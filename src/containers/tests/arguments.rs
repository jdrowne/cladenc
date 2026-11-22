use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use nix::unistd::{Gid, Uid};

use crate::containers::container::{self, Container, Mount, Shell};
use crate::containers::program::Limits;
use crate::outputs_directory::OutputsDirectory;

const NAME: &str = "cladenc-0123456789abcdef";

const SHELL: Shell<'static> = Shell {
    command: "run a harness",
    environment: &[("MODEL_KEY", "secret"), ("MODEL", "a-model")],
};

fn of_the_harness() -> Mount<'static> {
    Mount {
        from: Path::new("/outputs/harnesses/3"),
        to: "/opt/cladenc/harness",
        read_only: true,
    }
}

fn of_the_child() -> Mount<'static> {
    Mount {
        from: Path::new("/outputs/running/procreation-4/child"),
        to: "/var/opt/cladenc/child",
        read_only: false,
    }
}

fn arguments(command: &Command) -> Vec<String> {
    command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect()
}

fn run_with(mounts: &[Mount<'_>]) -> Vec<String> {
    let outputs_directory = OutputsDirectory::at(Path::new("/outputs"));
    let container = Container {
        outputs_directory: &outputs_directory,
        image: test_inputs::IMAGE,
        mounts,
        working_directory: "/var/opt/cladenc/child",
    };
    arguments(&container.docker_run(NAME))
}

#[test]
fn a_container_is_run_under_its_name_for_its_directed_evolution_as_the_user_that_runs_cladenc() {
    let user = format!("{}:{}", Uid::effective(), Gid::effective());
    assert_eq!(
        run_with(&[]),
        [
            "run",
            "--rm",
            "--pull",
            "never",
            "--name",
            NAME,
            "--label",
            "cladenc.directed-evolution=/outputs",
            "--add-host",
            "host.docker.internal:host-gateway",
            "--user",
            user.as_str(),
            "--workdir",
            "/var/opt/cladenc/child",
        ]
    );
}

#[test]
fn a_container_is_given_each_of_its_mounts_as_a_volume_after_everything_else() {
    let given = run_with(&[of_the_harness(), of_the_child()]);
    assert_eq!(
        given[14..],
        [
            "--volume",
            "/outputs/harnesses/3:/opt/cladenc/harness:ro",
            "--volume",
            "/outputs/running/procreation-4/child:/var/opt/cladenc/child",
        ]
    );
}

#[test]
fn a_mount_that_is_only_read_is_written_as_its_two_paths_and_ro() {
    assert_eq!(
        of_the_harness().written(),
        "/outputs/harnesses/3:/opt/cladenc/harness:ro"
    );
}

#[test]
fn a_mount_that_can_be_written_to_is_written_as_its_two_paths_alone() {
    assert_eq!(
        of_the_child().written(),
        "/outputs/running/procreation-4/child:/var/opt/cladenc/child"
    );
}

#[test]
fn docker_is_given_the_name_of_each_variable_of_a_shell_and_not_its_value() {
    let mut docker = Command::new("docker");
    SHELL.given_to(&mut docker);
    assert_eq!(arguments(&docker), ["--env", "MODEL_KEY", "--env", "MODEL"]);
}

#[test]
fn docker_finds_the_value_of_each_variable_of_a_shell_in_its_own_environment() {
    let mut docker = Command::new("docker");
    SHELL.given_to(&mut docker);
    let mut variables: Vec<(String, Option<String>)> = docker
        .get_envs()
        .map(|(name, value)| {
            (
                name.to_string_lossy().into_owned(),
                value.map(|value| value.to_string_lossy().into_owned()),
            )
        })
        .collect();
    variables.sort();
    assert_eq!(
        variables,
        [
            ("MODEL".to_owned(), Some("a-model".to_owned())),
            ("MODEL_KEY".to_owned(), Some("secret".to_owned())),
        ]
    );
}

#[test]
fn the_command_of_a_shell_is_given_to_sh_under_a_timer_with_the_seconds_of_the_timer() {
    let stop = AtomicBool::new(false);
    let mut docker = Command::new("docker");
    SHELL.timed(&mut docker, Limits::within(Duration::ZERO, &stop));
    let mut given = arguments(&docker);
    let _timer = given.remove(2);
    assert_eq!(given, ["sh", "-c", "run a harness", "5"]);
}

#[test]
fn a_timer_whose_deadline_has_passed_is_given_the_five_seconds_of_leeway_alone() {
    assert_eq!(container::seconds_of_the_timer(Instant::now()), 5);
}

#[test]
fn a_timer_is_given_the_seconds_left_with_a_part_of_a_second_counted_whole_and_five_more() {
    let deadline = Instant::now() + Duration::from_millis(59_900);
    assert_eq!(container::seconds_of_the_timer(deadline), 65);
}
