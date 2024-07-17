use std::{
    collections::HashSet,
    hash::{BuildHasher, RandomState},
    sync::{atomic::AtomicU32, Arc},
};

use anyhow::Result;
use axum::{
    extract::{self, Path, State},
    routing::{get, put},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

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

#[derive(Default)]
struct AppState {
    fs: FlagStore,
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

async fn flag(state: State<Arc<AppState>>, Path(team): Path<u32>) -> Json<String> {
    Json(state.fs.get_flag(team))
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
    state.fs.tick().await;
}

#[derive(Parser)]
struct Args {
    #[clap(default_value = "0.0.0.0:8080")]
    listen_on: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let app = Router::new()
        .route("/getflag/:team", get(flag))
        .route("/flags", put(flags))
        .route("/force-tick", put(force_tick))
        .with_state(Arc::new(AppState::default()));

    let listener = tokio::net::TcpListener::bind(&args.listen_on).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
