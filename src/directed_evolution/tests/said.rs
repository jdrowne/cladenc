use std::path::Path;
use std::time::Duration;

use crate::directed_evolution::Said;
use crate::directed_evolution::said::{Environments, Retrying};
use crate::file_system::tests::double::Memory;
use crate::genealogy::{Cause, CreationOutcome, Experiment, TrialResult};
use crate::journal::Started;
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::{Checked, HarnessEnding, Retry, SandboxError};

fn outputs_directory() -> OutputsDirectory {
    OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"))
}

fn diversification() -> Started {
    Started::Diversification {
        child: 3,
        progenitor: 1,
    }
}

fn trial() -> Started {
    Started::Trial {
        harness: 3,
        task: "fix-the-parser".to_owned(),
    }
}

fn baseline() -> Started {
    Started::Baseline {
        harness: 4,
        child: 3,
        progenitor: 1,
    }
}

fn started(action: &Started, again: bool) -> String {
    Said::Started {
        action,
        again,
        diversifications: 2,
        max_diversifications: 800,
    }
    .to_string()
}

fn established(outcome: CreationOutcome) -> String {
    Said::Established {
        harness: 4,
        experiment: Experiment {
            child: 3,
            progenitor: 1,
        },
        outcome,
        outputs_directory: &outputs_directory(),
    }
    .to_string()
}

fn diversified(outcome: CreationOutcome) -> String {
    Said::Diversified {
        child: 3,
        progenitor: 1,
        outcome,
        outputs_directory: &outputs_directory(),
    }
    .to_string()
}

fn trialled(result: TrialResult) -> String {
    Said::Trialled {
        harness: 3,
        task: "fix-the-parser",
        result,
        ending: HarnessEnding::Ended,
        outputs_directory: &outputs_directory(),
    }
    .to_string()
}

