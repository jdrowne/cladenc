use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::containers::{self, ContainerError, Environment, GraderImage, Launch};
use cladenc::genealogy::AttemptResult;
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const OWN: &str = "[verifier.environment]\n";
const REPORT: &str = "artifacts = [\"/app/report.txt\"]\n";
const OUT: &str = "artifacts = [\"/app/out\"]\n";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const COMPLETE: &str = "echo 1 > /logs/verifier/reward.txt";
const NOT_ON_THE_MACHINE: &str = "cladenc-test-not-on-the-machine:0";
const HOLDS_ITS_TESTS: &str = "COPY . /tests/\n";
const NEVER_READY: &str = "[verifier.environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";

#[derive(Clone, Copy)]
enum Grader<'a> {
    InTheEnvironment,
    BuiltWith(&'a str),
    Named(&'a str),
}

struct Attempt<'a> {
    task_toml: &'a str,
    run_command: &'a str,
    grader: &'a str,
    image: Grader<'a>,
}

struct Prepared {
    directory: TempDir,
    built: Vec<Image>,
    outputs_directory: OutputsDirectory,
    environment: Environment,
}

struct Attempted {
    _directory: TempDir,
    _built: Vec<Image>,
    outputs_directory: OutputsDirectory,
    result: Result<containers::Attempted, ContainerError>,
}

fn image_of_the_grader(
    outputs_directory: &OutputsDirectory,
    grader: Grader<'_>,
    environment: &Image,
    built: &mut Vec<Image>,
) -> GraderImage {
    match grader {
        Grader::InTheEnvironment => GraderImage {
            name: environment.name.clone(),
            holds_grader: false,
        },
        Grader::Named(name) => GraderImage {
            name: name.to_owned(),
            holds_grader: true,
        },
        Grader::BuiltWith(steps) => {
            let image = test_inputs::image();
            let tests = outputs_directory.task(TASK).join("tests");
            fs::write(
                tests.join("Dockerfile"),
                format!("FROM {}\n{steps}", test_inputs::IMAGE),
            )
            .unwrap();
            containers::build_grader_environment(
                outputs_directory,
                TASK,
                &image.name,
                Duration::from_secs(600),
                &AtomicBool::new(false),
            )
            .unwrap();
            fs::write(tests.join("test.sh"), "#!/bin/sh\necho from the task\n").unwrap();
            let name = image.name.clone();
            built.push(image);
            GraderImage {
                name,
                holds_grader: true,
            }
        }
    }
}

fn prepared(attempt: &Attempt<'_>) -> Prepared {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), attempt.task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    test_inputs::kept_grader(&path, "t1", attempt.grader);
    let configuration = TaskConfiguration::read(&outputs_directory.task(TASK)).unwrap();
    let image = test_inputs::image();
    containers::build_environment(
        &outputs_directory,
        TASK,
        &image.name,
        configuration.build_time_limit,
        &AtomicBool::new(false),
    )
    .unwrap();
    let mut built = Vec::new();
    let grader = image_of_the_grader(&outputs_directory, attempt.image, &image, &mut built);
    let environment = Environment {
        image: image.name.clone(),
        configuration,
        grader: Some(grader),
    };
    built.push(image);
    Prepared {
        directory,
        built,
        outputs_directory,
        environment,
    }
}

fn carried_out(prepared: Prepared, run_command: &str) -> Attempted {
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
    };
    let result = containers::run_attempt(
        &prepared.outputs_directory,
        &launch,
        containers::Attempt {
            harness: 0,
            task: TASK,
            environment: &prepared.environment,
        },
        &AtomicBool::new(false),
    );
    Attempted {
        _directory: prepared.directory,
        _built: prepared.built,
        outputs_directory: prepared.outputs_directory,
        result,
    }
}

fn attempted_as(attempt: &Attempt<'_>) -> Attempted {
    carried_out(prepared(attempt), attempt.run_command)
}

