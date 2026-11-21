---
name: only-quick-checks-run-before-a-push
description: "Before a push, the Mac runs only cargo fmt --all, each new test seen failing and then passing, and the test files the commit changes; clippy, coverage, the whole tiers, markdownlint, yamllint, the assert-* tools and the package are left to CI"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-21T20:02:03.887Z
---

# Only quick checks run before a push

Before a push, three things run on the Mac, and nothing else:

1. `cargo fmt --all`, which rewrites the files; the workflow only reports a bad format.
2. Each new or changed test, run once before its code is written, to see it fail, and once after, to see it pass.
3. The test files the commit changes, each run on its own with a filter, such as `cargo test --lib artificial_selection::tests::attempt` for a unit test file or `cargo test --test integration containers::procreation` for a file of a tier under `tests/`.

Everything else the workflows check is left to CI: `cargo clippy`, the coverage measure, a whole test tier, `markdownlint`, `yamllint`, the `assert-*` tools, and the `.deb` and its install on Debian.

**Why:** Once `target/` holds a build, those three take seconds on the Mac and catch the most common failures, a bad format and a broken test in a file the commit touches. A failure CI finds costs a run of about five minutes, a fix commit and a second run, with the session waiting on both. The other checks need tools the Mac lacks, take as long as CI, or behave differently off Linux, and CI runs every one of them on every push anyway.

**How to apply:**

- `cargo` is reached per [[cargo-is-not-on-the-shell-path]], and a run that compiles the dependencies goes in the background, per [[long-running-commands-in-the-background]].
- A failure of one of the three is fixed before the push. Nothing else holds a push back, per [[push-to-main]].
- A test that starts containers needs the Docker daemon on the Mac. When the daemon is not running, that test is left to CI.
- A commit that changes no Rust file runs only what applies to it, which can be nothing, as for a commit of `README.md` or a workflow alone.
- `cladenc` is also run by hand, to learn how it behaves before writing about it.
- A pass on the Mac proves nothing about CI. The runs the push starts are watched per [[every-run-is-watched]], and one that fails is fixed in the next commit, per [[a-failed-run-is-fixed-before-other-work]].