fn trialled_by_a_harness_that_ran_out_of_time(result: TrialResult) -> String {
    Said::Trialled {
        harness: 3,
        task: "fix-the-parser",
        result,
        ending: HarnessEnding::Stopped,
        outputs_directory: &outputs_directory(),
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

fn continuing(diversifications: u64, max_diversifications: u64) -> String {
    Said::Continuing {
        outputs_directory: Path::new("/outputs"),
        diversifications,
        max_diversifications,
        harnesses: diversifications + 1,
    }
    .to_string()
}

#[test]
fn the_outputs_directory_is_said_to_be_made_with_a_copy_of_the_primogenitor() {
    assert_eq!(
        Said::Making {
            outputs_directory: Path::new("/outputs"),
            primogenitor: Path::new("/primogenitor"),
        }
        .to_string(),
        "making the outputs directory /outputs with a copy of the primogenitor /primogenitor"
    );
}

#[test]
fn the_tasks_of_a_benchmark_are_said_to_be_copied_with_their_number() {
    assert_eq!(
        copying(3),
        "copying the 3 tasks of the benchmark /parsing into the outputs directory"
    );
}

#[test]
fn one_task_is_not_said_to_be_several() {
    assert_eq!(
        copying(1),
        "copying the 1 task of the benchmark /parsing into the outputs directory"
    );
}

#[test]
fn the_tasks_held_out_of_a_benchmark_are_said_with_how_many_of_its_tasks_they_are() {
    assert_eq!(
        Said::HoldingOut {
            benchmark: Path::new("/parsing"),
            selection: 6,
            corroboration: 2,
            tasks: 10,
        }
        .to_string(),
        "holding out 8 of the 10 tasks of the benchmark /parsing from every progenitor: 6 \
         selection tasks and 2 corroboration tasks"
    );
}

#[test]
fn one_selection_task_and_one_corroboration_task_are_not_said_as_several() {
    assert_eq!(
        Said::HoldingOut {
            benchmark: Path::new("/parsing"),
            selection: 1,
            corroboration: 1,
            tasks: 3,
        }
        .to_string(),
        "holding out 2 of the 3 tasks of the benchmark /parsing from every progenitor: 1 \
         selection task and 1 corroboration task"
    );
}

#[test]
fn a_trial_that_passed_after_its_harness_was_stopped_at_its_time_limit_is_said_with_both() {
    assert_eq!(
        trialled_by_a_harness_that_ran_out_of_time(TrialResult::Pass),
        "the trial of harness 3 on task fix-the-parser passed, after the harness was stopped at \
         its time limit: what the trial left is kept in /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn a_trial_that_failed_after_its_harness_was_stopped_at_its_time_limit_is_said_with_both() {
    assert_eq!(
        trialled_by_a_harness_that_ran_out_of_time(TrialResult::Fail),
        "the trial of harness 3 on task fix-the-parser failed, after the harness was stopped at \
         its time limit, as the verifier wrote a verdict that is not 1: what the trial left is \
         kept in /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn an_inconclusive_trial_whose_harness_was_stopped_at_its_time_limit_is_said_with_both() {
    assert_eq!(
        trialled_by_a_harness_that_ran_out_of_time(TrialResult::Inconclusive(Cause::NoVerdict)),
        "the trial of harness 3 on task fix-the-parser was inconclusive, after the harness was \
         stopped at its time limit, as the verifier left no verdict that can be read: what the \
         trial left is kept in /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn a_trial_on_a_holdout_task_is_said_with_the_directory_under_holdout_that_keeps_it() {
    assert_eq!(
        Said::Trialled {
            harness: 3,
            task: "parsing/fix-the-parser",
            result: TrialResult::Pass,
            ending: HarnessEnding::Ended,
            outputs_directory: &outputs_directory()
                .holding_out(&["parsing/fix-the-parser".to_owned()]),
        }
        .to_string(),
        "the trial of harness 3 on task parsing/fix-the-parser passed: what the trial left is \
         kept in /outputs/holdout/3/parsing/fix-the-parser"
    );
}

fn removed(sandboxes: u64) -> String {
    Said::Removed { sandboxes }.to_string()
}

#[test]
fn the_sandboxes_a_directed_evolution_left_running_are_said_to_be_removed_with_their_number() {
    assert_eq!(
        removed(2),
        "removed 2 sandboxes that the directed evolution left running"
    );
}

#[test]
fn one_sandbox_is_not_said_to_be_several() {
    assert_eq!(
        removed(1),
        "removed 1 sandbox that the directed evolution left running"
    );
}

#[test]
fn a_directed_evolution_that_is_continued_is_said_with_how_far_it_had_got() {
    assert_eq!(
        continuing(5, 800),
        "continuing the directed evolution in /outputs, which has started 5 of 800 \
         diversifications and holds 6 harnesses"
    );
}

#[test]
fn one_harness_is_not_said_to_be_several() {
    assert_eq!(
        continuing(0, 800),
        "continuing the directed evolution in /outputs, which has started 0 of 800 \
         diversifications and holds 1 harness"
    );
}

#[test]
fn one_diversification_is_not_said_to_be_several() {
    assert_eq!(
        continuing(1, 1),
        "continuing the directed evolution in /outputs, which has started 1 of 1 diversification \
         and holds 2 harnesses"
    );
}

#[test]
fn a_directed_evolution_that_had_finished_is_said_with_its_best_harness() {
    assert_eq!(
        Said::HadFinished {
            best_harness: 3,
            directory: Path::new("/outputs/harnesses/3"),
        }
        .to_string(),
        "the directed evolution had finished: its best harness is harness 3, kept in \
         /outputs/harnesses/3"
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
            outputs_directory: &outputs_directory(),
        }
        .to_string(),
        "the image cladenc-0123456789abcdef is built: what Docker printed is kept in /outputs/image"
    );
}

#[test]
fn an_image_that_was_not_built_is_said_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        Said::ImageNotBuilt {
            error: &SandboxError::Exit(1),
            outputs_directory: &outputs_directory(),
        }
        .to_string(),
        "the build of the image exited with status 1: what Docker printed is kept in /outputs/image"
    );
}

#[test]
fn the_environment_of_a_task_is_said_to_be_being_built_with_the_image_it_is_built_as() {
    assert_eq!(
        Said::Environments(Environments::Building {
            task: "benchmark/t1",
            image: "cladenc-environments-0123456789abcdef:0",
        })
        .to_string(),
        "building the environment of task benchmark/t1 as the image \
         cladenc-environments-0123456789abcdef:0"
    );
}

#[test]
fn the_environment_of_the_one_task_is_said_to_be_built_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        Said::Environments(Environments::Built {
            built: 1,
            tasks: 1,
            outputs_directory: &outputs_directory(),
        })
        .to_string(),
        "built the environment of the 1 task: what Docker printed is kept in /outputs/environments"
    );
}

