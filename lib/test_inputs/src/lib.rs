use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

pub const IMAGE: &str = "busybox:1.37.0";

pub const MODEL: &str = "a-model";

pub const MODEL_URL: &str = "http://models.example/v1";

pub const MODEL_KEY: &str = "a-made-up-key";

pub const SOLVE_ONE_MORE: &str = r#"if [ -d /evaluations ]
then
  touch solved
  echo "t$(($(wc -l < solved) + 1))" >> solved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub const SOLVE_ONE_MORE_OR_END_WITH_125: &str = r#"if [ -d /evaluations ]
then
  touch solved
  solved=$(($(wc -l < solved)))
  [ -e end-with-125-at ] && [ "$solved" -eq "$(cat end-with-125-at)" ] && exit 125
  echo "t$((solved + 1))" >> solved
else
  [ -e end-with-125 ] && exit 125
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub const SOLVE_WHAT_THE_IMAGE_SOLVES: &str = r#"if [ -d /evaluations ]
then
  cp /solved-by-the-image solved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub struct Image {
    pub name: String,
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rmi", "--force", &self.name])
            .output();
    }
}

pub fn image() -> Image {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    Image {
        name: format!(
            "cladenc-test-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ),
    }
}

pub fn dockerfile(directory: &Path, steps: &str) -> PathBuf {
    let file = directory.join("Dockerfile");
    fs::write(&file, format!("FROM {IMAGE}\n{steps}")).expect("the Dockerfile can be written");
    file
}

pub fn printed_in_a_container_of(image: &str, command: &str) -> String {
    let ran = Command::new("docker")
        .args(["run", "--rm", image, "sh", "-c", command])
        .output()
        .expect("docker can be called");
    String::from_utf8(ran.stdout).expect("what the container printed is text")
}

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

pub fn benchmark(directory: &Path, tasks: &[&str]) -> PathBuf {
    benchmark_named(directory, "benchmark", tasks)
}

pub fn benchmark_named(directory: &Path, name: &str, tasks: &[&str]) -> PathBuf {
    let benchmark = directory.join(name);
    fs::create_dir_all(&benchmark).expect("the benchmark can be made");
    for id in tasks {
        task(&benchmark, id);
    }
    benchmark
}

pub fn task(benchmark: &Path, id: &str) -> PathBuf {
    let task = benchmark.join(id);
    let check = task.join("check");
    fs::create_dir_all(task.join("files")).expect("the task can be made");
    fs::write(
        task.join("statement"),
        format!("Finish the work of {id}.\n"),
    )
    .expect("the statement can be written");
    fs::write(task.join("files/work.txt"), "to do\n").expect("the file can be written");
    fs::write(&check, format!("#!/bin/sh\ngrep -q -x -e {id} solved\n"))
        .expect("the check can be written");
    fs::set_permissions(&check, fs::Permissions::from_mode(0o750))
        .expect("the check can be given its permission bits");
    task
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

pub fn end_with_125_on(directory: &Path, task: &str) {
    let marker = directory
        .join("benchmark")
        .join(task)
        .join("files/end-with-125");
    fs::write(marker, "").expect("the marker can be written");
}

pub fn end_with_125_on_no_more(run_directory: &Path, task: &str) {
    let marker = run_directory
        .join("benchmarks/benchmark")
        .join(task)
        .join("files/end-with-125");
    fs::remove_file(marker).expect("the marker can be removed");
}

pub fn check(directory: &Path, task: &str, command: &str) {
    let check = directory.join("benchmark").join(task).join("check");
    fs::write(check, format!("#!/bin/sh\n{command}\n")).expect("the check can be written");
}

pub fn kept_check(run_directory: &Path, task: &str, command: &str) {
    check(&run_directory.join("benchmarks"), task, command);
}

pub fn model_key_file(directory: &Path, text: &str) -> PathBuf {
    let file = directory.join("model-key");
    fs::write(&file, text).expect("the file of the key can be written");
    file
}
