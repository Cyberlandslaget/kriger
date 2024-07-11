use chrono::{DateTime, Utc};
use color_eyre::Result;

// Ported from angrepa
pub(crate) fn get_instant_from_datetime(target: DateTime<Utc>) -> Result<tokio::time::Instant> {
    let time_since_start = Utc::now() - target;
    let instant = if time_since_start < chrono::Duration::seconds(0) {
        // The target time is in the future, we have to negate it
        tokio::time::Instant::now() + (-time_since_start).to_std()?
    } else {
        // The target time is in the past
        tokio::time::Instant::now() - time_since_start.to_std()?
    };
    Ok(instant)
}
