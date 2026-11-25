use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::LazyLock;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

fn start(directory: &Path, run_command: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--max-grader-health-check-retries", "0".into()),
        ("--max-task-health-check-retries", "0".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start
        .arg("start")
        .args(options)
        .stderr(fs::File::create(directory.join("printed")).unwrap());
    start
}

fn resume(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("resume")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .output()
        .unwrap()
}

struct InUse {
    second: Output,
    the_container_of_the_first_went_on: bool,
}

static RESUMED_WHILE_IN_USE: LazyLock<InUse> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let mut first = start(directory.path(), "hostname; sleep 300")
        .spawn()
        .unwrap();
    let id_file = directory
        .path()
        .join("outputs/running/attempt-0-benchmark+t1/stdout");
    test_inputs::wait_for_a_line(&mut first, &id_file, &directory.path().join("printed"));
    let second = resume(directory.path());
    let the_container_of_the_first_went_on = test_inputs::container_is_running(&id_file);
    kill(Pid::from_raw(first.id().cast_signed()), Signal::SIGINT).unwrap();
    first.wait().unwrap();
    InUse {
        second,
        the_container_of_the_first_went_on,
    }
});

static RESUMED_ONCE_THE_FILE_OF_ITS_KEY_IS_GONE: LazyLock<(PathBuf, Output)> =
    LazyLock::new(|| {
        let directory = test_inputs::temporary_directory();
        let key_file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
        start(directory.path(), "exit 125")
            .arg(format!("--model-key-file={}", key_file.display()))
            .status()
            .unwrap();
        fs::remove_file(&key_file).unwrap();
        let resumed = resume(directory.path());
        (key_file, resumed)
    });

#[test]
fn resume_on_a_directed_evolution_whose_file_of_the_key_is_gone_exits_with_2() {
    let (_key_file, resumed) = &*RESUMED_ONCE_THE_FILE_OF_ITS_KEY_IS_GONE;
    assert_eq!(resumed.status.code(), Some(2));
}

#[test]
fn resume_on_a_directed_evolution_whose_file_of_the_key_is_gone_names_the_file() {
    let (key_file, resumed) = &*RESUMED_ONCE_THE_FILE_OF_ITS_KEY_IS_GONE;
    assert_eq!(
        str::from_utf8(&resumed.stderr).unwrap(),
        format!(
            "cladenc: the key of the model cannot be read: {} is not a file that holds a key on \
             one line and nothing else\n",
            key_file.display()
        )
    );
}

#[test]
fn resume_on_a_missing_outputs_directory_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(resume(directory.path()).status.code(), Some(2));
}

#[test]
fn a_second_cladenc_on_an_outputs_directory_in_use_exits_with_2() {
    assert_eq!(RESUMED_WHILE_IN_USE.second.status.code(), Some(2));
}

#[test]
fn a_second_cladenc_on_an_outputs_directory_in_use_says_that_another_is_using_it() {
    assert_eq!(
        str::from_utf8(&RESUMED_WHILE_IN_USE.second.stderr).unwrap(),
        "cladenc: the outputs directory cannot be used: another cladenc is using it\n"
    );
}

#[test]
fn a_second_cladenc_on_an_outputs_directory_in_use_removes_no_container_of_the_first() {
    assert!(RESUMED_WHILE_IN_USE.the_container_of_the_first_went_on);
}
