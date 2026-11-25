use std::process::{Command, Output};
use std::sync::LazyLock;

use crate::shared::ENDED;

static STATUS: LazyLock<Output> = LazyLock::new(|| {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("status")
        .arg(format!(
            "--outputs-dir={}",
            ENDED.0.path().join("outputs").display()
        ))
        .output()
        .unwrap()
});

#[test]
fn status_exits_with_2_on_a_directed_evolution_that_has_ended() {
    assert_eq!(STATUS.status.code(), Some(2));
}

#[test]
fn status_says_of_a_directed_evolution_that_has_ended_to_call_report() {
    assert_eq!(
        str::from_utf8(&STATUS.stderr).unwrap(),
        "cladenc: the directed evolution has ended: call `cladenc report` to read its report\n"
    );
}

#[test]
fn status_prints_nothing_on_standard_output_of_a_directed_evolution_that_has_ended() {
    assert_eq!(STATUS.stdout, Vec::<u8>::new());
}
