use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use crate::artificial_selection::Action;
use crate::benchmark::Benchmark;
use crate::containers::{Attempted, Creation, HarnessEnding};
use crate::directed_evolution::Said;
use crate::directed_evolution::action::{Ended, Told};
use crate::directed_evolution::progress::Progress;
use crate::genealogy::{AttemptResult, CreationOutcome, Experiment, Genealogy};
use crate::journal::{self, Journal, Record, Settings, Started};
use crate::outputs_directory::OutputsDirectory;

const FIRST: &str = "benchmark/t1";
const SECOND: &str = "benchmark/t2";

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/abs/harness"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 2,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn seeded(seed: u64) -> Settings {
    Settings { seed, ..settings() }
}

fn with_a_build_command() -> Settings {
    Settings {
        harness_build_command: Some("make".to_owned()),
        ..settings()
    }
}

fn unwritable() -> Record {
    Record::Run {
        format: journal::FORMAT,
        settings: Box::new(Settings {
            primogenitor: PathBuf::from(OsStr::from_bytes(&[b'p', 0xff])),
            ..settings()
        }),
    }
}

fn seen<T>(
    settings: &Settings,
    steps: impl FnOnce(Progress<'_>) -> T,
) -> (T, Vec<String>, Vec<Record>) {
    let outputs_directory = tempfile::tempdir().unwrap();
    let journal = Journal::open(outputs_directory.path()).unwrap();
    let mut written: Vec<u8> = Vec::new();
    let returned = steps(Progress::new(settings, journal, &mut written));
    let said = String::from_utf8(written)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let journaled = fs::read_to_string(journal::path(outputs_directory.path()))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (returned, said, journaled)
}

fn returned<T>(settings: &Settings, steps: impl FnOnce(Progress<'_>) -> T) -> T {
    seen(settings, steps).0
}

fn said<T>(settings: &Settings, steps: impl FnOnce(Progress<'_>) -> T) -> Vec<String> {
    seen(settings, steps).1
}

fn journaled<T>(settings: &Settings, steps: impl FnOnce(Progress<'_>) -> T) -> Vec<Record> {
    seen(settings, steps).2
}

fn attempt(harness: u64, task: &str) -> Started {
    Started::Attempt {
        harness,
        task: task.to_owned(),
    }
}

fn end_of(action: Started) -> Told {
    let creation = Creation {
        outcome: CreationOutcome::Created,
        seconds: 2.5,
    };
    Told::Ended(match action {
        Started::Procreation { child, progenitor } => Ended::Procreated {
            child,
            progenitor,
            creation,
        },
        Started::Baseline {
            harness,
            child,
            progenitor,
        } => Ended::Established {
            harness,
            experiment: Experiment { child, progenitor },
            creation,
        },
        Started::Attempt { harness, task } => Ended::Attempted {
            harness,
            task,
            attempted: Attempted {
                result: AttemptResult::Completed,
                seconds: 1.5,
                ending: HarnessEnding::Ended,
            },
        },
    })
}

fn start(progress: &mut Progress<'_>) -> Started {
    let (action, again) = progress.take().unwrap();
    progress.set_running(&action, again);
    action
}

fn carry_out(progress: &mut Progress<'_>, actions: usize) {
    for _ in 0..actions {
        let action = start(progress);
        progress.hear(end_of(action), &OutputsDirectory::at(Path::new("/outputs")));
    }
}

fn left_attempting<'a>(progress: Progress<'a>, tasks: &[&str]) -> Progress<'a> {
    let mut genealogy = settings().genealogy();
    for task in tasks {
        genealogy.start_attempt(0, task).unwrap();
    }
    let unfinished = tasks.iter().map(|task| attempt(0, task)).collect();
    progress.resumed(genealogy, unfinished, tasks.len() as u64)
}

fn awaiting_a_baseline(progress: Progress<'_>, unfinished: Vec<Started>) -> Progress<'_> {
    let mut genealogy = settings().genealogy();
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Created)
        .unwrap();
    genealogy.start_procreation(child).unwrap();
    progress.resumed(genealogy, unfinished, 2)
}

fn benchmarked_in_full(genealogy: &mut Genealogy, harness: u64) {
    for task in [FIRST, SECOND] {
        genealogy.start_attempt(harness, task).unwrap();
        genealogy
            .end_attempt(harness, task, AttemptResult::Completed)
            .unwrap();
    }
}

fn two_harnesses_benchmarked_in_full() -> Genealogy {
    let mut genealogy = settings().genealogy();
    benchmarked_in_full(&mut genealogy, 0);
    let child = genealogy.start_procreation(0).unwrap();
    genealogy
        .end_procreation(child, CreationOutcome::Created)
        .unwrap();
    benchmarked_in_full(&mut genealogy, child);
    genealogy
}

fn progenitor_chosen(seed: u64) -> Option<u64> {
    let chosen = seeded(seed)
        .artificial_selection()
        .next_action(&two_harnesses_benchmarked_in_full(), 5);
    match chosen {
        Some(Action::Procreation { progenitor }) => Some(progenitor),
        _ => None,
    }
}

fn progenitor_taken(taken: Option<(Started, bool)>) -> Option<u64> {
    match taken {
        Some((Started::Procreation { progenitor, .. }, false)) => Some(progenitor),
        _ => None,
    }
}

#[test]
fn the_first_action_taken_is_an_attempt_of_the_primogenitor_on_the_first_task() {
    assert_eq!(
        returned(&settings(), |mut progress| progress.take()),
        Some((attempt(0, FIRST), false))
    );
}

#[test]
fn the_second_action_taken_is_an_attempt_of_the_primogenitor_on_the_next_task() {
    let taken = returned(&settings(), |mut progress| {
        progress.take();
        progress.take()
    });
    assert_eq!(taken, Some((attempt(0, SECOND), false)));
}

#[test]
fn no_action_is_taken_while_the_primogenitor_attempts_every_task() {
    let taken = returned(&settings(), |mut progress| {
        progress.take();
        progress.take();
        progress.take()
    });
    assert_eq!(taken, None);
}

#[test]
fn an_attempt_that_is_taken_is_journaled_as_started() {
    assert_eq!(
        journaled(&settings(), |mut progress| progress.take()),
        [Record::AttemptStarted {
            harness: 0,
            task: FIRST.to_owned()
        }]
    );
}

#[test]
fn taking_an_action_says_nothing() {
    assert!(said(&settings(), |mut progress| progress.take()).is_empty());
}

#[test]
fn once_the_primogenitor_is_benchmarked_in_full_its_procreation_is_taken() {
    let taken = returned(&settings(), |mut progress| {
        carry_out(&mut progress, 2);
        progress.take()
    });
    assert_eq!(
        taken,
        Some((
            Started::Procreation {
                child: 1,
                progenitor: 0
            },
            false
        ))
    );
}

#[test]
fn a_procreation_that_is_taken_is_journaled_as_started() {
    let journaled = journaled(&settings(), |mut progress| {
        carry_out(&mut progress, 2);
        progress.take()
    });
    assert_eq!(
        journaled.last(),
        Some(&Record::ProcreationStarted {
            child: 1,
            progenitor: 0
        })
    );
}

#[test]
fn no_action_is_taken_while_a_child_of_the_primogenitor_is_being_created() {
    let taken = returned(&settings(), |mut progress| {
        carry_out(&mut progress, 2);
        progress.take();
        progress.take()
    });
    assert_eq!(taken, None);
}

#[test]
fn a_child_that_was_created_is_given_its_first_task_next() {
    let taken = returned(&settings(), |mut progress| {
        carry_out(&mut progress, 3);
        progress.take()
    });
    assert_eq!(taken, Some((attempt(1, FIRST), false)));
}

#[test]
fn no_action_is_taken_once_the_most_procreations_are_started_and_every_harness_is_benchmarked() {
    let only_one_procreation = Settings {
        max_procreations: 1,
        ..settings()
    };
    let taken = returned(&only_one_procreation, |mut progress| {
        carry_out(&mut progress, 5);
        progress.take()
    });
    assert_eq!(taken, None);
}

#[test]
fn a_progenitor_is_chosen_with_the_count_of_the_actions_started_before_its_procreation() {
    let taken: Vec<Option<u64>> = (0..32)
        .map(|seed| {
            returned(&seeded(seed), |mut progress| {
                carry_out(&mut progress, 5);
                progenitor_taken(progress.take())
            })
        })
        .collect();
    let chosen: Vec<Option<u64>> = (0..32).map(progenitor_chosen).collect();
    assert_eq!(taken, chosen);
}

#[test]
fn a_resumed_directed_evolution_chooses_the_progenitor_an_uninterrupted_one_chooses() {
    let taken: Vec<Option<u64>> = (0..32)
        .map(|seed| {
            returned(&seeded(seed), |progress| {
                let genealogy = two_harnesses_benchmarked_in_full();
                progenitor_taken(progress.resumed(genealogy, Vec::new(), 5).take())
            })
        })
        .collect();
    let chosen: Vec<Option<u64>> = (0..32).map(progenitor_chosen).collect();
    assert_eq!(taken, chosen);
}

#[test]
fn an_action_that_a_resume_found_unfinished_is_taken_first_to_be_started_again() {
    let taken = returned(&settings(), |progress| {
        left_attempting(progress, &[FIRST]).take()
    });
    assert_eq!(taken, Some((attempt(0, FIRST), true)));
}

#[test]
fn an_action_taken_to_be_started_again_is_not_journaled_a_second_time() {
    let journaled = journaled(&settings(), |progress| {
        left_attempting(progress, &[FIRST]).take()
    });
    assert!(journaled.is_empty());
}

#[test]
fn the_unfinished_actions_of_a_resume_are_taken_in_the_order_they_were_started() {
    let taken = returned(&settings(), |progress| {
        let mut progress = left_attempting(progress, &[FIRST, SECOND]);
        progress.take();
        progress.take()
    });
    assert_eq!(taken, Some((attempt(0, SECOND), true)));
}

#[test]
fn after_the_unfinished_actions_of_a_resume_the_action_chosen_next_is_taken() {
    let taken = returned(&settings(), |progress| {
        let mut progress = left_attempting(progress, &[FIRST]);
        progress.take();
        progress.take()
    });
    assert_eq!(taken, Some((attempt(0, SECOND), false)));
}

#[test]
fn the_baseline_that_an_experiment_waits_for_is_taken_before_the_action_chosen_next() {
    let taken = returned(&settings(), |progress| {
        awaiting_a_baseline(progress, Vec::new()).take()
    });
    assert_eq!(
        taken,
        Some((
            Started::Baseline {
                harness: 3,
                child: 2,
                progenitor: 1
            },
            false
        ))
    );
}

#[test]
fn a_baseline_that_is_taken_is_journaled_as_started() {
    let journaled = journaled(&settings(), |progress| {
        awaiting_a_baseline(progress, Vec::new()).take()
    });
    assert_eq!(
        journaled,
        [Record::BaselineStarted {
            harness: 3,
            child: 2,
            progenitor: 1
        }]
    );
}

#[test]
fn an_unfinished_action_is_taken_before_the_baseline_that_an_experiment_waits_for() {
    let procreation = Started::Procreation {
        child: 2,
        progenitor: 1,
    };
    let taken = returned(&settings(), |progress| {
        awaiting_a_baseline(progress, vec![procreation.clone()]).take()
    });
    assert_eq!(taken, Some((procreation, true)));
}

#[test]
fn once_the_baseline_of_an_experiment_is_taken_the_action_chosen_next_is_taken() {
    let taken = returned(&settings(), |progress| {
        let mut progress = awaiting_a_baseline(progress, Vec::new());
        progress.take();
        progress.take()
    });
    assert_eq!(taken, Some((attempt(0, FIRST), false)));
}

#[test]
fn no_container_is_running_before_an_action_is_set_running() {
    assert_eq!(returned(&settings(), |progress| progress.running()), 0);
}

#[test]
fn an_action_that_is_taken_is_not_yet_running() {
    let running = returned(&settings(), |mut progress| {
        progress.take();
        progress.running()
    });
    assert_eq!(running, 0);
}

#[test]
fn an_action_set_running_is_one_more_container_running() {
    let running = returned(&settings(), |mut progress| {
        start(&mut progress);
        start(&mut progress);
        progress.running()
    });
    assert_eq!(running, 2);
}

#[test]
fn an_action_set_running_is_to_be_carried_out() {
    let carried_out = returned(&settings(), |mut progress| {
        let (action, again) = progress.take().unwrap();
        progress.set_running(&action, again)
    });
    assert!(carried_out);
}

#[test]
fn an_attempt_set_running_is_said_to_have_started() {
    assert_eq!(
        said(&settings(), |mut progress| start(&mut progress)),
        ["cladenc: started the attempt of harness 0 on task benchmark/t1"]
    );
}

#[test]
fn a_procreation_set_running_is_said_with_its_number_and_the_most_procreations() {
    let said = said(&settings(), |mut progress| {
        carry_out(&mut progress, 2);
        start(&mut progress)
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some("cladenc: started the procreation of harness 1 by harness 0, procreation 1 of 2")
    );
}

#[test]
fn a_baseline_set_running_is_said_to_have_started() {
    let said = said(&settings(), |progress| {
        start(&mut awaiting_a_baseline(progress, Vec::new()))
    });
    assert_eq!(
        said,
        [
            "cladenc: started the creation of harness 3, the singleton control group harness, by \
             the primogenitor from the source code of harness 1"
        ]
    );
}

#[test]
fn an_unfinished_action_set_running_is_said_to_have_started_again() {
    let said = said(&settings(), |progress| {
        start(&mut left_attempting(progress, &[FIRST]))
    });
    assert_eq!(
        said,
        ["cladenc: started again the attempt of harness 0 on task benchmark/t1"]
    );
}

#[test]
fn an_action_whose_start_could_not_be_journaled_is_not_to_be_carried_out() {
    let carried_out = returned(&settings(), |mut progress| {
        let (action, again) = progress.take().unwrap();
        progress.record(&unwritable());
        progress.set_running(&action, again)
    });
    assert!(!carried_out);
}

#[test]
fn an_action_whose_start_could_not_be_journaled_is_no_container_running() {
    let running = returned(&settings(), |mut progress| {
        let (action, again) = progress.take().unwrap();
        progress.record(&unwritable());
        progress.set_running(&action, again);
        progress.running()
    });
    assert_eq!(running, 0);
}

#[test]
fn an_action_whose_start_could_not_be_journaled_is_not_said_to_have_started() {
    let said = said(&settings(), |mut progress| {
        let (action, again) = progress.take().unwrap();
        progress.record(&unwritable());
        progress.set_running(&action, again)
    });
    assert!(said.is_empty());
}

#[test]
fn what_a_directed_evolution_is_given_to_say_is_written_after_the_name_of_cladenc() {
    assert_eq!(
        said(&settings(), |mut progress| progress.say(&Said::Building)),
        ["cladenc: building the primogenitor"]
    );
}

#[test]
fn a_primogenitor_that_has_a_build_command_awaits_its_build() {
    let awaits = returned(&with_a_build_command(), |progress| {
        progress.awaits_build(&AtomicBool::new(false))
    });
    assert!(awaits);
}

#[test]
fn a_primogenitor_that_has_no_build_command_awaits_no_build() {
    let awaits = returned(&settings(), |progress| {
        progress.awaits_build(&AtomicBool::new(false))
    });
    assert!(!awaits);
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_awaits_no_build() {
    let awaits = returned(&with_a_build_command(), |progress| {
        progress.awaits_build(&AtomicBool::new(true))
    });
    assert!(!awaits);
}

#[test]
fn a_primogenitor_that_is_built_awaits_no_build() {
    let awaits = returned(&with_a_build_command(), |mut progress| {
        progress.built();
        progress.awaits_build(&AtomicBool::new(false))
    });
    assert!(!awaits);
}

#[test]
fn the_build_of_the_primogenitor_is_journaled() {
    assert_eq!(
        journaled(&with_a_build_command(), |mut progress| progress.built()),
        [Record::PrimogenitorBuilt]
    );
}

#[test]
fn the_primogenitor_is_said_to_be_built() {
    assert_eq!(
        said(&with_a_build_command(), |mut progress| progress.built()),
        ["cladenc: the primogenitor is built"]
    );
}

#[test]
fn no_action_is_taken_before_the_primogenitor_is_built() {
    assert_eq!(
        returned(&with_a_build_command(), |mut progress| progress.take()),
        None
    );
}

#[test]
fn once_the_primogenitor_is_built_its_attempt_on_the_first_task_is_taken() {
    let taken = returned(&with_a_build_command(), |mut progress| {
        progress.built();
        progress.take()
    });
    assert_eq!(taken, Some((attempt(0, FIRST), false)));
}
