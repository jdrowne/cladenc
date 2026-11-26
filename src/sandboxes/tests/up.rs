use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::SandboxError;
use crate::sandboxes::program::{Ending, Limits};
use crate::sandboxes::sandbox::{Bringing, Run, Shell, Up};
use crate::sandboxes::tests::double::{Answer, Double};
use crate::sandboxes::tests::given::{self, EMPTIED, TIMER};

const RUNNING: &str = "/running";
const REMOVED: &str = "docker rm --force NAME";
const REFUSED: Answer = Answer::Refused("docker is not there");
const COPIED_OUT: [&str; 2] = ["cp", "--follow-link"];
const THROUGH: &str = "/directory/taken";
const KEPT: &str = "/directory/kept";

const PLAIN: Bringing<'static> = Bringing {
    image: test_inputs::IMAGE,
    working_directory: None,
    cpus: None,
    memory_megabytes: None,
    variables: &[],
    stays: Duration::from_secs(60),
};

const SET: Bringing<'static> = Bringing {
    working_directory: Some("/app"),
    cpus: Some(0.5),
    memory_megabytes: Some(4096),
    ..PLAIN
};

const SHELL: Shell<'static> = Shell {
    command: "true",
    environment: &[("MODE", "strict")],
};

fn outputs() -> OutputsDirectory {
    OutputsDirectory::at(Memory::default().shared(), Path::new("/outputs"))
}

fn start<'a>(
    docker: &'a Double,
    outputs_directory: &'a OutputsDirectory,
    bringing: &Bringing<'_>,
) -> Result<Up<'a>, SandboxError> {
    Up::start(
        given::host(docker, outputs_directory),
        bringing,
        Path::new(RUNNING),
    )
}

fn brought_up<'a>(docker: &'a Double, outputs_directory: &'a OutputsDirectory) -> Up<'a> {
    start(docker, outputs_directory, &PLAIN).unwrap()
}

fn ran(up: &Up<'_>) -> Result<Ending, SandboxError> {
    let stop = AtomicBool::new(false);
    let variables = [("KEY".to_owned(), "value".to_owned())];
    let run = Run {
        shell: &SHELL,
        user: "tester",
        variables: &variables,
    };
    let limits = Limits::within(Duration::from_secs(30), &stop);
    up.run(&run, Path::new(RUNNING), limits)
}

fn last_asked_of(docker: &Double, outputs_directory: &OutputsDirectory) -> Option<String> {
    docker.commands(outputs_directory).pop()
}

fn leaving_a_directory(docker: Double) -> Double {
    docker.doing(&COPIED_OUT, |directory, asked| {
        directory.create_dir_all(asked.kept.join("out")).unwrap();
        directory.write(asked.kept.join("out/a.txt"), "").unwrap();
        directory.set_mode(asked.kept.join("out"), 0o500).unwrap();
    })
}

fn leaving_a_file(docker: Double) -> Double {
    docker.doing(&COPIED_OUT, |directory, asked| {
        directory.create_dir_all(&asked.kept).unwrap();
        directory
            .write(asked.kept.join("report.txt"), "found\n")
            .unwrap();
    })
}

fn kept_of_what_is_at_a_path(leaving: impl FnOnce(Double) -> Double) -> Memory {
    let directory = Memory::default();
    let docker = leaving(Double::default().on(&directory));
    let outputs_directory = OutputsDirectory::at(directory.shared(), Path::new("/outputs"));
    let up = brought_up(&docker, &outputs_directory);
    up.copy_what_is_at("/app/out", Path::new(THROUGH), Path::new(KEPT));
    directory
}

#[test]
fn a_sandbox_that_stays_is_run_detached_and_sleeps_for_as_long_as_it_stays() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let _up = brought_up(&docker, &outputs_directory);
    assert_eq!(
        docker.commands(&outputs_directory)[0],
        "docker run --rm --pull never --name NAME --label cladenc.directed-evolution=OUTPUTS \
         --add-host host.docker.internal:host-gateway --env-file /dev/stdin --detach \
         debian:stable-slim sleep 60"
    );
}

#[test]
fn a_sandbox_is_given_its_working_directory_its_cpus_and_its_memory_in_megabytes() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let _up = start(&docker, &outputs_directory, &SET).unwrap();
    assert_eq!(
        docker.commands(&outputs_directory)[0],
        "docker run --rm --pull never --name NAME --label cladenc.directed-evolution=OUTPUTS \
         --add-host host.docker.internal:host-gateway --workdir /app --cpus 0.5 --memory 4096m \
         --env-file /dev/stdin --detach debian:stable-slim sleep 60"
    );
}

