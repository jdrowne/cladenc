---
name: the-layout-follows-rust-conventions
description: "The user's layout: Rust's and Cargo's conventions for where things go, with unit tests inside src/ as a tests/ directory beside each module's code, the other tiers under tests/, packages under crates/, and nothing named in Cargo.toml that Cargo finds itself; kept apart from the conventions are mod.rs for every module, no comments, and linter settings under etc/"
metadata:
  type: project
---

# The layout follows Rust's conventions

The repository is laid out the way Rust and Cargo expect, so that a Rust reader finds each thing where they look for it. It is the user's layout.

- **Unit tests are inside the library.** The unit tests of a module are a module named `tests` beside its code: the directory `src/<module>/tests/`, with a `mod.rs` that lists its files and nothing else. The last lines of `src/<module>/mod.rs` declare it, as `#[cfg(test)]` over `mod tests;`. A test file there reaches the library as `crate::…`.
- **The unit tests are always a directory.** No `tests.rs` file, and no `mod tests { … }` written into a source file, however few the tests are. One form holds for every module.
- **The other two tiers are under `tests/`.** `tests/integration/` and `tests/e2e/` are one test program each, and the `main.rs` of each lists its files.
- **The samples are under `tests/` too.** `tests/samples/` holds what the unit tests read, named from the top of the repository, as `tests/samples/Dockerfile`. `tests/e2e/samples/` holds what the end-to-end tier reads, reached with `test_inputs::sample`.
- **A package other than `cladenc` is under `crates/`,** per [[crates-stores-shared-code]].
- **`Cargo.toml` names nothing that Cargo finds itself:** no `[[bin]]` and no `[[test]]`. Cargo takes `src/main.rs` for the program, `src/lib.rs` for the library, and each directory of `tests/` that has a `main.rs` for a test program.

Three things are kept apart from Rust's conventions, each a rule of the user's:

- every module of `src/` is a directory with a `mod.rs`, per [[each-module-of-src-is-a-directory]];
- no comment is written, a doc comment included, per [[the-checks-the-code-faces]];
- the settings of a linter are under `etc/`, per [[a-tool-is-configured-under-etc]].

**Why:** The user chose Rust's conventions for this repository, where the user's other repositories name the same directories `test/` and `lib/`. My reading of the reason for the directory of tests, not the user's: the test code stays in files of its own, so the coverage measure leaves it out by the name of the directory, which a `mod tests { … }` inside a source file would not allow without a directive in the code.

**How to apply:**

- A new unit test goes in a file of `src/<module>/tests/`, and a new file there is added to that directory's `mod.rs`. A module that gets its first unit test gets the directory, its `mod.rs`, and the declaration at the foot of `src/<module>/mod.rs`.
- The three tiers are run as `cargo test --lib` for the unit tier, `--test integration` and `--test e2e`. No test target is named `unit`.
- With the unit tests inside it, Cargo builds the library twice, and the coverage tool counts a function by the better of its two builds unless it is told otherwise. The job `assert-coverage` adds the two builds up, per [[the-checks-the-code-faces]].
- An older issue that names `test/unit/<module>/`, `test/integration/`, `test/e2e/`, `test/lib/test_inputs/` or `lib/test_inputs` means `src/<module>/tests/`, `tests/integration/`, `tests/e2e/`, `crates/test_inputs/tests/` and `crates/test_inputs`.
- Nothing is `pub` for the unit tests, since they are inside the library, per [[an-item-is-as-private-as-its-uses-allow]].
