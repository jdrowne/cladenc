use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;
use test_inputs::Image;

const SOLVES_T1: &str = "RUN echo t1 > /solved-by-the-image\n";
const FAILS: &str = "RUN exit 3\n";

struct Ran {
    directory: TempDir,
    image: Option<Image>,
    status: Option<i32>,
    output: String,
    said: String,
}

fn start(directory: &Path, steps: &str) -> Command {
    let options = [
        ("--progenitor", test_inputs::progenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--dockerfile", test_inputs::dockerfile(directory, steps)),
        ("--docker-build-timeout", "60".into()),
        (
            "--harness-run-command",
            test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES.into(),
        ),
        ("--run-dir", directory.join("run")),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0".into()),
        ("--confidence", "99".into()),
        ("--initial-procreations", "1".into()),
        ("--workers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--evaluation-timeout", "30".into()),
        ("--seed", "7".into()),
    ]
    .map(|(option, value)| format!("{option}={}", value.display()));
    let mut start = Command::new(env!("CARGO_BIN_EXE_cladenc"));
    start.arg("start").args(options);
    start
}

fn image_named_in(said: &str) -> Option<Image> {
    said.lines()
        .find_map(|line| line.strip_prefix("cladenc: building the image "))
        .and_then(|rest| rest.split(' ').next())
        .map(|name| Image {
            name: name.to_owned(),
        })
}

fn ran(steps: &str) -> Ran {
    let directory = test_inputs::temporary_directory();
    let output = start(directory.path(), steps).output().unwrap();
    let said = String::from_utf8(output.stderr).unwrap();
    Ran {
        directory,
        image: image_named_in(&said),
        status: output.status.code(),
        output: String::from_utf8(output.stdout).unwrap(),
        said,
    }
}

fn name_of_the_image(ran: &Ran) -> &str {
    ran.image.as_ref().map_or("", |image| image.name.as_str())
}

fn is_named_by_cladenc(image: &str) -> bool {
    image.strip_prefix("cladenc-").is_some_and(|digits| {
        digits.len() == 16
            && digits
                .chars()
                .all(|digit| matches!(digit, '0'..='9' | 'a'..='f'))
    })
}

fn docker_holds(image: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", image])
        .output()
        .unwrap()
        .status
        .success()
}

fn kept_of_the_image(ran: &Ran) -> Vec<String> {
    let mut kept: Vec<String> = fs::read_dir(ran.directory.path().join("run/image"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    kept.sort();
    kept
}

fn arguments_of_a_docker_that_records_them() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let arguments = directory.path().join("arguments-of-docker");
    let path = format!(
        "{}:{}",
        test_inputs::sample("e2e", "fake").display(),
        env::var("PATH").unwrap()
    );
    start(directory.path(), SOLVES_T1)
        .env("PATH", path)
        .env("CLADENC_TEST_ARGUMENTS_OF_DOCKER", &arguments)
        .output()
        .unwrap();
    let recorded = fs::read_to_string(arguments).unwrap();
    (directory, recorded.lines().map(str::to_owned).collect())
}

#[test]
fn a_search_given_a_dockerfile_exits_with_0() {
    assert_eq!(ran(SOLVES_T1).status, Some(0));
}

#[test]
fn a_search_given_a_dockerfile_prints_the_harness_made_in_the_image_it_built() {
    let ran = ran(SOLVES_T1);
    assert_eq!(
        ran.output,
        format!(
            "{}\n",
            ran.directory.path().join("run/harnesses/1").display()
        )
    );
}

#[test]
fn the_image_cladenc_builds_is_named_cladenc_and_sixteen_hexadecimal_digits() {
    assert!(is_named_by_cladenc(name_of_the_image(&ran(SOLVES_T1))));
}

#[test]
fn the_image_stays_on_the_machine_when_the_search_ends() {
    assert!(docker_holds(name_of_the_image(&ran(SOLVES_T1))));
}

#[test]
fn the_run_directory_keeps_the_dockerfile_and_what_docker_printed() {
    assert_eq!(
        kept_of_the_image(&ran(SOLVES_T1)),
        ["Dockerfile", "context", "stderr", "stdout"]
    );
}

#[test]
fn a_dockerfile_that_is_not_built_exits_with_2() {
    assert_eq!(ran(FAILS).status, Some(2));
}

#[test]
fn a_dockerfile_that_is_not_built_is_reported_last_on_standard_error() {
    assert!(
        ran(FAILS)
            .said
            .lines()
            .last()
            .unwrap()
            .starts_with("cladenc: the image of the search was not built: its build exited with ")
    );
}

#[test]
fn a_dockerfile_that_is_not_built_prints_nothing_on_standard_output() {
    assert_eq!(ran(FAILS).output, "");
}

#[test]
fn a_dockerfile_that_is_not_built_leaves_no_journal() {
    assert!(
        !ran(FAILS)
            .directory
            .path()
            .join("run/journal.jsonl")
            .exists()
    );
}

#[test]
fn docker_is_asked_to_build_from_the_copy_of_the_dockerfile_with_an_empty_directory_of_files() {
    let (directory, arguments) = arguments_of_a_docker_that_records_them();
    let kept = directory.path().join("run/image");
    assert_eq!(
        [&arguments[..2], &arguments[3..6]].concat(),
        [
            "build".to_owned(),
            "--tag".to_owned(),
            "--file".to_owned(),
            kept.join("Dockerfile").display().to_string(),
            kept.join("context").display().to_string(),
        ]
    );
}

#[test]
fn every_container_is_started_from_the_image_docker_was_asked_to_build() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    let started_from = format!("\n{}\nsh\n-c\n", arguments[2]);
    assert!(arguments.join("\n").contains(&started_from));
}

#[test]
fn docker_is_asked_to_build_once() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    assert_eq!(
        arguments
            .iter()
            .filter(|argument| *argument == "build")
            .count(),
        1
    );
}

#[test]
fn docker_is_not_told_to_fetch_anything() {
    let (_directory, arguments) = arguments_of_a_docker_that_records_them();
    assert!(
        !arguments
            .iter()
            .any(|argument| argument.starts_with("--pull"))
    );
}
