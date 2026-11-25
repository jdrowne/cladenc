use std::process::{Command, Output};

use tempfile::TempDir;

use crate::shared::ENDED;

fn report_on_standard_error(output: &Output) -> String {
    let printed = str::from_utf8(&output.stderr).unwrap();
    let finished = printed
        .find("cladenc: the directed evolution finished ")
        .unwrap();
    let report = finished + printed[finished..].find('\n').unwrap() + 1;
    printed[report..].to_owned()
}

fn report_of(directory: &TempDir) -> String {
    let report = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--outputs-dir={}",
            directory.path().join("outputs").display()
        ))
        .output()
        .unwrap();
    String::from_utf8(report.stdout).unwrap()
}

#[test]
fn a_run_over_the_sample_programs_exits_with_0() {
    let (_directory, output) = &*ENDED;
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn the_harness_a_run_prints_is_the_one_the_samples_make_best() {
    let (directory, output) = &*ENDED;
    assert_eq!(
        str::from_utf8(&output.stdout).unwrap(),
        format!(
            "{}\n",
            directory.path().join("outputs/harnesses/1").display()
        )
    );
}

#[test]
fn standard_error_ends_with_what_report_prints_of_the_directed_evolution() {
    let (directory, output) = &*ENDED;
    assert_eq!(report_on_standard_error(output), report_of(directory));
}
