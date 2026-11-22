use std::fs;
use std::path::Path;

fn readme() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md")).unwrap()
}

#[test]
fn the_readme_says_that_every_harness_is_given_a_directory_whose_contents_are_kept() {
    assert!(readme().contains(
        "When the harness is started, the container holds `/logs/artifacts`, an empty \
         directory that belongs to the user the harness runs as, and what the harness left \
         there is kept as `artifacts/logs/artifacts`, though the task lists no such path."
    ));
}

#[test]
fn the_readme_says_when_the_commands_a_task_lists_under_collect_are_run() {
    assert!(readme().contains(
        "Where the task's `task.toml` lists commands under `verifier.collect`, `cladenc` runs \
         each in the container once the harness has ended or was stopped, before it keeps any \
         path, with the time limit that the entry gives as `timeout_sec`, or 60 seconds where \
         it gives none."
    ));
}

#[test]
fn the_readme_says_what_a_command_under_collect_that_fails_does_and_where_its_output_is() {
    assert!(readme().contains(
        "A command that fails does not end the attempt, and what each command printed is kept \
         in `collect`."
    ));
}
