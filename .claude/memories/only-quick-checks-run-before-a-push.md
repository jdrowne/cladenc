---
name: only-quick-checks-run-before-a-push
description: "No test runs locally and no change is compiled locally to check it: CI compiles the code and runs the unit, integration and end-to-end tests; seven checks run locally before a push: assert-no-inline-directives and assert-no-linter-config-files before every push, cargo fmt --all and the scripts of the jobs assert-no-empty-test-files, assert-no-oversized-source-files and assert-one-directory-per-module for a commit that changes a Rust file, and yamllint for a commit that changes a workflow; whether the code compiles, clippy, coverage, markdownlint, the other assert-* tools and the package are left to CI"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-22T14:28:41.967Z
---

# Only quick checks run before a push

No test runs locally, and no change is compiled locally to check it. Locally means where the session works, as opposed to CI. The unit, integration and end-to-end tests all run in CI, and only there. Before a push, seven checks run locally, and nothing else. Each is the job of the same name in `.github/workflows/release.yml`, and `cargo fmt --all` is the job `rustfmt`. Which of the seven run depends on the files the commit changes, and every command below runs from the top directory of the repository.

Two checks run before every push, whatever the commit changes:

- `assert-no-inline-directives`: no Rust, Markdown, TOML or YAML file holds an attribute or an inline directive that silences a tool.
- `assert-no-linter-config-files`: no file configures a linter or a formatter but `etc/clippy/clippy.toml`.

Each of the two is a tool that is installed locally, called with the flags its job gives it, each between parentheses so that one that fails does not stop the next:

```bash
(
  tools=clippy,coverage,markdownlint,rustc,rustdoc,rustfmt,yamllint
  git ls-files -z | xargs -0 assert-no-inline-directives --tools "$tools"
)
(
  linters=clippy,markdownlint,rustc,rustdoc,rustfmt,yamllint
  assert-no-linter-config-files --linters "$linters" \
    --exclude "$PWD/etc/clippy/clippy.toml" "$PWD"
)
```

Each job gives its tool the directory of the checkout, which in CI holds only the files git tracks. Here `assert-no-inline-directives` is given the files git tracks instead, so that it reads the same files as in CI and none that git ignores, such as the files under `target/`. Neither command asks for the verbose output the jobs ask for.

Four checks run for a commit that changes a Rust file:

- `cargo fmt --all`, which rewrites the files; the workflow only reports a bad format.
- `assert-no-empty-test-files`: every test file holds a test.
- `assert-no-oversized-source-files`: no Rust file has more than 1000 lines.
- `assert-one-directory-per-module`: no Rust file is directly in `src/` but `lib.rs` and `main.rs`.

The last three are the scripts of their jobs, as the workflow has them:

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

One check runs for a commit that changes a workflow:

- `yamllint`: each of the two workflows keeps the rules of the job `yamllint` that reads it. `.github/workflows/markdownlint.yml` has a job of that name too, with the same rules, for its own file.

It is the script of the two jobs, given both workflows at once, and `yamllint` is installed locally as well:

```bash
(
  EMPTY_LINES='empty-lines: {max: 0, max-start: 0, max-end: 1}'
  KEY_ORDERING='key-ordering: enable'
  TRUTHY='truthy: {allowed-values: ["true", "false", "on"]}'
  RULES="$EMPTY_LINES, $KEY_ORDERING, $TRUTHY"
  yamllint --strict --format github \
    --config-data "{extends: default, rules: {$RULES}}" \
    .github/workflows/release.yml .github/workflows/markdownlint.yml
)
```

Everything else the workflows check is left to CI: whether the code compiles, every test of every tier, `cargo clippy`, the coverage measure, `markdownlint`, the other `assert-*` tools, and the `.deb` and its install on Debian.

**Why:** Compiling the code and running the tests are the heavy part of the checks, and CI does both on every push anyway. The seven checks that run locally compile nothing and run no test, and together they take about a second. A commit that fails one of them in CI costs a failed run on `main`, a second commit and a second wait for CI, per [[a-failed-run-is-fixed-before-other-work]]. The two `assert-*` tools run before every push because the workflow `release` does not run for every commit: a commit of Markdown files alone starts no run of it, so a directive in one of those files would fail the run of a later push that has nothing to do with it.

**How to apply:**

- No `cargo test` is run locally: not one test, not one file, not a tier, with Docker or without it.
- No `cargo check`, `cargo build` or `cargo clippy` is run to check a change before its push: each of the three compiles the code.
- A commit that does not compile is found by the run of its push, in which every job that compiles fails. The fix is the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- A test is written before its code, per [[tests-are-written-before-code]], and CI compiles and runs it for the first time, in the run of the push that brings it with its code.
- `cargo` is reached per [[cargo-is-not-on-the-shell-path]].
- The order is `cargo fmt --all`, then `git add` of the files of the commit, then the scripts and the tools, all before `git commit`. The three scripts and `assert-no-inline-directives` read only the files git tracks, so a new file is checked only once it is staged.
- A check that passes prints nothing. Each line a check prints names a file, and that file is fixed before the push. A line of a script starts with `::error`. A line of an `assert-*` tool starts with the path of the file. `yamllint` prints lines that start with `::error` or `::warning`, and a warning fails the check as an error does. A file that `cargo fmt --all` cannot read as Rust is fixed before the push too. Nothing else holds a push back, per [[push-to-main]].
- A commit that changes no Rust file leaves out the four checks that read only Rust files, and a commit that changes neither workflow leaves out `yamllint`. A commit of `README.md` alone runs the two `assert-*` tools and nothing else.
- The workflow is the list: where a command here and its job disagree, the workflow is right, and the command here is corrected. A commit that changes the job of one of the seven checks changes the command of the check here as well.
- CI installs the newest release of each of the three tools on every run, so a tool can be an older release locally than in its job. A check that passes locally and fails in CI is a failed run like any other, per [[a-failed-run-is-fixed-before-other-work]].
- My own reading, which the user has not confirmed: a tool that is not found locally is not installed by the session. The user is told, and the check of that tool is left to CI for the push.
- The runs the push starts are watched per [[every-run-is-watched]].
- The autopilot has a reminder of its own for this rule, per "Reminders" in `.claude/skills/autopilot/SKILL.md`.
