use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use cladenc::benchmark::{Benchmark, TaskConfiguration};
use cladenc::file_system::Disk;
use cladenc::genealogy::{Cause, TrialResult};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use cladenc::sandboxes::{
    self, Copied, Environment, Host, Launch, Program, Retry, SandboxError, VerifierImage,
};
use tempfile::TempDir;
use test_inputs::Image;

const TASK: &str = "benchmark/t1";
const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const OWN: &str = "[verifier.environment]\n";
const REPORT: &str = "artifacts = [\"/app/report.txt\"]\n";
const OUT: &str = "artifacts = [\"/app/out\"]\n";
const WRITE_A_REPORT: &str = "echo found > report.txt";
const CHANGE_THE_WORK_AND_WRITE_A_REPORT: &str = "echo done > work.txt; echo found > report.txt";
const COMPLETE: &str = "echo 1 > /logs/verifier/reward.txt";
const PUT_OUT_WHERE_DOCKER_COPIES_NOTHING_FROM: &str = "ln -s /proc/1 out";
const LEAVE_A_FILE_NO_ONE_MAY_READ_IN_OUT: &str =
    "mkdir out && echo found > out/report.txt && chmod 000 out/report.txt";
const LEAVE_PERMISSION_BITS_IN_OUT: &str = "mkdir -p out/private out/open out/shut && echo found > \
                                            out/secret && chmod 700 out/private && chmod 1777 \
                                            out/open && chmod 000 out/shut out/secret && chmod \
                                            750 out";
const SAY_THE_PERMISSION_BITS_IN_OUT: &str =
    "cd /app && stat -c '%a %n' out out/private out/open out/shut out/secret";
const HOLDS_ITS_TESTS: &str = "COPY . /tests/\n";
const NEVER_READY: &str = "[verifier.environment.healthcheck]\ncommand = \"echo not ready; exit \
                           1\"\ninterval_sec = 0\nretries = 1\n";
#[derive(Clone, Copy)]
enum Verifier<'a> {
    InTheEnvironment,
    BuiltWith(&'a str),
}

struct Trial<'a> {
    task_toml: &'a str,
    run_command: &'a str,
    verifier: &'a str,
    image: Verifier<'a>,
}

struct Prepared {
    directory: TempDir,
    built: Vec<Image>,
    outputs_directory: OutputsDirectory,
    environment: Environment,
}

struct Trialled {
    _directory: TempDir,
    _built: Vec<Image>,
    outputs_directory: OutputsDirectory,
    result: Result<sandboxes::Trialled, SandboxError>,
}

struct Asked {
    trialled: Trialled,
    of_the_verifier: &'static [&'static str],
}

fn image_of_the_verifier(
    outputs_directory: &OutputsDirectory,
    verifier: Verifier<'_>,
    environment: &Image,
    built: &mut Vec<Image>,
) -> VerifierImage {
    match verifier {
        Verifier::InTheEnvironment => VerifierImage {
            name: environment.name.clone(),
            holds_verifier: false,
        },
        Verifier::BuiltWith(steps) => {
            let image = test_inputs::image();
            let tests = outputs_directory.task(TASK).join("tests");
            fs::write(
                tests.join("Dockerfile"),
                format!("FROM {}\n{steps}", test_inputs::IMAGE),
            )
            .unwrap();
            sandboxes::build_verifier_environment(
                Host {
                    docker: &Program,
                    outputs_directory,
                },
                TASK,
                &image.name,
                Duration::from_secs(600),
                &AtomicBool::new(false),
            )
            .unwrap();
            fs::write(tests.join("test.sh"), "#!/bin/sh\necho from the task\n").unwrap();
            let name = image.name.clone();
            built.push(image);
            VerifierImage {
                name,
                holds_verifier: true,
            }
        }
    }
}

