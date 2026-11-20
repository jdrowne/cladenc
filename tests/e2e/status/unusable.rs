use std::process::Command;

#[test]
fn status_on_a_missing_outputs_directory_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let status = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("status")
        .arg(format!(
            "--outputs-dir={}",
            directory.path().join("outputs").display()
        ))
        .output()
        .unwrap()
        .status;
    assert_eq!(status.code(), Some(2));
}
