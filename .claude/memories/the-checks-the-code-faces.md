---
name: the-checks-the-code-faces
description: "The code is written to pass the checks of release.yml: no comments, one assertion per test, no directive that silences a tool, no linter configuration file, no Rust file over 1000 lines, no test file without a test, every line covered, clippy and rustfmt clean"
metadata:
  node_type: memory
  type: project
  originSessionId: 16a1d745-ea25-4971-9454-4ef3ef668142
  modified: 2026-11-16T04:40:35.293Z
---

# The checks the code faces

`.github/workflows/release.yml` holds the code to a set of checks, and the code is written to pass them from its first line:

- `assert-no-comments`: no comment and no doc comment, in any file of the repository.
- `assert-one-assert-per-test`: one assertion in each test function, in `src`, `test` and `lib`.
- `assert-no-inline-directives`: no attribute or inline directive that silences clippy, coverage, markdownlint, rustc, rustdoc, rustfmt or yamllint.
- `assert-no-linter-config-files`: no configuration file for a linter or a formatter.
- `assert-no-oversized-source-files`: no Rust file of more than 1000 lines.
- `assert-no-empty-test-files`: every test file holds a test.
- `assert-coverage`: the tests reach every line, region and function.
- `clippy`, with every warning an error, and `rustfmt`.

**Why:** The build that the tests and the release hang on waits for each of these, so code that breaks one costs a whole run and releases nothing. No issue has to say these rules, per [[issues-have-no-fixed-form]], so a session that writes code finds them here.

**How to apply:** The workflow is the list: where it and this memory disagree, the workflow is right, and this memory is corrected. Write to the checks and leave the running of them to CI, per [[what-ci-can-do-is-done-in-ci]]. Code under `lib/` is held to the same checks, per [[lib-stores-shared-code]], and the tests of a change come before its code, per [[tests-are-written-before-code]].
