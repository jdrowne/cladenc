use std::env;
use std::fs;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::dataset::{Dataset, TaskConfiguration};
use cladenc::file_system::Disk;
use cladenc::genealogy::TrialResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{self, Environment, Host, Launch, Program, SandboxError};
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const TASK: &str = "dataset/t1";
const AS_1000: &str = "[agent]\nuser = 1000\n";
const VERIFIED_AS_1000: &str = "[verifier]\nuser = 1000\n";
const ELSEWHERE: &str = "[environment]\nworkdir = \"/srv/elsewhere\"\n";
const WITH_A_MODE: &str = "[environment.env]\nMODE = \"strict\"\n";
const WITH_A_JUDGE: &str = "[verifier.env]\nJUDGE = \"strict\"\n";
const IN_AN_IMAGE: &str = "[environment]\ndocker_image = \"debian:stable-slim\"\n";

struct Trialled {
    _directory: TempDir,
    _built: Option<Image>,
    outputs_directory: OutputsDirectory,
    result: Result<sandboxes::Trialled, SandboxError>,
}

struct Asked {
    trialled: Trialled,
    of_the_harness: &'static [&'static str],
    of_the_verifier: &'static [&'static str],
}

fn environment_of(outputs_directory: &OutputsDirectory) -> (Environment, Option<Image>) {
    let configuration = TaskConfiguration::read(&Disk, &outputs_directory.task(TASK)).unwrap();
    let built = configuration.docker_image.is_none().then(|| {
        let image = test_inputs::image();
        sandboxes::build_environment(
            Host {
                docker: &Program,
                outputs_directory,
            },
            TASK,
            &image.name,
            configuration.build_time_limit,
            &AtomicBool::new(false),
        )
        .unwrap();
        image
    });
    let image = built
        .as_ref()
        .map(|built| built.name.clone())
        .or_else(|| configuration.docker_image.clone())
        .unwrap();
    (
        Environment {
            image,
            configuration,
            verifier: None,
        },
        built,
    )
}

fn trialled(task_toml: &str, run_command: &str, verifier: &str) -> Trialled {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let dataset = test_inputs::dataset(directory.path(), &["t1"]);
    fs::write(dataset.join("t1/task.toml"), task_toml).unwrap();
    outputs_directory
        .copy_dataset(&Dataset::read(&Disk, &dataset).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", verifier);
    let (environment, built) = environment_of(&outputs_directory);
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let result = sandboxes::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        &launch,
        sandboxes::Trial {
            harness: 0,
            task: TASK,
            environment: &environment,
        },
        &AtomicBool::new(false),
        &|_| (),
    );
    Trialled {
        _directory: directory,
        _built: built,
        outputs_directory,
        result,
    }
}

fn kept(trialled: &Trialled, file: &str) -> String {
    fs::read_to_string(trialled.outputs_directory.trial(0, TASK).join(file)).unwrap()
}

fn printed_by_the_harness(task_toml: &str, run_command: &str) -> String {
    kept(&trialled(task_toml, run_command, FINISHED), "stdout")
}

fn printed_by_the_verifier(task_toml: &str, verifier: &str) -> String {
    kept(&trialled(task_toml, FINISH, verifier), "verifier/stdout")
}

fn asked(
    task_toml: &str,
    of_the_harness: &'static [&'static str],
    of_the_verifier: &'static [&'static str],
) -> Asked {
    Asked {
        trialled: trialled(
            task_toml,
            &test_inputs::one_after_another(of_the_harness),
            &test_inputs::one_after_another(of_the_verifier),
        ),
        of_the_harness,
        of_the_verifier,
    }
}

static WITH_A_USER_FOR_THE_HARNESS: LazyLock<Asked> = LazyLock::new(|| {
    asked(
        AS_1000,
        &["id -u", "stat -c %u /var/opt/cladenc/harness/harness.sh"],
        &["id -u"],
    )
});

static WITH_A_USER_FOR_THE_VERIFIER: LazyLock<Asked> = LazyLock::new(|| {
    asked(
        VERIFIED_AS_1000,
        &["id -u", FINISH],
        &["id -u", "stat -c %u /tests/test.sh", FINISHED],
    )
});

static WITH_A_WORKING_DIRECTORY: LazyLock<Asked> =
    LazyLock::new(|| asked(ELSEWHERE, &["pwd", "echo left > note"], &["pwd"]));

static WITH_A_VARIABLE_OF_THE_ENVIRONMENT: LazyLock<Asked> =
    LazyLock::new(|| asked(WITH_A_MODE, &["echo \"$MODE\""], &["echo \"$MODE\""]));

