---
name: only-quick-checks-run-before-a-push
description: "No test runs locally and no change is compiled locally to check it: CI compiles the code and runs the unit, integration and end-to-end tests; the eleven checks of the workflows that compile nothing run locally before a push: assert-no-comments, assert-no-inline-directives and assert-no-linter-config-files before every push; cargo fmt --all, assert-no-empty-test-files, assert-no-oversized-functions, assert-no-oversized-source-files, assert-one-assert-per-test and assert-one-directory-per-module for a commit that changes a Rust file; markdownlint for a commit that changes a Markdown file; yamllint for a commit that changes a workflow or an action; whether the code compiles, the tests, clippy, coverage and the package are left to CI"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-26T16:00:00.000Z
---

# Only quick checks run before a push

No test runs locally, and no change is compiled locally to check it. Locally means where the session works, as opposed to CI. The unit, integration and end-to-end tests all run in CI, and only there. Before a push, eleven checks run locally, and nothing else: every check of the workflows that compiles nothing and runs no test. Each is the job of the same name in `.github/workflows/release.yml`, but for two: `cargo fmt --all` is the job `rustfmt`, and `markdownlint` is the job of that name in `.github/workflows/markdownlint.yml`.

Which of the eleven run depends on the files the commit changes:

| A commit that changes | Runs |
| --- | --- |
| anything | `assert-no-comments`, `assert-no-inline-directives`, `assert-no-linter-config-files` |
| a Rust file | `cargo fmt --all`, `assert-no-empty-test-files`, `assert-no-oversized-functions`, `assert-no-oversized-source-files`, `assert-one-assert-per-test`, `assert-one-directory-per-module` |
| a Markdown file | `markdownlint` |
| a workflow or an action | `yamllint` |

The rows add up: a commit that changes a Rust file and a Markdown file runs the checks of the first three rows. Every command below runs from the top directory of the repository, each between parentheses so that one that fails does not stop the next.

Three checks run before every push, whatever the commit changes:

- `assert-no-comments`: no file holds a comment or a doc comment.
- `assert-no-inline-directives`: no Rust, Markdown, TOML or YAML file holds an attribute or an inline directive that silences a tool.
- `assert-no-linter-config-files`: no file configures a linter or a formatter but `etc/clippy/clippy.toml`.

Each of the three is a tool that is installed locally, called with the flags its job gives it:

```bash
(
  git ls-files -z | xargs -0 assert-no-comments --annotate
)
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

Each job gives its tool the directory of the checkout, which in CI holds only the files git tracks. Here `assert-no-comments` and `assert-no-inline-directives` are given the files git tracks instead, so that each reads the same files as in CI and none that git ignores, such as the files under `target/`. No command asks for the verbose output the jobs ask for.

Six checks run for a commit that changes a Rust file:

- `cargo fmt --all`, which rewrites the files; the workflow only reports a bad format.
- `assert-no-empty-test-files`: every test file holds a test.
- `assert-no-oversized-source-files`: no Rust file has more than 1000 lines.
- `assert-one-directory-per-module`: no Rust file is directly in `src/` but `lib.rs` and `main.rs`.
- `assert-no-oversized-functions`: no function has more than 50 statements.
- `assert-one-assert-per-test`: each test function holds one assertion.

The first three of the five commands below are the scripts of their jobs, as the workflow has them. The fourth takes the Python script of the job `assert-no-oversized-functions` out of the workflow and runs it, so the script is not copied here. The Python that runs it is in a virtual environment that is installed locally and holds `tree-sitter` and `tree-sitter-rust` at the versions the job names. The fifth is a tool that is installed locally, given the directories its job gives it:

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
(
  awk '
    /python - <<.PYTHON.$/ { inside = 1; next }
    /^          PYTHON$/ { inside = 0 }
    inside { sub(/^          /, ""); print }
  ' .github/workflows/release.yml |
    "$HOME/.local/share/assert-no-oversized-functions/bin/python" -
)
(
  assert-one-assert-per-test src tests crates
)
```

