---
name: only-quick-checks-run-before-a-push
description: "No test runs locally and no change is compiled locally to check it: CI compiles the code and runs the unit, integration and end-to-end tests; before a push of a commit that changes a Rust file, four checks run locally: cargo fmt --all, then the scripts of the jobs assert-no-empty-test-files, assert-no-oversized-source-files and assert-one-directory-per-module; whether the code compiles, clippy, coverage, the linters, the other assert-* tools and the package are left to CI"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-22T13:35:37.192Z
---

# Only quick checks run before a push

No test runs locally, and no change is compiled locally to check it. Locally means where the session works, as opposed to CI. The unit, integration and end-to-end tests all run in CI, and only there. Before a push, four checks run locally, and nothing else. Each is the job of the same name in `.github/workflows/release.yml`, and `cargo fmt --all` is the job `rustfmt`:

- `cargo fmt --all`, which rewrites the files; the workflow only reports a bad format.
- `assert-no-empty-test-files`: every test file holds a test.
- `assert-no-oversized-source-files`: no Rust file has more than 1000 lines.
- `assert-one-directory-per-module`: no Rust file is directly in `src/` but `lib.rs` and `main.rs`.

The last three are the scripts of their jobs, as the workflow has them, each between parentheses so that one that fails does not stop the next:

```bash
(
  tests=('tests/*.rs' 'src/*/tests/*.rs' 'crates/*/tests/*.rs')
  tests+=(':!*/main.rs' ':!*/mod.rs')
  found="$(git grep -LF '#[test]' -- "${tests[@]}" || true)"
  if [ -n "$found" ]; then
    for file in $found; do
      said="$file is a test file that holds no test"
      echo "::error file=$file::$said"
    done
    exit 1
  fi
)
(
  git ls-files -z '*.rs' | xargs -0 wc -l | awk -v limit=1000 '
    $2 != "total" && $1 > limit {
      said = $2 " has " $1 " lines, and the limit is " limit
      print "::error file=" $2 "::" said
      over = 1
    }
    END { exit over }'
)
(
  files=(':(glob)src/*.rs' ':!src/lib.rs' ':!src/main.rs')
  found="$(git ls-files -- "${files[@]}")"
  if [ -n "$found" ]; then
    for file in $found; do
      said="$file is a module in a file: move it to"
      echo "::error file=$file::$said ${file%.rs}/mod.rs"
    done
    exit 1
  fi
)
```

Everything else the workflows check is left to CI: whether the code compiles, every test of every tier, `cargo clippy`, the coverage measure, `markdownlint`, `yamllint`, the other `assert-*` tools, and the `.deb` and its install on Debian.

**Why:** Compiling the code and running the tests are the heavy part of the checks, and CI does both on every push anyway. The four checks that run locally compile nothing and run no test, and together they take under a second. A commit that fails one of them in CI costs a failed run on `main`, a second commit and a second wait for CI, per [[a-failed-run-is-fixed-before-other-work]].

**How to apply:**

- No `cargo test` is run locally: not one test, not one file, not a tier, with Docker or without it.
- No `cargo check`, `cargo build` or `cargo clippy` is run to check a change before its push: each of the three compiles the code.
- A commit that does not compile is found by the run of its push, in which every job that compiles fails. The fix is the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- A test is written before its code, per [[tests-are-written-before-code]], and CI compiles and runs it for the first time, in the run of the push that brings it with its code.
- `cargo` is reached per [[cargo-is-not-on-the-shell-path]].
- The order is `cargo fmt --all`, then `git add` of the files of the commit, then the three scripts, all before `git commit`. The scripts read only the files git tracks, so a new file is checked only once it is staged.
- The three scripts print nothing when every file passes. Each line they print starts with `::error` and names a file, and that file is fixed before the push. A file that `cargo fmt --all` cannot read as Rust is fixed before the push too. Nothing else holds a push back, per [[push-to-main]].
- A commit that changes no Rust file runs nothing locally, as for a commit of `README.md` or a workflow alone: each of the four checks reads Rust files and no others.
- The workflow is the list: where a script here and the script of its job disagree, the workflow is right, and the script here is corrected. A commit that changes one of the three jobs changes its script here as well.
- The runs the push starts are watched per [[every-run-is-watched]].
- The autopilot has a reminder of its own for this rule, per "Reminders" in `.claude/skills/autopilot/SKILL.md`.
