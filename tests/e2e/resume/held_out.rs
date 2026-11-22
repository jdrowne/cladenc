use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

fn start(directory: &Path) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3", "t4"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        (
            "--harness-run-command",
            test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.into(),
        ),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

struct Resumed {
    directory: TempDir,
    run_record_when_stopped: String,
}

fn stopped_at_its_first_attempt_and_resumed() -> Resumed {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path());
    test_inputs::end_with_125_on(directory.path(), "t1");
    start.output().unwrap();
    let outputs = directory.path().join("outputs");
    let run_record_when_stopped = run_record_of(&outputs);
    test_inputs::end_with_125_on_no_more(&outputs, "t1");
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .args(["resume", "--outputs-dir"])
        .arg(&outputs)
        .output()
        .unwrap();
    Resumed {
        directory,
        run_record_when_stopped,
    }
}

fn run_record_of(outputs: &Path) -> String {
    let journal = fs::read_to_string(outputs.join("journal.jsonl")).unwrap();
    journal.lines().next().unwrap().to_owned()
}

fn holdout_tasks_of(run_record: &str) -> Vec<String> {
    let run_record: Value = serde_json::from_str(run_record).unwrap();
    run_record["holdout_tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task.as_str().unwrap().to_owned())
        .collect()
}

fn tasks_kept_under_holdout_of(resumed: &Resumed, harness: &str) -> Vec<String> {
    let benchmark = resumed
        .directory
        .path()
        .join("outputs/holdout")
        .join(harness)
        .join("benchmark");
    let mut tasks: Vec<String> = fs::read_dir(benchmark)
        .unwrap()
        .map(|entry| format!("benchmark/{}", entry.unwrap().file_name().to_string_lossy()))
        .collect();
    tasks.sort();
    tasks
}

#[test]
fn resume_leaves_the_holdout_tasks_in_the_journal_as_they_were_drawn() {
    let resumed = stopped_at_its_first_attempt_and_resumed();
    assert_eq!(
        run_record_of(&resumed.directory.path().join("outputs")),
        resumed.run_record_when_stopped
    );
}

#[test]
fn resume_keeps_the_attempts_of_the_primogenitor_at_the_holdout_tasks_under_holdout() {
    let resumed = stopped_at_its_first_attempt_and_resumed();
    assert_eq!(
        tasks_kept_under_holdout_of(&resumed, "0"),
        holdout_tasks_of(&resumed.run_record_when_stopped)
    );
}

#[test]
fn resume_keeps_the_attempts_of_a_child_at_the_holdout_tasks_under_holdout() {
    let resumed = stopped_at_its_first_attempt_and_resumed();
    assert_eq!(
        tasks_kept_under_holdout_of(&resumed, "1"),
        holdout_tasks_of(&resumed.run_record_when_stopped)
    );
}
