use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use nix::sys::signal::kill;
use nix::unistd::Pid;
use tempfile::TempDir;

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

pub fn pid_is_written(pid_file: &Path) -> bool {
    fs::read_to_string(pid_file).is_ok_and(|pid| pid.ends_with('\n'))
}

pub fn process_is_gone(pid_file: &Path) -> bool {
    let pid = fs::read_to_string(pid_file)
        .expect("the file of the process id can be read")
        .trim()
        .parse()
        .expect("the file holds a process id");
    within_ten_seconds(|| kill(Pid::from_raw(pid), None).is_err())
}
