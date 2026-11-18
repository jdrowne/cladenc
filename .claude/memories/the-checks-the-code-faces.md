---
name: the-checks-the-code-faces
description: "The code is written to pass the checks of release.yml: no comments, one assertion per test, no directive that silences a tool, no linter configuration file but etc/clippy/clippy.toml, no Rust file over 1000 lines, no function over 50 statements, no test file without a test, each module of src a directory and no file beside a directory of its name, every line, region and function of src reached by the three tiers together and of crates/test_inputs by its own tests, rustfmt clean, and clippy clean with limits on each function's cognitive complexity, nesting, parameters and lines; the release waits for every one of them"
metadata:
  node_type: memory
  type: project
  originSessionId: 16a1d745-ea25-4971-9454-4ef3ef668142
  modified: 2026-11-18T03:22:09.655Z
---

# The checks the code faces

`.github/workflows/release.yml` holds the code to a set of checks, and the code is written to pass them from its first line:

- `assert-no-comments`: no comment and no doc comment, in any file of the repository.
- `assert-one-assert-per-test`: one assertion in each test function, in `src`, `tests` and `crates`.
- `assert-no-inline-directives`: no attribute or inline directive that silences clippy, coverage, markdownlint, rustc, rustdoc, rustfmt or yamllint.
- `assert-no-linter-config-files`: no configuration file for a linter or a formatter, and no `[lints]` table in a `Cargo.toml`. The one file let through is `etc/clippy/clippy.toml`, per [[a-tool-is-configured-under-etc]].
- `assert-no-oversized-source-files`: no Rust file of more than 1000 lines.
- `assert-no-oversized-functions`: no function of more than 50 statements. A statement is anything that stands directly in a block: a `let`, an expression with or without its `;`, the last expression of the block, an item. Every block inside the function is counted, a closure's among them. A function written inside another is counted by itself, and what is inside the parentheses of a macro call is not counted.
- `assert-no-empty-test-files`: every test file holds a test. A test file is a Rust file under `tests/`, under the `tests/` directory of a module of `src/`, or under the `tests/` directory of a package of `crates/`. A `main.rs` or a `mod.rs` there only lists files and is let through.
- `assert-one-directory-per-module`: no Rust file directly in `src/` but `lib.rs` and `main.rs`, so a module of `src/` is a directory, per [[each-module-of-src-is-a-directory]].
- `assert-coverage`: the tests of the three tiers, taken together, reach every line, region and function of the code under `src/`.
  - Cargo builds the library twice: once with the unit tests in it, which is the program of the unit tier, per [[the-layout-follows-rust-conventions]], and once for the `cladenc` program and the other two tiers. In the percentages that `cargo llvm-cov` prints, and in its `--fail-under-lines` and `--fail-under-regions`, a function counts by the better of its two builds, so a function that the unit tests reach in part and the other tiers reach the rest of would fail them. The job adds the two builds up instead.
  - Lines: `--fail-uncovered-lines 0`, which takes a line for reached when either build ran it.
  - Functions: `--fail-under-functions 100`, which takes a function for reached when either build ran it. A report that lists no file fails it, since the tool takes nothing counted for 0 percent.
  - Regions: a Python script written in the workflow, which reads the report as JSON, where the counts of the two builds are added, and fails on each region that nothing entered and on a report that lists no file.
  - The table the tool prints still goes by the better build, so a file can show under 100 percent there while the job passes. What decides is the two steps of the job.
  - The report reads two programs, `cladenc` and the unit tier's. `cargo llvm-cov report` passes over a program in an `out` directory that has no `fingerprint` directory beside it, so `build-coverage` hands on the unit tier's `fingerprint` directory with its program. The programs of the other two tiers are handed on without one and passed over, which loses nothing: they are built on the same build of the library as `cladenc` is.
  - `--ignore-filename-regex '(^|/)tests/'` keeps the test code out of the measure, the unit tests under `src/` among them. `crates/test_inputs` is out of it too: a report that is asked for at the top of the repository with no package named takes `cladenc` for the package to measure, and leaves out every file under the directory of another member of the workspace.
  - The three jobs that use `cargo-llvm-cov` install version 0.9.1 of it, and the version is raised by hand in all three at once. Mine, not the user's: a later version may count `--fail-uncovered-lines` by the better build, and the check is read again when the version is raised.
