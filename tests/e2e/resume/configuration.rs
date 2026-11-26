use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

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
            "--dataset",
            test_inputs::dataset(directory.path(), &["t1", "t2", "t3"]),
        ),
        ("--diversification-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", END_WITH_125_WHERE_MARKED.into()),
        ("--outputs-dir", directory.path().join("outputs")),
        ("--max-diversifications", "1".into()),
        ("--max-concurrent-sandboxes", "1".into()),
        ("--diversification-timeout", "30".into()),
        ("--max-verifier-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    fs::write(directory.path().join("dataset/t2/task.toml"), WITH_A_TOKEN).unwrap();
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

static RESUMED_WITH_THE_VARIABLE_NO_LONGER_SET: LazyLock<Output> =
    LazyLock::new(|| resume(stopped().path()).output().unwrap());

fn resumed_with_the_variable_set_to(value: &str) -> (TempDir, Output) {
    let directory = stopped();
    let output = resume(directory.path())
        .env("CLADENC_TEST_TOKEN", value)
        .output()
        .unwrap();
    (directory, output)
}

#[test]
fn a_variable_of_a_task_that_is_no_longer_set_is_reported_last_with_the_task_and_the_variable() {
    assert!(
        str::from_utf8(&RESUMED_WITH_THE_VARIABLE_NO_LONGER_SET.stderr)
            .unwrap()
            .ends_with(
                "cladenc: task dataset/t2 cannot be given a trial: its task.toml gives API_TOKEN \
                 under environment.env the value of CLADENC_TEST_TOKEN, which is not set where \
                 cladenc runs\n"
            )
    );
}

#[test]
fn a_resumed_directed_evolution_gives_a_variable_the_value_it_has_when_it_is_resumed() {
    let (directory, _output) = resumed_with_the_variable_set_to("another-token");
    let trial = test_inputs::kept_trial(&directory.path().join("outputs"), 0, "dataset/t2");
    assert_eq!(
        fs::read_to_string(trial.join("stdout")).unwrap(),
        "another-token\n"
    );
}