static WITH_A_VARIABLE_FOR_THE_VERIFIER: LazyLock<Asked> = LazyLock::new(|| {
    asked(
        WITH_A_JUDGE,
        &["echo \"${JUDGE-not given}\""],
        &["echo \"$JUDGE\""],
    )
});

static WITH_AN_IMAGE: LazyLock<Asked> =
    LazyLock::new(|| asked(IN_AN_IMAGE, &["pwd; ls /app"], &["pwd"]));

fn found_by_the_harness(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_harness,
        &kept(&asked.trialled, "stdout"),
    )
}

fn found_by_the_verifier(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_verifier,
        &kept(&asked.trialled, "verifier/stdout"),
    )
}

#[test]
fn the_harness_runs_as_the_user_the_task_names_by_number() {
    assert_eq!(
        found_by_the_harness(&WITH_A_USER_FOR_THE_HARNESS, "id -u"),
        "1000\n"
    );
}

#[test]
fn the_harness_runs_as_the_user_the_task_names_by_name() {
    assert_eq!(
        printed_by_the_harness("[agent]\nuser = \"nobody\"\n", "id -un"),
        "nobody\n"
    );
}

#[test]
fn the_source_code_of_the_harness_belongs_to_the_user_the_task_names() {
    assert_eq!(
        found_by_the_harness(
            &WITH_A_USER_FOR_THE_HARNESS,
            "stat -c %u /var/opt/cladenc/harness/harness.sh"
        ),
        "1000\n"
    );
}

#[test]
fn the_verifier_of_a_task_that_names_a_user_for_the_harness_alone_runs_as_the_user_of_the_environment()
 {
    assert_eq!(
        found_by_the_verifier(&WITH_A_USER_FOR_THE_HARNESS, "id -u"),
        "0\n"
    );
}

#[test]
fn the_verifier_runs_as_the_user_the_task_names_for_it() {
    assert_eq!(
        found_by_the_verifier(&WITH_A_USER_FOR_THE_VERIFIER, "id -u"),
        "1000\n"
    );
}

#[test]
fn the_harness_of_a_task_that_names_a_user_for_the_verifier_alone_runs_as_the_user_of_the_environment()
 {
    assert_eq!(
        found_by_the_harness(&WITH_A_USER_FOR_THE_VERIFIER, "id -u"),
        "0\n"
    );
}

#[test]
fn the_tests_of_the_task_belong_to_the_user_the_task_names_for_the_verifier() {
    assert_eq!(
        found_by_the_verifier(&WITH_A_USER_FOR_THE_VERIFIER, "stat -c %u /tests/test.sh"),
        "1000\n"
    );
}

#[test]
fn a_verifier_that_runs_as_another_user_than_the_harness_can_write_its_verdict() {
    assert!(matches!(
        WITH_A_USER_FOR_THE_VERIFIER.trialled.result,
        Ok(sandboxes::Trialled {
            result: TrialResult::Pass,
            ..
        })
    ));
}

#[test]
fn a_trial_whose_task_names_a_user_that_its_environment_does_not_hold_is_an_error() {
    assert!(matches!(
        trialled("[agent]\nuser = \"nobody-here\"\n", FINISH, FINISHED).result,
        Err(SandboxError::Io(_))
    ));
}

#[test]
fn the_harness_works_in_the_working_directory_the_task_names() {
    assert_eq!(
        found_by_the_harness(&WITH_A_WORKING_DIRECTORY, "pwd"),
        "/srv/elsewhere\n"
    );
}

#[test]
fn the_verifier_works_in_the_working_directory_the_task_names() {
    assert_eq!(
        found_by_the_verifier(&WITH_A_WORKING_DIRECTORY, "pwd"),
        "/srv/elsewhere\n"
    );
}

#[test]
fn what_the_harness_leaves_in_the_working_directory_the_task_names_is_kept_as_its_work() {
    assert_eq!(
        kept(&WITH_A_WORKING_DIRECTORY.trialled, "work/note"),
        "left\n"
    );
}

#[test]
fn the_harness_is_given_the_variables_of_the_environment_of_the_task() {
    assert_eq!(
        found_by_the_harness(&WITH_A_VARIABLE_OF_THE_ENVIRONMENT, "echo \"$MODE\""),
        "strict\n"
    );
}

#[test]
fn the_verifier_is_given_the_variables_of_the_environment_of_the_task() {
    assert_eq!(
        found_by_the_verifier(&WITH_A_VARIABLE_OF_THE_ENVIRONMENT, "echo \"$MODE\""),
        "strict\n"
    );
}

