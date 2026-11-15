use std::process::{Command, Output};

fn version() -> Output {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("--version")
        .output()
        .unwrap()
}

#[test]
fn the_version_is_printed_as_the_name_and_the_version_of_the_package() {
    assert_eq!(
        String::from_utf8(version().stdout).unwrap(),
        format!("cladenc {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn asking_for_the_version_exits_with_0() {
    assert_eq!(version().status.code(), Some(0));
}
