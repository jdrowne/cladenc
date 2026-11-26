use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use cladenc::sandboxes::{Docker, Program};
use tempfile::TempDir;

const LEAVE: &str = "mkdir -p left/locked/inner left/empty && echo found > left/report.txt && \
                     echo kept > left/locked/inner/file && ln -s report.txt left/link && ln \
                     left/report.txt left/twin.txt && mkfifo left/pipe && ln left/pipe \
                     left/pipe-twin && printf '#!/bin/sh\\n' > left/run.sh && chmod 750 \
                     left/run.sh && chmod 500 left/locked/inner left/locked && tar -cf left.tar \
                     -C left . && chmod -R u+w left";
const PRINT_THE_ARCHIVE: &str = "cat left.tar";
const NOTHING: &str = "true";
const EVERY_BIT: u32 = 0o7777;

struct Unpacked {
    directory: TempDir,
    result: io::Result<()>,
}

impl Unpacked {
    fn at(&self, path: &str) -> PathBuf {
        self.directory.path().join("into").join(path)
    }

    fn bits_of(&self, path: &str) -> u32 {
        fs::symlink_metadata(self.at(path))
            .unwrap()
            .permissions()
            .mode()
            & EVERY_BIT
    }

    fn reason(&self) -> String {
        self.result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .unwrap_or_default()
    }
}

fn sh(directory: &Path, script: &str) -> Command {
    let mut command = Command::new("sh");
    command.args(["-c", script]).current_dir(directory);
    command
}

fn unpacked_where(prepare: &str, print: &str) -> Unpacked {
    let directory = test_inputs::temporary_directory();
    let made = sh(directory.path(), &format!("{LEAVE} && {prepare}"))
        .status()
        .unwrap();
    made.success()
        .then_some(())
        .expect("the archive can be made");
    let result = Program.unpack(sh(directory.path(), print), &directory.path().join("into"));
    Unpacked { directory, result }
}

fn unpacked(print: &str) -> Unpacked {
    unpacked_where(NOTHING, print)
}

static UNPACKED: LazyLock<Unpacked> = LazyLock::new(|| unpacked(PRINT_THE_ARCHIVE));

static NOT_PRINTED: LazyLock<Unpacked> =
    LazyLock::new(|| unpacked("echo ' no such container ' >&2; exit 1"));

#[test]
fn an_archive_that_a_command_prints_is_unpacked() {
    assert!(UNPACKED.result.is_ok());
}

#[test]
fn a_file_of_an_archive_is_unpacked_with_what_it_holds() {
    assert_eq!(
        fs::read_to_string(UNPACKED.at("report.txt")).unwrap(),
        "found\n"
    );
}

#[test]
fn a_file_in_a_directory_that_cannot_be_written_in_is_unpacked() {
    assert_eq!(
        fs::read_to_string(UNPACKED.at("locked/inner/file")).unwrap(),
        "kept\n"
    );
}

#[test]
fn a_directory_that_cannot_be_written_in_is_unpacked_with_its_permission_bits() {
    assert_eq!(UNPACKED.bits_of("locked"), 0o500);
}

#[test]
fn a_file_is_unpacked_with_its_permission_bits() {
    assert_eq!(UNPACKED.bits_of("run.sh"), 0o750);
}

#[test]
fn a_directory_that_holds_nothing_is_unpacked() {
    assert!(UNPACKED.at("empty").is_dir());
}

#[test]
fn a_symbolic_link_is_unpacked_as_a_link_to_the_same_target() {
    assert_eq!(
        fs::read_link(UNPACKED.at("link")).unwrap(),
        Path::new("report.txt")
    );
}

#[test]
fn a_second_name_of_a_file_is_unpacked_with_what_the_file_holds() {
    assert_eq!(
        fs::read_to_string(UNPACKED.at("twin.txt")).unwrap(),
        "found\n"
    );
}

#[test]
fn a_named_pipe_is_left_out_of_what_is_unpacked() {
    assert!(fs::symlink_metadata(UNPACKED.at("pipe")).is_err());
}

#[test]
fn a_second_name_of_a_named_pipe_is_left_out_of_what_is_unpacked() {
    assert!(fs::symlink_metadata(UNPACKED.at("pipe-twin")).is_err());
}

#[test]
fn what_a_command_prints_once_its_archive_has_ended_is_read_to_its_end() {
    let unpacked = unpacked("cat left.tar; sleep 1; head -c 1000000 /dev/zero");
    assert!(unpacked.result.is_ok());
}

#[test]
fn a_command_that_fails_is_an_error_of_what_it_said_went_wrong() {
    assert_eq!(NOT_PRINTED.reason(), "no such container");
}

#[test]
fn a_command_that_printed_nothing_has_no_directory_made() {
    assert!(fs::symlink_metadata(NOT_PRINTED.at("")).is_err());
}

#[test]
fn a_command_that_fails_and_says_nothing_is_an_error_of_its_exit_status() {
    assert_eq!(unpacked("exit 3").reason(), "exit status: 3");
}

#[test]
fn a_command_that_fails_once_it_has_printed_an_archive_is_an_error_all_the_same() {
    let unpacked = unpacked("cat left.tar; echo 'the sandbox is gone' >&2; exit 1");
    assert_eq!(unpacked.reason(), "the sandbox is gone");
}

#[test]
fn a_command_that_fails_is_the_error_where_what_it_printed_is_no_archive_either() {
    let unpacked = unpacked("echo this is no archive; echo 'the sandbox is gone' >&2; exit 1");
    assert_eq!(unpacked.reason(), "the sandbox is gone");
}

#[test]
fn a_command_whose_program_cannot_be_started_is_an_error() {
    let directory = test_inputs::temporary_directory();
    let unpacked = Program.unpack(
        Command::new("/no/such/program"),
        &directory.path().join("into"),
    );
    assert!(unpacked.is_err());
}

#[test]
fn what_is_no_archive_is_an_error() {
    assert!(unpacked("echo this is no archive").result.is_err());
}

#[test]
fn an_archive_is_not_unpacked_where_a_file_is_in_the_place_of_the_directory() {
    assert!(
        unpacked_where("touch into", PRINT_THE_ARCHIVE)
            .result
            .is_err()
    );
}

#[test]
fn a_file_is_not_unpacked_where_a_directory_that_holds_something_has_its_name() {
    assert!(
        unpacked_where("mkdir -p into/run.sh/held", PRINT_THE_ARCHIVE)
            .result
            .is_err()
    );
}

#[test]
fn a_directory_is_not_unpacked_where_a_file_has_its_name() {
    assert!(
        unpacked_where("mkdir into && touch into/empty", PRINT_THE_ARCHIVE)
            .result
            .is_err()
    );
}
