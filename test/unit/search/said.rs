use std::path::Path;

use cladenc::genealogy::{EvaluationOutcome, ProcreationOutcome};
use cladenc::job::JobError;
use cladenc::journal::Started;
use cladenc::search::Said;
use cladenc::workspace::Workspace;

fn workspace() -> Workspace {
    Workspace::at(Path::new("/run"))
}

fn procreation() -> Started {
    Started::Procreation {
        child: 3,
        parent: 1,
    }
}

fn evaluation() -> Started {
    Started::Evaluation {
        harness: 3,
        task: "fix-the-parser".to_owned(),
    }
}

fn started(job: &Started, again: bool) -> String {
    Said::Started {
        job,
        again,
        evaluations: 5,
        budget: 800,
    }
    .to_string()
}

fn procreated(outcome: ProcreationOutcome) -> String {
    Said::Procreated {
        child: 3,
        parent: 1,
        outcome,
        workspace: &workspace(),
    }
    .to_string()
}

fn evaluated(outcome: EvaluationOutcome) -> String {
    Said::Evaluated {
        harness: 3,
        task: "fix-the-parser",
        outcome,
        workspace: &workspace(),
    }
    .to_string()
}

fn copying(tasks: u64) -> String {
    Said::CopyingTasks {
        benchmark: Path::new("/parsing"),
        tasks,
    }
    .to_string()
}

fn continuing(harnesses: u64) -> String {
    Said::Continuing {
        run_directory: Path::new("/run"),
        started: 5,
        budget: 800,
        harnesses,
    }
    .to_string()
}

#[test]
fn the_run_directory_is_said_to_be_made_with_a_copy_of_the_progenitor() {
    assert_eq!(
        Said::Making {
            run_directory: Path::new("/run"),
            progenitor: Path::new("/progenitor"),
        }
        .to_string(),
        "making the run directory /run with a copy of the progenitor /progenitor"
    );
}

#[test]
fn the_tasks_of_a_benchmark_are_said_to_be_copied_with_their_number() {
    assert_eq!(
        copying(3),
        "copying the 3 tasks of the benchmark /parsing into the run directory"
    );
}

#[test]
fn one_task_is_not_said_to_be_several() {
    assert_eq!(
        copying(1),
        "copying the 1 task of the benchmark /parsing into the run directory"
    );
}

#[test]
fn a_search_that_is_continued_is_said_with_how_far_it_had_got() {
    assert_eq!(
        continuing(2),
        "continuing the search in /run, which has started 5 of its 800 evaluations and holds 2 \
         harnesses"
    );
}

#[test]
fn one_harness_is_not_said_to_be_several() {
    assert_eq!(
        continuing(1),
        "continuing the search in /run, which has started 5 of its 800 evaluations and holds 1 \
         harness"
    );
}

#[test]
fn a_search_that_had_finished_is_said_with_its_best_harness() {
    assert_eq!(
        Said::HadFinished {
            best_harness: 3,
            directory: Path::new("/run/harnesses/3"),
        }
        .to_string(),
        "the search had finished: its best harness is harness 3, kept in /run/harnesses/3"
    );
}

#[test]
fn the_progenitor_is_said_to_be_being_built() {
    assert_eq!(Said::Building.to_string(), "building the progenitor");
}

#[test]
fn the_progenitor_is_said_to_be_built() {
    assert_eq!(Said::Built.to_string(), "the progenitor is built");
}

#[test]
fn a_build_of_the_progenitor_that_failed_is_said_with_where_what_it_printed_is_kept() {
    assert_eq!(
        Said::BuildFailed {
            error: &JobError::Exit(2),
            workspace: &workspace(),
        }
        .to_string(),
        "the build of the progenitor exited with status 2: what it printed is kept in \
         /run/builds/0"
    );
}

#[test]
fn a_procreation_is_said_to_have_started() {
    assert_eq!(
        started(&procreation(), false),
        "started the procreation of harness 3 by harness 1"
    );
}

#[test]
fn a_procreation_that_is_run_again_is_said_to_have_started_again() {
    assert_eq!(
        started(&procreation(), true),
        "started again the procreation of harness 3 by harness 1"
    );
}