fn attempted(task_toml: &str, run_command: &str, grader: &str) -> Attempted {
    attempted_as(&Attempt {
        task_toml,
        run_command,
        grader,
        image: Grader::InTheEnvironment,
    })
}

fn graded_in_an_image_built_with(steps: &str, task_toml: &str, grader: &str) -> Attempted {
    attempted_as(&Attempt {
        task_toml,
        run_command: WRITE_A_REPORT,
        grader,
        image: Grader::BuiltWith(steps),
    })
}

fn graded_in_its_image(task_toml: &str, grader: &str) -> Attempted {
    graded_in_an_image_built_with(HOLDS_ITS_TESTS, task_toml, grader)
}

fn kept_at(attempted: &Attempted, file: &str) -> PathBuf {
    attempted.outputs_directory.attempt(0, TASK).join(file)
}

fn kept(attempted: &Attempted, file: &str) -> String {
    fs::read_to_string(kept_at(attempted, file)).unwrap()
}

fn printed_by_the_grader(task_toml: &str, run_command: &str, grader: &str) -> String {
    kept(&attempted(task_toml, run_command, grader), "grader/stdout")
}

fn printed_by_the_grader_in_its_image(task_toml: &str, grader: &str) -> String {
    kept(&graded_in_its_image(task_toml, grader), "grader/stdout")
}

fn result_of(attempted: &Attempted) -> Option<AttemptResult> {
    attempted.result.as_ref().ok().map(|ended| ended.result)
}

fn separate_and(more: &str) -> String {
    format!("{more}\n{SEPARATE}")
}

fn containers_while_the_grader_runs() -> usize {
    let prepared = prepared(&Attempt {
        task_toml: SEPARATE,
        run_command: WRITE_A_REPORT,
        grader: "echo started; sleep 5; echo 1 > /logs/verifier/reward.txt",
        image: Grader::InTheEnvironment,
    });
    let started = prepared
        .outputs_directory
        .running_attempt(0, TASK)
        .join("grader/stdout");
    let label = format!(
        "label=cladenc.directed-evolution={}",
        fs::canonicalize(prepared.outputs_directory.path())
            .unwrap()
            .display()
    );
    thread::scope(|scope| {
        let counted = scope.spawn(|| {
            (0..6).any(|_| {
                test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&started))
            });
            let listed = Command::new("docker")
                .args(["ps", "--quiet", "--filter", &label])
                .output()
                .unwrap();
            String::from_utf8(listed.stdout).unwrap().lines().count()
        });
        carried_out(prepared, WRITE_A_REPORT);
        counted.join().unwrap()
    })
}

#[test]
fn a_grader_that_runs_separately_finds_the_working_directory_as_the_environment_made_it() {
    assert_eq!(
        printed_by_the_grader(SEPARATE, "echo done > work.txt", "cat work.txt"),
        "to do\n"
    );
}

#[test]
fn a_grader_that_runs_separately_finds_no_file_that_the_harness_made() {
    assert_eq!(
        printed_by_the_grader(SEPARATE, WRITE_A_REPORT, "ls"),
        "work.txt\n"
    );
}

#[test]
fn a_grader_that_runs_in_a_copy_of_the_environment_is_given_the_tests_of_the_task() {
    assert_eq!(
        printed_by_the_grader(SEPARATE, WRITE_A_REPORT, "ls /tests"),
        "test.sh\n"
    );
}

#[test]
fn a_grader_that_runs_in_a_copy_of_the_environment_finds_tests_a_symbolic_link_under_var_opt() {
    assert_eq!(
        printed_by_the_grader(SEPARATE, WRITE_A_REPORT, "readlink /tests"),
        "/var/opt/cladenc/harbor/tests\n"
    );
}

#[test]
fn a_grader_that_runs_separately_finds_logs_a_symbolic_link_to_a_directory_under_var_log() {
    assert_eq!(
        printed_by_the_grader(SEPARATE, WRITE_A_REPORT, "readlink /logs"),
        "/var/log/cladenc/harbor\n"
    );
}