fn prepared(trial: &Trial<'_>) -> Prepared {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory =
        OutputsDirectory::create(crate::real::disk(), &path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    fs::write(benchmark.join("t1/task.toml"), trial.task_toml).unwrap();
    outputs_directory
        .copy_benchmark(&Benchmark::read(&Disk, &benchmark).unwrap())
        .unwrap();
    test_inputs::kept_verifier(&path, "t1", trial.verifier);
    let configuration = TaskConfiguration::read(&Disk, &outputs_directory.task(TASK)).unwrap();
    let image = test_inputs::image();
    sandboxes::build_environment(
        Host {
            docker: &Program,
            outputs_directory: &outputs_directory,
        },
        TASK,
        &image.name,
        configuration.build_time_limit,
        &AtomicBool::new(false),
    )
    .unwrap();
    let mut built = Vec::new();
    let verifier = image_of_the_verifier(&outputs_directory, trial.image, &image, &mut built);
    let environment = Environment {
        image: image.name.clone(),
        configuration,
        verifier: Some(verifier),
    };
    built.push(image);
    Prepared {
        directory,
        built,
        outputs_directory,
        environment,
    }
}

fn carried_out(prepared: Prepared, run_command: &str) -> Trialled {
    carried_out_with(prepared, run_command, 0, &|_| ())
}

fn carried_out_with(
    prepared: Prepared,
    run_command: &str,
    max_retries: u64,
    retrying: &dyn Fn(Retry),
) -> Trialled {
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
        max_verifier_health_check_retries: max_retries,
        max_task_health_check_retries: 0,
    };
    let result = sandboxes::run_trial(
        Host {
            docker: &Program,
            outputs_directory: &prepared.outputs_directory,
        },
        &launch,
        sandboxes::Trial {
            harness: 0,
            task: TASK,
            environment: &prepared.environment,
        },
        &AtomicBool::new(false),
        retrying,
    );
    Trialled {
        _directory: prepared.directory,
        _built: prepared.built,
        outputs_directory: prepared.outputs_directory,
        result,
    }
}

fn sandboxes_at_each_retry(task_toml: &str, max_retries: u64) -> Vec<usize> {
    let prepared = prepared(&Trial {
        task_toml,
        run_command: WRITE_A_REPORT,
        verifier: COMPLETE,
        image: Verifier::InTheEnvironment,
    });
    let label = label_of(&prepared.outputs_directory);
    let counted = RefCell::new(Vec::new());
    carried_out_with(prepared, WRITE_A_REPORT, max_retries, &|_| {
        counted.borrow_mut().push(sandboxes_with(&label));
    });
    counted.into_inner()
}

fn trialled_as(trial: &Trial<'_>) -> Trialled {
    carried_out(prepared(trial), trial.run_command)
}

fn trialled(task_toml: &str, run_command: &str, verifier: &str) -> Trialled {
    trialled_as(&Trial {
        task_toml,
        run_command,
        verifier,
        image: Verifier::InTheEnvironment,
    })
}

fn verified_in_an_image_built_with(steps: &str, task_toml: &str, verifier: &str) -> Trialled {
    trialled_as(&Trial {
        task_toml,
        run_command: WRITE_A_REPORT,
        verifier,
        image: Verifier::BuiltWith(steps),
    })
}

fn verified_in_its_image(task_toml: &str, verifier: &str) -> Trialled {
    verified_in_an_image_built_with(HOLDS_ITS_TESTS, task_toml, verifier)
}

fn kept_at(trialled: &Trialled, file: &str) -> PathBuf {
    trialled.outputs_directory.trial(0, TASK).join(file)
}

fn kept(trialled: &Trialled, file: &str) -> String {
    fs::read_to_string(kept_at(trialled, file)).unwrap()
}

fn printed_by_the_verifier(task_toml: &str, run_command: &str, verifier: &str) -> String {
    kept(
        &trialled(task_toml, run_command, verifier),
        "verifier/stdout",
    )
}

static BITS_FOUND_BY_THE_VERIFIER: LazyLock<String> = LazyLock::new(|| {
    printed_by_the_verifier(
        &separate_and(OUT),
        LEAVE_PERMISSION_BITS_IN_OUT,
        SAY_THE_PERMISSION_BITS_IN_OUT,
    )
});

fn found_by_the_verifier_with_its_bits(said: &str) -> bool {
    BITS_FOUND_BY_THE_VERIFIER.lines().any(|line| line == said)
}