#[test]
fn a_whole_number_of_cpus_is_given_to_docker_with_no_fraction() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let bringing = Bringing {
        cpus: Some(4.0),
        ..PLAIN
    };
    let _up = start(&docker, &outputs_directory, &bringing).unwrap();
    assert_eq!(
        given::following("--cpus", &docker.commands(&outputs_directory)[0]),
        ["4"]
    );
}

#[test]
fn the_variables_of_a_sandbox_are_given_to_docker_on_its_standard_input() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let variables = [("MODE".to_owned(), "strict".to_owned())];
    let bringing = Bringing {
        variables: &variables,
        ..PLAIN
    };
    let _up = start(&docker, &outputs_directory, &bringing).unwrap();
    assert_eq!(docker.asked()[0].input, "MODE=strict\n");
}

#[test]
fn what_docker_says_of_a_sandbox_it_brings_up_is_kept_where_the_sandbox_runs() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let _up = brought_up(&docker, &outputs_directory);
    assert_eq!(docker.asked()[0].kept, Path::new("/running/stderr"));
}

#[test]
fn a_sandbox_that_docker_cannot_be_asked_to_bring_up_is_an_error() {
    let docker = Double::default().answering(&["--detach"], &[REFUSED]);
    let outputs_directory = outputs();
    assert!(matches!(
        start(&docker, &outputs_directory, &PLAIN),
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn a_sandbox_that_docker_cannot_be_asked_to_bring_up_is_removed_all_the_same() {
    let docker = Double::default().answering(&["--detach"], &[REFUSED]);
    let outputs_directory = outputs();
    drop(start(&docker, &outputs_directory, &PLAIN));
    assert_eq!(
        last_asked_of(&docker, &outputs_directory).as_deref(),
        Some(REMOVED)
    );
}

#[test]
fn a_sandbox_that_docker_does_not_bring_up_names_the_file_that_holds_what_docker_said() {
    let docker = Double::default().answering(&["--detach"], &[Answer::Exited(1)]);
    let outputs_directory = outputs();
    assert!(matches!(
        start(&docker, &outputs_directory, &PLAIN),
        Err(SandboxError::Docker(said)) if said == Path::new("/running/stderr")
    ));
}

#[test]
fn a_sandbox_whose_settings_docker_does_not_print_is_an_error() {
    let docker = Double::default().answering(&["inspect"], &[REFUSED]);
    let outputs_directory = outputs();
    assert!(matches!(
        start(&docker, &outputs_directory, &PLAIN),
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn a_sandbox_whose_settings_docker_does_not_print_is_removed() {
    let docker = Double::default().answering(&["inspect"], &[REFUSED]);
    let outputs_directory = outputs();
    drop(start(&docker, &outputs_directory, &PLAIN));
    assert_eq!(
        last_asked_of(&docker, &outputs_directory).as_deref(),
        Some(REMOVED)
    );
}

#[test]
fn docker_is_asked_for_the_user_and_the_working_directory_of_the_image_of_a_sandbox() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let _up = brought_up(&docker, &outputs_directory);
    assert_eq!(
        docker.commands(&outputs_directory)[1],
        "docker inspect --format {{println .Config.User}}{{.Config.WorkingDir}} NAME"
    );
}

#[test]
fn the_user_of_a_sandbox_is_the_one_its_image_sets() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("harness\n/app\n")]);
    let outputs_directory = outputs();
    assert_eq!(brought_up(&docker, &outputs_directory).user(), "harness");
}

#[test]
fn the_user_of_a_sandbox_whose_image_sets_none_is_root() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    assert_eq!(brought_up(&docker, &outputs_directory).user(), "0");
}

#[test]
fn the_work_of_a_sandbox_is_copied_out_of_the_working_directory_its_image_sets() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("harness\n/app\n")]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    up.keep_work(Path::new("/running/work"));
    assert_eq!(
        docker.commands(&outputs_directory)[2],
        "docker cp NAME:/app/. -"
    );
}

#[test]
fn the_work_of_a_sandbox_is_unpacked_where_it_is_kept() {
    let docker = Double::default().answering(&["inspect"], &[Answer::Printed("harness\n/app\n")]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    up.keep_work(Path::new("/running/work"));
    assert_eq!(docker.asked()[2].kept, Path::new("/running/work"));
}

#[test]
fn no_work_is_copied_out_of_a_sandbox_whose_working_directory_is_the_root() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.keep_work(Path::new("/running/work"));
    assert_eq!(docker.asked().len(), 2);
}

