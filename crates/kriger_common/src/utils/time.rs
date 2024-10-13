// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use chrono::{DateTime, Utc};
use color_eyre::eyre;

// Ported from angrepa
pub fn get_instant_from_datetime(target: DateTime<Utc>) -> eyre::Result<tokio::time::Instant> {
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
