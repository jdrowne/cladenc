use std::sync::atomic::AtomicBool;

use crate::directed_evolution::Found;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;
use crate::sandboxes::tests::double::Double;

fn with_a_key(directory: &Memory, settings: &mut Settings) {
    settings.model_key_file = Some(test_inputs::model_key_file_in(
        directory,
        directory.path(),
        "a-made-up-key\n",
    ));
}

fn ended() -> Evolved {
    given::started(Double::default().changing(), given::unchanged)
}

fn found_by(evolved: &Evolved) -> Option<Found> {
    evolved.result.as_ref().ok().cloned()
}

#[test]
fn resuming_a_directed_evolution_that_ended_returns_what_the_directed_evolution_returned() {
    let ended = ended();
    let found = found_by(&ended);
    let resumed = given::resumed(ended, Double::default());
    assert_eq!((found.is_some(), found_by(&resumed)), (true, found));
}

#[test]
fn resuming_a_directed_evolution_that_ended_leaves_its_journal_unchanged() {
    let ended = ended();
    let journal = ended.journal();
    assert_eq!(given::resumed(ended, Double::default()).journal(), journal);
}

#[test]
fn resuming_a_directed_evolution_that_ended_says_that_it_had_finished_and_nothing_else() {
    let ended = ended();
    let had_finished = format!(
        "cladenc: the directed evolution had finished: its best harness is harness 0, kept in {}\n",
        ended.outputs().join("harnesses/0").display()
    );
    assert_eq!(given::resumed(ended, Double::default()).said, had_finished);
}

#[test]
fn resuming_a_directed_evolution_that_ended_returns_it_even_when_asked_to_stop() {
    let ended = ended();
    let found = found_by(&ended);
    let resumed = given::resumed_until(ended, Double::default(), &AtomicBool::new(true));
    assert_eq!((found.is_some(), found_by(&resumed)), (true, found));
}

#[test]
fn resuming_a_directed_evolution_that_ended_asks_nothing_of_docker() {
    let resumed = given::resumed(ended(), Double::default());
    assert_eq!(resumed.commands(), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_that_ended_is_resumed_without_its_key() {
    let ended = given::started(Double::default(), with_a_key);
    let found = found_by(&ended);
    ended
        .files()
        .remove_file(ended.files().path().join("model-key"))
        .unwrap();
    let resumed = given::resumed(ended, Double::default());
    assert_eq!((found.is_some(), found_by(&resumed)), (true, found));
}
