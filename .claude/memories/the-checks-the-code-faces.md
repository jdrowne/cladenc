---
name: the-checks-the-code-faces
description: "The checks the code faces are the jobs of release.yml with the numbers of etc/clippy/clippy.toml, and they are read there, not here; kept here is only what the workflow does not say: how clippy counts its limits on a function, how to get under one, where a new check job is named, why a coverage table can show under 100 percent while the job passes, and what puts a function in the wrong group of the check on the unit tier"
metadata:
  node_type: memory
  type: project
  originSessionId: 16a1d745-ea25-4971-9454-4ef3ef668142
  modified: 2026-11-23T03:30:36.994Z
---

# The checks the code faces

The checks are the jobs of `.github/workflows/release.yml`, with the numbers of `etc/clippy/clippy.toml`. They are read there, where they are always right, and are not listed here. Whether the code passes them is what a run of CI says, per [[ci-says-whether-the-code-is-right]].

What follows is what the workflow does not say.

## How clippy counts its limits

`clippy` holds each function to four limits, in `src`, `tests` and `crates` alike:

- cognitive complexity, as `clippy::cognitive_complexity` counts it: one, and one more for each `if`, `match` of more than one arm, match guard, `while`, `for` and `?`, less the returns;
- nesting, as `clippy::excessive_nesting` counts it: one level for an `impl`, a `trait` or an inline `mod`, and one for each pair of braces of a block, the function's own among them. A closure's braces are a level, and so are the braces of each arm of an `if` written on one line. A method has two levels left inside its body and a free function three;
- parameters, `self` counted;
- lines between the braces of a function, blank lines not counted.

## Getting under a limit

- A function that would take one parameter too many is given a value that carries two of them together, as `Limits` carries a deadline and a stop flag in `src/containers/program.rs`, or what it needs is put in a value it already takes, as the time limits are in `Launch`.
- A block that would be one level too deep is flattened: a `let … else { break }` at the head of a loop becomes a condition of the loop, or the inner part becomes a function of its own.
- No lint is silenced to get under a limit, since `assert-no-inline-directives` refuses the attribute.

## Coverage

- Cargo builds the library twice, once with the unit tests in it, per [[the-layout-follows-rust-conventions]], and once for the `cladenc` program and the other two tiers. `assert-coverage` adds the counts of the two builds up, so a function that the unit tests reach in part and the other tiers reach the rest of passes. The table `cargo llvm-cov` prints goes by the better build, so a file can show under 100 percent there while the job passes.
- Every job that uses `cargo-llvm-cov` installs the same version of it, and the version is raised by hand in all of them at once. A later version may count `--fail-uncovered-lines` by the better build, and the check is read again when the version is raised.
- `assert-coverage-of-the-unit-tier` finds the group of each function, per [[the-tiers-follow-the-standard-test-pyramid]], from the names the function's own statements call. The names are sets at the head of the job's script:
  - A new way for the code to reach the file system or a program is added to those sets in the commit that brings it.
  - A method of the code's own that has one of those names and takes no argument, such as a `status()`, puts its caller in the second group by mistake. Such a name is not given to one.

## The workflow itself

- A job that waits for no other job and is not a build is named in the `needs` of `release`. A check added as a job of its own is added there in the same commit.
- The workflow has no empty line, no line over 80 columns and no comment, so a script written in it has none either.
- The local form of `assert-no-oversized-functions`, per [[only-quick-checks-run-before-a-push]], takes every script of the workflow that starts with `python - <<'PYTHON'`. Another script of the workflow ends its here-document with another word.

**Why:** The user asked on 2026-11-22 why memories describe how the software works when the source of truth can be read, here the workflow. This memory used to list each job and what it does. The list went stale whenever a job changed, the memory itself said the workflow was right where the two disagreed, and the list did not prevent the misses it was written against. The reason it gave for the list, that code is to pass every check from the start, was never the user's, per [[ci-says-whether-the-code-is-right]].

**How to apply:**

- Read the workflow and `etc/clippy/clippy.toml` for what a check asks. Nothing is added here that can be read there.
- Add here only what a session would otherwise learn from a failed run and the workflow would not have told it.
- Code under `crates/` is held to the same checks, per [[crates-stores-shared-code]], and the tests of a change come before its code, per [[tests-are-written-before-code]].
