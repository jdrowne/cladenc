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
use cladenc::workspace::Workspace;
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

fn label_of(run_directory: &Path) -> String {
    format!(
        "cladenc.search={}",
        fs::canonicalize(run_directory).unwrap().display()
    )
}

fn left_by_a_search_in(run: &str) -> Left {
    let directory = test_inputs::temporary_directory();
    let run_directory = directory.path().join(run);
    fs::create_dir(&run_directory).unwrap();
    let started = Command::new("docker")
        .args(["run", "--detach", "--rm", "--pull", "never", "--label"])
        .arg(label_of(&run_directory))
        .args([test_inputs::IMAGE, "sleep", "300"])
        .output()
        .unwrap();
    let id_file = directory.path().join("id");
    fs::write(&id_file, started.stdout).unwrap();
    Left { directory, id_file }
}

fn removed_from(left: &Left, run: &str) -> io::Result<u64> {
    containers::remove_left_behind(&Workspace::at(&left.directory.path().join(run)))
}

fn the_container_of_an_evaluation_is_listed_under_the_label_of_its_search() -> bool {
    let directory = test_inputs::temporary_directory();
    let progenitor = test_inputs::progenitor(directory.path());
    let run_directory = directory.path().join("run");
    let workspace = Workspace::create(&run_directory, &progenitor).unwrap();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1"]);
    workspace
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
    };
    let id_file = workspace
        .running_evaluation(0, "benchmark/t1")
        .join("work/id");
    let stop = AtomicBool::new(false);
    thread::scope(|scope| {
        let listed = scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            let listed = Command::new("docker")
                .args(["ps", "--quiet", "--no-trunc", "--filter"])
                .arg(format!("label={}", label_of(&run_directory)))
                .output()
                .unwrap();
            stop.store(true, Ordering::SeqCst);
            String::from_utf8(listed.stdout).unwrap()
        });
        let _ = containers::run_evaluation(&workspace, &launch, 0, "benchmark/t1", 60, &stop);
        let id = fs::read_to_string(&id_file).unwrap();
        listed.join().unwrap().starts_with(id.trim())
    })
}

#[test]
fn a_container_that_carries_the_label_of_the_search_is_removed() {
    let left = left_by_a_search_in("run");
    let _ = removed_from(&left, "run");
    assert!(test_inputs::container_is_gone(&left.id_file));
}

#[test]
fn the_number_of_containers_removed_is_returned() {
    let left = left_by_a_search_in("run");
    assert_eq!(removed_from(&left, "run").ok(), Some(1));
}

#[test]
fn a_search_that_left_no_container_has_none_removed() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        containers::remove_left_behind(&Workspace::at(directory.path())).ok(),
        Some(0)
    );
}

#[test]
fn a_run_directory_that_is_not_there_has_no_container_removed() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        containers::remove_left_behind(&Workspace::at(&directory.path().join("missing"))).ok(),
        Some(0)
    );
}

#[test]
fn the_container_of_another_search_is_left_running() {
    let left = left_by_a_search_in("another");
    fs::create_dir(left.directory.path().join("run")).unwrap();
    let _ = removed_from(&left, "run");
    assert!(test_inputs::container_is_running(&left.id_file));
}

#[test]
fn a_search_reached_through_a_symbolic_link_is_the_same_search() {
    let left = left_by_a_search_in("run");
    symlink(
        left.directory.path().join("run"),
        left.directory.path().join("link"),
    )
    .unwrap();
    assert_eq!(removed_from(&left, "link").ok(), Some(1));
}

#[test]
fn the_container_of_an_evaluation_carries_the_label_of_its_search() {
    assert!(the_container_of_an_evaluation_is_listed_under_the_label_of_its_search());
}
