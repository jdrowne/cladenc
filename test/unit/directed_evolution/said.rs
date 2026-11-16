use std::path::Path;

use cladenc::containers::ContainerError;
use cladenc::directed_evolution::Said;
use cladenc::genealogy::{ProcreationOutcome, Score};
use cladenc::journal::Started;
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

fn assessment() -> Started {
    Started::Assessment {
        harness: 3,
        task: "fix-the-parser".to_owned(),
    }
}

fn started(action: &Started, again: bool) -> String {
    Said::Started {
        action,
        again,
        assessments: 5,
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

fn scored(score: Score) -> String {
    Said::Scored {
        harness: 3,
        task: "fix-the-parser",
        score,
        workspace: &workspace(),
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
        continuing(2),
        "continuing the directed evolution in /run, which has started 5 of its 800 assessments \
         and holds 2 harnesses"
    );
}

#[test]
fn one_harness_is_not_said_to_be_several() {
    assert_eq!(
        continuing(1),
        "continuing the directed evolution in /run, which has started 5 of its 800 assessments \
         and holds 1 harness"
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
            workspace: &workspace(),
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
            workspace: &workspace(),
        }
        .to_string(),
        "the build of the image exited with status 1: what Docker printed is kept in /run/image"
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
            error: &ContainerError::Exit(2),
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
fn an_assessment_is_said_to_have_started_with_its_number_and_the_budget() {
    assert_eq!(
        started(&assessment(), false),
        "started the assessment of harness 3 on task fix-the-parser, assessment 5 of 800"
    );
}

#[test]
fn an_assessment_that_is_run_again_is_said_to_have_started_again_with_no_number() {
    assert_eq!(
        started(&assessment(), true),
        "started again the assessment of harness 3 on task fix-the-parser"
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
fn a_harness_scored_a_success_is_said_with_where_what_the_assessment_left_is_kept() {
    assert_eq!(
        scored(Score::Success),
        "harness 3 was scored a success on task fix-the-parser: what the assessment left is kept \
         in /run/assessments/3/fix-the-parser"
    );
}

#[test]
fn a_harness_scored_a_failure_is_said_to_have_been_scored_a_failure() {
    assert_eq!(
        scored(Score::Failure),
        "harness 3 was scored a failure on task fix-the-parser: what the assessment left is kept \
         in /run/assessments/3/fix-the-parser"
    );
}

#[test]
fn a_harness_whose_assessment_timed_out_is_said_to_have_been_scored_a_failure_for_it() {
    assert_eq!(
        scored(Score::TimedOut),
        "harness 3 was scored a failure on task fix-the-parser, as the assessment ran past its \
         time limit: what the assessment left is kept in /run/assessments/3/fix-the-parser"
    );
}

#[test]
fn an_action_that_stops_the_directed_evolution_is_said_with_what_became_of_it() {
    assert_eq!(
        Said::Failed {
            action: &assessment(),
            error: &ContainerError::Exit(2),
        }
        .to_string(),
        "the assessment of harness 3 on task fix-the-parser exited with status 2"
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
            assessments: 800,
            harnesses: 31,
            best_harness: 12,
            directory: Path::new("/run/harnesses/12"),
        }
        .to_string(),
        "the directed evolution finished with 800 assessments started and 31 harnesses: its best \
         harness is harness 12, kept in /run/harnesses/12"
    );
}