#[test]
fn logs_verifier_of_a_grader_that_runs_separately_belongs_to_the_user_the_grader_runs_as() {
    assert_eq!(
        printed_by_the_grader(
            "[verifier]\nenvironment_mode = \"separate\"\nuser = 1000\n",
            WRITE_A_REPORT,
            "stat -c %u /var/log/cladenc/harbor/verifier"
        ),
        "1000\n"
    );
}

#[test]
fn a_path_the_task_lists_is_at_the_same_path_in_the_container_of_the_grader() {
    assert_eq!(
        printed_by_the_grader(&separate_and(REPORT), WRITE_A_REPORT, "cat /app/report.txt"),
        "found\n"
    );
}

#[test]
fn a_harness_completes_a_task_whose_grader_runs_separately_by_what_was_kept() {
    let attempted = attempted(
        &separate_and(REPORT),
        WRITE_A_REPORT,
        "grep -q found /app/report.txt && echo 1 > /logs/verifier/reward.txt",
    );
    assert_eq!(result_of(&attempted), Some(AttemptResult::Completed));
}

#[test]
fn a_harness_does_not_complete_a_task_by_work_that_its_separate_grader_is_not_given() {
    let attempted = attempted(
        SEPARATE,
        "echo done > work.txt",
        "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt",
    );
    assert_eq!(result_of(&attempted), Some(AttemptResult::NotCompleted));
}

#[test]
fn a_path_is_given_to_the_grader_at_its_own_path_and_not_at_its_destination() {
    assert_eq!(
        printed_by_the_grader(
            &separate_and(
                "[[artifacts]]\nsource = \"/app/report.txt\"\ndestination = \"kept/elsewhere.txt\"\n"
            ),
            WRITE_A_REPORT,
            "cat /app/report.txt"
        ),
        "found\n"
    );
}

#[test]
fn a_directory_that_was_kept_takes_the_place_of_what_the_image_holds_at_its_path() {
    assert_eq!(
        printed_by_the_grader(
            &separate_and("artifacts = [\"/app\"]\n"),
            "rm work.txt && echo found > report.txt",
            "ls /app"
        ),
        "report.txt\n"
    );
}

#[test]
fn what_the_patterns_of_the_task_left_out_is_not_given_to_the_grader() {
    assert_eq!(
        printed_by_the_grader(
            &separate_and("[[artifacts]]\nsource = \"/app/out\"\nexclude = [\"*.tmp\"]\n"),
            "mkdir out && echo a > out/a.txt && touch out/draft.tmp",
            "ls /app/out"
        ),
        "a.txt\n"
    );
}

#[test]
fn a_grader_is_run_where_a_path_the_task_lists_was_not_in_the_container_of_the_harness() {
    let attempted = attempted(
        &separate_and("artifacts = [\"/app/never/made.txt\"]\n"),
        WRITE_A_REPORT,
        COMPLETE,
    );
    assert_eq!(result_of(&attempted), Some(AttemptResult::Completed));
}

#[test]
fn a_file_given_to_the_grader_keeps_no_setuid_bit() {
    assert_eq!(
        printed_by_the_grader(
            &separate_and("artifacts = [\"/app/tool\"]\n"),
            "cp /bin/true tool && chmod 4755 tool",
            "stat -c %a /app/tool"
        ),
        "755\n"
    );
}

#[test]
fn a_symbolic_link_that_points_outside_the_path_it_was_kept_with_is_not_given_to_the_grader() {
    assert_eq!(
        printed_by_the_grader(
            &separate_and(OUT),
            "mkdir out && echo a > out/a.txt && ln -s /tests/test.sh out/answer",
            "ls /app/out"
        ),
        "a.txt\n"
    );
}

#[test]
fn a_symbolic_link_that_stays_inside_the_path_it_was_kept_with_is_given_to_the_grader() {
    assert_eq!(
        printed_by_the_grader(
            &separate_and(OUT),
            "mkdir out && echo a > out/a.txt && ln -s a.txt out/link",
            "cat /app/out/link"
        ),
        "a\n"
    );
}

