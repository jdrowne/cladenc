use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use crate::dataset::Dataset;
use crate::directed_evolution::action::{Ended, Told};
use crate::directed_evolution::progress::Progress;
use crate::directed_evolution::{DirectedEvolutionError, Found};
use crate::file_system::tests::double::Memory;
use crate::genealogy::{CreationOutcome, Experiment, TrialResult};
use crate::journal::{self, Journal, Record, Settings, Started};
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::{Checked, Creation, HarnessEnding, Retry, SandboxError, Trialled};

const FIRST: &str = "dataset/t1";
const SECOND: &str = "dataset/t2";
const STARTED_THE_FIRST_TRIAL: &str = "cladenc: started the trial of harness 0 on task dataset/t1";
const THE_FIRST_TRIAL_FAILED: &str =
    "cladenc: the trial of harness 0 on task dataset/t1 exited with status 3";
const ENDED_WITH_THE_FIRST_TRIAL: &str = "the directed evolution stopped before its end: the \
     trial of harness 0 on task dataset/t1 exited with status 3";

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/abs/harness"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        datasets: vec![Dataset {
            directory: PathBuf::from("/abs/dataset"),
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_diversifications: 2,
        max_concurrent_sandboxes: 8,
        diversification_timeout: 3600,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn selecting_by_the_second_task() -> Settings {
    Settings {
        holdout_tasks: vec![SECOND.to_owned()],
        ..settings()
    }
}

fn run_record(primogenitor: &OsStr) -> Record {
    Record::Run {
        format: journal::FORMAT,
        settings: Box::new(Settings {
            primogenitor: PathBuf::from(primogenitor),
            ..settings()
        }),
    }
}

fn unwritable() -> Record {
    run_record(OsStr::from_bytes(&[b'p', 0xff]))
}

