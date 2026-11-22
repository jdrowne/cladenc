use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const END_WITH_125_WHERE_MARKED: &str = "[ -e end-with-125 ] && exit 125; echo \"$API_TOKEN\"";
const WITH_A_TOKEN: &str = "[environment.env]\nAPI_TOKEN = \"${CLADENC_TEST_TOKEN}\"\n";

fn stopped() -> TempDir {
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
        ("--harness-run-command", END_WITH_125_WHERE_MARKED.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    fs::write(
        directory.path().join("benchmark/t2/task.toml"),
        WITH_A_TOKEN,
    )
    .unwrap();
    test_inputs::end_with_125_on(directory.path(), "t2");
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .env("CLADENC_TEST_TOKEN", "a-made-up-token")
        .output()
        .unwrap();
    test_inputs::end_with_125_on_no_more(&directory.path().join("outputs"), "t2");
    directory
}

fn resume(directory: &Path) -> Command {
    let mut resume = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    resume.arg("resume").arg(format!(
        "--outputs-dir={}",
        directory.join("outputs").display()
    ));
    resume
}

fn resumed_with_the_variable_no_longer_set() -> Output {
    resume(stopped().path()).output().unwrap()
}

fn resumed_with_the_variable_set_to(value: &str) -> (TempDir, Output) {
    let directory = stopped();
    let output = resume(directory.path())
        .env("CLADENC_TEST_TOKEN", value)
        .output()
        .unwrap();
    (directory, output)
}

#[test]
fn a_variable_of_a_task_that_is_no_longer_set_stops_cladenc_resume_with_exit_status_2() {
    assert_eq!(
        resumed_with_the_variable_no_longer_set().status.code(),
        Some(2)
    );
}

#[test]
fn a_variable_of_a_task_that_is_no_longer_set_is_reported_last_with_the_task_and_the_variable() {
    assert!(
        String::from_utf8(resumed_with_the_variable_no_longer_set().stderr)
            .unwrap()
            .ends_with(
                "cladenc: task benchmark/t2 cannot be attempted: its task.toml gives API_TOKEN \
                 under environment.env the value of CLADENC_TEST_TOKEN, which is not set where \
                 cladenc runs\n"
            )
    );
}

#[test]
fn a_resumed_directed_evolution_whose_variables_are_set_runs_to_its_end() {
    assert_eq!(
        resumed_with_the_variable_set_to("a-made-up-token")
            .1
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn a_resumed_directed_evolution_gives_a_variable_the_value_it_has_when_it_is_resumed() {
    let (directory, _output) = resumed_with_the_variable_set_to("another-token");
    let attempt = test_inputs::kept_attempt(&directory.path().join("outputs"), 0, "benchmark/t2");
    assert_eq!(
        fs::read_to_string(attempt.join("stdout")).unwrap(),
        "another-token\n"
    );
}