#[test]
fn what_is_given_to_the_grader_belongs_to_the_user_the_grader_runs_as() {
    assert_eq!(
        printed_by_the_grader(
            &format!("{REPORT}\n[verifier]\nenvironment_mode = \"separate\"\nuser = 1000\n"),
            "echo found > report.txt && chmod 600 report.txt",
            "cat /app/report.txt"
        ),
        "found\n"
    );
}

#[test]
fn what_a_grader_that_runs_separately_left_in_logs_verifier_is_kept_with_the_attempt() {
    let attempted = attempted(
        SEPARATE,
        WRITE_A_REPORT,
        "echo noted > /logs/verifier/notes.txt",
    );
    assert_eq!(
        kept(&attempted, "grader/logs/verifier/notes.txt"),
        "noted\n"
    );
}

#[test]
fn a_grader_that_runs_separately_past_its_time_limit_is_timed_out() {
    let attempted = attempted(
        "[verifier]\nenvironment_mode = \"separate\"\ntimeout_sec = 2\n",
        WRITE_A_REPORT,
        "sleep 300",
    );
    assert_eq!(result_of(&attempted), Some(AttemptResult::TimedOut));
}

#[test]
fn the_container_of_the_harness_is_gone_while_the_grader_runs_in_its_own() {
    assert_eq!(containers_while_the_grader_runs(), 1);
}

#[test]
fn a_grader_in_an_image_built_from_the_tests_of_the_task_is_the_grader_the_image_holds() {
    assert_eq!(
        printed_by_the_grader_in_its_image(SEPARATE, "echo from the image"),
        "from the image\n"
    );
}

#[test]
fn nothing_is_made_at_tests_in_a_container_whose_image_holds_the_grader() {
    assert_eq!(
        printed_by_the_grader_in_its_image(
            SEPARATE,
            "[ -L /tests ] || echo no link; ls -d /var/opt/cladenc/harbor 2> /dev/null"
        ),
        "no link\n"
    );
}

#[test]
fn an_image_built_from_the_tests_of_the_task_is_not_the_environment_of_the_task() {
    assert_eq!(
        printed_by_the_grader_in_its_image(SEPARATE, "ls /app 2> /dev/null || echo no /app"),
        "no /app\n"
    );
}

#[test]
fn a_path_is_given_to_a_grader_whose_image_holds_no_directory_for_it() {
    assert_eq!(
        printed_by_the_grader_in_its_image(&separate_and(REPORT), "cat /app/report.txt"),
        "found\n"
    );
}

#[test]
fn the_grader_runs_in_the_working_directory_that_its_own_environment_names() {
    assert_eq!(
        printed_by_the_grader_in_its_image(&format!("{OWN}workdir = \"/srv\"\n"), "pwd"),
        "/srv\n"
    );
}

#[test]
fn the_container_of_the_grader_is_given_the_variables_of_its_own_environment() {
    assert_eq!(
        printed_by_the_grader(
            "[verifier.environment.env]\nMODE = \"strict\"\n",
            WRITE_A_REPORT,
            "echo \"$MODE\""
        ),
        "strict\n"
    );
}

#[test]
fn a_grader_with_an_environment_of_its_own_is_given_no_variable_of_that_of_the_task() {
    assert_eq!(
        printed_by_the_grader(
            &format!("[environment.env]\nMODE = \"of the task\"\n\n{OWN}"),
            WRITE_A_REPORT,
            "echo \"${MODE-not set}\""
        ),
        "not set\n"
    );
}

#[test]
fn a_grader_with_no_environment_of_its_own_is_given_the_variables_of_that_of_the_task() {
    assert_eq!(
        printed_by_the_grader(
            &format!("[environment.env]\nMODE = \"of the task\"\n\n{SEPARATE}"),
            WRITE_A_REPORT,
            "echo \"$MODE\""
        ),
        "of the task\n"
    );
}

