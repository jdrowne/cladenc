use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;

const REFUSED: Answer = Answer::Refused("docker is not there");

fn with_a_key(directory: &Memory, settings: &mut Settings) {
    settings.model_key_file = Some(test_inputs::model_key_file_in(
        directory,
        directory.path(),
        "a-made-up-key\n",
    ));
}

fn failed() -> Evolved {
    let docker = Double::default().answering(&["--detach"], &[REFUSED]);
    given::started(docker, given::unchanged)
}

fn resumed_by(docker: Double) -> Evolved {
    given::resumed(failed(), docker)
}

#[test]
fn a_directed_evolution_that_failed_can_be_resumed_to_its_end() {
    let best = resumed_by(Double::default()).result.ok();
    assert_eq!(best.map(|found| found.best_harness), Some(0));
}

#[test]
fn a_directed_evolution_that_is_resumed_tells_how_far_it_had_come() {
    let said = resumed_by(Double::default()).said;
    assert!(said.contains("continuing the directed evolution in"));
}

#[test]
fn a_trial_that_had_not_ended_is_started_again() {
    let said = resumed_by(Double::default()).said;
    assert!(said.contains("started again the trial of harness 0 on task benchmark/t1"));
}

#[test]
fn the_containers_that_were_left_running_are_removed() {
    let docker = Double::default().answering(&["ps"], &[Answer::Printed("a\nb\n")]);
    assert_eq!(resumed_by(docker).count_of("docker rm --force a b"), 1);
}

#[test]
fn the_containers_that_were_removed_are_told() {
    let docker = Double::default().answering(&["ps"], &[Answer::Printed("a\nb\n")]);
    let said = resumed_by(docker).said;
    assert!(said.contains("removed 2 containers that the directed evolution left running"));
}

#[test]
fn nothing_is_told_of_containers_where_none_was_left_running() {
    let said = resumed_by(Double::default()).said;
    assert!(!said.contains("removed"));
}

#[test]
fn a_directed_evolution_whose_containers_cannot_be_listed_is_not_resumed() {
    let docker = Double::default().answering(&["ps"], &[REFUSED]);
    assert!(matches!(
        resumed_by(docker).result,
        Err(DirectedEvolutionError::Left(_))
    ));
}

#[test]
fn a_directed_evolution_whose_file_of_the_key_is_gone_is_not_resumed() {
    let docker = Double::default().answering(&["--detach"], &[REFUSED]);
    let evolved = given::started(docker, with_a_key);
    evolved
        .files()
        .remove_file(evolved.files().path().join("model-key"))
        .unwrap();
    let evolved = given::resumed(evolved, Double::default());
    assert!(matches!(
        evolved.result,
        Err(DirectedEvolutionError::Key(_))
    ));
}

#[test]
fn a_directed_evolution_whose_environments_cannot_be_built_again_is_not_resumed() {
    let docker = Double::default().answering(&["build"], &[Answer::Exited(1)]);
    assert!(matches!(
        resumed_by(docker).result,
        Err(DirectedEvolutionError::Environment { .. })
    ));
}

#[test]
fn a_resume_first_asks_docker_for_the_containers_that_carry_the_label_of_its_directed_evolution() {
    let commands = resumed_by(Double::default()).commands();
    assert_eq!(
        commands.first().map(String::as_str),
        Some("docker ps --all --quiet --filter label=cladenc.directed-evolution=OUTPUTS")
    );
}

#[test]
fn the_journal_of_a_directed_evolution_holds_the_file_of_the_key_and_not_the_key() {
    let journal = given::started(Double::default(), with_a_key).journal();
    assert_eq!(
        (
            journal.contains("a-made-up-key"),
            journal.contains("/model-key\"")
        ),
        (false, true)
    );
}

#[test]
fn a_directed_evolution_says_its_key_nowhere() {
    let said = given::started(Double::default(), with_a_key).said;
    assert!(!said.contains("a-made-up-key"));
}

#[test]
fn a_directed_evolution_that_is_not_resumed_for_its_key_journals_nothing_more() {
    let docker = Double::default().answering(&["--detach"], &[REFUSED]);
    let evolved = given::started(docker, with_a_key);
    let journal = evolved.journal();
    evolved
        .files()
        .remove_file(evolved.files().path().join("model-key"))
        .unwrap();
    assert_eq!(
        given::resumed(evolved, Double::default()).journal(),
        journal
    );
}
