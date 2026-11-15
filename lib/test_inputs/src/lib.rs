use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use nix::sys::signal::kill;
use nix::unistd::Pid;
use tempfile::TempDir;

pub const IMAGE: &str = "busybox:1.37.0";

pub const MODEL: &str = "a-model";

pub const MODEL_URL: &str = "http://models.example/v1";

pub const MODEL_KEY: &str = "a-made-up-key";

pub const SOLVE_ONE_MORE: &str = r#"touch solved
echo "t$(($(wc -l < solved) + 1))" >> solved"#;

pub const SOLVE_ONE_MORE_OR_END_WITH_125: &str = r#"touch solved
solved=$(($(wc -l < solved)))
[ -e end-with-125-at ] && [ "$solved" -eq "$(cat end-with-125-at)" ] && exit 125
echo "t$((solved + 1))" >> solved"#;

pub fn temporary_directory() -> TempDir {
    tempfile::tempdir().expect("a temporary directory can be made")
}

pub fn progenitor(directory: &Path) -> PathBuf {
    let progenitor = directory.join("progenitor");
    let script = progenitor.join("harness.sh");
    fs::create_dir_all(progenitor.join("prompts")).expect("the progenitor can be made");
    fs::write(&script, "#!/bin/sh\necho harness\n").expect("the script can be written");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o750))
        .expect("the script can be given its permission bits");
    fs::write(progenitor.join("prompts/system.txt"), "Solve the task.\n")
        .expect("the prompt can be written");
    symlink("prompts/system.txt", progenitor.join("prompt.txt"))
        .expect("the symbolic link can be made");
    progenitor
}

pub fn task_file(directory: &Path, tasks: &[&str]) -> PathBuf {
    let task_file = directory.join("tasks.txt");
    let lines: String = tasks.iter().map(|task| format!("{task}\n")).collect();
    fs::write(&task_file, lines).expect("the task file can be written");
    task_file
}

pub fn sample(tier: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test")
        .join(tier)
        .join("samples")
        .join(name)
}

pub fn within_ten_seconds(mut holds: impl FnMut() -> bool) -> bool {
    (0..1000).any(|_| {
        thread::sleep(Duration::from_millis(10));
        holds()
    })
}

pub fn line_is_written(file: &Path) -> bool {
    fs::read_to_string(file).is_ok_and(|line| line.ends_with('\n'))
}

pub fn process_is_gone(pid_file: &Path) -> bool {
    let pid = fs::read_to_string(pid_file)
        .expect("the file of the process id can be read")
        .trim()
        .parse()
        .expect("the file holds a process id");
    within_ten_seconds(|| kill(Pid::from_raw(pid), None).is_err())
}

pub fn container_is_gone(id_file: &Path) -> bool {
    let id = fs::read_to_string(id_file).expect("the file of the container id can be read");
    let filter = format!("id={}", id.trim());
    within_ten_seconds(|| {
        Command::new("docker")
            .args(["ps", "--all", "--quiet", "--filter", &filter])
            .output()
            .is_ok_and(|listed| listed.status.success() && listed.stdout.is_empty())
    })
}

pub fn end_with_125_at(progenitor: &Path, solved: u64) {
    fs::write(progenitor.join("end-with-125-at"), format!("{solved}\n"))
        .expect("the marker can be written");
}

pub fn end_with_125_no_more(run_directory: &Path) {
    for harness in fs::read_dir(run_directory.join("harnesses")).expect("the harnesses are there") {
        let marker = harness
            .expect("a harness can be listed")
            .path()
            .join("end-with-125-at");
        if marker.exists() {
            fs::remove_file(marker).expect("the marker can be removed");
        }
    }
}

pub fn model_key_file(directory: &Path, text: &str) -> PathBuf {
    let file = directory.join("model-key");
    fs::write(&file, text).expect("the file of the key can be written");
    file
}