#[test]
fn the_environments_of_several_tasks_are_said_to_be_built_with_how_many_tasks_there_are() {
    assert_eq!(
        Said::Environments(Environments::Built {
            built: 3,
            tasks: 3,
            outputs_directory: &outputs_directory(),
        })
        .to_string(),
        "built the environments of the 3 tasks: what Docker printed is kept in \
         /outputs/environments"
    );
}

#[test]
fn the_environments_built_for_some_of_the_tasks_are_said_with_how_many_of_how_many() {
    assert_eq!(
        Said::Environments(Environments::Built {
            built: 2,
            tasks: 3,
            outputs_directory: &outputs_directory(),
        })
        .to_string(),
        "built the environments of 2 of the 3 tasks: what Docker printed is kept in \
         /outputs/environments"
    );
}

#[test]
fn the_one_environment_built_among_several_tasks_is_said_as_one() {
    assert_eq!(
        Said::Environments(Environments::Built {
            built: 1,
            tasks: 3,
            outputs_directory: &outputs_directory(),
        })
        .to_string(),
        "built the environment of 1 of the 3 tasks: what Docker printed is kept in \
         /outputs/environments"
    );
}

#[test]
fn an_image_that_a_task_names_is_said_to_be_taken_as_the_environment_of_the_task() {
    assert_eq!(
        Said::Environments(Environments::Taking {
            task: "benchmark/t1",
            image: "registry.example/tasks/parser:3",
        })
        .to_string(),
        "taking the image registry.example/tasks/parser:3 as the environment of task \
         benchmark/t1, as its task.toml says"
    );
}

#[test]
fn an_environment_that_was_not_built_is_said_with_its_task_and_where_what_docker_printed_is_kept() {
    assert_eq!(
        Said::Environments(Environments::NotBuilt {
            task: "benchmark/t1",
            error: &SandboxError::Exit(1),
            outputs_directory: &outputs_directory(),
        })
        .to_string(),
        "the build of the environment of task benchmark/t1 exited with status 1: what Docker \
         printed is kept in /outputs/environments/benchmark/t1"
    );
}

#[test]
fn a_verifier_environment_is_said_to_be_being_built_with_the_image_it_is_built_as() {
    assert_eq!(
        Said::Environments(Environments::VerifierBuilding {
            task: "benchmark/t1",
            image: "cladenc-environments-0123456789abcdef:0-verifier",
        })
        .to_string(),
        "building the verifier environment of task benchmark/t1 as the image \
         cladenc-environments-0123456789abcdef:0-verifier"
    );
}

#[test]
fn an_image_that_a_task_names_for_its_verifier_is_said_to_be_taken_as_its_environment() {
    assert_eq!(
        Said::Environments(Environments::VerifierTaking {
            task: "benchmark/t1",
            image: "registry.example/verifiers/parser:3",
        })
        .to_string(),
        "taking the image registry.example/verifiers/parser:3 as the verifier environment of \
         task benchmark/t1, as its task.toml says"
    );
}

