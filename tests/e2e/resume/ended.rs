use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;

struct Resumed {
    directory: TempDir,
    run: Output,
    journal_before: Vec<u8>,
    resumed: Output,
    journal_after: Vec<u8>,
}

static RESUMED: LazyLock<Resumed> = LazyLock::new(|| {
    let (directory, run) = ended();
    let journal = directory.path().join("outputs/journal.jsonl");
    let journal_before = fs::read(&journal).unwrap();
    let resumed = resume(directory.path());
    let journal_after = fs::read(&journal).unwrap();
    Resumed {
        directory,
        run,
        journal_before,
        resumed,
        journal_after,
    }
});

static REFUSED_FOR_QUIET: LazyLock<Output> =
    LazyLock::new(|| resume_with(RESUMED.directory.path(), Some("--quiet")));

fn ended() -> (TempDir, Output) {
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
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", test_inputs::SOLVE_ONE_MORE.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let output = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    (directory, output)
}

fn resume(directory: &Path) -> Output {
    resume_with(directory, None)
}

fn report_of(directory: &Path) -> String {
    let report = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .output()
        .unwrap();
    String::from_utf8(report.stdout).unwrap()
}

fn resume_with(directory: &Path, another: Option<&str>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .args(another)
        .output()
        .unwrap()
}

#[test]
fn resume_on_a_run_that_ended_prints_the_path_the_run_printed() {
    assert_eq!(RESUMED.resumed.stdout, RESUMED.run.stdout);
}

#[test]
fn resume_on_a_run_that_ended_exits_with_0() {
    assert_eq!(RESUMED.resumed.status.code(), Some(0));
}

#[test]
fn resume_on_a_run_that_ended_leaves_the_journal_unchanged() {
    assert_eq!(RESUMED.journal_after, RESUMED.journal_before);
}

#[test]
fn resume_on_a_run_that_ended_says_on_standard_error_that_the_directed_evolution_had_finished() {
    assert!(
        str::from_utf8(&RESUMED.resumed.stderr)
            .unwrap()
            .starts_with(
                "cladenc: the directed evolution had finished: its best harness is harness "
            )
    );
}

#[test]
fn resume_on_a_run_that_ended_prints_on_standard_error_what_report_prints() {
    let said = str::from_utf8(&RESUMED.resumed.stderr).unwrap();
    assert_eq!(
        said.split_once('\n').map(|(_, report)| report.to_owned()),
        Some(report_of(RESUMED.directory.path()))
    );
}

#[test]
fn quiet_which_is_no_flag_of_resume_exits_with_2() {
    assert_eq!(REFUSED_FOR_QUIET.status.code(), Some(2));
}

#[test]
fn quiet_which_is_no_flag_of_resume_is_refused_as_an_argument_that_is_not_expected() {
    assert!(
        str::from_utf8(&REFUSED_FOR_QUIET.stderr)
            .unwrap()
            .starts_with("error: unexpected argument '--quiet' found\n")
    );
}

#[test]
fn quiet_which_is_no_flag_of_resume_prints_nothing_on_standard_output() {
    assert_eq!(REFUSED_FOR_QUIET.stdout, Vec::<u8>::new());
}
