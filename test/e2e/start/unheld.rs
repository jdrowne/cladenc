use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;
use test_inputs::Image;

struct Stopped {
    directory: TempDir,
    image: Image,
    output: Output,
}

fn start(directory: &Path, image: &str) -> Command {
    let options = [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", image.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--run-dir", directory.join("run")),
        ("--max-procreations", "1".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--attempt-timeout", "30".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn stopped() -> Stopped {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let output = start(directory.path(), &image.name).output().unwrap();
    Stopped {
        directory,
        image,
        output,
    }
}

fn kept_of_what_docker_said(stopped: &Stopped) -> PathBuf {
    stopped
        .directory
        .path()
        .join("run/running/attempt-0-benchmark+t1/stderr")
}

fn what_docker_said(stopped: &Stopped) -> String {
    fs::read_to_string(kept_of_what_docker_said(stopped)).unwrap()
}

fn resumed_once_the_machine_holds_the_image(stopped: &Stopped) -> Output {
    Command::new("docker")
        .args(["tag", test_inputs::IMAGE, &stopped.image.name])
        .output()
        .unwrap();
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .args(["resume", "--run-dir"])
        .arg(stopped.directory.path().join("run"))
        .output()
        .unwrap()
}

#[test]
fn a_directed_evolution_whose_image_the_machine_does_not_hold_exits_with_1() {
    assert_eq!(stopped().output.status.code(), Some(1));
}

#[test]
fn a_directed_evolution_whose_image_the_machine_does_not_hold_stops_at_its_first_container() {
    let stopped = stopped();
    let said = String::from_utf8_lossy(&stopped.output.stderr);
    assert_eq!(
        said.lines().last().unwrap_or_default(),
        format!(
            "cladenc: the directed evolution stopped before its end: the attempt of harness 0 \
             on task benchmark/t1 could not be carried out: Docker did not start its container, \
             and {} holds what Docker said",
            kept_of_what_docker_said(&stopped).display()
        )
    );
}

#[test]
fn docker_says_of_an_image_the_machine_does_not_hold_that_there_is_no_such_image() {
    assert!(what_docker_said(&stopped()).contains("No such image"));
}

#[test]
fn docker_does_not_look_in_a_registry_for_an_image_the_machine_does_not_hold() {
    assert!(!what_docker_said(&stopped()).contains("Unable to find image"));
}

#[test]
fn a_directed_evolution_that_stopped_for_its_image_is_continued_once_the_machine_holds_the_image() {
    let stopped = stopped();
    assert_eq!(
        resumed_once_the_machine_holds_the_image(&stopped)
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn a_directed_evolution_continued_once_its_image_is_there_prints_the_directory_of_a_harness() {
    let stopped = stopped();
    let resumed = resumed_once_the_machine_holds_the_image(&stopped);
    let harnesses = stopped.directory.path().join("run/harnesses");
    assert!(
        String::from_utf8_lossy(&resumed.stdout).starts_with(&format!("{}/", harnesses.display()))
    );
}
