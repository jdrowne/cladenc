use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", run_command.into()),
        ("--outputs-dir", directory.join("outputs")),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "60".into()),
        ("--attempt-timeout", "60".into()),
        ("--holdout-percentage", "50".into()),
        ("--holdout-corroboration-percentage", "50".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options).stderr(Stdio::null());
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

fn resumed_while_in_use() -> InUse {
    let directory = test_inputs::temporary_directory();
    let mut first = start(directory.path(), "hostname > id; sleep 300")
        .spawn()
        .unwrap();
    let id_file = directory
        .path()
        .join("outputs/running/attempt-0-benchmark+t1/work/id");
    test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
    let second = resume(directory.path());
    let the_container_of_the_first_went_on = test_inputs::container_is_running(&id_file);
    kill(Pid::from_raw(first.id().cast_signed()), Signal::SIGINT).unwrap();
    first.wait().unwrap();
    InUse {
        second,
        the_container_of_the_first_went_on,
    }
}

fn resumed_once_the_file_of_its_key_is_gone() -> (PathBuf, Output) {
    let directory = test_inputs::temporary_directory();
    let key_file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
    start(directory.path(), "exit 125")
        .arg(format!("--model-key-file={}", key_file.display()))
        .status()
        .unwrap();
    fs::remove_file(&key_file).unwrap();
    (key_file, resume(directory.path()))
}

#[test]
fn resume_on_a_directed_evolution_whose_file_of_the_key_is_gone_exits_with_2() {
    let (_key_file, resumed) = resumed_once_the_file_of_its_key_is_gone();
    assert_eq!(resumed.status.code(), Some(2));
}

#[test]
fn resume_on_a_directed_evolution_whose_file_of_the_key_is_gone_names_the_file() {
    let (key_file, resumed) = resumed_once_the_file_of_its_key_is_gone();
    assert_eq!(
        String::from_utf8(resumed.stderr).unwrap(),
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
    assert_eq!(resumed_while_in_use().second.status.code(), Some(2));
}

#[test]
fn a_second_cladenc_on_an_outputs_directory_in_use_says_that_another_is_using_it() {
    assert_eq!(
        String::from_utf8(resumed_while_in_use().second.stderr).unwrap(),
        "cladenc: the outputs directory cannot be used: another cladenc is using it\n"
    );
}

#[test]
fn a_second_cladenc_on_an_outputs_directory_in_use_removes_no_container_of_the_first() {
    assert!(resumed_while_in_use().the_container_of_the_first_went_on);
}