fn seen<T>(
    settings: &Settings,
    steps: impl FnOnce(Progress<'_>) -> T,
) -> (T, Vec<String>, Vec<Record>) {
    let directory = Memory::default();
    let journal = Journal::open(&directory, directory.path()).unwrap();
    let mut written: Vec<u8> = Vec::new();
    let returned = steps(Progress::new(settings, journal, &mut written));
    let said = String::from_utf8(written)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let journaled = directory
        .read_to_string(journal::path(directory.path()))
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

fn outputs_directory() -> OutputsDirectory {
    OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"))
}

fn trial(harness: u64, task: &str) -> Started {
    Started::Trial {
        harness,
        task: task.to_owned(),
    }
}

fn first_diversification() -> Started {
    Started::Diversification {
        child: 1,
        progenitor: 0,
    }
}

fn first_trial_started() -> Record {
    Record::TrialStarted {
        harness: 0,
        task: FIRST.to_owned(),
    }
}

fn end_of(action: Started, well: bool) -> Told {
    let creation = Creation {
        outcome: if well {
            CreationOutcome::Created
        } else {
            CreationOutcome::Failed
        },
        seconds: 2.5,
    };
    let result = if well {
        TrialResult::Pass
    } else {
        TrialResult::Fail
    };
    Told::Ended(match action {
        Started::Diversification { child, progenitor } => Ended::Diversified {
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
        Started::Trial { harness, task } => Ended::Trialled {
            harness,
            task,
            trialled: Trialled {
                result,
                seconds: 1.5,
                ending: HarnessEnding::Ended,
            },
        },
    })
}

fn failed(action: Started, error: SandboxError) -> Told {
    Told::Ended(Ended::Failed { action, error })
}

fn retrying() -> Told {
    Told::Retrying {
        harness: 0,
        task: FIRST.to_owned(),
        retry: Retry {
            checked: Checked::Task,
            number: 1,
            most: 3,
            wait: Duration::from_secs(1),
        },
    }
}

fn start(progress: &mut Progress<'_>) -> Started {
    let (action, again) = progress.take().unwrap();
    progress.set_running(&action, again);
    action
}

fn carry_out(progress: &mut Progress<'_>, actions: usize, badly: &[Started]) {
    for _ in 0..actions {
        let action = start(progress);
        let well = !badly.contains(&action);
        progress.hear(end_of(action, well), &outputs_directory());
    }
}

fn stop_the_first_trial(progress: &mut Progress<'_>) {
    start(progress);
    progress.hear(Told::Ended(Ended::Stopped), &outputs_directory());
}

fn fail_the_first_trial(progress: &mut Progress<'_>) {
    let action = start(progress);
    progress.hear(failed(action, SandboxError::Exit(3)), &outputs_directory());
}

fn fail_both_trials(progress: &mut Progress<'_>) {
    let first = start(progress);
    let second = start(progress);
    progress.hear(failed(first, SandboxError::Exit(3)), &outputs_directory());
    progress.hear(
        failed(second, SandboxError::TimeLimit),
        &outputs_directory(),
    );
}

fn awaiting_a_baseline(progress: Progress<'_>) -> Progress<'_> {
    let mut genealogy = settings().genealogy();
    let child = genealogy.start_diversification(0).unwrap();
    genealogy
        .end_diversification(child, CreationOutcome::Created)
        .unwrap();
    genealogy.start_diversification(child).unwrap();
    progress.resumed(genealogy, Vec::new(), 2)
}

fn finish(progress: Progress<'_>) -> Result<Found, DirectedEvolutionError> {
    progress.finish(&outputs_directory(), &AtomicBool::new(false))
}

fn error_of(finished: Result<Found, DirectedEvolutionError>) -> Option<String> {
    finished.err().map(|error| error.to_string())
}

#[test]
fn a_trial_that_ended_is_journaled_with_its_result_and_how_long_it_took() {
    let journaled = journaled(&settings(), |mut progress| {
        carry_out(&mut progress, 1, &[]);
    });
    assert_eq!(
        journaled.last(),
        Some(&Record::TrialFinished {
            harness: 0,
            task: FIRST.to_owned(),
            result: TrialResult::Pass,
            seconds: 1.5
        })
    );
}

#[test]
fn a_trial_that_failed_is_journaled_as_such() {
    let journaled = journaled(&settings(), |mut progress| {
        carry_out(&mut progress, 1, &[trial(0, FIRST)]);
    });
    assert_eq!(
        journaled.last(),
        Some(&Record::TrialFinished {
            harness: 0,
            task: FIRST.to_owned(),
            result: TrialResult::Fail,
            seconds: 1.5
        })
    );
}

#[test]
fn a_trial_that_passed_is_said_with_where_what_it_left_is_kept() {
    let said = said(&settings(), |mut progress| {
        carry_out(&mut progress, 1, &[]);
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some(
            "cladenc: the trial of harness 0 on task dataset/t1 passed: what the trial left \
             is kept in /outputs/trials/0/dataset/t1"
        )
    );
}

#[test]
fn a_trial_that_failed_is_said_to_have_failed() {
    let said = said(&settings(), |mut progress| {
        carry_out(&mut progress, 1, &[trial(0, FIRST)]);
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some(
            "cladenc: the trial of harness 0 on task dataset/t1 failed, as the verifier \
             wrote a verdict that is not 1: what the trial left is kept in \
             /outputs/trials/0/dataset/t1"
        )
    );
}

#[test]
fn a_trial_that_ended_is_one_sandbox_fewer_running() {
    let running = returned(&settings(), |mut progress| {
        start(&mut progress);
        carry_out(&mut progress, 1, &[]);
        progress.running()
    });
    assert_eq!(running, 1);
}

#[test]
fn a_diversification_that_ended_is_journaled_with_its_outcome_and_how_long_it_took() {
    let journaled = journaled(&settings(), |mut progress| {
        carry_out(&mut progress, 3, &[]);
    });
    assert_eq!(
        journaled.last(),
        Some(&Record::DiversificationFinished {
            child: 1,
            outcome: CreationOutcome::Created,
            seconds: 2.5
        })
    );
}

#[test]
fn a_diversification_that_failed_is_journaled_as_failed() {
    let journaled = journaled(&settings(), |mut progress| {
        carry_out(&mut progress, 3, &[first_diversification()]);
    });
    assert_eq!(
        journaled.last(),
        Some(&Record::DiversificationFinished {
            child: 1,
            outcome: CreationOutcome::Failed,
            seconds: 2.5
        })
    );
}

#[test]
fn a_diversification_that_created_a_harness_is_said_with_where_the_harness_is_kept() {
    let said = said(&settings(), |mut progress| {
        carry_out(&mut progress, 3, &[]);
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some(
            "cladenc: the diversification of harness 1 by harness 0 created harness 1, kept in \
             /outputs/harnesses/1"
        )
    );
}

#[test]
fn a_diversification_that_failed_is_said_with_where_what_the_harness_printed_is_kept() {
    let said = said(&settings(), |mut progress| {
        carry_out(&mut progress, 3, &[first_diversification()]);
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some(
            "cladenc: the diversification of harness 1 by harness 0 failed: what the harness \
             printed is kept in /outputs/diversifications/1"
        )
    );
}

#[test]
fn a_child_that_was_created_is_given_its_first_task() {
    let taken = returned(&settings(), |mut progress| {
        carry_out(&mut progress, 3, &[]);
        progress.take()
    });
    assert_eq!(taken, Some((trial(1, FIRST), false)));
}

#[test]
fn a_progenitor_whose_diversification_failed_is_cloned_again() {
    let taken = returned(&settings(), |mut progress| {
        carry_out(&mut progress, 3, &[first_diversification()]);
        progress.take()
    });
    assert_eq!(
        taken,
        Some((
            Started::Diversification {
                child: 2,
                progenitor: 0
            },
            false
        ))
    );
}

#[test]
fn a_baseline_that_ended_is_journaled_with_its_outcome_and_how_long_it_took() {
    let journaled = journaled(&settings(), |progress| {
        carry_out(&mut awaiting_a_baseline(progress), 1, &[]);
    });
    assert_eq!(
        journaled.last(),
        Some(&Record::BaselineFinished {
            harness: 3,
            outcome: CreationOutcome::Created,
            seconds: 2.5
        })
    );
}

#[test]
fn a_baseline_that_created_the_singleton_control_group_harness_is_said_with_where_it_is_kept() {
    let said = said(&settings(), |progress| {
        carry_out(&mut awaiting_a_baseline(progress), 1, &[]);
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some(
            "cladenc: the creation of harness 3, the singleton control group harness, by the \
             primogenitor from a second clone of harness 1 created harness 3, kept in \
             /outputs/harnesses/3"
        )
    );
}

#[test]
fn an_action_that_was_stopped_is_one_sandbox_fewer_running() {
    let running = returned(&settings(), |mut progress| {
        stop_the_first_trial(&mut progress);
        progress.running()
    });
    assert_eq!(running, 0);
}

#[test]
fn an_action_that_was_stopped_is_not_journaled_as_finished() {
    let journaled = journaled(&settings(), |mut progress| {
        stop_the_first_trial(&mut progress);
    });
    assert_eq!(journaled, [first_trial_started()]);
}

#[test]
fn nothing_is_said_of_an_action_that_was_stopped() {
    let said = said(&settings(), |mut progress| {
        stop_the_first_trial(&mut progress);
    });
    assert_eq!(said, [STARTED_THE_FIRST_TRIAL]);
}

#[test]
fn a_trial_that_is_tried_again_in_a_new_sandbox_is_said() {
    let said = said(&settings(), |mut progress| {
        progress.hear(retrying(), &outputs_directory());
    });
    assert_eq!(
        said,
        [
            "cladenc: the trial of harness 0 on task dataset/t1 is not carried out yet: the \
             health check of the task did not pass, and cladenc tries again in a new sandbox \
             after 1 second, retry 1 of 3"
        ]
    );
}

#[test]
fn a_trial_that_is_tried_again_in_a_new_sandbox_is_not_journaled() {
    let journaled = journaled(&settings(), |mut progress| {
        progress.hear(retrying(), &outputs_directory());
    });
    assert!(journaled.is_empty());
}

#[test]
fn a_trial_that_is_tried_again_in_a_new_sandbox_is_still_a_sandbox_running() {
    let running = returned(&settings(), |mut progress| {
        start(&mut progress);
        progress.hear(retrying(), &outputs_directory());
        progress.running()
    });
    assert_eq!(running, 1);
}

#[test]
fn an_action_that_failed_is_said_with_what_went_wrong() {
    let said = said(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
    });
    assert_eq!(said, [STARTED_THE_FIRST_TRIAL, THE_FIRST_TRIAL_FAILED]);
}

#[test]
fn an_action_that_failed_is_one_sandbox_fewer_running() {
    let running = returned(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        progress.running()
    });
    assert_eq!(running, 0);
}

#[test]
fn an_action_that_failed_is_not_journaled_as_finished() {
    let journaled = journaled(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
    });
    assert_eq!(journaled, [first_trial_started()]);
}

#[test]
fn an_action_that_failed_halts_the_directed_evolution() {
    let halted = returned(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        progress.halted(&AtomicBool::new(false))
    });
    assert!(halted);
}

#[test]
fn a_directed_evolution_whose_actions_end_without_failing_is_not_halted() {
    let halted = returned(&settings(), |mut progress| {
        carry_out(
            &mut progress,
            3,
            &[trial(0, FIRST), first_diversification()],
        );
        progress.halted(&AtomicBool::new(false))
    });
    assert!(!halted);
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_is_halted() {
    let halted = returned(&settings(), |progress| {
        progress.halted(&AtomicBool::new(true))
    });
    assert!(halted);
}

#[test]
fn an_action_is_not_to_be_carried_out_once_another_failed() {
    let carried_out = returned(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        let (action, again) = progress.take().unwrap();
        progress.set_running(&action, again)
    });
    assert!(!carried_out);
}

#[test]
fn a_directed_evolution_ends_with_the_action_that_failed() {
    let error = returned(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        error_of(finish(progress))
    });
    assert_eq!(error.as_deref(), Some(ENDED_WITH_THE_FIRST_TRIAL));
}

#[test]
fn a_directed_evolution_ends_with_the_first_of_the_actions_that_failed() {
    let error = returned(&settings(), |mut progress| {
        fail_both_trials(&mut progress);
        error_of(finish(progress))
    });
    assert_eq!(error.as_deref(), Some(ENDED_WITH_THE_FIRST_TRIAL));
}

#[test]
fn an_action_that_failed_after_another_is_said_with_what_went_wrong() {
    let said = said(&settings(), |mut progress| {
        fail_both_trials(&mut progress);
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some("cladenc: the trial of harness 0 on task dataset/t2 ran past its time limit")
    );
}

#[test]
fn a_directed_evolution_that_ends_with_a_failure_does_not_journal_that_it_finished() {
    let journaled = journaled(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        finish(progress)
    });
    assert_eq!(journaled, [first_trial_started()]);
}

#[test]
fn a_directed_evolution_that_ends_with_a_failure_is_not_said_to_have_finished() {
    let said = said(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        finish(progress)
    });
    assert_eq!(said, [STARTED_THE_FIRST_TRIAL, THE_FIRST_TRIAL_FAILED]);
}

#[test]
fn a_directed_evolution_that_is_asked_to_stop_says_so_with_the_sandboxes_running() {
    let said = said(&settings(), |mut progress| {
        start(&mut progress);
        progress.stopping();
    });
    assert_eq!(
        said,
        [
            STARTED_THE_FIRST_TRIAL,
            "cladenc: asked to stop, with 1 sandbox running"
        ]
    );
}

#[test]
fn a_directed_evolution_says_once_that_it_is_stopping() {
    let said = said(&settings(), |mut progress| {
        progress.stopping();
        progress.stopping();
    });
    assert_eq!(said, ["cladenc: asked to stop, with 0 sandboxes running"]);
}

#[test]
fn a_directed_evolution_that_stops_for_a_failure_says_so_with_the_sandboxes_running() {
    let said = said(&settings(), |mut progress| {
        let first = start(&mut progress);
        start(&mut progress);
        progress.hear(failed(first, SandboxError::Exit(3)), &outputs_directory());
        progress.stopping();
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some("cladenc: stopping the directed evolution, with 1 sandbox running")
    );
}

#[test]
fn a_record_that_is_written_is_journaled() {
    let journaled = journaled(&settings(), |mut progress| {
        progress.record(&run_record(OsStr::new("/abs/harness")));
    });
    assert_eq!(journaled, [run_record(OsStr::new("/abs/harness"))]);
}

#[test]
fn a_record_that_cannot_be_written_is_not_journaled() {
    let journaled = journaled(&settings(), |mut progress| {
        progress.record(&unwritable());
    });
    assert!(journaled.is_empty());
}

#[test]
fn a_journal_that_cannot_be_written_halts_the_directed_evolution() {
    let halted = returned(&settings(), |mut progress| {
        progress.record(&unwritable());
        progress.halted(&AtomicBool::new(false))
    });
    assert!(halted);
}

#[test]
fn a_directed_evolution_whose_journal_cannot_be_written_ends_with_an_error_that_says_so() {
    let error = returned(&settings(), |mut progress| {
        progress.record(&unwritable());
        error_of(finish(progress))
    });
    assert!(error.is_some_and(|error| {
        error.starts_with(
            "the directed evolution stopped before its end: its journal could not be written: ",
        )
    }));
}

#[test]
fn the_action_that_failed_first_is_kept_when_the_journal_cannot_be_written_later() {
    let error = returned(&settings(), |mut progress| {
        fail_the_first_trial(&mut progress);
        progress.record(&unwritable());
        error_of(finish(progress))
    });
    assert_eq!(error.as_deref(), Some(ENDED_WITH_THE_FIRST_TRIAL));
}

#[test]
fn a_directed_evolution_that_finishes_gives_its_best_harness_and_where_it_is_kept() {
    let found = returned(&settings(), |progress| finish(progress).ok());
    assert_eq!(
        found,
        Some(Found {
            best_harness: 0,
            directory: PathBuf::from("/outputs/harnesses/0")
        })
    );
}

#[test]
fn a_directed_evolution_that_finishes_journals_its_best_harness() {
    assert_eq!(
        journaled(&settings(), finish),
        [Record::DirectedEvolutionFinished { best_harness: 0 }]
    );
}

#[test]
fn a_directed_evolution_that_finishes_says_what_it_started_and_its_best_harness() {
    assert_eq!(
        said(&settings(), finish),
        [
            "cladenc: the directed evolution finished with 0 trials started and 1 harness: its \
             best harness is harness 0, kept in /outputs/harnesses/0"
        ]
    );
}

#[test]
fn the_best_harness_is_the_one_heard_to_complete_the_most_selection_tasks() {
    let found = returned(&selecting_by_the_second_task(), |mut progress| {
        carry_out(&mut progress, 5, &[trial(0, SECOND), trial(1, FIRST)]);
        finish(progress).ok()
    });
    assert_eq!(
        found,
        Some(Found {
            best_harness: 1,
            directory: PathBuf::from("/outputs/harnesses/1")
        })
    );
}

#[test]
fn a_directed_evolution_that_finishes_counts_the_trials_and_the_harnesses_it_heard_of() {
    let said = said(&selecting_by_the_second_task(), |mut progress| {
        carry_out(&mut progress, 5, &[trial(0, SECOND), trial(1, FIRST)]);
        finish(progress)
    });
    assert_eq!(
        said.last().map(String::as_str),
        Some(
            "cladenc: the directed evolution finished with 4 trials started and 2 harnesses: \
             its best harness is harness 1, kept in /outputs/harnesses/1"
        )
    );
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_ends_with_an_error_that_says_so() {
    let error = returned(&settings(), |progress| {
        error_of(progress.finish(&outputs_directory(), &AtomicBool::new(true)))
    });
    assert_eq!(
        error.as_deref(),
        Some("the directed evolution stopped before its end: it was asked to stop")
    );
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_does_not_journal_that_it_finished() {
    let journaled = journaled(&settings(), |progress| {
        progress.finish(&outputs_directory(), &AtomicBool::new(true))
    });
    assert!(journaled.is_empty());
}

#[test]
fn a_directed_evolution_that_was_asked_to_stop_is_not_said_to_have_finished() {
    let said = said(&settings(), |progress| {
        progress.finish(&outputs_directory(), &AtomicBool::new(true))
    });
    assert!(said.is_empty());
}