static VERIFIED_SEPARATELY: LazyLock<Asked> = LazyLock::new(|| {
    let of_the_verifier: &'static [&'static str] = &[
        "cat work.txt",
        "ls",
        "ls /tests",
        "readlink /tests",
        "readlink /logs",
        "echo noted > /logs/verifier/notes.txt",
        "grep -q done work.txt && echo 1 > /logs/verifier/reward.txt",
    ];
    Asked {
        trialled: trialled(
            SEPARATE,
            CHANGE_THE_WORK_AND_WRITE_A_REPORT,
            &test_inputs::one_after_another(of_the_verifier),
        ),
        of_the_verifier,
    }
});

static VERIFIED_SEPARATELY_ON_ITS_REPORT: LazyLock<Asked> = LazyLock::new(|| {
    let of_the_verifier: &'static [&'static str] = &[
        "cat /app/report.txt",
        "grep -q found /app/report.txt && echo 1 > /logs/verifier/reward.txt",
    ];
    Asked {
        trialled: trialled(
            &separate_and(REPORT),
            WRITE_A_REPORT,
            &test_inputs::one_after_another(of_the_verifier),
        ),
        of_the_verifier,
    }
});

static VERIFIED_IN_ITS_IMAGE: LazyLock<Asked> = LazyLock::new(|| {
    let of_the_verifier: &'static [&'static str] = &[
        "echo from the image",
        "[ -L /tests ] || echo no link; ls -d /var/opt/cladenc/harbor 2> /dev/null",
        "ls /app 2> /dev/null || echo no /app",
    ];
    Asked {
        trialled: verified_in_its_image(SEPARATE, &test_inputs::one_after_another(of_the_verifier)),
        of_the_verifier,
    }
});

fn found_by_the_verifier(asked: &Asked, command: &str) -> String {
    test_inputs::printed_by(
        command,
        asked.of_the_verifier,
        &kept(&asked.trialled, "verifier/stdout"),
    )
}

fn printed_by_the_verifier_in_its_image(task_toml: &str, verifier: &str) -> String {
    kept(
        &verified_in_its_image(task_toml, verifier),
        "verifier/stdout",
    )
}

fn result_of(trialled: &Trialled) -> Option<TrialResult> {
    trialled.result.as_ref().ok().map(|ended| ended.result)
}

fn separate_and(more: &str) -> String {
    format!("{more}\n{SEPARATE}")
}

fn label_of(outputs_directory: &OutputsDirectory) -> String {
    format!(
        "label=cladenc.directed-evolution={}",
        fs::canonicalize(outputs_directory.path())
            .unwrap()
            .display()
    )
}

fn sandboxes_with(label: &str) -> usize {
    let listed = Command::new("docker")
        .args(["ps", "--quiet", "--filter", label])
        .output()
        .unwrap();
    String::from_utf8(listed.stdout).unwrap().lines().count()
}

fn sandboxes_while_the_verifier_runs() -> usize {
    let prepared = prepared(&Trial {
        task_toml: SEPARATE,
        run_command: WRITE_A_REPORT,
        verifier: "echo started; sleep 5; echo 1 > /logs/verifier/reward.txt",
        image: Verifier::InTheEnvironment,
    });
    let started = prepared
        .outputs_directory
        .running_trial(0, TASK)
        .join("verifier/stdout");
    let label = label_of(&prepared.outputs_directory);
    thread::scope(|scope| {
        let counted = scope.spawn(|| {
            (0..6).any(|_| {
                test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&started))
            });
            sandboxes_with(&label)
        });
        carried_out(prepared, WRITE_A_REPORT);
        counted.join().unwrap()
    })
}

#[test]
fn a_verifier_that_runs_separately_finds_the_working_directory_as_the_environment_made_it() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_SEPARATELY, "cat work.txt"),
        "to do\n"
    );
}

#[test]
fn a_verifier_that_runs_separately_finds_no_file_that_the_harness_made() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_SEPARATELY, "ls"),
        "work.txt\n"
    );
}