- `assert-coverage-of-test-inputs`: the tests of `crates/test_inputs`, which are in `crates/test_inputs/tests/`, reach every line, region and function of the package. They alone are counted: that a tier of `cladenc` calls a function of the package counts for nothing. The one job builds the tests, runs them and reports, with `cargo llvm-cov --package test_inputs --test '*'`, so cargo's own directories lie beside each test program and the report reads it. `--test '*'` runs every test program of the package and leaves out a unit-test build of its library, which holds no test: the package is built once, and `--fail-under-functions`, `--fail-under-lines` and `--fail-under-regions` at 100 hold as the tool counts them. Naming the package is what puts `crates/test_inputs` in the report, and `--ignore-filename-regex '(^|/)tests/'` is what keeps the tests themselves out of the measure. The four harness run commands are constants and hold no line the measure counts. The tests bring up containers, so the job fetches the `busybox:1.37.0` image first. A report that lists no file fails the check, since the tool takes nothing counted for 0 percent. Among the user's other repositories none measures its test files; two give each package of shared code, one that only tests use among them, tests of its own and a full coverage gate, and three leave their test helpers unmeasured.
- `clippy`, with every warning an error, and `rustfmt`.
- `clippy::self_named_module_files`, which `clippy` is given as an error: no Rust file beside a directory of its name, in `src`, `tests` and `crates` alike. A module that has files of its own under it is a directory, and its own code is in the `mod.rs` of that directory.
- The limits `clippy` holds each function to, in `src`, `tests` and `crates` alike. `etc/clippy/clippy.toml` sets the numbers:
  - a cognitive complexity of 15 at most, as `clippy::cognitive_complexity` counts it: one, and one more for each `if`, `match` of more than one arm, match guard, `while`, `for` and `?`, less the returns;
  - a nesting of 4 at most, as `clippy::excessive_nesting` counts it: one level for an `impl`, a `trait` or an inline `mod`, and one for each pair of braces of a block, the function's own among them, so a method has two levels left inside its body and a free function three;
  - 5 parameters at most, `self` counted;
  - 200 lines at most between the braces of a function, blank lines not counted.

**Why:** The release waits for every one of these, so code that breaks one releases nothing, and its failed run is fixed before other work, per [[a-failed-run-is-fixed-before-other-work]]. No issue has to say these rules, per [[issues-have-no-fixed-form]], so a session that writes code finds them here.

**How to apply:** The workflow is the list: where it and this memory disagree, the workflow is right, and this memory is corrected. Write to the checks and leave the running of them to CI, per [[what-ci-can-do-is-done-in-ci]]. Code under `crates/` is held to the same checks, per [[crates-stores-shared-code]], and the tests of a change come before its code, per [[tests-are-written-before-code]].

- A function that would take a sixth parameter is given a value that carries two of them together, as `Limits` carries a deadline and a stop flag in `src/containers/program.rs`, or what it needs is put in a value it already takes, as the time limits are in `Launch`.
- A block that would be a fifth level is flattened: a `let … else { break }` at the head of a loop becomes a condition of the loop, or the inner part becomes a function of its own.
- No lint is silenced to get under a limit, since `assert-no-inline-directives` refuses the attribute.
- The user's rule for the workflow: a job that waits for no other job and is not a build is named in the `needs` of `release`. A check added as a job of its own is added there in the same commit. The builds, `build-coverage`, `build-debug` and `build-release`, wait for nothing and are not named there.
- `assert-no-oversized-functions` is a Python script written in the workflow, which reads each Rust file with `tree-sitter` and fails on a file it cannot read. The count of regions in `assert-coverage` is a second one. The workflow has no empty line, no line over 80 columns and no comment, so neither script has any.
