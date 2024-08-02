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

// Taken from angrepa
pub(crate) fn get_current_tick(
    start_time: DateTime<Utc>,
    current_time: DateTime<Utc>,
    tick_duration_secs: u64,
) -> i64 {
    let since_start = current_time - start_time;

    // "ew float" - Jonas
    let ticks_after_start = (since_start.num_seconds() as f64) / (tick_duration_secs as f64);

    // Round down so that we don't overshoot the tick. For example, if we're 1 ms early before the
    // start time, we're at tick -1, not 0.
    ticks_after_start.floor() as i64
}

#[cfg(test)]
mod tests {
    use crate::utils::get_current_tick;
    use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Utc};

    fn expect_tick(offset: TimeDelta, expected_tick: i64) {
        // CTF starts at Jan 1st 2024, 08:00 AM UTC
        let start_time: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            ),
            Utc,
        );
        let tick = get_current_tick(start_time, start_time + offset, 120);
        assert_eq!(tick, expected_tick);
    }

    #[test]
    fn should_have_correct_tick_exactly_at_start() {
        expect_tick(Duration::seconds(0), 0);
    }

    #[test]
    fn should_have_correct_tick_right_before_start() {
        expect_tick(Duration::seconds(-1), -1);
    }

    #[test]
    fn should_have_correct_tick_right_after_start() {
        expect_tick(Duration::seconds(1), 0);
    }

    #[test]
    fn should_have_correct_tick_one_hour_after_start() {
        expect_tick(Duration::hours(1), 30);
    }

    #[test]
    fn should_have_correct_tick_almost_one_hour_after_start() {
        expect_tick(Duration::minutes(59) + Duration::seconds(59), 29);
    }
    #[test]
    fn should_have_correct_tick_one_hour_before_start() {
        expect_tick(Duration::hours(-1), -30);
    }
}
