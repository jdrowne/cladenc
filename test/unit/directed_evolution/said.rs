use std::path::Path;

use cladenc::containers::ContainerError;
use cladenc::directed_evolution::Said;
use cladenc::genealogy::{AttemptResult, ProcreationOutcome};
use cladenc::journal::Started;
use cladenc::run_directory::RunDirectory;

fn run_directory() -> RunDirectory {
    RunDirectory::at(Path::new("/run"))
}

fn procreation() -> Started {
    Started::Procreation {
        child: 3,
        progenitor: 1,
    }
}

fn attempt() -> Started {
    Started::Attempt {
        harness: 3,
        task: "fix-the-parser".to_owned(),
    }
}

fn started(action: &Started, again: bool) -> String {
    Said::Started {
        action,
        again,
        max_procreations: 800,
    }
    .to_string()
}

fn procreated(outcome: ProcreationOutcome) -> String {
    Said::Procreated {
        child: 3,
        progenitor: 1,
        outcome,
        run_directory: &run_directory(),
    }
    .to_string()
}

fn attempted(result: AttemptResult) -> String {
    Said::Attempted {
        harness: 3,
        task: "fix-the-parser",
        result,
        run_directory: &run_directory(),
    }
    .to_string()
}

const IMAGE: &str = "cladenc-0123456789abcdef";

fn copying(tasks: u64) -> String {
    Said::CopyingTasks {
        benchmark: Path::new("/parsing"),
        tasks,
    }
    .to_string()
}

fn continuing(procreations: u64, max_procreations: u64) -> String {
    Said::Continuing {
        run_directory: Path::new("/run"),
        procreations,
        max_procreations,
        harnesses: procreations + 1,
    }
    .to_string()
}

#[test]
fn the_run_directory_is_said_to_be_made_with_a_copy_of_the_primogenitor() {
    assert_eq!(
        Said::Making {
            run_directory: Path::new("/run"),
            primogenitor: Path::new("/primogenitor"),
        }
        .to_string(),
        "making the run directory /run with a copy of the primogenitor /primogenitor"
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

fn removed(containers: u64) -> String {
    Said::Removed { containers }.to_string()
}

#[test]
fn the_containers_a_directed_evolution_left_running_are_said_to_be_removed_with_their_number() {
    assert_eq!(
        removed(2),
        "removed 2 containers that the directed evolution left running"
    );
}

#[test]
fn one_container_is_not_said_to_be_several() {
    assert_eq!(
        removed(1),
        "removed 1 container that the directed evolution left running"
    );
}

#[test]
fn a_directed_evolution_that_is_continued_is_said_with_how_far_it_had_got() {
    assert_eq!(
        continuing(5, 800),
        "continuing the directed evolution in /run, which has started 5 of 800 procreations and \
         holds 6 harnesses"
    );
}

#[test]
fn one_harness_is_not_said_to_be_several() {
    assert_eq!(
        continuing(0, 800),
        "continuing the directed evolution in /run, which has started 0 of 800 procreations and \
         holds 1 harness"
    );
}

#[test]
fn one_procreation_is_not_said_to_be_several() {
    assert_eq!(
        continuing(1, 1),
        "continuing the directed evolution in /run, which has started 1 of 1 procreation and \
         holds 2 harnesses"
    );
}

#[test]
fn a_directed_evolution_that_had_finished_is_said_with_its_best_harness() {
    assert_eq!(
        Said::HadFinished {
            best_harness: 3,
            directory: Path::new("/run/harnesses/3"),
        }
        .to_string(),
        "the directed evolution had finished: its best harness is harness 3, kept in \
         /run/harnesses/3"
    );
}

#[test]
fn an_image_is_said_to_be_being_built_with_its_name_and_its_dockerfile() {
    assert_eq!(
        Said::BuildingImage {
            image: IMAGE,
            dockerfile: Path::new("/images/Dockerfile"),
        }
        .to_string(),
        "building the image cladenc-0123456789abcdef from the Dockerfile /images/Dockerfile"
    );
}

#[test]
fn an_image_is_said_to_be_built_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        Said::ImageBuilt {
            image: IMAGE,
            run_directory: &run_directory(),
        }
        .to_string(),
        "the image cladenc-0123456789abcdef is built: what Docker printed is kept in /run/image"
    );
}

