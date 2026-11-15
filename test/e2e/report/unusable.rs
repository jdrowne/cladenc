use std::process::Command;

#[test]
fn report_on_a_missing_run_directory_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let status = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("report")
        .arg(format!(
            "--run-dir={}",
            directory.path().join("run").display()
        ))
        .output()
        .unwrap()
        .status;
    assert_eq!(status.code(), Some(2));
}
