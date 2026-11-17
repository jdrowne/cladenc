---
name: the-checks-the-code-faces
description: "The code is written to pass the checks of release.yml: no comments, one assertion per test, no directive that silences a tool, no linter configuration file but etc/clippy/clippy.toml, no Rust file over 1000 lines, no test file without a test, every line covered, rustfmt clean, and clippy clean with limits on each function's cognitive complexity, nesting, parameters and lines"
metadata:
  node_type: memory
  type: project
  originSessionId: 16a1d745-ea25-4971-9454-4ef3ef668142
  modified: 2026-11-17T04:24:41.779Z
---

# The checks the code faces

`.github/workflows/release.yml` holds the code to a set of checks, and the code is written to pass them from its first line:

- `assert-no-comments`: no comment and no doc comment, in any file of the repository.
- `assert-one-assert-per-test`: one assertion in each test function, in `src`, `test` and `lib`.
- `assert-no-inline-directives`: no attribute or inline directive that silences clippy, coverage, markdownlint, rustc, rustdoc, rustfmt or yamllint.
- `assert-no-linter-config-files`: no configuration file for a linter or a formatter, and no `[lints]` table in a `Cargo.toml`. The one file let through is `etc/clippy/clippy.toml`, per [[a-tool-is-configured-under-etc]].
- `assert-no-oversized-source-files`: no Rust file of more than 1000 lines.
- `assert-no-empty-test-files`: every test file holds a test.
- `assert-coverage`: the tests reach every line, region and function.
- `clippy`, with every warning an error, and `rustfmt`.
- The limits `clippy` holds each function to, in `src`, `test` and `lib` alike. `etc/clippy/clippy.toml` sets the numbers:
  - a cognitive complexity of 15 at most, as `clippy::cognitive_complexity` counts it: one, and one more for each `if`, `match` of more than one arm, match guard, `while`, `for` and `?`, less the returns;
  - a nesting of 4 at most, as `clippy::excessive_nesting` counts it: one level for an `impl`, a `trait` or an inline `mod`, and one for each pair of braces of a block, the function's own among them, so a method has two levels left inside its body and a free function three;
  - 5 parameters at most, `self` counted;
  - 200 lines at most between the braces of a function, blank lines not counted.

**Why:** A check that fails makes the run of the push fail, and a failed run is fixed before other work, per [[a-failed-run-is-fixed-before-other-work]], so code that breaks one costs a run and a second commit. No issue has to say these rules, per [[issues-have-no-fixed-form]], so a session that writes code finds them here.

**How to apply:** The workflow is the list: where it and this memory disagree, the workflow is right, and this memory is corrected. Write to the checks and leave the running of them to CI, per [[what-ci-can-do-is-done-in-ci]]. Code under `lib/` is held to the same checks, per [[lib-stores-shared-code]], and the tests of a change come before its code, per [[tests-are-written-before-code]].

- A function that would take a sixth parameter is given a value that carries two of them together, as `Limits` carries a deadline and a stop flag in `src/containers/program.rs`, or what it needs is put in a value it already takes, as the time limits are in `Launch`.
- A block that would be a fifth level is flattened: a `let … else { break }` at the head of a loop becomes a condition of the loop, or the inner part becomes a function of its own.
- No lint is silenced to get under a limit, since `assert-no-inline-directives` refuses the attribute.

Mine, not the user's: nothing checks the number of statements in a function. `clippy` has no lint for it.
