---
name: what-ci-can-do-is-done-in-ci
description: "A check that the workflows run is not run locally: no local test tier, clippy, format check, coverage, markdownlint, yamllint or assert tool before a push; the change is pushed and CI checks it"
metadata:
  type: feedback
---

# What CI can do is done in CI

A check that a workflow of this repository runs is left to that workflow. It is not run locally before a push; the push is made, and its runs are waited for per [[the-work-waits-for-ci]].

**Why:** A commit held back while tests run on the user's machine is a change that is not yet pushed, and running the workflows' checks locally is nothing the user asks for.

**How to apply:**

- Make the change, with its tests written first per [[tests-are-written-before-code]], then commit and push per [[push-to-main]]. Do not run the unit, integration or end-to-end tier, `cargo clippy`, a format check, the coverage measure, `markdownlint`, `yamllint` or an `assert-*` tool beforehand.
- The runs the push starts are watched per [[every-run-is-watched]], and one that fails is fixed in the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- What CI cannot do stays local: applying `cargo fmt --all` to the files, since the workflow only checks the format, and running `cladenc` by hand to learn how it behaves before writing about it.
- The rule covers every local run of a test or a check, the run that shows a new test failing before its code is written included. A push that changes only files no workflow starts on, such as `README.md` alone for the release workflow, is still pushed without a local run.
