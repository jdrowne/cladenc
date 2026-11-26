use std::fs;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::thread;

use cladenc::dataset::Dataset;
use cladenc::file_system::Disk;
use cladenc::genealogy::{CreationOutcome, TrialResult};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Host, Launch, Program, SandboxError};
use nix::unistd::Uid;
use tempfile::TempDir;

const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";

const MODEL: Model<'static> = Model {
    name: test_inputs::MODEL,
    url: test_inputs::MODEL_URL,
    key: None,
};

type Diversified = (
    TempDir,
    OutputsDirectory,
    Result<sandboxes::Creation, SandboxError>,
);

fn made_at(name: &str) -> (TempDir, OutputsDirectory) {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let outputs_directory = OutputsDirectory::create(
        crate::real::disk(),
        &directory.path().join(name),
        &primogenitor,
    )
    .unwrap();
    (directory, outputs_directory)
}

fn started_from(
    image: &str,
    run_command: &str,
    outputs_directory: &OutputsDirectory,
    time_limit: u64,
) -> Result<sandboxes::Creation, SandboxError> {
    let launch = Launch {
        image,
        run_command,
        build_command: None,
        model: MODEL,
        diversification_timeout: time_limit,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    sandboxes::run_diversification(
        Host {
            docker: &Program,
            outputs_directory,
        },
        &launch,
        1,
        0,
        &AtomicBool::new(false),
    )
}

fn diversified_within(run_command: &str, time_limit: u64) -> Diversified {
    let (directory, outputs_directory) = made_at("outputs");
    let result = started_from(
        test_inputs::IMAGE,
        run_command,
        &outputs_directory,
        time_limit,
    );
    (directory, outputs_directory, result)
}

fn diversified(run_command: &str) -> Diversified {
    diversified_within(run_command, 30)
}

static NOT_STARTED_BY_DOCKER: LazyLock<Diversified> = LazyLock::new(|| {
    let (directory, outputs_directory) = made_at("outputs");
    let result = started_from("Not An Image", "exit 0", &outputs_directory, 30);
    (directory, outputs_directory, result)
});

static IN_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE: LazyLock<Diversified> = LazyLock::new(|| {
    let (directory, outputs_directory) = made_at("outputs");
    let result = started_from(NOT_ON_THE_MACHINE, "exit 0", &outputs_directory, 30);
    (directory, outputs_directory, result)
});

fn what_docker_said_of(diversified: &Diversified) -> String {
    let (_directory, outputs_directory, _result) = diversified;
    fs::read_to_string(outputs_directory.running_diversification(1).join("stderr")).unwrap()
}

fn left_in_the_child(run_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = diversified(run_command);
    fs::read_to_string(outputs_directory.harness(1).join(file)).unwrap()
}

fn kept_of_the_diversification(run_command: &str, file: &str) -> String {
    let (_directory, outputs_directory, _result) = diversified(run_command);
    fs::read_to_string(outputs_directory.diversification(1).join(file)).unwrap()
}

fn left_by_a_harness_given_a_key(run_command: &str, file: &str) -> String {
    let (_directory, outputs_directory) = made_at("outputs");
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: Model {
            key: Some(test_inputs::MODEL_KEY),
            ..MODEL
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let _ = sandboxes::run_diversification(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        1,
        0,
        &AtomicBool::new(false),
    );
    fs::read_to_string(outputs_directory.harness(1).join(file)).unwrap()
}

fn heard_from_a_harness_that_says(said: &str) -> String {
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let hearing = thread::spawn(move || {
        let mut heard = String::new();
        let _ = listener
            .accept()
            .and_then(|(mut harness, _)| harness.read_to_string(&mut heard));
        heard
    });
    let _ = diversified(&format!(
        "bash -c 'echo {said} > /dev/tcp/host.docker.internal/{port}'"
    ));
    drop(TcpStream::connect(("127.0.0.1", port)));
    hearing.join().unwrap()
}

fn outcome_of(run_command: &str) -> CreationOutcome {
    let (_directory, _outputs_directory, result) = diversified(run_command);
    result.unwrap().outcome
}

#[test]
fn the_working_directory_of_a_harness_is_the_clone_that_becomes_its_child() {
    assert_eq!(
        left_in_the_child("pwd > where", "where"),
        "/var/opt/cladenc/work\n"
    );
}

#[test]
fn a_harness_is_told_where_its_source_code_is() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_HARNESS\" > told", "told"),
        "/var/opt/cladenc/harness\n"
    );
}

#[test]
fn the_source_code_a_harness_is_told_of_is_that_of_its_progenitor() {
    assert_eq!(
        left_in_the_child("cp \"$CLADENC_HARNESS/harness.sh\" copied", "copied"),
        "#!/bin/sh\necho harness\n"
    );
}

#[test]
fn a_harness_is_given_the_diversification_prompt_in_a_diversification() {
    assert_eq!(
        left_in_the_child("printf %s \"$CLADENC_INSTRUCTION\" > given", "given"),
        sandboxes::DIVERSIFICATION_PROMPT
    );
}

#[test]
fn a_harness_is_told_the_model_and_the_address_it_is_served_at() {
    assert_eq!(
        left_in_the_child("echo \"$CLADENC_MODEL $CLADENC_MODEL_URL\" > told", "told"),
        format!("{} {}\n", test_inputs::MODEL, test_inputs::MODEL_URL)
    );
}

#[test]
fn a_harness_is_given_the_key_the_model_is_called_with() {
    assert_eq!(
        left_by_a_harness_given_a_key("printf %s \"$CLADENC_MODEL_KEY\" > given", "given"),
        test_inputs::MODEL_KEY
    );
}

