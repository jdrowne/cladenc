use std::error::Error;
use std::io;
use std::path::PathBuf;

use crate::containers::{self, ContainerError};

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn what_was_stopped_is_worded() {
    assert_eq!(worded(ContainerError::Stopped), "was stopped");
}

#[test]
fn what_could_not_be_carried_out_is_worded_with_the_system_s_reason() {
    assert_eq!(
        worded(ContainerError::Io(io::Error::from_raw_os_error(2))),
        "could not be carried out: No such file or directory (os error 2)"
    );
}

#[test]
fn a_container_that_docker_did_not_start_is_worded_with_the_file_that_holds_what_docker_said() {
    assert_eq!(
        worded(ContainerError::Docker(PathBuf::from(
            "/outputs/running/procreation-1/stderr"
        ))),
        "could not be carried out: Docker did not start its container, \
         and /outputs/running/procreation-1/stderr holds what Docker said"
    );
}

#[test]
fn the_instruction_says_where_the_source_code_to_improve_is() {
    assert!(
        containers::INSTRUCTION
            .contains("The current directory holds a copy of your own source code")
    );
}

#[test]
fn the_instruction_says_where_the_attempts_are() {
    assert!(containers::INSTRUCTION.contains(
        "than you do. /var/log/cladenc/attempts holds the attempts you are shown: one directory \
         for each benchmark, and in it one directory for each task. In the directory of a task, "
    ));
}

#[test]
fn the_instruction_says_that_the_child_is_judged_by_tasks_the_progenitor_is_not_shown() {
    assert!(
        containers::INSTRUCTION
            .ends_with(" The new harness is judged by tasks that you are not shown.")
    );
}

#[test]
fn the_instruction_says_where_the_line_of_the_harness_and_the_children_it_made_before_are() {
    assert!(containers::INSTRUCTION.contains(
        " /var/log/cladenc/line holds one directory for each harness you descend from, named by \
         its id, and /var/log/cladenc/children one for each harness you made before, created or \
         not. "
    ));
}

#[test]
fn the_instruction_says_where_the_source_code_and_the_attempts_of_such_a_harness_are() {
    assert!(containers::INSTRUCTION.contains(
        " In the directory of such a harness, the directory named harness holds its source \
         code, the directory named attempts holds its attempts, laid out as yours are, "
    ));
}

#[test]
fn the_instruction_says_where_the_record_of_the_procreation_of_such_a_harness_is() {
    assert!(containers::INSTRUCTION.contains(
        "the directory named procreation holds what the harness that made it printed, in the \
         files named stdout and stderr, with the id of that harness in the file named \
         progenitor and one word in the file named outcome, created, unchanged, failed, \
         build_failed or timed_out, "
    ));
}

#[test]
fn the_instruction_says_where_what_the_build_of_such_a_harness_printed_is() {
    assert!(
        containers::INSTRUCTION
            .contains("and the directory named build holds what its build printed. ")
    );
}

#[test]
fn the_instruction_says_that_a_directory_that_would_hold_nothing_is_not_there() {
    assert!(
        containers::INSTRUCTION.contains(" A directory that would hold nothing is not there. ")
    );
}

#[test]
fn the_instruction_says_what_the_line_and_the_children_are_to_be_read_for() {
    assert!(containers::INSTRUCTION.contains(
        " Use all of them to find what to improve, and to see what was changed before and what \
         came of it. "
    ));
}

#[test]
fn the_instruction_asks_for_a_change_that_answers_several_attempts_and_not_one() {
    assert!(containers::INSTRUCTION.contains(
        " Make a change to how the harness works on any task: one that answers something you \
         saw in several of your attempts, not in one. "
    ));
}

#[test]
fn the_instruction_asks_for_source_code_that_names_no_task_and_holds_no_answer() {
    assert!(containers::INSTRUCTION.contains(
        " Leave nothing in the source code that names a task, a file of a task or the answer \
         to one. "
    ));
}

#[test]
fn the_instruction_says_that_taking_out_what_does_not_help_is_an_improvement() {
    assert!(
        containers::INSTRUCTION
            .contains(" Taking out what does not help is an improvement as well. ")
    );
}

#[test]
fn the_instruction_says_that_the_child_may_be_asked_to_improve_its_own_source_code_in_its_turn() {
    assert!(containers::INSTRUCTION.contains(
        "becomes a new harness, which attempts tasks in its turn. It may then be asked, as you \
         are now, to improve a copy of its own source code, so a change that makes it better \
         at that counts as well. "
    ));
}

#[test]
fn the_instruction_says_that_a_current_directory_left_as_it_was_makes_no_new_harness() {
    assert!(containers::INSTRUCTION.contains(
        " counts as well. Where you leave the current directory as it was, no new harness is \
         made: a change that you only print in your answer changes no file. The new harness "
    ));
}

#[test]
fn the_instruction_does_not_say_that_every_task_the_harness_attempted_is_there() {
    assert!(!containers::INSTRUCTION.contains("each task you have attempted"));
}

