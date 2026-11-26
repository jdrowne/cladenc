use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;
use crate::sandboxes::SandboxError;
use crate::sandboxes::tests::double::{Answer, Double};

const BUILD: [&str; 2] = ["run", "make"];
const BUILD_OF_THE_CHILD: [&str; 2] = ["make", "builds/1"];
const A_TRIAL: [&str; 2] = ["run a harness", "trial-0-benchmark+t1"];
const BUILT: &str = r#"{"record":"primogenitor_built"}"#;

fn with_a_build_command(_: &Memory, settings: &mut Settings) {
    settings.harness_build_command = Some("make".to_owned());
}

fn built_by(docker: Double) -> Evolved {
    given::started(docker, with_a_build_command)
}

fn not_built() -> Evolved {
    built_by(Double::default().answering(&BUILD, &[Answer::Exited(1)]))
}

#[test]
fn a_primogenitor_with_a_build_command_is_built_before_it_trials_a_task() {
    let said = built_by(Double::default()).said;
    assert!(said.contains("the primogenitor is built"));
}

#[test]
fn a_primogenitor_that_is_built_trials_every_task() {
    assert_eq!(built_by(Double::default()).count_of("--detach"), 2);
}

#[test]
fn a_primogenitor_whose_build_fails_ends_the_directed_evolution_with_the_cause() {
    assert!(matches!(
        not_built().result,
        Err(DirectedEvolutionError::Build(SandboxError::Exit(1)))
    ));
}

#[test]
fn a_primogenitor_whose_build_fails_is_told_with_the_cause() {
    let said = not_built().said;
    assert!(said.contains("the build of the primogenitor exited with status 1"));
}

#[test]
fn a_primogenitor_whose_build_fails_trials_no_task() {
    assert_eq!(not_built().count_of("--detach"), 0);
}

#[test]
fn a_directed_evolution_that_is_stopped_while_the_primogenitor_is_built_was_asked_to_stop() {
    let stopped = built_by(Double::default().answering(&BUILD, &[Answer::Stopped]));
    assert!(matches!(stopped.result, Err(DirectedEvolutionError::Asked)));
}

fn with_a_selection_task(directory: &Memory, settings: &mut Settings) {
    with_a_build_command(directory, settings);
    settings.holdout_tasks = vec!["benchmark/t1".to_owned()];
}

fn with_a_child_that_does_not_build() -> Evolved {
    let docker = Double::default()
        .changing()
        .answering(&BUILD_OF_THE_CHILD, &[Answer::Exited(1)]);
    built_by(docker)
}

fn stopped_at_a_trial_and_resumed() -> Evolved {
    let docker = Double::default().answering(&A_TRIAL, &[Answer::Exited(125)]);
    given::resumed(built_by(docker), Double::default())
}

fn builds_of(evolved: &Evolved, source_code: &str) -> usize {
    let mounted = format!(" --volume OUTPUTS/{source_code}:/var/opt/cladenc/harness ");
    evolved
        .commands()
        .iter()
        .filter(|command| command.contains(&mounted) && command.contains(" make "))
        .count()
}

#[test]
fn the_build_of_the_primogenitor_is_journaled_right_after_the_run_record() {
    assert_eq!(
        built_by(Double::default()).journal().lines().nth(1),
        Some(BUILT)
    );
}

#[test]
fn the_primogenitor_of_a_directed_evolution_is_built_once() {
    let evolved = built_by(Double::default().changing());
    assert_eq!(builds_of(&evolved, "harnesses/0"), 1);
}

#[test]
fn a_child_is_built_once_in_the_clone_that_its_progenitor_diversified() {
    let evolved = built_by(Double::default().changing());
    assert_eq!(builds_of(&evolved, "running/diversification-1/child"), 1);
}

#[test]
fn a_directed_evolution_that_builds_its_harnesses_finds_the_best_among_them() {
    let docker = Double::default()
        .changing()
        .verifying(|trial| trial.starts_with("trial-1-"));
    assert_eq!(
        given::started(docker, with_a_selection_task).best_harness(),
        Some(1)
    );
}

#[test]
fn a_child_that_does_not_build_is_journaled_as_one_whose_build_failed() {
    let journal = with_a_child_that_does_not_build().journal();
    assert!(
        journal.contains(
            r#"{"record":"diversification_finished","child":1,"outcome":"build_failed","#
        )
    );
}

#[test]
fn a_directed_evolution_goes_on_to_its_end_past_a_child_that_did_not_build() {
    assert_eq!(with_a_child_that_does_not_build().best_harness(), Some(0));
}

#[test]
fn a_child_that_did_not_build_is_said_with_where_what_its_build_printed_is_kept() {
    let evolved = with_a_child_that_does_not_build();
    assert!(evolved.said.contains(&format!(
        "cladenc: the diversification of harness 1 by harness 0 made a harness that did not build: \
         what its build printed is kept in {}\n",
        evolved.outputs().join("builds/1").display()
    )));
}

#[test]
fn a_directed_evolution_whose_primogenitor_did_not_build_journals_its_run_record_alone() {
    assert_eq!(not_built().journal().lines().count(), 1);
}

#[test]
fn a_primogenitor_whose_build_fails_is_said_last_with_where_what_its_build_printed_is_kept() {
    let evolved = not_built();
    assert!(evolved.said.ends_with(&format!(
        "cladenc: the build of the primogenitor exited with status 1: what it printed is kept in \
         {}\n",
        evolved.outputs().join("builds/0").display()
    )));
}

#[test]
fn a_directed_evolution_says_that_it_builds_its_primogenitor_and_then_that_it_is_built() {
    let said = built_by(Double::default()).said;
    assert!(
        said.contains("cladenc: building the primogenitor\ncladenc: the primogenitor is built\n")
    );
}

#[test]
fn a_directed_evolution_whose_primogenitor_did_not_build_builds_it_when_resumed() {
    let resumed = given::resumed(not_built(), Double::default());
    assert_eq!(resumed.journal().lines().nth(1), Some(BUILT));
}

#[test]
fn a_directed_evolution_whose_primogenitor_was_built_on_resuming_goes_on_to_its_end() {
    let resumed = given::resumed(not_built(), Double::default());
    assert!(resumed.result.is_ok());
}

#[test]
fn a_resumed_directed_evolution_whose_primogenitor_was_built_does_not_build_it_again() {
    assert_eq!(
        builds_of(&stopped_at_a_trial_and_resumed(), "harnesses/0"),
        0
    );
}

#[test]
fn a_resumed_directed_evolution_whose_primogenitor_was_built_goes_on_to_its_end() {
    assert!(stopped_at_a_trial_and_resumed().result.is_ok());
}

#[test]
fn a_directed_evolution_stopped_while_an_environment_is_built_does_not_build_its_primogenitor() {
    let stopped = built_by(Double::default().answering(&["build"], &[Answer::Stopped]));
    assert_eq!(builds_of(&stopped, "harnesses/0"), 0);
}