#[test]
fn an_image_that_was_not_built_is_said_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        Said::ImageNotBuilt {
            error: &ContainerError::Exit(1),
            run_directory: &run_directory(),
        }
        .to_string(),
        "the build of the image exited with status 1: what Docker printed is kept in /run/image"
    );
}

#[test]
fn the_primogenitor_is_said_to_be_being_built() {
    assert_eq!(Said::Building.to_string(), "building the primogenitor");
}

#[test]
fn the_primogenitor_is_said_to_be_built() {
    assert_eq!(Said::Built.to_string(), "the primogenitor is built");
}

#[test]
fn a_build_of_the_primogenitor_that_failed_is_said_with_where_what_it_printed_is_kept() {
    assert_eq!(
        Said::BuildFailed {
            error: &ContainerError::Exit(2),
            run_directory: &run_directory(),
        }
        .to_string(),
        "the build of the primogenitor exited with status 2: what it printed is kept in \
         /run/builds/0"
    );
}

#[test]
fn a_procreation_is_said_to_have_started_with_its_number_and_the_most_there_may_be() {
    assert_eq!(
        started(&procreation(), false),
        "started the procreation of harness 3 by harness 1, procreation 3 of 800"
    );
}

#[test]
fn a_procreation_that_is_run_again_is_said_to_have_started_again_with_no_number() {
    assert_eq!(
        started(&procreation(), true),
        "started again the procreation of harness 3 by harness 1"
    );
}

#[test]
fn an_attempt_is_said_to_have_started_with_no_number() {
    assert_eq!(
        started(&attempt(), false),
        "started the attempt of harness 3 on task fix-the-parser"
    );
}

#[test]
fn an_attempt_that_is_run_again_is_said_to_have_started_again_with_no_number() {
    assert_eq!(
        started(&attempt(), true),
        "started again the attempt of harness 3 on task fix-the-parser"
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
fn a_harness_that_completed_a_task_is_said_with_where_what_the_attempt_left_is_kept() {
    assert_eq!(
        attempted(AttemptResult::Completed),
        "harness 3 completed task fix-the-parser: what the attempt left is kept \
         in /run/attempts/3/fix-the-parser"
    );
}

#[test]
fn a_harness_that_did_not_complete_a_task_is_said_not_to_have_completed_it() {
    assert_eq!(
        attempted(AttemptResult::NotCompleted),
        "harness 3 did not complete task fix-the-parser: what the attempt left is kept \
         in /run/attempts/3/fix-the-parser"
    );
}

#[test]
fn a_harness_whose_attempt_timed_out_is_said_not_to_have_completed_the_task_for_it() {
    assert_eq!(
        attempted(AttemptResult::TimedOut),
        "harness 3 did not complete task fix-the-parser, as the attempt ran past its \
         time limit: what the attempt left is kept in /run/attempts/3/fix-the-parser"
    );
}

#[test]
fn an_action_that_stops_the_directed_evolution_is_said_with_what_became_of_it() {
    assert_eq!(
        Said::Failed {
            action: &attempt(),
            error: &ContainerError::Exit(2),
        }
        .to_string(),
        "the attempt of harness 3 on task fix-the-parser exited with status 2"
    );
}

#[test]
fn a_directed_evolution_asked_to_stop_is_said_with_the_number_of_containers_running() {
    assert_eq!(
        Said::Stopping {
            asked: true,
            running: 2,
        }
        .to_string(),
        "asked to stop, with 2 containers running"
    );
}

#[test]
fn a_directed_evolution_that_stops_over_a_failure_is_said_to_be_stopping() {
    assert_eq!(
        Said::Stopping {
            asked: false,
            running: 1,
        }
        .to_string(),
        "stopping the directed evolution, with 1 container running"
    );
}

#[test]
fn the_end_of_a_directed_evolution_is_said_with_its_best_harness_and_where_it_is_kept() {
    assert_eq!(
        Said::Finished {
            attempts: 800,
            harnesses: 31,
            best_harness: 12,
            directory: Path::new("/run/harnesses/12"),
        }
        .to_string(),
        "the directed evolution finished with 800 attempts started and 31 harnesses: its best \
         harness is harness 12, kept in /run/harnesses/12"
    );
}
