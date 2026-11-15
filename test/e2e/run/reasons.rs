use std::process::Command;

#[test]
fn a_run_an_evaluator_stopped_says_which_evaluation_and_how_it_ended() {
    let directory = test_inputs::temporary_directory();
    let options = [
        ("--progenitor", test_inputs::progenitor(directory.path())),
        ("--tasks", test_inputs::task_file(directory.path(), &["t1"])),
        ("--procreator", test_inputs::sample("e2e", "exit-0.sh")),
        ("--evaluator", test_inputs::sample("e2e", "exit-2.sh")),
        ("--run-dir", directory.path().join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "0".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--evaluation-timeout", "60".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let ended = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("run")
        .args(options)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(ended.stderr).unwrap().lines().last(),
        Some(
            "cladenc: the search stopped before its end: \
             the evaluation of harness 0 on task t1 exited with status 2"
        )
    );
}
