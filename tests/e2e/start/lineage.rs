use std::fs;
use std::process::Command;

use tempfile::TempDir;

const KEEP_WHAT_IS_GIVEN_TO_READ: &str = "if [ -d /var/log/cladenc/attempts ]; then ls \
                                          /var/log/cladenc > given; fi";

fn ran(run_command: &str, max_procreations: &str) -> TempDir {
    let directory = test_inputs::temporary_directory();
    let options = [
        (
            "--primogenitor",
            test_inputs::primogenitor(directory.path()),
        ),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory.path(), &["t1", "t2", "t3"]),
        ),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", max_procreations.into()),
        ("--genealogy-growth", "0".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--attempt-timeout", "30".into()),
        ("--holdout-percentage", "50".into()),
        ("--holdout-corroboration-percentage", "50".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    directory
}

fn given_to_the_maker_of(child: &str) -> String {
    let directory = ran(KEEP_WHAT_IS_GIVEN_TO_READ, "2");
    fs::read_to_string(
        directory
            .path()
            .join("outputs/harnesses")
            .join(child)
            .join("given"),
    )
    .unwrap()
}

#[test]
fn the_primogenitor_that_procreates_for_the_first_time_is_given_its_attempts_and_nothing_else() {
    assert_eq!(given_to_the_maker_of("1"), "attempts\n");
}

#[test]
fn the_harness_that_procreates_next_is_given_the_child_it_made_or_the_line_it_descends_from() {
    assert!(
        ["attempts\nchildren\n", "attempts\nline\n"].contains(&given_to_the_maker_of("2").as_str())
    );
}

#[test]
fn the_outputs_directory_records_which_harness_made_a_child_and_how_the_procreation_ended() {
    let directory = ran("touch changed", "1");
    let kept = directory.path().join("outputs/procreations/1");
    assert_eq!(
        ["progenitor", "outcome"].map(|file| fs::read_to_string(kept.join(file)).unwrap()),
        ["0\n", "created\n"]
    );
}

#[test]
fn the_outputs_directory_records_a_procreation_that_changed_nothing_as_unchanged() {
    let directory = ran("exit 0", "1");
    let kept = directory.path().join("outputs/procreations/1");
    assert_eq!(
        ["progenitor", "outcome"].map(|file| fs::read_to_string(kept.join(file)).unwrap()),
        ["0\n", "unchanged\n"]
    );
}
