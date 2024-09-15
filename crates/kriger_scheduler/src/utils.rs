use chrono::{DateTime, Utc};
use kriger_common::server::runtime::CompetitionConfig;

// Taken from angrepa
pub(crate) fn get_current_non_offsetting_tick(
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

pub(crate) fn is_team_excluded(config: &CompetitionConfig, team_id: &str) -> bool {
    if let Some(nop_team) = &config.nop_team {
        if nop_team == team_id {
            return true;
        }
    }
    if let Some(self_team) = &config.self_team {
        if self_team == team_id {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_current_non_offsetting_tick, is_team_excluded};
    use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Utc};
    use kriger_common::server::runtime::CompetitionConfig;

    const NOP_TEAM_ID: &str = "1";
    const SELF_TEAM_ID: &str = "4";

    fn expect_tick(offset: TimeDelta, expected_tick: i64) {
        // CTF starts at Jan 1st 2024, 08:00 AM UTC
        let start_time: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            ),
            Utc,
        );
        let tick = get_current_non_offsetting_tick(start_time, start_time + offset, 120);
        assert_eq!(tick, expected_tick);
    }

    fn create_competition_config() -> CompetitionConfig {
        // We do not care about other fields here except nop_team and self_team
        CompetitionConfig {
            start: Default::default(),
            tick: 0,
            tick_start: 0,
            flag_validity: 0,
            flag_format: "".to_string(),
            nop_team: Some(NOP_TEAM_ID.to_string()),
            self_team: Some(SELF_TEAM_ID.to_string()),
        }
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

    #[test]
    fn should_exclude_nop_team() {
        let config = create_competition_config();
        assert!(is_team_excluded(&config, NOP_TEAM_ID));
    }

    #[test]
    fn should_exclude_self_team() {
        let config = create_competition_config();
        assert!(is_team_excluded(&config, SELF_TEAM_ID));
    }

    #[test]
    fn should_not_exclude_other_team() {
        let config = create_competition_config();
        assert!(!is_team_excluded(&config, "1337"));
    }
}