One check runs for a commit that changes a Markdown file:

- `markdownlint`: every Markdown file keeps the rules of `markdownlint` but the limit on the length of a line.

It is a tool that is installed locally, called with the flags its job gives it, and given the Markdown files git tracks where the job gives it every Markdown file of the checkout:

```bash
(
  git ls-files -z '*.md' | xargs -0 markdownlint --dot --disable MD013 --
)
```

One check runs for a commit that changes a workflow or an action:

- `yamllint`: each of the two workflows keeps the rules of the job `yamllint` that reads it, and so does each action under `.github/actions/`, which the job of `.github/workflows/release.yml` reads as well. `.github/workflows/markdownlint.yml` has a job of that name too, with the same rules, for its own file.

It is the script of the two jobs, given both workflows and the directory of the actions at once, and `yamllint` is installed locally as well:

```bash
(
  EMPTY_LINES='empty-lines: {max: 0, max-start: 0, max-end: 1}'
  KEY_ORDERING='key-ordering: enable'
  TRUTHY='truthy: {allowed-values: ["true", "false", "on"]}'
  RULES="$EMPTY_LINES, $KEY_ORDERING, $TRUTHY"
  yamllint --strict --format github \
    --config-data "{extends: default, rules: {$RULES}}" \
    .github/workflows/release.yml .github/workflows/markdownlint.yml \
    .github/actions
)
```

Everything else the workflows check is left to CI: whether the code compiles, every test of every tier, `cargo clippy`, the two coverage measures, and the `.deb` and its install on Debian.

**Why:** Compiling the code and running the tests are the heavy part of the checks, and CI does both on every push anyway. The eleven checks that run locally compile nothing and run no test, and together they take under two seconds. A commit that fails one of them in CI costs a failed run on `main`, a second commit and a second wait for CI, per [[a-failed-run-is-fixed-before-other-work]]. The three `assert-*` tools of every push each read files of more than one kind, and the workflow `release` does not run for every commit: a commit of Markdown files alone starts no run of it, so a finding in such a commit would fail the run of a later push that has nothing to do with it.

**How to apply:**

- No `cargo test` is run locally: not one test, not one file, not a tier, with Docker or without it.
- No `cargo check`, `cargo build` or `cargo clippy` is run to check a change before its push: each of the three compiles the code.
- A commit that does not compile is found by the run of its push, in which every job that compiles fails. The fix is the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- A test is written before its code, per [[tests-are-written-before-code]], and CI compiles and runs it for the first time, in the run of the push that brings it with its code.
- `cargo` is reached per [[cargo-is-not-on-the-shell-path]].
- The order is `cargo fmt --all`, then `git add` of the files of the commit, then the other checks, all before `git commit`. Most of the checks read only the files git tracks, so a new file is checked only once it is staged.
- A check that passes prints nothing. Each line a check prints names a file and what is wrong in it, and that file is fixed before the push. A warning of `yamllint` fails its check as an error does. A file that `cargo fmt --all` cannot read as Rust is fixed before the push too. Nothing else holds a push back, per [[push-to-main]].
- A commit of `README.md` alone runs the three checks of every push and `markdownlint`, and nothing else. A commit of a memory is the same, since a memory is a Markdown file.
- The workflow is the list: where a command here and its job disagree, the workflow is right, and the command here is corrected. A commit that changes the job of one of the eleven checks changes the command of the check here as well.
- CI installs the newest release of each tool on every run, so a tool can be an older release locally than in its job. A check that passes locally and fails in CI is a failed run like any other, per [[a-failed-run-is-fixed-before-other-work]].
- A tool that is not found locally is not installed by the session, and neither is another version of `tree-sitter` or `tree-sitter-rust` when the job names one. The user is told, and the check is left to CI for the push.
- The runs the push starts are watched per [[every-run-is-watched]].
- The autopilot has a reminder of its own for this rule, per "Reminders" in `.claude/skills/autopilot/SKILL.md`.
