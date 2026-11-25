use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use nix::unistd::{Gid, Uid};

use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given::{self, LAUNCH, TIMER};
use crate::containers::{self, ContainerError, Creation, INSTRUCTION, Launch};
use crate::file_system::tests::double::Memory;
use crate::genealogy::{CreationOutcome, Experiment};
use crate::model::Model;
use crate::outputs_directory::OutputsDirectory;

const BUILT: Launch<'static> = Launch {
    build_command: Some("make"),
    ..LAUNCH
};

const KEYED: Launch<'static> = Launch {
    model: Model {
        key: Some(test_inputs::MODEL_KEY),
        ..LAUNCH.model
    },
    ..LAUNCH
};

const PROCREATION: [&str; 2] = ["run", "run a harness"];
const BUILD: [&str; 2] = ["run", "make"];
const HARNESS: &str = ":/var/opt/cladenc/harness";
const WORK: &str = ":/var/opt/cladenc/work";
const EXPERIMENT: Experiment = Experiment {
    child: 1,
    progenitor: 0,
};

struct Procreated {
    directory: Memory,
    outputs_directory: OutputsDirectory,
    docker: Double,
    result: Result<Creation, ContainerError>,
}

impl Procreated {
    fn outcome(&self) -> Option<CreationOutcome> {
        self.result.as_ref().ok().map(|creation| creation.outcome)
    }

    fn commands(&self) -> Vec<String> {
        self.docker.commands(&self.outputs_directory)
    }
}

fn procreated_by(docker: Double, launch: &Launch<'_>) -> Procreated {
    let (directory, outputs_directory) = given::made();
    let docker = docker.on(&directory);
    let result = containers::run_procreation(
        given::host(&docker, &outputs_directory),
        launch,
        1,
        0,
        &AtomicBool::new(false),
    );
    Procreated {
        directory,
        outputs_directory,
        docker,
        result,
    }
}

fn taking(docker: Double, needles: &[&str], milliseconds: u64) -> Double {
    docker.doing(needles, move |_, _| {
        thread::sleep(Duration::from_millis(milliseconds));
    })
}

fn changing_what_it_runs_from() -> Double {
    Double::default().doing(&PROCREATION, |files, asked| {
        let runs_from = asked
            .arguments
            .iter()
            .find_map(|argument| argument.strip_suffix(HARNESS));
        if let Some(runs_from) = runs_from {
            files
                .write(Path::new(runs_from).join("harness.sh"), "changed\n")
                .unwrap();
        }
    })
}

fn removing_what_the_progenitor_added() -> Double {
    Double::default()
        .changing()
        .doing(&PROCREATION, |files, asked| {
            let work = asked
                .arguments
                .iter()
                .find_map(|argument| argument.strip_suffix(WORK));
            if let Some(work) = work.filter(|_| asked.kept.ends_with("baseline-3")) {
                files.remove_file(Path::new(work).join("changed")).unwrap();
            }
        })
}

fn control_made_by(docker: Double) -> Procreated {
    let (directory, outputs_directory) = given::made();
    let docker = docker.on(&directory);
    let result = containers::run_baseline(
        given::host(&docker, &outputs_directory),
        &LAUNCH,
        2,
        EXPERIMENT,
        &AtomicBool::new(false),
    );
    Procreated {
        directory,
        outputs_directory,
        docker,
        result,
    }
}

#[test]
fn a_harness_procreates_in_a_container_that_holds_its_copy_its_work_and_its_trials() {
    let procreated = procreated_by(Double::default(), &LAUNCH);
    assert_eq!(
        procreated.commands(),
        [format!(
            "docker run --rm --pull never --name NAME --label cladenc.directed-evolution=OUTPUTS \
             --add-host host.docker.internal:host-gateway --user {}:{} \
             --workdir /var/opt/cladenc/work \
             --volume OUTPUTS/running/procreation-1/progenitor:/var/opt/cladenc/harness \
             --volume OUTPUTS/running/procreation-1/child:/var/opt/cladenc/work \
             --volume OUTPUTS/trials/0:/var/log/cladenc/trials:ro \
             --env CLADENC_HARNESS --env CLADENC_INSTRUCTION --env CLADENC_MODEL \
             --env CLADENC_MODEL_URL debian:stable-slim sh -c {TIMER} run a harness 35",
            Uid::effective(),
            Gid::effective()
        )]
    );
}

#[test]
fn a_progenitor_is_told_where_its_source_code_is_what_to_do_and_which_model_to_call() {
    let procreated = procreated_by(Double::default(), &LAUNCH);
    assert_eq!(
        procreated.docker.asked()[0].variables,
        [
            ("CLADENC_HARNESS", "/var/opt/cladenc/harness"),
            ("CLADENC_INSTRUCTION", INSTRUCTION),
            ("CLADENC_MODEL", test_inputs::MODEL),
            ("CLADENC_MODEL_URL", test_inputs::MODEL_URL),
        ]
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
    );
}

