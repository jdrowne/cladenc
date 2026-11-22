---
name: ci-runs-the-test-binaries-directly
description: "release.yml compiles each test tier once and runs the test binary by its path, not through cargo, so a test finds none of the variables cargo sets when it runs a test, such as CARGO_PKG_NAME; a test that needs a variable from where it runs reads PATH"
metadata:
  type: project
---

# CI runs the test binaries directly

`.github/workflows/release.yml` compiles each test tier once, with `cargo test --no-run`, and the jobs that test run the binary by its path. No `cargo` starts the tests there.

**Why:** `cargo test` puts variables of its own into the environment of a test, `CARGO_PKG_NAME` and `CARGO_MANIFEST_DIR` among them, and CI starts the tests without it. A test that reads one of them when it runs fails in CI, the only place the tests run, per [[only-quick-checks-run-before-a-push]], which costs a run and a fix, per [[a-failed-run-is-fixed-before-other-work]].

**How to apply:** A test that needs a variable that is set where the test runs reads `PATH`, which the end-to-end tier already relies on, and compares with what `env::var("PATH")` gives. `env!("CARGO_MANIFEST_DIR")` and `env!("CARGO_BIN_EXE_cladenc")` are read when the test is compiled, and work in both places. Related: [[only-quick-checks-run-before-a-push]], [[the-checks-the-code-faces]].