#[test]
fn a_verifier_environment_that_was_not_built_is_said_with_where_what_docker_printed_is_kept() {
    assert_eq!(
        Said::Environments(Environments::VerifierNotBuilt {
            task: "benchmark/t1",
            error: &SandboxError::Exit(1),
            outputs_directory: &outputs_directory(),
        })
        .to_string(),
        "the build of the verifier environment of task benchmark/t1 exited with status 1: \
         what Docker printed is kept in /outputs/environments/benchmark/t1/verifier"
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
            error: &SandboxError::Exit(2),
            outputs_directory: &outputs_directory(),
        }
        .to_string(),
        "the build of the primogenitor exited with status 2: what it printed is kept in \
         /outputs/builds/0"
    );
}

#[test]
fn a_diversification_is_said_to_have_started_with_its_number_and_the_most_there_may_be() {
    assert_eq!(
        started(&diversification(), false),
        "started the diversification of harness 3 by harness 1, diversification 2 of 800"
    );
}

#[test]
fn the_number_of_a_diversification_is_how_many_were_started_and_not_the_id_of_its_child() {
    assert!(started(&diversification(), false).ends_with(", diversification 2 of 800"));
}

#[test]
fn a_baseline_is_said_to_have_started_with_no_number() {
    assert_eq!(
        started(&baseline(), false),
        "started the creation of harness 4, the singleton control group harness, by the \
         primogenitor from a second clone of harness 1"
    );
}

#[test]
fn a_baseline_that_is_established_again_is_said_to_have_started_again() {
    assert_eq!(
        started(&baseline(), true),
        "started again the creation of harness 4, the singleton control group harness, by the \
         primogenitor from a second clone of harness 1"
    );
}

#[test]
fn a_baseline_that_created_its_harness_is_said_with_where_the_harness_is_kept() {
    assert_eq!(
        established(CreationOutcome::Created),
        "the creation of harness 4, the singleton control group harness, by the primogenitor \
         from a second clone of harness 1 created harness 4, kept in /outputs/harnesses/4"
    );
}

#[test]
fn a_baseline_that_failed_is_said_with_where_what_the_primogenitor_printed_is_kept() {
    assert_eq!(
        established(CreationOutcome::Failed),
        "the creation of harness 4, the singleton control group harness, by the primogenitor \
         from a second clone of harness 1 failed: what the primogenitor printed is kept in \
         /outputs/baselines/4"
    );
}

#[test]
fn a_singleton_control_group_harness_that_did_not_build_is_said_with_where_its_build_is_kept() {
    assert_eq!(
        established(CreationOutcome::BuildFailed),
        "the creation of harness 4, the singleton control group harness, by the primogenitor \
         from a second clone of harness 1 made a harness that did not build: what its build \
         printed is kept in /outputs/builds/4"
    );
}

#[test]
fn a_baseline_that_timed_out_is_said_to_have_run_past_its_time_limit() {
    assert_eq!(
        established(CreationOutcome::TimedOut),
        "the creation of harness 4, the singleton control group harness, by the primogenitor \
         from a second clone of harness 1 ran past its time limit: what the primogenitor \
         printed is kept in /outputs/baselines/4"
    );
}

#[test]
fn a_baseline_that_left_the_source_code_of_the_progenitor_unchanged_is_said_to_have_created_no_harness()
 {
    assert_eq!(
        established(CreationOutcome::Unchanged),
        "the creation of harness 4, the singleton control group harness, by the primogenitor \
         from a second clone of harness 1 left the clone of harness 1 \
         unchanged, and created no harness: what the primogenitor printed is kept in \
         /outputs/baselines/4"
    );
}

#[test]
fn a_diversification_that_is_run_again_is_said_to_have_started_again_with_no_number() {
    assert_eq!(
        started(&diversification(), true),
        "started again the diversification of harness 3 by harness 1"
    );
}

#[test]
fn a_trial_is_said_to_have_started_with_no_number() {
    assert_eq!(
        started(&trial(), false),
        "started the trial of harness 3 on task fix-the-parser"
    );
}

