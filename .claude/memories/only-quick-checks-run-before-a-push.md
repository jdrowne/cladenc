---
name: only-quick-checks-run-before-a-push
description: "No test runs on the Mac: the unit, integration and end-to-end tests run in CI only; before a push the Mac runs cargo fmt --all and a compile check that runs no test, and clippy, coverage, the linters, the assert-* tools and the package are left to CI as well"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-22T01:27:53.000Z
---

# Only quick checks run before a push

No test runs on the Mac. The unit, integration and end-to-end tests all run in CI, and only there. Before a push, two things run on the Mac, and nothing else:

1. `cargo fmt --all`, which rewrites the files; the workflow only reports a bad format.
2. `cargo check --all-targets`, which compiles the code and the tests and runs none of them.

Everything else the workflows check is left to CI: every test of every tier, `cargo clippy`, the coverage measure, `markdownlint`, `yamllint`, the `assert-*` tools, and the `.deb` and its install on Debian.

**Why:** Running the tests on the Mac overheats it. The tiers compile and run for minutes at a time and bring up containers all the while, and CI runs every one of them on every push anyway.

**How to apply:**

- No `cargo test` is run on the Mac: not one test, not one file, not a tier, with Docker or without it.
- A test is written before its code, per [[tests-are-written-before-code]], and CI runs it for the first time, in the run of the push that brings it with its code.
- `cargo` is reached per [[cargo-is-not-on-the-shell-path]], and a check that compiles the dependencies goes in the background, per [[long-running-commands-in-the-background]].
- A failure of the format or of the compile check is fixed before the push. Nothing else holds a push back, per [[push-to-main]].
- A commit that changes no Rust file runs neither of the two, as for a commit of `README.md` or a workflow alone.
- The runs the push starts are watched per [[every-run-is-watched]], and one that fails is fixed in the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- The autopilot has a reminder of its own for this rule, per "Reminders" in `.claude/skills/autopilot/SKILL.md`.
