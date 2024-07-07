use std::hash::{BuildHasher, RandomState};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use clap::Parser;

mod util {
    pub(crate) fn un_hex(a: &[u8], b: &mut [u8]) -> Option<()> {
        if a.len() != 2 * b.len() {
            return None;
        }

        fn un_hex_char(a: u8) -> Option<u8> {
            if a.is_ascii_hexdigit() {
                if a.is_ascii_digit() {
                    Some(a - b'0')
                } else if a.is_ascii_uppercase() {
                    Some(a - b'A' + 10)
                } else {
                    Some(a - b'a' + 10)
                }
            } else {
                None
            }
        }

        for i in 0..b.len() {
            b[i] = un_hex_char(a[2 * i])? << 4 | un_hex_char(a[2 * i + 1])?;
        }

        Some(())
    }

    pub(crate) fn hex(a: &[u8], b: &mut [u8]) -> Option<()> {
        if 2 * a.len() != b.len() {
            return None;
        }

        fn hex_char(a: u8) -> u8 {
            if a < 10 {
                a + b'0'
            } else {
                a - 10 + b'a'
            }
        }

        for i in 0..a.len() {
            b[2 * i] = hex_char(a[i] >> 4);
            b[2 * i + 1] = hex_char(a[i] & 0xf);
        }

        Some(())
    }
}

async fn flag(State(random_state): State<RandomState>) -> String {
    let n: u32 = rand::random();

    let mut buf = [0u8; 12];
    buf[..4].copy_from_slice(&n.to_le_bytes());
    let hash = random_state.hash_one(&buf[..4]).to_le_bytes();
    buf[4..].copy_from_slice(&hash);

    let mut out_buf = vec![0u8; 24];
    util::hex(&buf, &mut out_buf);

    String::from_utf8(out_buf).unwrap()
}

async fn validate(
    State(random_state): State<RandomState>,
    Path(flag): Path<String>,
) -> &'static str {
    let mut buf = [0u8; 12];
    let Some(()) = util::un_hex(flag.as_bytes(), &mut buf) else {
        return "invalid flag (expected 12 bytes, hex-encoded)";
    };

    // very secure hash indeed
    let hash = random_state.hash_one(&buf[..4]);

    if &buf[4..] == &hash.to_le_bytes() {
        "correct flag"
    } else {
        "incorrect flag"
    }
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
        .route("/flag", get(flag))
        .route("/validate/:flag", get(validate))
        .with_state(RandomState::new());

    let listener = tokio::net::TcpListener::bind(&args.listen_on).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