#[test]
fn a_progenitor_is_given_the_key_of_the_model_where_there_is_one() {
    let procreated = procreated_by(Double::default(), &KEYED);
    assert!(procreated.docker.asked()[0].variables.contains(&(
        "CLADENC_MODEL_KEY".to_owned(),
        test_inputs::MODEL_KEY.to_owned()
    )));
}

#[test]
fn what_a_progenitor_prints_is_kept_with_its_procreation() {
    let procreated = procreated_by(Double::default(), &LAUNCH);
    assert_eq!(
        procreated.docker.asked()[0].kept,
        procreated.outputs_directory.running_procreation(1)
    );
}

#[test]
fn a_progenitor_that_leaves_its_copy_as_it_was_has_made_no_child() {
    let procreated = procreated_by(Double::default(), &LAUNCH);
    assert_eq!(procreated.outcome(), Some(CreationOutcome::Unchanged));
}

#[test]
fn a_progenitor_that_changes_its_copy_has_created_a_child() {
    let procreated = procreated_by(Double::default().changing(), &LAUNCH);
    assert_eq!(procreated.outcome(), Some(CreationOutcome::Created));
}

#[test]
fn the_seconds_of_a_procreation_are_those_its_progenitor_ran() {
    let docker = taking(Double::default(), &PROCREATION, 20);
    let seconds = procreated_by(docker, &LAUNCH)
        .result
        .map(|creation| creation.seconds);
    assert!(seconds.is_ok_and(|seconds| seconds >= 0.02));
}

#[test]
fn the_seconds_of_a_procreation_leave_out_those_the_build_of_its_child_ran() {
    let docker = taking(Double::default().changing(), &BUILD, 500);
    let seconds = procreated_by(docker, &BUILT)
        .result
        .map(|creation| creation.seconds);
    assert!(seconds.is_ok_and(|seconds| seconds < 0.5));
}

#[test]
fn the_seconds_of_a_baseline_are_those_the_primogenitor_ran() {
    let docker = taking(Double::default(), &PROCREATION, 20);
    let seconds = control_made_by(docker)
        .result
        .map(|creation| creation.seconds);
    assert!(seconds.is_ok_and(|seconds| seconds >= 0.02));
}

#[test]
fn a_progenitor_that_changes_only_the_source_code_it_runs_from_has_made_no_child() {
    let procreated = procreated_by(changing_what_it_runs_from(), &LAUNCH);
    assert_eq!(procreated.outcome(), Some(CreationOutcome::Unchanged));
}

#[test]
fn the_copy_that_a_progenitor_changed_becomes_the_child() {
    let procreated = procreated_by(Double::default().changing(), &LAUNCH);
    assert!(
        procreated
            .directory
            .is_file(procreated.outputs_directory.harness(1).join("changed"))
    );
}

#[test]
fn the_outcome_of_a_procreation_is_kept_with_what_its_progenitor_printed() {
    let procreated = procreated_by(Double::default().changing(), &LAUNCH);
    let outcome = procreated.outputs_directory.procreation(1).join("outcome");
    assert_eq!(
        procreated.directory.read_to_string(outcome).unwrap(),
        "created\n"
    );
}

#[test]
fn a_progenitor_that_exits_with_another_status_than_zero_has_failed() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Exited(1)]);
    assert_eq!(
        procreated_by(docker, &LAUNCH).outcome(),
        Some(CreationOutcome::Failed)
    );
}

#[test]
fn a_progenitor_that_runs_past_the_time_limit_has_timed_out() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::TimedOut]);
    assert_eq!(
        procreated_by(docker, &LAUNCH).outcome(),
        Some(CreationOutcome::TimedOut)
    );
}

#[test]
fn the_container_of_a_progenitor_that_runs_past_the_time_limit_is_removed() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::TimedOut]);
    assert_eq!(
        procreated_by(docker, &LAUNCH).commands().pop().as_deref(),
        Some("docker rm --force NAME")
    );
}

#[test]
fn a_progenitor_that_leaves_something_that_is_not_a_harness_file_has_failed() {
    let docker = given::leaving_a_socket(Double::default(), &PROCREATION, WORK);
    assert_eq!(
        procreated_by(docker, &LAUNCH).outcome(),
        Some(CreationOutcome::Failed)
    );
}