#[test]
fn a_trial_that_is_run_again_is_said_to_have_started_again_with_no_number() {
    assert_eq!(
        started(&trial(), true),
        "started again the trial of harness 3 on task fix-the-parser"
    );
}

#[test]
fn a_diversification_that_created_its_child_is_said_with_where_the_child_is_kept() {
    assert_eq!(
        diversified(CreationOutcome::Created),
        "the diversification of harness 3 by harness 1 created harness 3, kept in \
         /outputs/harnesses/3"
    );
}

#[test]
fn a_diversification_that_failed_is_said_with_where_what_the_harness_printed_is_kept() {
    assert_eq!(
        diversified(CreationOutcome::Failed),
        "the diversification of harness 3 by harness 1 failed: what the harness printed is kept in \
         /outputs/diversifications/3"
    );
}

#[test]
fn a_diversification_that_left_the_progenitor_s_clone_unchanged_is_said_to_have_created_no_harness()
{
    assert_eq!(
        diversified(CreationOutcome::Unchanged),
        "the diversification of harness 3 by harness 1 left the clone of harness 1 \
         unchanged, and created no harness: what the harness printed is kept in \
         /outputs/diversifications/3"
    );
}

#[test]
fn a_child_that_did_not_build_is_said_with_where_what_its_build_printed_is_kept() {
    assert_eq!(
        diversified(CreationOutcome::BuildFailed),
        "the diversification of harness 3 by harness 1 made a harness that did not build: what its \
         build printed is kept in /outputs/builds/3"
    );
}

#[test]
fn a_diversification_that_timed_out_is_said_to_have_run_past_its_time_limit() {
    assert_eq!(
        diversified(CreationOutcome::TimedOut),
        "the diversification of harness 3 by harness 1 ran past its time limit: what the harness \
         printed is kept in /outputs/diversifications/3"
    );
}

