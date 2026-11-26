fn readme() -> String {
    test_inputs::README.to_owned()
}

#[test]
fn the_readme_says_that_what_a_harness_left_in_logs_artifacts_is_kept() {
    assert!(readme().contains(
        "What a harness left in `/logs/artifacts` is kept as `artifacts/logs/artifacts`, \
         though the task lists no such path."
    ));
}

#[test]
fn the_readme_says_when_the_commands_a_task_lists_under_collect_are_run() {
    assert!(readme().contains(
        "Where the task's `task.toml` lists commands under `verifier.collect`, which Harbor calls \
         collection hooks, `cladenc` runs each in the sandbox once the harness has ended or \
         was stopped, before it keeps any path, with the time limit that the entry gives as \
         `timeout_sec`, or 60 seconds where it gives none."
    ));
}

#[test]
fn the_readme_says_what_a_command_under_collect_that_fails_does_and_where_its_output_is() {
    assert!(readme().contains(
        "A command that fails does not end the trial, and what each command printed is kept \
         in `collection-hooks`."
    ));
}