#[test]
fn a_procreation_that_is_stopped_was_stopped() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Stopped]);
    assert!(matches!(
        procreated_by(docker, &LAUNCH).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn a_procreation_that_docker_does_not_start_names_the_file_that_holds_what_docker_said() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Exited(125)]);
    let procreated = procreated_by(docker, &LAUNCH);
    let said = procreated
        .outputs_directory
        .running_procreation(1)
        .join("stderr");
    assert!(matches!(&procreated.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn a_procreation_whose_record_cannot_be_put_in_its_place_is_not_carried_out() {
    let (directory, outputs_directory) = given::made();
    directory
        .create_dir_all(outputs_directory.procreation(1).join("in the way"))
        .unwrap();
    let docker = Double::default().on(&directory);
    let result = containers::run_procreation(
        given::host(&docker, &outputs_directory),
        &LAUNCH,
        1,
        0,
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(ContainerError::Io(_))));
}

#[test]
fn a_child_whose_build_exits_with_zero_and_leaves_only_harness_files_is_created() {
    let procreated = procreated_by(Double::default().changing(), &BUILT);
    assert_eq!(procreated.outcome(), Some(CreationOutcome::Created));
}

#[test]
fn a_child_is_built_in_a_container_that_holds_what_its_progenitor_left() {
    let procreated = procreated_by(Double::default().changing(), &BUILT);
    assert_eq!(
        given::following("--volume", &procreated.commands()[1]),
        ["OUTPUTS/running/procreation-1/child:/var/opt/cladenc/harness"]
    );
}

#[test]
fn what_the_build_of_a_child_prints_is_kept_with_its_builds() {
    let procreated = procreated_by(Double::default().changing(), &BUILT);
    assert_eq!(
        procreated.docker.asked()[1].kept,
        procreated.outputs_directory.build(1)
    );
}

#[test]
fn a_copy_that_its_progenitor_left_as_it_was_is_not_built() {
    let procreated = procreated_by(Double::default(), &BUILT);
    assert_eq!(procreated.docker.asked().len(), 1);
}

#[test]
fn a_child_whose_build_exits_with_another_status_than_zero_is_not_created() {
    let docker = Double::default()
        .changing()
        .answering(&BUILD, &[Answer::Exited(2)]);
    assert_eq!(
        procreated_by(docker, &BUILT).outcome(),
        Some(CreationOutcome::BuildFailed)
    );
}

#[test]
fn a_child_whose_build_leaves_something_that_is_not_a_harness_file_is_not_created() {
    let docker = given::leaving_a_socket(Double::default().changing(), &BUILD, HARNESS);
    assert_eq!(
        procreated_by(docker, &BUILT).outcome(),
        Some(CreationOutcome::BuildFailed)
    );
}

#[test]
fn a_child_whose_build_runs_past_the_time_limit_of_the_procreation_has_timed_out() {
    let docker = Double::default()
        .changing()
        .answering(&BUILD, &[Answer::TimedOut]);
    assert_eq!(
        procreated_by(docker, &BUILT).outcome(),
        Some(CreationOutcome::TimedOut)
    );
}

#[test]
fn a_procreation_that_is_stopped_while_its_child_is_built_was_stopped() {
    let docker = Double::default()
        .changing()
        .answering(&BUILD, &[Answer::Stopped]);
    assert!(matches!(
        procreated_by(docker, &BUILT).result,
        Err(ContainerError::Stopped)
    ));
}

#[test]
fn the_child_of_a_progenitor_that_failed_is_not_built() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Exited(1)]);
    assert_eq!(procreated_by(docker, &BUILT).docker.asked().len(), 1);
}

#[test]
fn a_progenitor_that_failed_has_failed_whatever_the_build_of_its_child_would_do() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Exited(1)]);
    assert_eq!(
        procreated_by(docker, &BUILT).outcome(),
        Some(CreationOutcome::Failed)
    );
}

#[test]
fn the_time_limit_of_a_procreation_covers_its_progenitor_and_the_build_of_its_child_together() {
    let docker = taking(Double::default().changing(), &PROCREATION, 1100);
    let commands = procreated_by(docker, &BUILT).commands();
    let seconds_left = commands[1]
        .rsplit(' ')
        .next()
        .and_then(|seconds| seconds.parse::<u64>().ok());
    assert!(seconds_left.is_some_and(|seconds| seconds < 35));
}

#[test]
fn a_child_whose_build_docker_does_not_start_names_the_file_that_holds_what_docker_said() {
    let docker = Double::default()
        .changing()
        .answering(&BUILD, &[Answer::Exited(125)]);
    let procreated = procreated_by(docker, &BUILT);
    let said = procreated.outputs_directory.build(1).join("stderr");
    assert!(matches!(&procreated.result, Err(ContainerError::Docker(kept)) if *kept == said));
}

