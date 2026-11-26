use std::path::Path;
use std::process::{Command, Output};

use crate::shared::ENDED;

fn report(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--outputs-dir={}",
            directory.join("outputs").display()
        ))
        .output()
        .unwrap()
}

fn table_of(directory: &Path) -> String {
    String::from_utf8(report(directory).stdout).unwrap()
}

fn up_to_the_seconds(table: &str) -> String {
    let seconds = table
        .lines()
        .nth(7)
        .and_then(|header| header.find("trial seconds"))
        .unwrap();
    let above: String = table
        .lines()
        .take(7)
        .map(|line| format!("{line}\n"))
        .collect();
    let rows: String = table
        .lines()
        .skip(7)
        .map(|line| format!("{}\n", line.get(..seconds).unwrap_or(line).trim_end()))
        .collect();
    above + &rows
}

#[test]
fn report_prints_the_table() {
    let (directory, _run) = &*ENDED;
    assert_eq!(
        up_to_the_seconds(&table_of(directory.path())),
        "model a-model at http://models.example/v1\n\
         started 1 of 1 diversification, best harness 1\n\
         completed by at least one harness: selection tasks 1 of 1, corroboration tasks 1 of 1, non-holdout tasks 1 of 1\n\
         on the corroboration tasks: only the best harness completed 1, only the primogenitor completed 0, and luck alone gives a lead at least as large with likelihood 0.500\n\
         experiments: conducted 0, the progenitor's child created in 0, the singleton control group harness created in 0, both created in 0\n\
         on the corroboration tasks of the experiments: no experiment created both the progenitor's child and the singleton control group harness, so there are no two harnesses to compare\n\
         \n\
         id  progenitor  experiment  state    selection completed  selection not completed  corroboration completed  corroboration not completed  non-holdout completed  non-holdout not completed  clade completed  clade not completed  clade score\n\
         0   -           -           created  0                    1                        0                        1                            0                      1                          1                1                    0.500\n\
         1   0           -           created  1                    0                        1                        0                            1                      0                          1                0                    1.000\n"
    );
}

#[test]
fn report_exits_with_0() {
    let (directory, _run) = &*ENDED;
    assert_eq!(report(directory.path()).status.code(), Some(0));
}
