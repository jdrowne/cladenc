use std::env;
use std::path::Path;
use std::process::Command;

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--run-dir", directory.join("run")),
        ("--max-procreations", "1".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--attempt-timeout", "60".into()),
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
                .join("run/running/attempt-0-benchmark+t1/stderr")
                .display()
        )
    );
}

#[test]
fn a_run_stopped_by_a_docker_that_a_signal_ended_says_which_attempt_and_which_signal() {
    let directory = test_inputs::temporary_directory();
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "killed").display(),
        env::var("PATH").unwrap()
    );
    let mut start = start(directory.path(), "exit 0");
    start
        .env("PATH", path)
        .env("CLADENC_TEST_SIGNAL_OF_DOCKER", "TERM");
    assert_eq!(
        said_last(&mut start),
        "cladenc: the directed evolution stopped before its end: the attempt of harness 0 on \
         task benchmark/t1 was ended by signal 15"
    );
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_says_that_its_attempt_could_not_be_carried_out() {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), "exit 0");
    start.env("PATH", directory.path());
    assert!(said_last(&mut start).starts_with(
        "cladenc: the directed evolution stopped before its end: the attempt of harness 0 on \
         task benchmark/t1 could not be carried out: "
    ));
}

#[test]
fn a_run_on_a_machine_where_docker_is_not_found_exits_with_1() {
    let directory = test_inputs::temporary_directory();
    let mut start = start(directory.path(), "exit 0");
    start.env("PATH", directory.path());
    assert_eq!(start.output().unwrap().status.code(), Some(1));
}