#[test]
fn the_instruction_says_what_each_attempt_keeps_of_the_task_and_of_the_harness() {
    assert!(containers::INSTRUCTION.contains(
        "the file named specification says what you were asked to do, the file named result \
         reads completed, not_completed or timed_out, the file named ending reads ended, or \
         stopped where you ran past the time limit of the task and were stopped, the directory \
         named work holds what you left in the working directory of the task, the file named \
         changes lists every path you added, changed or removed in the container of the task, \
         one to a line after A, C or D, "
    ));
}

#[test]
fn the_instruction_names_no_attempt_whose_harness_was_never_started() {
    assert!(!containers::INSTRUCTION.contains("not_started"));
}

#[test]
fn the_instruction_says_where_what_was_left_at_the_paths_the_task_lists_under_artifacts_is() {
    assert!(containers::INSTRUCTION.contains(
        "one to a line after A, C or D, the directory named artifacts holds what you left at \
         the paths that the task names to be kept, where it names any, and what you left in \
         /logs/artifacts, in logs/artifacts, "
    ));
}

#[test]
fn the_instruction_says_where_what_the_commands_of_the_task_printed_is() {
    assert!(containers::INSTRUCTION.contains(
        " the directory named collection-hooks holds what the commands that the task runs before \
         those paths are kept printed, one directory for each, where it runs any, the files \
         named stdout and stderr hold what you printed, "
    ));
}

#[test]
fn the_instruction_says_where_what_the_verifier_printed_and_left_is() {
    assert!(containers::INSTRUCTION.contains(
        "the files named stdout and stderr hold what you printed, the directory named verifier \
         holds what the verifier of the task printed, in the files named stdout and stderr, and \
         the files it left in logs/verifier, "
    ));
}

#[test]
fn the_instruction_says_where_what_the_health_check_printed_is() {
    assert!(containers::INSTRUCTION.contains(
        ", and the directory named health-check holds what the health check of the environment \
         of the task printed the last time it ran, in the files named stdout and stderr. Once "
    ));
}

#[test]
fn the_instruction_says_how_the_verifier_judged_the_attempt() {
    assert!(containers::INSTRUCTION.contains(
        " Once you had ended or were stopped, the verifier ran the tests of the task, which you \
         are not shown, on what you left, or, where the task has its verifier run in a container \
         of its own, on nothing but what the directory named artifacts holds, and wrote its \
         score in logs/verifier/reward.txt, or one score for each label in \
         logs/verifier/reward.json: the task was completed where every score is 1. "
    ));
}

#[test]
fn a_program_that_exited_with_another_status_is_worded() {
    assert_eq!(worded(ContainerError::Exit(2)), "exited with status 2");
}

#[test]
fn a_program_that_ran_past_its_time_limit_is_worded() {
    assert_eq!(worded(ContainerError::TimeLimit), "ran past its time limit");
}

#[test]
fn a_build_that_left_what_a_harness_cannot_hold_is_worded() {
    assert_eq!(
        worded(ContainerError::OtherFiles),
        "left something that is not a regular file, a directory or a symbolic link"
    );
}

#[test]
fn a_program_ended_by_a_signal_is_worded() {
    assert_eq!(worded(ContainerError::Signal(9)), "was ended by signal 9");
}

#[test]
fn a_verifier_that_was_never_run_is_worded_with_the_tries_and_where_the_health_check_is_kept() {
    assert_eq!(
        worded(ContainerError::VerifierHealthCheck {
            tries: 4,
            kept: PathBuf::from("/outputs/running/attempt-3-parsing+t1/verifier/health-check"),
        }),
        "was not verified: the health check of its verifier container did not pass in 4 \
         tries, so the verifier was not run, and \
         /outputs/running/attempt-3-parsing+t1/verifier/health-check holds what the health check \
         printed the last time it ran"
    );
}

#[test]
fn one_try_of_the_health_check_of_a_verifier_is_not_worded_as_several() {
    assert!(
        worded(ContainerError::VerifierHealthCheck {
            tries: 1,
            kept: PathBuf::from("/kept"),
        })
        .starts_with(
            "was not verified: the health check of its verifier container did not pass in 1 \
             try, so the verifier was not run, "
        )
    );
}

#[test]
fn a_harness_that_was_never_started_is_worded_with_the_tries_and_where_the_health_check_is() {
    assert_eq!(
        worded(ContainerError::TaskHealthCheck {
            tries: 4,
            kept: PathBuf::from("/outputs/running/attempt-3-parsing+t1/health-check"),
        }),
        "was not carried out: the health check of the task did not pass in 4 tries, so the \
         harness was not started, and /outputs/running/attempt-3-parsing+t1/health-check holds \
         what the health check printed the last time it ran"
    );
}

#[test]
fn one_try_of_the_health_check_of_a_task_is_not_worded_as_several() {
    assert!(
        worded(ContainerError::TaskHealthCheck {
            tries: 1,
            kept: PathBuf::from("/kept"),
        })
        .starts_with(
            "was not carried out: the health check of the task did not pass in 1 try, so the \
             harness was not started, "
        )
    );
}