#[test]
fn a_harness_reaches_the_machine_cladenc_runs_on_as_host_docker_internal() {
    assert_eq!(heard_from_a_harness_that_says("reached"), "reached\n");
}

#[test]
fn a_harness_can_read_the_ended_trials_of_its_progenitor_and_none_still_running() {
    let (directory, outputs_directory) = made_at("outputs");
    let dataset = test_inputs::dataset(directory.path(), &["t1", "t2"]);
    outputs_directory
        .copy_dataset(&Dataset::read(&Disk, &dataset).unwrap())
        .unwrap();
    outputs_directory.start_trial(0, "dataset/t1").unwrap();
    outputs_directory
        .end_trial(0, "dataset/t1", TrialResult::Fail)
        .unwrap();
    outputs_directory.start_trial(0, "dataset/t2").unwrap();
    let _ = started_from(
        test_inputs::IMAGE,
        "ls /var/log/cladenc/trials /var/log/cladenc/trials/dataset > evidence",
        &outputs_directory,
        30,
    );
    assert_eq!(
        fs::read_to_string(outputs_directory.harness(1).join("evidence")).unwrap(),
        "/var/log/cladenc/trials:\ndataset\n\n/var/log/cladenc/trials/dataset:\nt1\n"
    );
}

#[test]
fn a_harness_cannot_write_into_the_trials_of_its_progenitor() {
    let (_directory, outputs_directory, _result) =
        diversified("touch /var/log/cladenc/trials/written");
    assert!(!outputs_directory.trials(0).join("written").exists());
}

#[test]
fn the_sandbox_of_a_diversification_holds_nothing_under_var_lib_that_is_named_after_cladenc() {
    assert_eq!(
        left_in_the_child(
            "ls -d /var/lib/cladenc > held 2> /dev/null; echo looked >> held",
            "held"
        ),
        "looked\n"
    );
}

#[test]
fn a_harness_finds_nothing_at_the_top_of_the_filesystem_that_its_image_does_not_hold() {
    assert_eq!(
        left_in_the_child("ls / > top", "top"),
        test_inputs::printed_in_a_sandbox_of(test_inputs::IMAGE, "ls /")
    );
}

#[test]
fn a_harness_runs_as_the_user_cladenc_runs_as() {
    assert_eq!(
        left_in_the_child("id -u > user", "user"),
        format!("{}\n", Uid::effective())
    );
}

#[test]
fn the_standard_input_of_a_harness_is_empty() {
    assert_eq!(left_in_the_child("cat > input", "input"), "");
}

#[test]
fn the_standard_output_of_a_harness_is_kept_with_its_diversification() {
    assert_eq!(kept_of_the_diversification("echo out", "stdout"), "out\n");
}

#[test]
fn the_standard_error_of_a_harness_is_kept_with_its_diversification() {
    assert_eq!(
        kept_of_the_diversification("echo err >&2", "stderr"),
        "err\n"
    );
}

#[test]
fn a_harness_that_changes_its_clone_and_exits_with_0_creates_its_child() {
    let (_directory, _outputs_directory, result) = diversified("touch changed");
    assert!(matches!(
        result,
        Ok(sandboxes::Creation {
            outcome: CreationOutcome::Created,
            ..
        })
    ));
}

#[test]
fn the_time_limit_removes_the_sandbox_of_a_harness() {
    let (_directory, outputs_directory, _result) = diversified_within(
        "hostname > id; sleep \
                                                                       300",
        5,
    );
    assert!(test_inputs::sandbox_is_gone(
        &outputs_directory.harness(1).join("id")
    ));
}

#[test]
fn a_sandbox_that_docker_does_not_start_is_an_error() {
    let (_directory, _outputs_directory, result) = &*NOT_STARTED_BY_DOCKER;
    assert!(matches!(result, Err(SandboxError::Docker(_))));
}

#[test]
fn what_docker_said_is_kept_under_running() {
    let (_directory, outputs_directory, _result) = &*NOT_STARTED_BY_DOCKER;
    let said =
        fs::read_to_string(outputs_directory.running_diversification(1).join("stderr")).unwrap();
    assert!(said.contains("invalid reference format"));
}

#[test]
fn docker_says_of_an_image_the_machine_does_not_hold_that_there_is_no_such_image() {
    assert!(what_docker_said_of(&IN_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE).contains("No such image"));
}

#[test]
fn docker_does_not_look_in_a_registry_for_an_image_the_machine_does_not_hold() {
    assert!(
        !what_docker_said_of(&IN_AN_IMAGE_THAT_IS_NOT_ON_THE_MACHINE)
            .contains("Unable to find image")
    );
}

#[test]
fn an_outputs_directory_whose_path_holds_a_comma_a_quote_and_a_space_can_be_mounted() {
    let (_directory, outputs_directory) = made_at("outputs,a\"b c");
    let result = started_from(test_inputs::IMAGE, "touch made", &outputs_directory, 30);
    assert!(matches!(
        result,
        Ok(sandboxes::Creation {
            outcome: CreationOutcome::Created,
            ..
        })
    ));
}

#[test]
fn a_harness_that_exits_with_0_and_leaves_its_clone_as_it_was_creates_no_child() {
    assert_eq!(outcome_of("exit 0"), CreationOutcome::Unchanged);
}

#[test]
fn a_harness_that_leaves_a_file_no_one_may_read_in_its_clone_creates_a_child_that_holds_it() {
    assert_eq!(
        left_in_the_child("echo kept > secret && chmod 000 secret", "secret"),
        "kept\n"
    );
}

#[test]
fn a_harness_that_leaves_a_directory_no_one_may_list_in_its_clone_creates_its_child() {
    assert_eq!(
        outcome_of("mkdir locked && touch locked/file && chmod 000 locked"),
        CreationOutcome::Created
    );
}
