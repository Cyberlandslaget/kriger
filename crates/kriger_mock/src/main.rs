use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, RandomState},
    sync::{atomic::AtomicU32, Arc},
};

use anyhow::Result;
use axum::{
    extract::{self, Path, Query, State},
    routing::{get, put},
    Json, Router,
};
use clap::Parser;
use rand::{distributions::DistString, SeedableRng};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::debug;

mod util;

// Actual flag format:
//
// struct {
//     int32_t tick;
//     int32_t team;
//     intidk_t hash of the above;
// };
//
// We need to remember:
// - current tick
// - which flags have already been submitted
//
// Note: base36 encoding is annoying so instead this just takes each byte in the buffer mod 36.
#[derive(Default)]
struct FlagStore {
    tick: AtomicU32,
    // submitted[tick % 5] contains set of teams whose flags have been submitted
    submitted: [Mutex<HashSet<u32>>; 5],
    hasher: RandomState,
}

struct TickerControl {
    tx: tokio::sync::mpsc::Sender<()>,
}

struct AppState {
    fs: Arc<FlagStore>,
    ticker: TickerControl,
}

impl TickerControl {
    async fn tick(&self) {
        self.tx.send(()).await.unwrap();
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
enum FlagStatus {
    AlreadySubmitted,
    Accepted,
    Rejected,
    Expired,
    InvalidEncoding,
}

struct ProcessedFlag {
    tick: u32,
    team: u32,
    status: FlagStatus,
}

impl FlagStore {
    pub async fn tick(&self) {
        let old_tick = self.tick.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.submitted[((old_tick + 1) % 5) as usize]
            .lock()
            .await
            .clear();

        debug!("Tick!");
    }

    fn complete(&self, buf: &mut [u8; 31]) {
        let hash = self.hasher.hash_one(&buf[0..14]);
        buf[14..22].copy_from_slice(&hash.to_le_bytes());
        // we need more data
        let hash = self.hasher.hash_one(&buf[0..15]);
        buf[22..30].copy_from_slice(&hash.to_le_bytes());
        let hash = self.hasher.hash_one(&buf[0..16]);
        buf[30..31].copy_from_slice(&hash.to_le_bytes()[..1]);
    }

    fn get_flag_at_time(&self, tick: u32, team: u32) -> String {
        let mut buf = [0u8; 31];
        util::encode_u32(&mut buf[0..7], tick);
        util::encode_u32(&mut buf[7..14], team);

        self.complete(&mut buf);
        util::encode(&mut buf[14..]);

        let mut flag = String::from_utf8(buf.to_vec()).unwrap();
        flag.push('=');
        flag
    }

    pub fn get_flag(&self, team: u32) -> String {
        let tick = self.tick.load(std::sync::atomic::Ordering::Acquire);
        self.get_flag_at_time(tick, team)
    }

    const INVALID: ProcessedFlag = ProcessedFlag {
        tick: 0,
        team: 0,
        status: FlagStatus::InvalidEncoding,
    };

    pub async fn verify_flag(&self, flag: &str) -> ProcessedFlag {
        let current_tick = self.tick.load(std::sync::atomic::Ordering::Acquire);

        // Read data from flag, and decode
        if flag.len() != 32 || flag.as_bytes().last() != Some(&b'=') {
            return Self::INVALID;
        }
        let mut buf = [0u8; 31];
        buf.copy_from_slice(&flag.as_bytes()[..31]);

        let Some(tick) = util::decode_u32(&mut buf[..7]) else {
            return Self::INVALID;
        };
        let Some(team) = util::decode_u32(&mut buf[7..14]) else {
            return Self::INVALID;
        };
        // Just to ensure the entire string is correctly encoded
        let Some(()) = util::decode(&mut buf[14..]) else {
            return Self::INVALID;
        };

        let expected_flag = self.get_flag_at_time(tick, team);

        let status;
        if current_tick as i64 - tick as i64 >= 5 {
            status = FlagStatus::Expired;
        } else if flag == &expected_flag {
            let mut hm = self.submitted[tick as usize % 5].lock().await;
            if hm.contains(&team) {
                status = FlagStatus::AlreadySubmitted;
            } else {
                hm.insert(team);
                status = FlagStatus::Accepted;
            }
        } else {
            status = FlagStatus::Rejected;
        }

        ProcessedFlag { tick, team, status }
    }
}

async fn getflag(state: State<Arc<AppState>>, Path(team): Path<u32>) -> Json<String> {
    Json(state.fs.get_flag(team))
}

async fn getflags(state: State<Arc<AppState>>, Path(team): Path<u32>) -> Json<Vec<String>> {
    let current_tick = state.fs.tick.load(std::sync::atomic::Ordering::Acquire);

    // this is a little racy but it's fiine
    let first_tick = if current_tick < 6 {
        1
    } else {
        current_tick - 4
    };

    Json(
        (first_tick..=current_tick)
            .map(|tick| state.fs.get_flag_at_time(tick, team))
            .collect(),
    )
}

#[derive(Serialize)]
struct FlagsResponseItem {
    msg: String,
    flag: String,
    status: bool,
}

async fn flags(
    state: State<Arc<AppState>>,
    Json(flags): extract::Json<Vec<String>>,
) -> Json<Vec<FlagsResponseItem>> {
    let mut responses = vec![];

    for flag in flags {
        let flag_info = state.fs.verify_flag(&flag).await;
        let msg = match flag_info.status {
            FlagStatus::Accepted => format!("[{flag}] Accepted: 100 flag points"),
            FlagStatus::Rejected | FlagStatus::InvalidEncoding => {
                format!("[{flag}] Denied: invalid flag")
            }
            FlagStatus::AlreadySubmitted => format!("[{flag}] Denied: flag already claimed"),
            FlagStatus::Expired => format!("[{flag}] Denied: flag too old"),
        };

        responses.push(FlagsResponseItem {
            msg,
            flag,
            status: flag_info.status == FlagStatus::Accepted,
        });
    }

    Json(responses)
}

async fn force_tick(state: State<Arc<AppState>>) {
    state.ticker.tick().await;
}

async fn autotick(fs: Arc<FlagStore>, duration: Option<u64>) -> TickerControl {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    if let Some(duration) = duration {
        tokio::task::spawn(async move {
            loop {
                let ticker = async {
                    let mut it = tokio::time::interval(std::time::Duration::from_secs(duration));

                    // Intentionally not skipping first tick
                    loop {
                        it.tick().await;
                        fs.tick().await;
                    }
                };

                tokio::select! {
                    _ = ticker => unreachable!(),
                    _ = rx.recv() => {},
                };
            }
        });
    } else {
        tokio::task::spawn(async move {
            loop {
                rx.recv().await.unwrap();
                fs.tick().await;
            }
        });
    }

    TickerControl { tx }
}

#[derive(Deserialize, Debug)]
struct FlagIdQuery {
    service: Option<String>,
    team: Option<u32>,
}

#[derive(Serialize)]
struct FlagIdResponse(
    HashMap<
        String, /* chall */
        HashMap<
            u32, /* team */
            HashMap<u32 /* tick */, HashMap<&'static str /* hint name */, String>>,
        >,
    >,
);

async fn flag_id(
    state: State<Arc<AppState>>,
    query: Query<FlagIdQuery>,
) -> Result<Json<FlagIdResponse>, String> {
    println!("{query:?}");

    if query.service.is_none() && query.team.is_none() {
        return Err("You must specify a service or a team".into());
    }

    let service_names;
    let teams;
    if let Some(ref service) = query.service {
        service_names = vec![service.as_str()];
    } else {
        service_names = vec![
            "Idk-1",
            "Idk-2",
            "AnotherChall-1",
            "RSA-1",
            "RSA-2",
            "RSA-3",
        ];
    }
    if let Some(team) = query.team {
        teams = vec![team];
    } else {
        teams = (0..10).collect();
    }

    let ticks = 0..=state.fs.tick.load(std::sync::atomic::Ordering::Acquire);

    // TODO: sort data
    let data = service_names
        .iter()
        .map(|&s| {
            (
                s.to_owned(),
                teams
                    .iter()
                    .map(|&i| {
                        (
                            i,
                            ticks
                                .clone()
                                .map(|t| {
                                    (
                                        t,
                                        [("hint", get_hint_thing(&state.fs.hasher, s, i, t))]
                                            .into_iter()
                                            .collect(),
                                    )
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            )
        })
        .collect();

    Ok(Json(FlagIdResponse(data)))
}

fn get_hint_thing(hasher: &RandomState, s: &str, i: u32, t: u32) -> String {
    let a = hasher.hash_one((s, i, t));
    let mut seed = [0; 32];
    seed[0..8].copy_from_slice(&a.to_le_bytes());
    let mut rng = rand::rngs::StdRng::from_seed(seed);
    rand::distributions::Alphanumeric.sample_string(&mut rng, 10)
}

#[derive(Parser)]
struct Args {
    #[clap(default_value = "0.0.0.0:8080")]
    listen_on: String,

    #[clap(long)]
    /// Number of seconds between each tick
    autotick: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    let fs = Arc::new(FlagStore::default());
    let ticker = autotick(fs.clone(), args.autotick).await;
    let state = Arc::new(AppState { fs, ticker });

    let app = Router::new()
        .route("/getflag/:team", get(getflag))
        .route("/getflags/:team", get(getflags))
        .route("/flags", put(flags))
        .route("/forceTick", put(force_tick))
        .route("/flagId", get(flag_id))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.listen_on).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
