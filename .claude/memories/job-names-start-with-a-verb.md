---
name: job-names-start-with-a-verb
description: "The first word of the name of a new CI job is a verb that says what the job does, as in summarize-commits; a composite action under .github/actions/ is named the same way; a job named otherwise keeps its name"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 9f1bac3d-c3fb-4ecc-865f-f3474baaafda
  modified: 2026-11-26T15:37:14.211Z
---

# Job names start with a verb

The first word of the name of a new job in `.github/workflows/` is a verb that says what the job does, as in `summarize-commits` or `assert-no-comments`. A composite action under `.github/actions/` is named the same way.

**Why:** A verb first tells a reader of the page of a run what the job does, and not only what it is about.

**How to apply:** Name each job and each action that a change adds with a verb first. A job named otherwise, such as `clippy` or `unit-test-debug`, keeps its name unless the user asks for another.
