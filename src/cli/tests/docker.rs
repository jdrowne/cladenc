use crate::cli::tests::given::{self, Ran};
use crate::cli::{Command, ResumeOptions};
use crate::file_system::tests::double::Memory;
use crate::journal;
use crate::sandboxes::tests::double::{Answer, Double};

const REFUSED: Answer = Answer::Refused("docker is not there");
const UNREACHABLE: Answer = Answer::Refused("Cannot connect to the Docker daemon");
const LEFT: Answer = Answer::Printed("0123456789ab\n");
const UNREMOVED: Answer = Answer::Refused("cannot remove sandbox 0123456789ab");
const ASKS_FOR_A_CPU: &str = "schema_version = \"1.4\"\n\n[environment]\ncpus = 1\n";

fn started_with(docker: Double, directory: &Memory) -> Ran {
    let options = given::options(directory);
    given::executed(docker, directory, Command::Start(Box::new(options)))
}

fn stopped() -> Memory {
    let directory = Memory::default();
    let docker = Double::default().answering(&["--detach"], &[REFUSED]);
    started_with(docker, &directory);
    directory
}

fn resumed_with(docker: Double, directory: &Memory) -> Ran {
    let outputs_dir = directory.path().join("outputs");
    let resume = Command::Resume(ResumeOptions { outputs_dir });
    given::executed(docker, directory, resume)
}

fn unreachable() -> Double {
    Double::default().answering(&["ps"], &[UNREACHABLE])
}

fn unremoving() -> Double {
    Double::default()
        .answering(&["ps"], &[LEFT])
        .answering(&["rm"], &[UNREMOVED])
}

fn ended_by_a_signal() -> Ran {
    let docker = Double::default().answering(&["--detach"], &[Answer::Signal(15)]);
    started_with(docker, &Memory::default())
}

fn with_a_docker_that_says_nothing_of_the_machine() -> Ran {
    let directory = Memory::default();
    let options = given::options(&directory);
    let task_toml = directory.path().join("benchmark/t2/task.toml");
    directory.write(task_toml, ASKS_FOR_A_CPU).unwrap();
    let start = Command::Start(Box::new(options));
    given::executed(Double::default(), &directory, start)
}

fn said_last(ran: &Ran) -> &str {
    ran.errors.lines().last().unwrap_or_default()
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_gives_exit_status_2() {
    assert_eq!(resumed_with(unreachable(), &stopped()).status, 2);
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_says_what_docker_said() {
    assert_eq!(
        said_last(&resumed_with(unreachable(), &stopped())),
        "cladenc: the sandboxes that the directed evolution left running cannot be removed: \
         Cannot connect to the Docker daemon"
    );
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_prints_nothing_on_standard_output() {
    assert_eq!(resumed_with(unreachable(), &stopped()).output, "");
}

#[test]
fn a_resume_whose_docker_cannot_be_asked_continues_nothing() {
    let directory = stopped();
    let journal = journal::path(&directory.path().join("outputs"));
    let before = directory.read_to_string(&journal).ok();
    resumed_with(unreachable(), &directory);
    assert_eq!(
        (before.is_some(), directory.read_to_string(&journal).ok()),
        (true, before)
    );
}

#[test]
fn a_resume_whose_docker_does_not_remove_a_sandbox_gives_exit_status_2() {
    assert_eq!(resumed_with(unremoving(), &stopped()).status, 2);
}

#[test]
fn a_resume_whose_docker_does_not_remove_a_sandbox_says_what_docker_said() {
    assert_eq!(
        said_last(&resumed_with(unremoving(), &stopped())),
        "cladenc: the sandboxes that the directed evolution left running cannot be removed: \
         cannot remove sandbox 0123456789ab"
    );
}

#[test]
fn a_start_whose_docker_a_signal_ended_gives_exit_status_1() {
    assert_eq!(ended_by_a_signal().status, 1);
}

#[test]
fn a_start_whose_docker_a_signal_ended_says_which_trial_and_which_signal() {
    assert_eq!(
        said_last(&ended_by_a_signal()),
        "cladenc: the directed evolution stopped before its end: the trial of harness 0 on \
         task benchmark/t1 was ended by signal 15"
    );
}

#[test]
fn a_start_whose_docker_does_not_say_what_the_machine_has_gives_exit_status_2() {
    assert_eq!(with_a_docker_that_says_nothing_of_the_machine().status, 2);
}

#[test]
fn a_start_whose_docker_does_not_say_what_the_machine_has_is_reported_last() {
    assert_eq!(
        said_last(&with_a_docker_that_says_nothing_of_the_machine()),
        "cladenc: what the machine has cannot be told: Docker did not say how many CPUs and how \
         much memory it has"
    );
}
