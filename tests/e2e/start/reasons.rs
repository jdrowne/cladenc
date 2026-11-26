use std::path::Path;
use std::process::{Command, Output};

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--diversification-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-diversifications", "1".into()),
        ("--max-concurrent-sandboxes", "1".into()),
        ("--diversification-timeout", "60".into()),
        ("--max-verifier-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn said_last(ended: &Output) -> String {
    let said = str::from_utf8(&ended.stderr).unwrap();
    said.lines().last().unwrap_or_default().to_owned()
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_says_that_the_first_environment_is_not_built() {
    let directory = test_inputs::temporary_directory();
    let ended = start(directory.path(), "exit 0")
        .env("PATH", directory.path())
        .output()
        .unwrap();
    assert_eq!(
        said_last(&ended),
        "cladenc: the environment of task benchmark/t1 was not built: its build could not be \
         carried out: No such file or directory (os error 2)"
    );
}
