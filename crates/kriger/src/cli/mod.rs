use indicatif::ProgressBar;
use std::time::Duration;

pub(crate) mod args;
pub(crate) mod create;
pub(crate) mod deploy;
mod emoji;
mod model;

fn log(p: &ProgressBar, message: String) {
    p.suspend(|| {
        println!("  {message}");
    });
}

fn format_duration_secs(duration: &Duration) -> String {
    let secs_fractional = duration.as_millis() as f32 / 1000f32;
    format!("{secs_fractional:.2}s")
}
