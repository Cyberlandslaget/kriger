use futures::Future;
use indicatif::ProgressBar;
use std::time::Duration;

pub(crate) mod args;
pub(crate) mod create;
pub(crate) mod deploy;
mod emoji;
mod models;

fn log(p: &ProgressBar, message: String) {
    p.suspend(|| {
        println!("  {message}");
    });
}

fn format_duration_secs(duration: &Duration) -> String {
    let secs_fractional = duration.as_millis() as f32 / 1000f32;
    format!("{secs_fractional:.2}s")
}

/// Displays a spinner in the console while the future is running. The caller is responsible for
/// displaying a message signifying the completion.
async fn with_spinner<F, Fut, T, E>(message: &'static str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(130));
    pb.set_message(message);

    let res = f().await;
    pb.finish_and_clear();

    res
}
