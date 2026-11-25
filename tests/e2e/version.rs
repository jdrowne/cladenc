use std::process::{Command, Output};
use std::sync::LazyLock;

static VERSION: LazyLock<Output> = LazyLock::new(|| {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("--version")
        .output()
        .unwrap()
});

#[test]
fn the_version_is_printed_as_the_name_and_the_version_of_the_package() {
    assert_eq!(
        str::from_utf8(&VERSION.stdout).unwrap(),
        format!("cladenc {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn asking_for_the_version_exits_with_0() {
    assert_eq!(VERSION.status.code(), Some(0));
}
