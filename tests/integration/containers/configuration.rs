use std::env;
use std::fs;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Environment, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const FINISH: &str = "echo done > work.txt";
const FINISHED: &str = "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt";
const TASK: &str = "benchmark/t1";
const AS_1000: &str = "[agent]\nuser = 1000\n";
const GRADED_AS_1000: &str = "[verifier]\nuser = 1000\n";
const ELSEWHERE: &str = "[environment]\nworkdir = \"/srv/elsewhere\"\n";
const WITH_A_MODE: &str = "[environment.env]\nMODE = \"strict\"\n";
const WITH_A_JUDGE: &str = "[verifier.env]\nJUDGE = \"strict\"\n";
const IN_AN_IMAGE: &str = "[environment]\ndocker_image = \"debian:stable-slim\"\n";

struct Attempted {
    _directory: TempDir,
    _built: Option<Image>,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
}

fn environment_of(outputs_directory: &OutputsDirectory) -> (Environment, Option<Image>) {
    let configuration = TaskConfiguration::read(&outputs_directory.task(TASK)).unwrap();
    let built = configuration.docker_image.is_none().then(|| {
        let image = test_inputs::image();
        containers::build_environment(
            outputs_directory,
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
            grader: None,
        },
        built,
    )
}

fn attempted(task_toml: &str, run_command: &str, grader: &str) -> Attempted {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", grader);
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
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
    };
    let result = containers::run_attempt(
        &outputs_directory,
        &launch,
        containers::Attempt {
            harness: 0,
            task: TASK,
            environment: &environment,
        },
        &AtomicBool::new(false),
        &|_| (),
    );
    Attempted {
        _directory: directory,
        _built: built,
        outputs_directory,
        result,
    }
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(attempted.outputs_directory.attempt(0, TASK).join(file)).unwrap()
}

fn printed_by_the_harness(task_toml: &str, run_command: &str) -> String {
    kept(&attempted(task_toml, run_command, FINISHED), "stdout")
}

fn printed_by_the_grader(task_toml: &str, grader: &str) -> String {
    kept(&attempted(task_toml, FINISH, grader), "grader/stdout")
}

#[test]
fn the_harness_runs_as_the_user_the_task_names_by_number() {
    assert_eq!(printed_by_the_harness(AS_1000, "id -u"), "1000\n");
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
        printed_by_the_harness(AS_1000, "stat -c %u /var/opt/cladenc/harness/harness.sh"),
        "1000\n"
    );
}

#[test]
fn the_grader_of_a_task_that_names_a_user_for_the_harness_alone_runs_as_the_user_of_the_environment()
 {
    assert_eq!(printed_by_the_grader(AS_1000, "id -u"), "0\n");
}

#[test]
fn the_grader_runs_as_the_user_the_task_names_for_it() {
    assert_eq!(printed_by_the_grader(GRADED_AS_1000, "id -u"), "1000\n");
}

#[test]
fn the_harness_of_a_task_that_names_a_user_for_the_grader_alone_runs_as_the_user_of_the_environment()
 {
    assert_eq!(printed_by_the_harness(GRADED_AS_1000, "id -u"), "0\n");
}

#[test]
fn the_tests_of_the_task_belong_to_the_user_the_task_names_for_the_grader() {
    assert_eq!(
        printed_by_the_grader(GRADED_AS_1000, "stat -c %u /tests/test.sh"),
        "1000\n"
    );
}

#[test]
fn a_grader_that_runs_as_another_user_than_the_harness_can_score() {
    assert!(matches!(
        attempted(GRADED_AS_1000, FINISH, FINISHED).result,
        Ok(containers::Attempted {
            result: AttemptResult::Completed,
            ..
        })
    ));
}

#[test]
fn an_attempt_whose_task_names_a_user_that_its_environment_does_not_hold_is_an_error() {
    assert!(matches!(
        attempted("[agent]\nuser = \"nobody-here\"\n", FINISH, FINISHED).result,
        Err(ContainerError::Io(_))
    ));
}

#[test]
fn the_harness_works_in_the_working_directory_the_task_names() {
    assert_eq!(printed_by_the_harness(ELSEWHERE, "pwd"), "/srv/elsewhere\n");
}

#[test]
fn the_grader_works_in_the_working_directory_the_task_names() {
    assert_eq!(printed_by_the_grader(ELSEWHERE, "pwd"), "/srv/elsewhere\n");
}

#[test]
fn what_the_harness_leaves_in_the_working_directory_the_task_names_is_kept_as_its_work() {
    let attempted = attempted(ELSEWHERE, "echo left > note", FINISHED);
    assert_eq!(kept(&attempted, "work/note"), "left\n");
}

#[test]
fn the_harness_is_given_the_variables_of_the_environment_of_the_task() {
    assert_eq!(
        printed_by_the_harness(WITH_A_MODE, "echo \"$MODE\""),
        "strict\n"
    );
}

#[test]
fn the_grader_is_given_the_variables_of_the_environment_of_the_task() {
    assert_eq!(
        printed_by_the_grader(WITH_A_MODE, "echo \"$MODE\""),
        "strict\n"
    );
}

#[test]
fn the_grader_is_given_the_variables_the_task_names_for_it() {
    assert_eq!(
        printed_by_the_grader(WITH_A_JUDGE, "echo \"$JUDGE\""),
        "strict\n"
    );
}

#[test]
fn the_harness_is_not_given_the_variables_the_task_names_for_the_grader() {
    assert_eq!(
        printed_by_the_harness(WITH_A_JUDGE, "echo \"${JUDGE-not given}\""),
        "not given\n"
    );
}

#[test]
fn a_variable_the_task_names_for_the_grader_takes_the_place_of_one_of_the_environment() {
    assert_eq!(
        printed_by_the_grader(
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
fn a_variable_reaches_the_container_with_its_spaces_its_quotes_and_its_signs() {
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
fn an_attempt_is_carried_out_in_the_image_the_task_names() {
    assert_eq!(printed_by_the_harness(IN_AN_IMAGE, "pwd; ls /app"), "/\n");
}

#[test]
fn the_grader_of_a_task_that_names_an_image_runs_in_a_container_of_that_image() {
    assert_eq!(printed_by_the_grader(IN_AN_IMAGE, "pwd"), "/\n");
}

#[test]
fn the_harness_is_given_no_more_cpus_than_its_task_asks_for() {
    assert_eq!(
        printed_by_the_harness("[environment]\ncpus = 0.5\n", "cat /sys/fs/cgroup/cpu.max"),
        "50000 100000\n"
    );
}

#[test]
fn the_grader_is_given_no_more_cpus_than_its_task_asks_for() {
    assert_eq!(
        printed_by_the_grader("[environment]\ncpus = 2\n", "cat /sys/fs/cgroup/cpu.max"),
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
fn the_grader_is_given_no_more_memory_than_its_task_asks_for_in_the_older_spelling() {
    assert_eq!(
        printed_by_the_grader(
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