#[test]
fn a_verifier_that_runs_in_a_copy_of_the_environment_is_given_the_tests_of_the_task() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_SEPARATELY, "ls /tests"),
        "test.sh\n"
    );
}

#[test]
fn a_verifier_that_runs_in_a_copy_of_the_environment_finds_tests_a_symbolic_link_under_var_opt() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_SEPARATELY, "readlink /tests"),
        "/var/opt/cladenc/harbor/tests\n"
    );
}

#[test]
fn a_verifier_that_runs_separately_finds_logs_a_symbolic_link_to_a_directory_under_var_log() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_SEPARATELY, "readlink /logs"),
        "/var/log/cladenc/harbor\n"
    );
}

#[test]
fn logs_verifier_of_a_verifier_that_runs_separately_belongs_to_the_user_the_verifier_runs_as() {
    assert_eq!(
        printed_by_the_verifier(
            "[verifier]\nenvironment_mode = \"separate\"\nuser = 1000\n",
            WRITE_A_REPORT,
            "stat -c %u /var/log/cladenc/harbor/verifier"
        ),
        "1000\n"
    );
}

#[test]
fn a_path_the_task_lists_is_at_the_same_path_in_the_verifier_sandbox() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_SEPARATELY_ON_ITS_REPORT, "cat /app/report.txt"),
        "found\n"
    );
}

#[test]
fn a_harness_completes_a_task_whose_verifier_runs_separately_by_what_was_kept() {
    assert_eq!(
        result_of(&VERIFIED_SEPARATELY_ON_ITS_REPORT.trialled),
        Some(TrialResult::Pass)
    );
}

#[test]
fn a_harness_does_not_complete_a_task_by_work_that_its_separate_verifier_is_not_given() {
    assert_eq!(
        result_of(&VERIFIED_SEPARATELY.trialled),
        Some(TrialResult::Inconclusive(Cause::NoVerdict))
    );
}

#[test]
fn a_path_is_given_to_the_verifier_at_its_own_path_and_not_at_its_destination() {
    assert_eq!(
        printed_by_the_verifier(
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
        printed_by_the_verifier(
            &separate_and("artifacts = [\"/app\"]\n"),
            "rm work.txt && echo found > report.txt",
            "ls /app"
        ),
        "report.txt\n"
    );
}

#[test]
fn a_directory_no_one_can_write_in_is_given_to_the_verifier_with_all_it_holds() {
    let trialled = trialled(
        &separate_and(OUT),
        "mkdir -p out/locked && echo found > out/locked/report.txt && chmod 500 out/locked out",
        "grep -q found /app/out/locked/report.txt && echo 1 > /logs/verifier/reward.txt",
    );
    assert_eq!(result_of(&trialled), Some(TrialResult::Pass));
}

#[test]
fn what_the_patterns_of_the_task_left_out_is_not_given_to_the_verifier() {
    assert_eq!(
        printed_by_the_verifier(
            &separate_and("[[artifacts]]\nsource = \"/app/out\"\nexclude = [\"*.tmp\"]\n"),
            "mkdir out && echo a > out/a.txt && touch out/draft.tmp",
            "ls /app/out"
        ),
        "a.txt\n"
    );
}

#[test]
fn a_trial_is_given_no_result_where_a_path_its_verifier_would_be_given_cannot_be_copied_out() {
    let trialled = trialled(
        &separate_and(OUT),
        PUT_OUT_WHERE_DOCKER_COPIES_NOTHING_FROM,
        "true",
    );
    assert!(matches!(
        trialled.result,
        Err(SandboxError::NotCopiedOut { left: Copied::Artifact(path), .. }) if path == "/app/out"
    ));
}

#[test]
fn a_file_no_one_may_read_at_a_kept_path_is_given_to_the_verifier() {
    let trialled = trialled(
        &separate_and(OUT),
        LEAVE_A_FILE_NO_ONE_MAY_READ_IN_OUT,
        "grep -q found /app/out/report.txt && echo 1 > /logs/verifier/reward.txt",
    );
    assert_eq!(result_of(&trialled), Some(TrialResult::Pass));
}

#[test]
fn a_verifier_is_run_where_a_path_the_task_lists_was_not_in_the_sandbox_of_the_harness() {
    let trialled = trialled(
        &separate_and("artifacts = [\"/app/never/made.txt\"]\n"),
        WRITE_A_REPORT,
        COMPLETE,
    );
    assert_eq!(result_of(&trialled), Some(TrialResult::Pass));
}

#[test]
fn a_file_given_to_the_verifier_keeps_no_setuid_bit() {
    assert_eq!(
        printed_by_the_verifier(
            &separate_and("artifacts = [\"/app/tool\"]\n"),
            "cp /bin/true tool && chmod 4755 tool",
            "stat -c %a /app/tool"
        ),
        "755\n"
    );
}

#[test]
fn a_symbolic_link_that_points_outside_the_path_it_was_kept_with_is_not_given_to_the_verifier() {
    assert_eq!(
        printed_by_the_verifier(
            &separate_and(OUT),
            "mkdir out && echo a > out/a.txt && ln -s /tests/test.sh out/answer",
            "ls /app/out"
        ),
        "a.txt\n"
    );
}

#[test]
fn a_symbolic_link_that_stays_inside_the_path_it_was_kept_with_is_given_to_the_verifier() {
    assert_eq!(
        printed_by_the_verifier(
            &separate_and(OUT),
            "mkdir out && echo a > out/a.txt && ln -s a.txt out/link",
            "cat /app/out/link"
        ),
        "a\n"
    );
}

#[test]
fn what_is_given_to_the_verifier_belongs_to_the_user_the_verifier_runs_as() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("{REPORT}\n[verifier]\nenvironment_mode = \"separate\"\nuser = 1000\n"),
            "echo found > report.txt && chmod 600 report.txt",
            "cat /app/report.txt"
        ),
        "found\n"
    );
}

