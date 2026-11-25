use std::path::Path;

use super::ContainerError;
use super::container::Up;

pub(super) const VERIFIER: &str = "/tests/test.sh";
pub(super) const VERIFIER_LOGS: &str = "/logs/verifier";

const LOGS: &str = "/logs";
const KEPT_LOGS: &str = "/var/log/cladenc/harbor";
const ARTIFACTS: &str = "/logs/artifacts";
const TESTS: &str = "/tests";
const GIVEN_TESTS: &str = "/var/opt/cladenc/harbor/tests";
const LINKED_WHERE_NOTHING_IS: &str =
    r#"[ -e "$1" ] || [ -L "$1" ] || { rm -rf "$2" && mkdir -p "$2" && ln -s "$2" "$1"; }"#;
const MADE: &str = r#"mkdir -p "$3" && chown "$4" "$3""#;
const EMPTIED: &str = r#"rm -rf "$3" && mkdir -p "$3" && chown "$4" "$3""#;
const LINKED: &str = r#"rm -rf "$1" && ln -s "$2" "$1""#;

pub(super) fn link_logs(up: &Up<'_>) -> Result<(), ContainerError> {
    up.run_as_root(LINKED_WHERE_NOTHING_IS, &[LOGS, KEPT_LOGS])
}

pub(super) fn make_artifacts(up: &Up<'_>, owner: &str) -> Result<(), ContainerError> {
    up.run_as_root(
        &format!("{LINKED_WHERE_NOTHING_IS} && {MADE}"),
        &[LOGS, KEPT_LOGS, ARTIFACTS, owner],
    )
}

pub(super) fn empty_verifier(up: &Up<'_>, owner: &str) -> Result<(), ContainerError> {
    up.run_as_root(
        &format!("{LINKED_WHERE_NOTHING_IS} && {EMPTIED}"),
        &[LOGS, KEPT_LOGS, VERIFIER_LOGS, owner],
    )
}

pub(super) fn give_tests(up: &Up<'_>, tests: &Path, owner: &str) -> Result<(), ContainerError> {
    up.put(tests, GIVEN_TESTS, owner)
        .and_then(|()| up.run_as_root(LINKED, &[TESTS, GIVEN_TESTS]))
}
