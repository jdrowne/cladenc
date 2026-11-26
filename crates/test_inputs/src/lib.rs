use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

const LOOK: Duration = Duration::from_millis(10);
const TEN_SECONDS: u32 = 1_000;
const TWO_MINUTES: u32 = 12_000;

const MARK: char = '\u{1e}';

const ROFF_ESCAPES: [(&str, &str); 6] = [
    ("\\-", "-"),
    ("\\(aq", "'"),
    ("\\fB", ""),
    ("\\fP", ""),
    ("\\&", ""),
    ("\\e", "\\"),
];

const MANUAL_PAGES: [(&str, &str); 5] = [
    ("cladenc", include_str!("../../../docs/man/man1/cladenc.1")),
    (
        "cladenc-report",
        include_str!("../../../docs/man/man1/cladenc-report.1"),
    ),
    (
        "cladenc-resume",
        include_str!("../../../docs/man/man1/cladenc-resume.1"),
    ),
    (
        "cladenc-start",
        include_str!("../../../docs/man/man1/cladenc-start.1"),
    ),
    (
        "cladenc-status",
        include_str!("../../../docs/man/man1/cladenc-status.1"),
    ),
];

pub const README: &str = include_str!("../../../README.md");

pub const PACKAGE: &str = include_str!("../../../etc/nfpm/nfpm.yaml");

pub const IMAGE: &str = "debian:stable-slim";

pub const MODEL: &str = "a-model";

pub const MODEL_URL: &str = "http://models.example/v1";

pub const MODEL_KEY: &str = "a-made-up-key";

pub const SOLVE_ONE_MORE: &str = r#"if [ -d /var/log/cladenc/trials ]
then
  touch solved
  echo "t$(($(wc -l < solved) + 1))" >> solved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub const SOLVE_EVERY_TASK: &str = r#"if [ -d /var/log/cladenc/trials ]
then
  for task in t1 t2 t3 t4 t5; do echo "$task"; done > solved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub const SOLVE_ONE_MORE_OR_END_WITH_125: &str = r#"if [ -d /var/log/cladenc/trials ]
then
  touch solved
  echo "t$(($(wc -l < solved) + 1))" >> solved
else
  [ -e end-with-125 ] && exit 125
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub const SOLVE_WHAT_THE_IMAGE_SOLVES: &str = r#"if [ -d /var/log/cladenc/trials ]
then
  cp /solved-by-the-image solved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

pub struct Image {
    pub name: String,
}

pub trait Files {
    fn make_directories(&self, directory: &Path);

    fn write(&self, file: &Path, text: &str);

    fn set_mode(&self, file: &Path, mode: u32);

    fn link(&self, target: &str, link: &Path);
}

pub struct Disk;

impl Files for Disk {
    fn make_directories(&self, directory: &Path) {
        fs::create_dir_all(directory).expect("the directory can be made");
    }

    fn write(&self, file: &Path, text: &str) {
        fs::write(file, text).expect("the file can be written");
    }

    fn set_mode(&self, file: &Path, mode: u32) {
        fs::set_permissions(file, fs::Permissions::from_mode(mode))
            .expect("the file can be given its permission bits");
    }

    fn link(&self, target: &str, link: &Path) {
        symlink(target, link).expect("the symbolic link can be made");
    }
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

pub fn environment_image_repository() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "cladenc-test-environments-{}-{}",
        process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    )
}

pub fn dockerfile(directory: &Path, steps: &str) -> PathBuf {
    dockerfile_in(&Disk, directory, steps)
}

pub fn dockerfile_in(files: &dyn Files, directory: &Path, steps: &str) -> PathBuf {
    let file = directory.join("Dockerfile");
    files.write(&file, &format!("FROM {IMAGE}\n{steps}"));
    file
}