#[test]
fn the_verifier_is_given_the_variables_the_task_names_for_it() {
    assert_eq!(
        found_by_the_verifier(&WITH_A_VARIABLE_FOR_THE_VERIFIER, "echo \"$JUDGE\""),
        "strict\n"
    );
}

#[test]
fn the_harness_is_not_given_the_variables_the_task_names_for_the_verifier() {
    assert_eq!(
        found_by_the_harness(
            &WITH_A_VARIABLE_FOR_THE_VERIFIER,
            "echo \"${JUDGE-not given}\""
        ),
        "not given\n"
    );
}

#[test]
fn a_variable_the_task_names_for_the_verifier_takes_the_place_of_one_of_the_environment() {
    assert_eq!(
        printed_by_the_verifier(
            "[environment.env]\nMODE = \"strict\"\n[verifier.env]\nMODE = \"lenient\"\n",
            "echo \"$MODE\""
        ),
        "lenient\n"
    );
}

#[test]
fn a_variable_is_given_the_value_it_has_where_cladenc_runs() {
    assert_eq!(
        printed_by_the_harness(
            "[environment.env]\nSEARCHED = \"${PATH}\"\n",
            "echo \"$SEARCHED\""
        ),
        format!("{}\n", env::var("PATH").unwrap())
    );
}

#[test]
fn a_variable_reaches_the_sandbox_with_its_spaces_its_quotes_and_its_signs() {
    assert_eq!(
        printed_by_the_harness(
            "[environment.env]\nNOTE = ' a \"quoted\" = #sign  '\n",
            "printf '[%s]' \"$NOTE\""
        ),
        "[ a \"quoted\" = #sign  ]"
    );
}

#[test]
fn a_task_may_name_the_variables_that_docker_itself_reads() {
    assert_eq!(
        printed_by_the_harness(
            "[environment.env]\nHOME = \"/srv/home\"\nPATH = \"/srv/bin:/usr/bin:/bin\"\n",
            "echo \"$HOME $PATH\""
        ),
        "/srv/home /srv/bin:/usr/bin:/bin\n"
    );
}

#[test]
fn a_variable_of_the_task_does_not_take_the_place_of_what_cladenc_tells_the_harness() {
    assert_eq!(
        printed_by_the_harness(
            "[environment.env]\nCLADENC_MODEL = \"another-model\"\n",
            "echo \"$CLADENC_MODEL\""
        ),
        format!("{}\n", test_inputs::MODEL)
    );
}

#[test]
fn a_trial_is_carried_out_in_the_image_the_task_names() {
    assert_eq!(found_by_the_harness(&WITH_AN_IMAGE, "pwd; ls /app"), "/\n");
}

#[test]
fn the_verifier_of_a_task_that_names_an_image_runs_in_a_sandbox_of_that_image() {
    assert_eq!(found_by_the_verifier(&WITH_AN_IMAGE, "pwd"), "/\n");
}

#[test]
fn the_harness_is_given_no_more_cpus_than_its_task_asks_for() {
    assert_eq!(
        printed_by_the_harness("[environment]\ncpus = 0.5\n", "cat /sys/fs/cgroup/cpu.max"),
        "50000 100000\n"
    );
}

#[test]
fn the_verifier_is_given_no_more_cpus_than_its_task_asks_for() {
    assert_eq!(
        printed_by_the_verifier("[environment]\ncpus = 2\n", "cat /sys/fs/cgroup/cpu.max"),
        "200000 100000\n"
    );
}

#[test]
fn the_harness_is_given_no_more_memory_than_its_task_asks_for() {
    assert_eq!(
        printed_by_the_harness(
            "[environment]\nmemory_mb = 256\n",
            "cat /sys/fs/cgroup/memory.max"
        ),
        "268435456\n"
    );
}

#[test]
fn the_verifier_is_given_no_more_memory_than_its_task_asks_for_in_the_older_spelling() {
    assert_eq!(
        printed_by_the_verifier(
            "[environment]\nmemory = '256M'\n",
            "cat /sys/fs/cgroup/memory.max"
        ),
        "268435456\n"
    );
}

#[test]
fn a_harness_whose_task_asks_for_no_cpus_and_no_memory_is_limited_in_neither() {
    assert_eq!(
        printed_by_the_harness("", "cat /sys/fs/cgroup/cpu.max /sys/fs/cgroup/memory.max"),
        "max 100000\nmax\n"
    );
}