#[test]
fn what_a_verifier_that_runs_separately_left_in_logs_verifier_is_kept_with_the_trial() {
    assert_eq!(
        kept(
            &VERIFIED_SEPARATELY.trialled,
            "verifier/logs/verifier/notes.txt"
        ),
        "noted\n"
    );
}

#[test]
fn the_sandbox_of_the_harness_is_gone_while_the_verifier_runs_in_its_own() {
    assert_eq!(sandboxes_while_the_verifier_runs(), 1);
}

#[test]
fn a_verifier_in_an_image_built_from_the_tests_of_the_task_is_the_verifier_the_image_holds() {
    assert_eq!(
        found_by_the_verifier(&VERIFIED_IN_ITS_IMAGE, "echo from the image"),
        "from the image\n"
    );
}

#[test]
fn nothing_is_made_at_tests_in_a_sandbox_whose_image_holds_the_verifier() {
    assert_eq!(
        found_by_the_verifier(
            &VERIFIED_IN_ITS_IMAGE,
            "[ -L /tests ] || echo no link; ls -d /var/opt/cladenc/harbor 2> /dev/null"
        ),
        "no link\n"
    );
}

#[test]
fn an_image_built_from_the_tests_of_the_task_is_not_the_environment_of_the_task() {
    assert_eq!(
        found_by_the_verifier(
            &VERIFIED_IN_ITS_IMAGE,
            "ls /app 2> /dev/null || echo no /app"
        ),
        "no /app\n"
    );
}

#[test]
fn a_path_is_given_to_a_verifier_whose_image_holds_no_directory_for_it() {
    assert_eq!(
        printed_by_the_verifier_in_its_image(&separate_and(REPORT), "cat /app/report.txt"),
        "found\n"
    );
}

#[test]
fn the_verifier_runs_in_the_working_directory_that_its_own_environment_names() {
    assert_eq!(
        printed_by_the_verifier_in_its_image(&format!("{OWN}workdir = \"/srv\"\n"), "pwd"),
        "/srv\n"
    );
}

#[test]
fn the_verifier_sandbox_is_given_the_variables_of_its_own_environment() {
    assert_eq!(
        printed_by_the_verifier(
            "[verifier.environment.env]\nMODE = \"strict\"\n",
            WRITE_A_REPORT,
            "echo \"$MODE\""
        ),
        "strict\n"
    );
}

