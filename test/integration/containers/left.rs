use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::{self, Launch};
use cladenc::model::Model;
use cladenc::outputs_directory::OutputsDirectory;
use tempfile::TempDir;

const SAY_THE_CONTAINER_AND_STAY: &str = "hostname > id; sleep 300";

struct Left {
    directory: TempDir,
    id_file: PathBuf,
}

impl Drop for Left {
    fn drop(&mut self) {
        let id = fs::read_to_string(&self.id_file).unwrap_or_default();
        let _ = Command::new("docker")
            .args(["rm", "--force", id.trim()])
            .output();
    }
}

fn label_of(path: &Path) -> String {
    format!(
        "cladenc.directed-evolution={}",
        fs::canonicalize(path).unwrap().display()
    )
}

fn left_by_a_directed_evolution_in(outputs: &str) -> Left {
    let directory = test_inputs::temporary_directory();
    let path = directory.path().join(outputs);
    fs::create_dir(&path).unwrap();
    let started = Command::new("docker")
        .args(["run", "--detach", "--rm", "--pull", "never", "--label"])
        .arg(label_of(&path))
        .args([test_inputs::IMAGE, "sleep", "300"])
        .output()
        .unwrap();
    let id_file = directory.path().join("id");
    fs::write(&id_file, started.stdout).unwrap();
    Left { directory, id_file }
}

fn removed_from(left: &Left, outputs: &str) -> io::Result<u64> {
    containers::remove_left_behind(&OutputsDirectory::at(&left.directory.path().join(outputs)))
}

fn the_container_of_an_attempt_is_listed_under_the_label_of_its_directed_evolution() -> bool {
    let directory = test_inputs::temporary_directory();
    let primogenitor = test_inputs::primogenitor(directory.path());
    let path = directory.path().join("outputs");
    let outputs_directory = OutputsDirectory::create(&path, &primogenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    outputs_directory
        .copy_benchmark(&Benchmark::read(&benchmark).unwrap())
        .unwrap();
    let launch = Launch {
        image: test_inputs::IMAGE,
        run_command: SAY_THE_CONTAINER_AND_STAY,
        build_command: None,
        model: Model {
            name: test_inputs::MODEL,
            url: test_inputs::MODEL_URL,
            key: None,
        },
        procreation_timeout: 60,
        attempt_timeout: 60,
    };
    let id_file = outputs_directory
        .running_attempt(0, "benchmark/t1")
        .join("work/id");
    let stop = AtomicBool::new(false);
    thread::scope(|scope| {
        let listed = scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            let listed = Command::new("docker")
                .args(["ps", "--quiet", "--no-trunc", "--filter"])
                .arg(format!("label={}", label_of(&path)))
                .output()
                .unwrap();
            stop.store(true, Ordering::SeqCst);
            String::from_utf8(listed.stdout).unwrap()
        });
        let _ = containers::run_attempt(&outputs_directory, &launch, 0, "benchmark/t1", &stop);
        let id = fs::read_to_string(&id_file).unwrap();
        listed.join().unwrap().starts_with(id.trim())
    })
}

#[test]
fn a_container_that_carries_the_label_of_the_directed_evolution_is_removed() {
    let left = left_by_a_directed_evolution_in("outputs");
    let _ = removed_from(&left, "outputs");
    assert!(test_inputs::container_is_gone(&left.id_file));
}

#[test]
fn the_number_of_containers_removed_is_returned() {
    let left = left_by_a_directed_evolution_in("outputs");
    assert_eq!(removed_from(&left, "outputs").ok(), Some(1));
}

#[test]
fn a_directed_evolution_that_left_no_container_has_none_removed() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        containers::remove_left_behind(&OutputsDirectory::at(directory.path())).ok(),
        Some(0)
    );
}

#[test]
fn an_outputs_directory_that_is_not_there_has_no_container_removed() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        containers::remove_left_behind(&OutputsDirectory::at(&directory.path().join("missing")))
            .ok(),
        Some(0)
    );
}

#[test]
fn the_container_of_another_directed_evolution_is_left_running() {
    let left = left_by_a_directed_evolution_in("another");
    fs::create_dir(left.directory.path().join("outputs")).unwrap();
    let _ = removed_from(&left, "outputs");
    assert!(test_inputs::container_is_running(&left.id_file));
}

#[test]
fn a_directed_evolution_reached_through_a_symbolic_link_is_the_same_directed_evolution() {
    let left = left_by_a_directed_evolution_in("outputs");
    symlink(
        left.directory.path().join("outputs"),
        left.directory.path().join("link"),
    )
    .unwrap();
    assert_eq!(removed_from(&left, "link").ok(), Some(1));
}

#[test]
fn the_container_of_an_attempt_carries_the_label_of_its_directed_evolution() {
    assert!(the_container_of_an_attempt_is_listed_under_the_label_of_its_directed_evolution());
}
