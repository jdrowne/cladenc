use std::time::Duration;

use crate::dataset::HealthCheck;
use crate::dataset::configuration::{self, ConfigurationError};
use crate::file_system::tests::double::Memory;

const CHECKED: &str = "[environment.healthcheck]\ncommand = \"test -e /ready\"\n";

fn read(written: &str) -> Result<Option<HealthCheck>, ConfigurationError> {
    let directory = Memory::default();
    directory
        .write(directory.path().join("task.toml"), written)
        .unwrap();
    configuration::read_with(&directory, directory.path(), &|_| None)
        .map(|configuration| configuration.health_check)
}

fn checked(more: &str) -> HealthCheck {
    read(&format!("{CHECKED}{more}")).unwrap().unwrap()
}

fn refused(more: &str) -> Option<ConfigurationError> {
    read(&format!("{CHECKED}{more}")).err()
}

fn not_a_wait(key: &'static str) -> Option<ConfigurationError> {
    Some(ConfigurationError::NotAWait { key })
}

#[test]
fn a_task_that_gives_no_health_check_has_none() {
    assert_eq!(read(""), Ok(None));
}

#[test]
fn the_command_of_a_health_check_is_read() {
    assert_eq!(checked("").command, "test -e /ready");
}

#[test]
fn a_health_check_whose_command_is_written_with_nothing_in_it_is_none() {
    assert_eq!(
        read("[environment.healthcheck]\ncommand = \"\"\n"),
        Ok(None)
    );
}

#[test]
fn the_settings_of_a_health_check_with_no_command_in_it_are_not_looked_at() {
    assert_eq!(
        read("[environment.healthcheck]\ncommand = \"\"\ninterval_sec = -1\n"),
        Ok(None)
    );
}

#[test]
fn a_health_check_is_run_again_after_5_seconds_where_the_task_gives_no_interval() {
    assert_eq!(checked("").interval, Duration::from_secs(5));
}

#[test]
fn a_run_of_a_health_check_has_30_seconds_where_the_task_gives_no_time_limit() {
    assert_eq!(checked("").time_limit, Duration::from_secs(30));
}

#[test]
fn a_health_check_has_no_start_period_where_the_task_gives_none() {
    assert_eq!(checked("").start_period, Duration::ZERO);
}

#[test]
fn a_health_check_is_run_again_after_5_seconds_in_its_start_period_where_the_task_gives_no_interval()
 {
    assert_eq!(checked("").start_interval, Duration::from_secs(5));
}

#[test]
fn a_health_check_may_fail_3_times_where_the_task_gives_no_retries() {
    assert_eq!(checked("").retries, 3);
}

#[test]
fn the_interval_of_a_health_check_is_read_with_its_part_of_a_second() {
    assert_eq!(
        checked("interval_sec = 0.5\n").interval,
        Duration::from_millis(500)
    );
}

#[test]
fn the_interval_of_a_health_check_is_read_where_it_is_a_whole_number() {
    assert_eq!(
        checked("interval_sec = 2\n").interval,
        Duration::from_secs(2)
    );
}

#[test]
fn an_interval_of_0_seconds_is_taken() {
    assert_eq!(checked("interval_sec = 0\n").interval, Duration::ZERO);
}

#[test]
fn the_time_limit_of_a_run_of_a_health_check_is_read() {
    assert_eq!(
        checked("timeout_sec = 2.5\n").time_limit,
        Duration::from_millis(2500)
    );
}

#[test]
fn the_start_period_of_a_health_check_is_read() {
    assert_eq!(
        checked("start_period_sec = 60\n").start_period,
        Duration::from_secs(60)
    );
}

#[test]
fn the_interval_of_the_start_period_of_a_health_check_is_read() {
    assert_eq!(
        checked("start_interval_sec = 1\n").start_interval,
        Duration::from_secs(1)
    );
}

#[test]
fn the_retries_of_a_health_check_are_read() {
    assert_eq!(checked("retries = 7\n").retries, 7);
}

#[test]
fn a_setting_under_a_health_check_that_harbor_does_not_name_is_let_through() {
    assert_eq!(checked("flavour = 5\n").command, "test -e /ready");
}

#[test]
fn the_health_check_of_a_verifier_environment_is_not_the_health_check_of_the_task() {
    assert_eq!(
        read("[verifier.environment.healthcheck]\ncommand = \"test -e /ready\"\n"),
        Ok(None)
    );
}

#[test]
fn a_time_limit_of_0_seconds_for_a_run_of_a_health_check_is_refused() {
    assert_eq!(
        refused("timeout_sec = 0\n"),
        Some(ConfigurationError::NotATimeLimit {
            key: "environment.healthcheck.timeout_sec"
        })
    );
}

#[test]
fn an_interval_below_0_is_refused() {
    assert_eq!(
        refused("interval_sec = -1\n"),
        not_a_wait("environment.healthcheck.interval_sec")
    );
}

#[test]
fn an_interval_that_is_not_a_number_of_seconds_is_refused() {
    assert_eq!(
        refused("interval_sec = nan\n"),
        not_a_wait("environment.healthcheck.interval_sec")
    );
}

#[test]
fn a_start_period_below_0_is_refused() {
    assert_eq!(
        refused("start_period_sec = -0.5\n"),
        not_a_wait("environment.healthcheck.start_period_sec")
    );
}

#[test]
fn an_interval_of_the_start_period_below_0_is_refused() {
    assert_eq!(
        refused("start_interval_sec = -5\n"),
        not_a_wait("environment.healthcheck.start_interval_sec")
    );
}

#[test]
fn retries_of_0_are_refused() {
    assert_eq!(
        refused("retries = 0\n"),
        Some(ConfigurationError::NotAbove0 {
            key: "environment.healthcheck.retries"
        })
    );
}

#[test]
fn retries_below_0_are_refused() {
    assert_eq!(
        refused("retries = -3\n"),
        Some(ConfigurationError::NotAbove0 {
            key: "environment.healthcheck.retries"
        })
    );
}

#[test]
fn the_longest_a_health_check_takes_is_every_run_at_its_time_limit_and_every_interval() {
    assert_eq!(checked("").longest(), Duration::from_secs(135));
}

#[test]
fn the_longest_a_health_check_takes_holds_its_start_period() {
    assert_eq!(
        checked("start_period_sec = 60\n").longest(),
        Duration::from_secs(195)
    );
}

#[test]
fn the_longest_a_health_check_takes_is_counted_for_retries_beyond_any_count() {
    assert_eq!(
        checked("retries = 9223372036854775807\n").longest(),
        Duration::from_secs(30 + 35 * u64::from(u32::MAX))
    );
}