#[test]
fn a_progenitor_is_shown_the_source_code_and_the_trials_of_each_harness_it_descends_from() {
    let (directory, outputs_directory) = given::made();
    let docker = Double::default().changing();
    let docker = docker.on(&directory);
    let host = given::host(&docker, &outputs_directory);
    let stop = AtomicBool::new(false);
    containers::run_procreation(host, &LAUNCH, 1, 0, &stop).unwrap();
    containers::run_procreation(host, &LAUNCH, 2, 1, &stop).unwrap();
    assert_eq!(
        given::following("--volume", &docker.commands(&outputs_directory)[1]),
        [
            "OUTPUTS/running/procreation-2/progenitor:/var/opt/cladenc/harness",
            "OUTPUTS/running/procreation-2/child:/var/opt/cladenc/work",
            "OUTPUTS/trials/1:/var/log/cladenc/trials:ro",
            "OUTPUTS/harnesses/0:/var/log/cladenc/line/0/harness:ro",
            "OUTPUTS/trials/0:/var/log/cladenc/line/0/trials:ro",
        ]
    );
}

#[test]
fn the_singleton_control_group_harness_is_created_from_a_copy_of_the_source_of_the_progenitor() {
    let made = control_made_by(Double::default());
    assert_eq!(
        given::following("--volume", &made.commands()[0]),
        [
            "OUTPUTS/running/baseline-2/primogenitor:/var/opt/cladenc/harness",
            "OUTPUTS/running/baseline-2/singleton-control-group-harness:/var/opt/cladenc/work",
            "OUTPUTS/trials/0:/var/log/cladenc/trials:ro",
        ]
    );
}

#[test]
fn the_primogenitor_is_shown_each_child_its_progenitor_made_before_that_of_the_experiment() {
    let (directory, outputs_directory) = given::made();
    let docker = Double::default().changing();
    let docker = docker.on(&directory);
    let host = given::host(&docker, &outputs_directory);
    let stop = AtomicBool::new(false);
    containers::run_procreation(host, &LAUNCH, 1, 0, &stop).unwrap();
    containers::run_procreation(host, &LAUNCH, 2, 0, &stop).unwrap();
    let experiment = Experiment {
        child: 2,
        progenitor: 0,
    };
    containers::run_baseline(host, &LAUNCH, 3, experiment, &stop).unwrap();
    assert_eq!(
        given::following("--volume", &docker.commands(&outputs_directory)[2])[3..],
        [
            "OUTPUTS/harnesses/1:/var/log/cladenc/children/1/harness:ro",
            "OUTPUTS/trials/1:/var/log/cladenc/children/1/trials:ro",
            "OUTPUTS/procreations/1:/var/log/cladenc/children/1/procreation:ro",
        ]
    );
}

#[test]
fn a_primogenitor_that_leaves_the_copy_as_it_was_has_made_no_harness() {
    assert_eq!(
        control_made_by(Double::default()).outcome(),
        Some(CreationOutcome::Unchanged)
    );
}

#[test]
fn a_primogenitor_that_changes_the_copy_has_created_the_singleton_control_group_harness() {
    assert_eq!(
        control_made_by(Double::default().changing()).outcome(),
        Some(CreationOutcome::Created)
    );
}

#[test]
fn a_primogenitor_that_exits_with_another_status_than_zero_has_failed() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Exited(3)]);
    assert_eq!(
        control_made_by(docker).outcome(),
        Some(CreationOutcome::Failed)
    );
}

#[test]
fn what_the_primogenitor_leaves_is_compared_with_the_source_code_of_the_progenitor() {
    let (directory, outputs_directory) = given::made();
    let docker = removing_what_the_progenitor_added().on(&directory);
    let host = given::host(&docker, &outputs_directory);
    let stop = AtomicBool::new(false);
    containers::run_procreation(host, &LAUNCH, 1, 0, &stop).unwrap();
    containers::run_procreation(host, &LAUNCH, 2, 1, &stop).unwrap();
    let experiment = Experiment {
        child: 2,
        progenitor: 1,
    };
    let made = containers::run_baseline(host, &LAUNCH, 3, experiment, &stop);
    assert_eq!(
        made.ok().map(|creation| creation.outcome),
        Some(CreationOutcome::Created)
    );
}

#[test]
fn what_the_primogenitor_prints_is_kept_with_its_baseline() {
    let made = control_made_by(Double::default());
    assert_eq!(
        made.docker.asked()[0].kept,
        made.outputs_directory.running_baseline(2)
    );
}

#[test]
fn the_experiment_of_a_baseline_is_kept_with_what_the_primogenitor_printed() {
    let made = control_made_by(Double::default().changing());
    let child = made.outputs_directory.baseline(2).join("child");
    assert_eq!(made.directory.read_to_string(child).unwrap(), "1\n");
}

#[test]
fn a_baseline_that_is_stopped_was_stopped() {
    let docker = Double::default().answering(&PROCREATION, &[Answer::Stopped]);
    assert!(matches!(
        control_made_by(docker).result,
        Err(ContainerError::Stopped)
    ));
}