#[test]
fn a_verifier_with_an_environment_of_its_own_is_given_no_variable_of_that_of_the_task() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("[environment.env]\nMODE = \"of the task\"\n\n{OWN}"),
            WRITE_A_REPORT,
            "echo \"${MODE-not set}\""
        ),
        "not set\n"
    );
}

#[test]
fn a_verifier_with_no_environment_of_its_own_is_given_the_variables_of_that_of_the_task() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("[environment.env]\nMODE = \"of the task\"\n\n{SEPARATE}"),
            WRITE_A_REPORT,
            "echo \"$MODE\""
        ),
        "of the task\n"
    );
}

#[test]
fn a_verifier_that_runs_separately_is_given_the_variables_under_verifier() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("{SEPARATE}\n[verifier.env]\nJUDGE = \"strict\"\n"),
            WRITE_A_REPORT,
            "echo \"$JUDGE\""
        ),
        "strict\n"
    );
}

#[test]
fn the_verifier_sandbox_is_given_no_more_cpus_than_its_own_environment_asks_for() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("{OWN}cpus = 0.5\n"),
            WRITE_A_REPORT,
            "cat /sys/fs/cgroup/cpu.max"
        ),
        "50000 100000\n"
    );
}

#[test]
fn the_verifier_sandbox_is_given_no_more_memory_than_its_own_environment_asks_for() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("{OWN}memory_mb = 256\n"),
            WRITE_A_REPORT,
            "cat /sys/fs/cgroup/memory.max"
        ),
        "268435456\n"
    );
}

#[test]
fn a_verifier_with_an_environment_of_its_own_is_not_held_to_the_memory_of_that_of_the_task() {
    assert_eq!(
        printed_by_the_verifier(
            &format!("[environment]\nmemory_mb = 256\n\n{OWN}"),
            WRITE_A_REPORT,
            "cat /sys/fs/cgroup/memory.max"
        ),
        "max\n"
    );
}

#[test]
fn a_verifier_sandbox_is_removed_before_the_wait_for_a_retry() {
    assert_eq!(sandboxes_at_each_retry(NEVER_READY, 2), [0, 0]);
}

#[test]
fn what_the_health_check_of_a_verifier_printed_is_kept_with_what_is_kept_of_the_verifier() {
    let trialled = trialled(
        "[verifier.environment.healthcheck]\ncommand = \"echo ready\"\n",
        WRITE_A_REPORT,
        COMPLETE,
    );
    assert_eq!(kept(&trialled, "verifier/health-check/stdout"), "ready\n");
}

#[test]
fn a_verifier_whose_health_check_passes_is_run() {
    let trialled = trialled(
        "[verifier.environment.healthcheck]\ncommand = \"test -e /app/work.txt\"\n",
        WRITE_A_REPORT,
        COMPLETE,
    );
    assert_eq!(result_of(&trialled), Some(TrialResult::Pass));
}

#[test]
fn a_path_that_cannot_be_put_in_the_verifier_sandbox_is_an_error() {
    let trialled = verified_in_an_image_built_with(
        "RUN touch /app\nCOPY . /tests/\n",
        &separate_and(REPORT),
        COMPLETE,
    );
    assert!(matches!(trialled.result, Err(SandboxError::Io(_))));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_finds_the_bits_the_harness_left_on_a_kept_directory() {
    assert!(found_by_the_verifier_with_its_bits("750 out"));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_finds_the_bits_of_a_directory_inside_a_kept_one() {
    assert!(found_by_the_verifier_with_its_bits("700 out/private"));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_finds_the_sticky_bit_the_harness_left_on_a_directory() {
    assert!(found_by_the_verifier_with_its_bits("1777 out/open"));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_finds_a_directory_no_one_may_list_as_it_was_left() {
    assert!(found_by_the_verifier_with_its_bits("0 out/shut"));
}

#[test]
fn a_verifier_in_a_sandbox_of_its_own_finds_a_file_no_one_may_read_as_it_was_left() {
    assert!(found_by_the_verifier_with_its_bits("0 out/secret"));
}