pub fn printed_in_a_sandbox_of(image: &str, command: &str) -> String {
    let ran = Command::new("docker")
        .args(["run", "--rm", image, "sh", "-c", command])
        .output()
        .expect("docker can be called");
    String::from_utf8(ran.stdout).expect("what the sandbox printed is text")
}

pub fn temporary_directory() -> TempDir {
    tempfile::tempdir().expect("a temporary directory can be made")
}

pub fn primogenitor(directory: &Path) -> PathBuf {
    primogenitor_in(&Disk, directory)
}

pub fn primogenitor_in(files: &dyn Files, directory: &Path) -> PathBuf {
    let primogenitor = directory.join("primogenitor");
    let script = primogenitor.join("harness.sh");
    files.make_directories(&primogenitor.join("prompts"));
    files.write(&script, "#!/bin/sh\necho harness\n");
    files.set_mode(&script, 0o750);
    files.write(
        &primogenitor.join("prompts/system.txt"),
        "Solve the task.\n",
    );
    files.link("prompts/system.txt", &primogenitor.join("prompt.txt"));
    primogenitor
}

pub fn benchmark(directory: &Path, tasks: &[&str]) -> PathBuf {
    benchmark_in(&Disk, directory, tasks)
}

pub fn benchmark_in(files: &dyn Files, directory: &Path, tasks: &[&str]) -> PathBuf {
    benchmark_named_in(files, directory, "benchmark", tasks)
}

pub fn benchmark_named_in(
    files: &dyn Files,
    directory: &Path,
    name: &str,
    tasks: &[&str],
) -> PathBuf {
    let benchmark = directory.join(name);
    files.make_directories(&benchmark);
    for id in tasks {
        task_in(files, &benchmark, id);
    }
    benchmark
}

pub fn task(benchmark: &Path, id: &str) -> PathBuf {
    task_in(&Disk, benchmark, id)
}

pub fn task_in(files: &dyn Files, benchmark: &Path, id: &str) -> PathBuf {
    let task = benchmark.join(id);
    let verifier = task.join("tests/test.sh");
    files.make_directories(&task.join("tests"));
    files.make_directories(&task.join("environment/workspace"));
    files.write(
        &task.join("environment/Dockerfile"),
        &format!("FROM {IMAGE}\nWORKDIR /app\nCOPY workspace .\n"),
    );
    files.write(
        &task.join("instruction.md"),
        &format!("Finish the work of {id}.\n"),
    );
    files.write(&task.join("task.toml"), "schema_version = \"1.4\"\n");
    files.write(&task.join("environment/workspace/work.txt"), "to do\n");
    files.write(
        &verifier,
        &format!(
            "#!/bin/sh\nif grep -q -x -e {id} solved; then echo 1; else echo 0; fi \
             > /logs/verifier/reward.txt\n"
        ),
    );
    files.set_mode(&verifier, 0o750);
    task
}

pub fn kept_trial(outputs_directory: &Path, harness: u64, task: &str) -> PathBuf {
    ["trials", "holdout"]
        .iter()
        .map(|kept| {
            outputs_directory
                .join(kept)
                .join(harness.to_string())
                .join(task)
        })
        .find(|trial| trial.is_dir())
        .expect("the trial is kept in the outputs directory")
}

pub fn without_seconds(journal: &str) -> String {
    journal
        .lines()
        .map(|line| match line.split_once(",\"seconds\":") {
            Some((record, _)) => format!("{record}}}\n"),
            None => format!("{line}\n"),
        })
        .collect()
}

pub fn manual(page: &str) -> String {
    let roff = MANUAL_PAGES
        .iter()
        .find(|(name, _)| *name == page)
        .map(|(_, roff)| *roff)
        .expect("the manual page is one of those that are written");
    text_of_roff(roff)
}

