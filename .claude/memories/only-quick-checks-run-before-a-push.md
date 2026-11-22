---
name: only-quick-checks-run-before-a-push
description: "No test runs locally and no change is compiled locally to check it: CI compiles the code and runs the unit, integration and end-to-end tests; before a push only cargo fmt --all runs locally, and whether the code compiles, clippy, coverage, the linters, the assert-* tools and the package are left to CI"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-22T13:22:32.037Z
---

# Only quick checks run before a push

No test runs locally, and no change is compiled locally to check it. Locally means where the session works, as opposed to CI. The unit, integration and end-to-end tests all run in CI, and only there. Before a push, one command runs locally, and nothing else:

- `cargo fmt --all`, which rewrites the files; the workflow only reports a bad format.

Everything else the workflows check is left to CI: whether the code compiles, every test of every tier, `cargo clippy`, the coverage measure, `markdownlint`, `yamllint`, the `assert-*` tools, and the `.deb` and its install on Debian.

**Why:** Compiling the code and running the tests are the heavy part of the checks, and CI does both on every push anyway.

**How to apply:**

- No `cargo test` is run locally: not one test, not one file, not a tier, with Docker or without it.
- No `cargo check`, `cargo build` or `cargo clippy` is run to check a change before its push: each of the three compiles the code.
- A commit that does not compile is found by the run of its push, in which every job that compiles fails. The fix is the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- A test is written before its code, per [[tests-are-written-before-code]], and CI compiles and runs it for the first time, in the run of the push that brings it with its code.
- `cargo` is reached per [[cargo-is-not-on-the-shell-path]].
- A file that `cargo fmt --all` cannot read as Rust is fixed before the push. Nothing else holds a push back, per [[push-to-main]].
- A commit that changes no Rust file runs nothing locally, as for a commit of `README.md` or a workflow alone.
- The runs the push starts are watched per [[every-run-is-watched]].
- The autopilot has a reminder of its own for this rule, per "Reminders" in `.claude/skills/autopilot/SKILL.md`.
