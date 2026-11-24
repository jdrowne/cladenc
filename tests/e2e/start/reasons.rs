use std::path::Path;
use std::process::Command;

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
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

fn said_last(start: &mut Command) -> String {
    let ended = start.output().unwrap();
    let said = String::from_utf8(ended.stderr).unwrap();
    said.lines().last().unwrap_or_default().to_owned()
}

#[test]
fn a_run_a_container_stopped_says_which_attempt_and_where_what_docker_said_is_kept() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        said_last(&mut start(directory.path(), "exit 125")),
        format!(
            "cladenc: the directed evolution stopped before its end: the attempt of harness 0 \
             on task benchmark/t1 could not be carried out: Docker did not start its container, \
             and {} holds what Docker said",
            directory
                .path()
                .join("outputs/running/attempt-0-benchmark+t1/stderr")
                .display()
        )
    );
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_says_that_the_first_environment_is_not_built() {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), "exit 0");
    start.env("PATH", directory.path());
    assert_eq!(
        said_last(&mut start),
        "cladenc: the environment of task benchmark/t1 was not built: its build could not be \
         carried out: No such file or directory (os error 2)"
    );
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), "exit 0");
    start.env("PATH", directory.path());
    assert_eq!(start.output().unwrap().status.code(), Some(2));
}