pub fn manual_on_one_line(page: &str) -> String {
    manual(page)
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn text_of_roff(roff: &str) -> String {
    roff.lines()
        .map(text_of_roff_line)
        .collect::<Vec<String>>()
        .join("\n")
}

fn text_of_roff_line(line: &str) -> String {
    let said = match line.strip_prefix('.') {
        None => line,
        Some(request) => request
            .strip_prefix("SH ")
            .or_else(|| request.strip_prefix("SS "))
            .unwrap_or_default(),
    };
    ROFF_ESCAPES
        .iter()
        .fold(said.to_owned(), |text, (escape, meaning)| {
            text.replace(escape, meaning)
        })
}

pub fn within_ten_seconds(holds: impl FnMut() -> bool) -> bool {
    within(TEN_SECONDS, holds)
}

fn within(looks: u32, mut holds: impl FnMut() -> bool) -> bool {
    (0..looks).any(|_| {
        thread::sleep(LOOK);
        holds()
    })
}

pub fn line_is_written(file: &Path) -> bool {
    fs::read_to_string(file).is_ok_and(|line| line.ends_with('\n'))
}

pub fn wait_for_a_line(program: &mut Child, file: &Path, printed: &Path) {
    within(TWO_MINUTES, || {
        line_is_written(file) || program.try_wait().is_ok_and(|ended| ended.is_some())
    });
    if !line_is_written(file) {
        let _ = program.kill();
        let ended = program.wait().expect("the program can be waited for");
        let printed = fs::read_to_string(printed).unwrap_or_default();
        panic!("the program wrote no line before it ended with {ended}, and printed:\n{printed}");
    }
}

pub fn sandbox_is_gone(id_file: &Path) -> bool {
    let id = fs::read_to_string(id_file).expect("the file of the sandbox id can be read");
    let filter = format!("id={}", id.trim());
    within_ten_seconds(|| {
        Command::new("docker")
            .args(["ps", "--all", "--quiet", "--filter", &filter])
            .output()
            .is_ok_and(|listed| listed.status.success() && listed.stdout.is_empty())
    })
}

pub fn sandbox_is_running(id_file: &Path) -> bool {
    let id = fs::read_to_string(id_file).expect("the file of the sandbox id can be read");
    let filter = format!("id={}", id.trim());
    Command::new("docker")
        .args(["ps", "--quiet", "--filter", &filter])
        .output()
        .is_ok_and(|listed| listed.status.success() && !listed.stdout.is_empty())
}

pub fn end_with_125_on(directory: &Path, task: &str) {
    let marker = directory
        .join("benchmark")
        .join(task)
        .join("environment/workspace/end-with-125");
    fs::write(marker, "").expect("the marker can be written");
}

pub fn end_with_125_on_no_more(outputs_directory: &Path, task: &str) {
    let marker = outputs_directory
        .join("benchmarks/benchmark")
        .join(task)
        .join("environment/workspace/end-with-125");
    fs::remove_file(marker).expect("the marker can be removed");
}

pub fn kept_verifier(outputs_directory: &Path, task: &str, command: &str) {
    let verifier = outputs_directory
        .join("benchmarks/benchmark")
        .join(task)
        .join("tests/test.sh");
    fs::write(verifier, format!("#!/bin/sh\n{command}\n")).expect("the verifier can be written");
}

pub fn model_key_file(directory: &Path, text: &str) -> PathBuf {
    model_key_file_in(&Disk, directory, text)
}

pub fn model_key_file_in(files: &dyn Files, directory: &Path, text: &str) -> PathBuf {
    let file = directory.join("model-key");
    files.write(&file, text);
    file
}

pub fn one_after_another(commands: &[&str]) -> String {
    commands
        .iter()
        .map(|command| format!("printf '\\036'\n{command}\n"))
        .collect()
}

pub fn printed_by(command: &str, commands: &[&str], printed: &str) -> String {
    let place = commands
        .iter()
        .position(|one| *one == command)
        .expect("the command is one of those that were run");
    printed
        .split(MARK)
        .nth(place + 1)
        .expect("every command was run after its mark")
        .to_owned()
}