#[test]
fn a_command_is_run_in_a_sandbox_as_its_user_by_sh_under_a_timer() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    let _ended = ran(&up);
    assert_eq!(
        docker.commands(&outputs_directory)[2],
        format!(
            "docker exec --env-file /dev/stdin --env MODE --user tester NAME sh -c {TIMER} true 35"
        )
    );
}

#[test]
fn the_variables_of_a_command_run_in_a_sandbox_are_given_on_the_standard_input_of_docker() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    let _ended = ran(&up);
    assert_eq!(docker.asked()[2].input, "KEY=value\n");
}

#[test]
fn the_values_of_the_variables_of_a_shell_are_given_in_the_environment_of_docker() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    let _ended = ran(&up);
    assert_eq!(
        docker.asked()[2].variables,
        [("MODE".to_owned(), "strict".to_owned())]
    );
}

#[test]
fn a_command_that_docker_does_not_start_in_a_sandbox_names_what_docker_said() {
    let docker = Double::default().answering(&["exec"], &[Answer::Exited(125)]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert!(matches!(
        ran(&up),
        Err(SandboxError::Docker(said)) if said == Path::new("/running/stderr")
    ));
}

#[test]
fn the_exit_status_of_a_command_run_in_a_sandbox_is_that_of_the_command() {
    let docker = Double::default().answering(&["exec"], &[Answer::Exited(7)]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert!(matches!(ran(&up), Ok(Ending::Exited(7))));
}

#[test]
fn a_command_run_in_a_sandbox_past_its_time_limit_has_timed_out() {
    let docker = Double::default().answering(&["exec"], &[Answer::TimedOut]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert!(matches!(ran(&up), Ok(Ending::TimedOut)));
}

#[test]
fn a_directory_is_put_in_a_sandbox_emptied_first_and_then_given_to_its_owner() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.put(Path::new("/from"), "/to", "owner").unwrap();
    assert_eq!(
        docker.commands(&outputs_directory)[2..],
        [
            format!("docker exec --user 0 NAME sh -c {EMPTIED} sh /to"),
            "docker cp /from/. NAME:/to".to_owned(),
            "docker exec --user 0 NAME chown -R owner /to".to_owned(),
        ]
    );
}

#[test]
fn a_file_is_put_in_a_sandbox_where_room_is_made_for_it_and_then_given_to_its_owner() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.put_file(Path::new("/from/file"), "/to/file", "owner")
        .unwrap();
    assert_eq!(
        docker.commands(&outputs_directory)[2..],
        [
            r#"docker exec --user 0 NAME sh -c rm -rf "$1" && mkdir -p "${1%/*}/" sh /to/file"#,
            "docker cp /from/file NAME:/to/file",
            "docker exec --user 0 NAME chown -R owner /to/file",
        ]
    );
}

#[test]
fn a_directory_that_docker_does_not_copy_into_a_sandbox_is_an_error() {
    let docker = Double::default().answering(&["cp"], &[REFUSED]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert!(matches!(
        up.put(Path::new("/from"), "/to", "owner"),
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn what_docker_does_not_copy_into_a_sandbox_is_given_to_no_owner() {
    let docker = Double::default().answering(&["cp"], &[REFUSED]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    let _put = up.put(Path::new("/from"), "/to", "owner");
    assert_eq!(docker.asked().len(), 4);
}

#[test]
fn a_script_is_run_in_a_sandbox_as_root_with_its_arguments() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.run_as_root("mkdir \"$1\"", &["/made"]).unwrap();
    assert_eq!(
        docker.commands(&outputs_directory)[2],
        "docker exec --user 0 NAME sh -c mkdir \"$1\" sh /made"
    );
}

#[test]
fn a_script_that_docker_cannot_be_asked_to_run_as_root_is_an_error() {
    let docker = Double::default().answering(&["exec"], &[REFUSED]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert!(matches!(
        up.run_as_root("mkdir \"$1\"", &["/made"]),
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn every_program_of_a_sandbox_is_ended_as_root_with_the_signal_that_cannot_be_caught() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.end_everything();
    assert_eq!(
        docker.commands(&outputs_directory)[2],
        "docker exec --user 0 NAME sh -c kill -9 -1"
    );
}

#[test]
fn docker_is_asked_what_changed_in_a_sandbox() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    let _changes = up.changes();
    assert_eq!(docker.commands(&outputs_directory)[2], "docker diff NAME");
}

#[test]
fn the_changes_of_a_sandbox_are_the_lines_docker_prints() {
    let printed = [Answer::Printed("C /app\nA /app/new\n")];
    let docker = Double::default().answering(&["diff"], &printed);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert_eq!(up.changes(), ["C /app", "A /app/new"]);
}

#[test]
fn a_sandbox_whose_changes_docker_does_not_print_is_taken_to_have_none() {
    let docker = Double::default().answering(&["diff"], &[REFUSED]);
    let outputs_directory = outputs();
    let up = brought_up(&docker, &outputs_directory);
    assert!(up.changes().is_empty());
}

#[test]
fn docker_is_asked_to_print_an_archive_of_what_a_directory_of_a_sandbox_holds() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.copy_out_of("/logs/verifier", Path::new("/running/verifier"));
    assert_eq!(
        docker.commands(&outputs_directory)[2],
        "docker cp NAME:/logs/verifier/. -"
    );
}

#[test]
fn what_a_directory_of_a_sandbox_holds_is_unpacked_where_it_is_kept() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.copy_out_of("/logs/verifier", Path::new("/running/verifier"));
    assert_eq!(docker.asked()[2].kept, Path::new("/running/verifier"));
}

#[test]
fn docker_is_asked_to_print_an_archive_of_what_is_at_a_path_of_a_sandbox_with_a_link_followed() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.copy_what_is_at(
        "/logs/artifacts",
        Path::new("/running/taken"),
        Path::new("/running/artifacts"),
    );
    assert_eq!(
        docker.commands(&outputs_directory)[2],
        "docker cp --follow-link NAME:/logs/artifacts -"
    );
}

#[test]
fn what_is_at_a_path_of_a_sandbox_is_unpacked_apart_from_where_it_is_kept() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    let up = brought_up(&docker, &outputs_directory);
    up.copy_what_is_at(
        "/logs/artifacts",
        Path::new("/running/taken"),
        Path::new("/running/artifacts"),
    );
    assert_eq!(docker.asked()[2].kept, Path::new("/running/taken"));
}

#[test]
fn a_directory_at_a_path_of_a_sandbox_is_kept_under_the_name_it_is_given() {
    let directory = kept_of_what_is_at_a_path(leaving_a_directory);
    assert_eq!(directory.names_in(KEPT), ["a.txt"]);
}

#[test]
fn a_file_at_a_path_of_a_sandbox_is_kept_under_the_name_it_is_given() {
    let directory = kept_of_what_is_at_a_path(leaving_a_file);
    assert_eq!(
        directory.read_to_string(KEPT).ok().as_deref(),
        Some("found\n")
    );
}

#[test]
fn a_directory_copied_out_of_a_sandbox_that_cannot_be_written_in_is_kept_as_one_that_can() {
    let directory = kept_of_what_is_at_a_path(leaving_a_directory);
    assert_eq!(directory.mode(KEPT) & 0o700, 0o700);
}

#[test]
fn the_place_where_a_path_of_a_sandbox_was_unpacked_is_removed() {
    let directory = kept_of_what_is_at_a_path(leaving_a_directory);
    assert!(!directory.exists(THROUGH));
}

#[test]
fn what_was_left_where_a_path_of_a_sandbox_is_unpacked_is_not_kept_in_its_place() {
    let directory = kept_of_what_is_at_a_path(|docker| {
        leaving_a_directory(docker).doing(&["inspect"], |directory, _| {
            directory.create_dir_all(THROUGH).unwrap();
            directory
                .write(Path::new(THROUGH).join("earlier"), "")
                .unwrap();
        })
    });
    assert!(directory.is_dir(KEPT));
}

#[test]
fn a_path_that_docker_does_not_copy_out_of_a_sandbox_keeps_nothing() {
    let directory = kept_of_what_is_at_a_path(|docker| docker.answering(&COPIED_OUT, &[REFUSED]));
    assert!(!directory.exists(KEPT));
}

#[test]
fn a_sandbox_that_is_no_longer_used_is_removed() {
    let (docker, outputs_directory) = (Double::default(), outputs());
    drop(brought_up(&docker, &outputs_directory));
    assert_eq!(
        last_asked_of(&docker, &outputs_directory).as_deref(),
        Some(REMOVED)
    );
}
