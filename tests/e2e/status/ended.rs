use std::process::{Command, Output};

fn status_of_one_that_has_ended() -> Output {
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
        (
            "--harness-run-command",
            test_inputs::SOLVE_EVERY_TASK.into(),
        ),
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
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("status")
        .arg(format!(
            "--outputs-dir={}",
            directory.path().join("outputs").display()
        ))
        .output()
        .unwrap()
}

#[test]
fn status_exits_with_2_on_a_directed_evolution_that_has_ended() {
    assert_eq!(status_of_one_that_has_ended().status.code(), Some(2));
}

#[test]
fn status_says_of_a_directed_evolution_that_has_ended_to_call_report() {
    assert_eq!(
        String::from_utf8(status_of_one_that_has_ended().stderr).unwrap(),
        "cladenc: the directed evolution has ended: call `cladenc report` to read its report\n"
    );
}

#[test]
fn status_prints_nothing_on_standard_output_of_a_directed_evolution_that_has_ended() {
    assert_eq!(status_of_one_that_has_ended().stdout, Vec::<u8>::new());
}
