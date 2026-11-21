---
name: tests-are-written-before-code
description: "The tests of a change are written before the code that makes them pass, and both land in the same commit"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-16T04:40:39.611Z
---

# Tests are written before code

The tests of a change are written before its code.

**Why:** A test written first says what the code is for before the code can shape it. An issue does not have to say it, per [[issues-have-no-fixed-form]].

**How to apply:** Write the tests an issue lists, then the code that makes them pass. Both land in the commit that closes the issue, per [[a-commit-closes-at-least-one-issue]]; the tests are never pushed ahead of the code as a commit of their own. `cargo` is reached per [[cargo-is-not-on-the-shell-path]], and a run that compiles the dependencies goes in the background, per [[long-running-commands-in-the-background]].