#[test]
fn a_grader_that_runs_separately_is_given_the_variables_under_verifier() {
    assert_eq!(
        printed_by_the_grader(
            &format!("{SEPARATE}\n[verifier.env]\nJUDGE = \"strict\"\n"),
            WRITE_A_REPORT,
            "echo \"$JUDGE\""
        ),
        "strict\n"
    );
}

#[test]
fn the_container_of_the_grader_is_given_no_more_cpus_than_its_own_environment_asks_for() {
    assert_eq!(
        printed_by_the_grader(
            &format!("{OWN}cpus = 0.5\n"),
            WRITE_A_REPORT,
            "cat /sys/fs/cgroup/cpu.max"
        ),
        "50000 100000\n"
    );
}

#[test]
fn the_container_of_the_grader_is_given_no_more_memory_than_its_own_environment_asks_for() {
    assert_eq!(
        printed_by_the_grader(
            &format!("{OWN}memory_mb = 256\n"),
            WRITE_A_REPORT,
            "cat /sys/fs/cgroup/memory.max"
        ),
        "268435456\n"
    );
}

#[test]
fn a_grader_with_an_environment_of_its_own_is_not_held_to_the_memory_of_that_of_the_task() {
    assert_eq!(
        printed_by_the_grader(
            &format!("[environment]\nmemory_mb = 256\n\n{OWN}"),
            WRITE_A_REPORT,
            "cat /sys/fs/cgroup/memory.max"
        ),
        "max\n"
    );
}

#[test]
fn a_harness_does_not_complete_a_task_whose_grader_s_health_check_does_not_pass() {
    let attempted = attempted(NEVER_READY, WRITE_A_REPORT, COMPLETE);
    assert_eq!(result_of(&attempted), Some(AttemptResult::NotCompleted));
}

#[test]
fn what_the_health_check_of_a_grader_printed_is_kept_with_what_is_kept_of_the_grader() {
    let attempted = attempted(NEVER_READY, WRITE_A_REPORT, COMPLETE);
    assert_eq!(
        kept(&attempted, "grader/health-check/stdout"),
        "not ready\n"
    );
}

#[test]
fn a_grader_whose_health_check_does_not_pass_is_not_run() {
    let attempted = attempted(NEVER_READY, WRITE_A_REPORT, COMPLETE);
    assert!(!kept_at(&attempted, "grader/stdout").exists());
}

#[test]
fn a_grader_whose_health_check_passes_is_run() {
    let attempted = attempted(
        "[verifier.environment.healthcheck]\ncommand = \"test -e /app/work.txt\"\n",
        WRITE_A_REPORT,
        COMPLETE,
    );
    assert_eq!(result_of(&attempted), Some(AttemptResult::Completed));
}

#[test]
fn a_health_check_of_a_grader_that_exits_with_status_125_is_an_error_of_docker() {
    let attempted = attempted(
        "[verifier.environment.healthcheck]\ncommand = \"exit 125\"\n",
        WRITE_A_REPORT,
        COMPLETE,
    );
    assert!(matches!(attempted.result, Err(ContainerError::Docker(_))));
}

#[test]
fn a_grader_whose_container_docker_did_not_start_is_an_error_that_names_what_docker_said() {
    let attempted = attempted_as(&Attempt {
        task_toml: SEPARATE,
        run_command: WRITE_A_REPORT,
        grader: COMPLETE,
        image: Grader::Named(NOT_ON_THE_MACHINE),
    });
    assert!(matches!(
        &attempted.result,
        Err(ContainerError::Docker(said)) if said.ends_with("grader/stderr")
    ));
}

#[test]
fn a_path_that_cannot_be_put_in_the_container_of_the_grader_is_an_error() {
    let attempted = graded_in_an_image_built_with(
        "RUN touch /app\nCOPY . /tests/\n",
        &separate_and(REPORT),
        COMPLETE,
    );
    assert!(matches!(attempted.result, Err(ContainerError::Io(_))));
}