#[test]
fn a_trial_that_passed_is_said_with_where_what_the_trial_left_is_kept() {
    assert_eq!(
        trialled(TrialResult::Pass),
        "the trial of harness 3 on task fix-the-parser passed: what the trial left is kept in \
         /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn a_trial_whose_verdict_was_not_1_is_said_to_have_failed_for_it() {
    assert_eq!(
        trialled(TrialResult::Fail),
        "the trial of harness 3 on task fix-the-parser failed, as the verifier wrote a verdict \
         that is not 1: what the trial left is kept in /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_is_said_to_be_inconclusive_for_it() {
    assert_eq!(
        trialled(TrialResult::Inconclusive(Cause::NoVerdict)),
        "the trial of harness 3 on task fix-the-parser was inconclusive, as the verifier left \
         no verdict that can be read: what the trial left is kept in \
         /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn a_trial_whose_verifier_timed_out_is_said_to_be_inconclusive_for_it() {
    assert_eq!(
        trialled(TrialResult::Inconclusive(Cause::TimedOut)),
        "the trial of harness 3 on task fix-the-parser was inconclusive, as the verifier ran \
         past its time limit: what the trial left is kept in /outputs/trials/3/fix-the-parser"
    );
}

#[test]
fn a_trial_whose_verifier_left_no_verdict_is_not_said_to_have_failed() {
    assert!(!trialled(TrialResult::Inconclusive(Cause::NoVerdict)).contains("failed"));
}

#[test]
fn a_trial_whose_verifier_timed_out_is_not_said_to_have_failed() {
    assert!(!trialled(TrialResult::Inconclusive(Cause::TimedOut)).contains("failed"));
}

#[test]
fn an_action_that_stops_the_directed_evolution_is_said_with_what_became_of_it() {
    assert_eq!(
        Said::Failed {
            action: &trial(),
            error: &SandboxError::Exit(2),
        }
        .to_string(),
        "the trial of harness 3 on task fix-the-parser exited with status 2"
    );
}

#[test]
fn a_directed_evolution_asked_to_stop_is_said_with_the_number_of_sandboxes_running() {
    assert_eq!(
        Said::Stopping {
            asked: true,
            running: 2,
        }
        .to_string(),
        "asked to stop, with 2 sandboxes running"
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
        "stopping the directed evolution, with 1 sandbox running"
    );
}

#[test]
fn the_end_of_a_directed_evolution_is_said_with_its_best_harness_and_where_it_is_kept() {
    assert_eq!(
        Said::Finished {
            trials: 800,
            harnesses: 31,
            best_harness: 12,
            directory: Path::new("/outputs/harnesses/12"),
        }
        .to_string(),
        "the directed evolution finished with 800 trials started and 31 harnesses: its best \
         harness is harness 12, kept in /outputs/harnesses/12"
    );
}

fn retrying(checked: Checked, wait: Duration) -> String {
    Said::Retrying(Retrying {
        harness: 3,
        task: "parsing/fix-the-parser",
        retry: Retry {
            checked,
            number: 2,
            most: 5,
            wait,
        },
    })
    .to_string()
}

fn retrying_after(wait: Duration) -> String {
    retrying(Checked::Verifier, wait)
}

#[test]
fn a_retry_of_the_health_check_of_a_task_is_said_with_its_trial_its_wait_and_its_number() {
    assert_eq!(
        retrying(Checked::Task, Duration::from_secs(10)),
        "the trial of harness 3 on task parsing/fix-the-parser is not carried out yet: the \
         health check of the task did not pass, and cladenc tries again in a new sandbox \
         after 10 seconds, retry 2 of 5"
    );
}

#[test]
fn a_wait_of_1_second_before_a_retry_of_the_health_check_of_a_task_is_not_said_as_several() {
    assert!(
        retrying(Checked::Task, Duration::from_secs(1))
            .ends_with(" in a new sandbox after 1 second, retry 2 of 5")
    );
}

#[test]
fn a_retry_of_the_health_check_of_a_verifier_is_said_with_its_trial_its_wait_and_its_number() {
    assert_eq!(
        retrying_after(Duration::from_secs(10)),
        "the trial of harness 3 on task parsing/fix-the-parser is not verified yet: the health \
         check of its verifier sandbox did not pass, and cladenc tries again in a new \
         sandbox after 10 seconds, retry 2 of 5"
    );
}

#[test]
fn a_wait_of_1_second_before_a_retry_is_not_said_as_several() {
    assert!(
        retrying_after(Duration::from_secs(1))
            .ends_with(" in a new sandbox after 1 second, retry 2 of 5")
    );
}

#[test]
fn a_wait_of_a_part_of_a_second_before_a_retry_is_said_with_the_part() {
    assert!(
        retrying_after(Duration::from_millis(200))
            .ends_with(" in a new sandbox after 0.2 seconds, retry 2 of 5")
    );
}

#[test]
fn a_verifier_that_was_never_run_is_said_with_what_stopped_it_from_being_run() {
    assert_eq!(
        Said::Failed {
            action: &trial(),
            error: &SandboxError::VerifierHealthCheck {
                tries: 1,
                kept: "/outputs/running/trial-3-fix-the-parser/verifier/health-check".into(),
            },
        }
        .to_string(),
        "the trial of harness 3 on task fix-the-parser was not verified: the health check of \
         its verifier sandbox did not pass in 1 try, so the verifier was not run, and \
         /outputs/running/trial-3-fix-the-parser/verifier/health-check holds what the health \
         check printed the last time it ran"
    );
}

#[test]
fn a_harness_that_was_never_started_is_said_with_what_stopped_it_from_being_started() {
    assert_eq!(
        Said::Failed {
            action: &trial(),
            error: &SandboxError::TaskHealthCheck {
                tries: 1,
                kept: "/outputs/running/trial-3-fix-the-parser/health-check".into(),
            },
        }
        .to_string(),
        "the trial of harness 3 on task fix-the-parser was not carried out: the health check \
         of the task did not pass in 1 try, so the harness was not started, and \
         /outputs/running/trial-3-fix-the-parser/health-check holds what the health check \
         printed the last time it ran"
    );
}