#[test]
fn an_evaluation_is_said_to_have_started_with_its_number_and_the_budget() {
    assert_eq!(
        started(&evaluation(), false),
        "started the evaluation of harness 3 on task fix-the-parser, evaluation 5 of 800"
    );
}

#[test]
fn an_evaluation_that_is_run_again_is_said_to_have_started_again_with_no_number() {
    assert_eq!(
        started(&evaluation(), true),
        "started again the evaluation of harness 3 on task fix-the-parser"
    );
}

#[test]
fn a_procreation_that_created_its_child_is_said_with_where_the_child_is_kept() {
    assert_eq!(
        procreated(ProcreationOutcome::Created),
        "the procreation of harness 3 by harness 1 created harness 3, kept in /run/harnesses/3"
    );
}

#[test]
fn a_procreation_that_failed_is_said_with_where_what_the_harness_printed_is_kept() {
    assert_eq!(
        procreated(ProcreationOutcome::Failed),
        "the procreation of harness 3 by harness 1 failed: what the harness printed is kept in \
         /run/procreations/3"
    );
}

#[test]
fn a_child_that_did_not_build_is_said_with_where_what_its_build_printed_is_kept() {
    assert_eq!(
        procreated(ProcreationOutcome::BuildFailed),
        "the procreation of harness 3 by harness 1 made a harness that did not build: what its \
         build printed is kept in /run/builds/3"
    );
}

#[test]
fn a_procreation_that_timed_out_is_said_to_have_run_past_its_time_limit() {
    assert_eq!(
        procreated(ProcreationOutcome::TimedOut),
        "the procreation of harness 3 by harness 1 ran past its time limit: what the harness \
         printed is kept in /run/procreations/3"
    );
}

#[test]
fn an_evaluation_that_succeeded_is_said_with_where_what_it_left_is_kept() {
    assert_eq!(
        evaluated(EvaluationOutcome::Success),
        "the evaluation of harness 3 on task fix-the-parser succeeded: what it left is kept in \
         /run/evaluations/3/fix-the-parser"
    );
}

#[test]
fn an_evaluation_that_failed_is_said_to_have_failed() {
    assert_eq!(
        evaluated(EvaluationOutcome::Failure),
        "the evaluation of harness 3 on task fix-the-parser failed: what it left is kept in \
         /run/evaluations/3/fix-the-parser"
    );
}

#[test]
fn an_evaluation_that_timed_out_is_said_to_have_run_past_its_time_limit() {
    assert_eq!(
        evaluated(EvaluationOutcome::TimedOut),
        "the evaluation of harness 3 on task fix-the-parser ran past its time limit: what it \
         left is kept in /run/evaluations/3/fix-the-parser"
    );
}

#[test]
fn a_job_that_stops_the_search_is_said_with_what_became_of_it() {
    assert_eq!(
        Said::Failed {
            job: &evaluation(),
            error: &JobError::Exit(2),
        }
        .to_string(),
        "the evaluation of harness 3 on task fix-the-parser exited with status 2"
    );
}

#[test]
fn a_search_asked_to_stop_is_said_with_the_number_of_jobs_running() {
    assert_eq!(
        Said::Stopping {
            asked: true,
            running: 2,
        }
        .to_string(),
        "asked to stop, with 2 jobs running"
    );
}

#[test]
fn a_search_that_stops_over_a_failure_is_said_to_be_stopping() {
    assert_eq!(
        Said::Stopping {
            asked: false,
            running: 1,
        }
        .to_string(),
        "stopping the search, with 1 job running"
    );
}

#[test]
fn the_end_of_a_search_is_said_with_its_best_harness_and_where_it_is_kept() {
    assert_eq!(
        Said::Finished {
            evaluations: 800,
            harnesses: 31,
            best_harness: 12,
            directory: Path::new("/run/harnesses/12"),
        }
        .to_string(),
        "the search finished with 800 evaluations started and 31 harnesses: its best harness is \
         harness 12, kept in /run/harnesses/12"
    );
}
