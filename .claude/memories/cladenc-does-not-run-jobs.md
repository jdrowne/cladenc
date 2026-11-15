---
name: cladenc-does-not-run-jobs
description: "What cladenc does is not called a job: it brings up a container, for a procreation or for an evaluation; an older issue that says job means one of those"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 1363dd2e-d43d-4955-a13c-888d1b3e9bc9
  modified: 2026-11-15T23:42:20.077Z
---

# cladenc does not run jobs

`cladenc` does not run jobs, and the word job is not used for what it does. It brings up a container, for a procreation or for an evaluation.

**Why:** The user's rule, and the phrase about bringing up a container is the user's own. My reading of the reason, not the user's: a person who uses `cladenc` gives it and sees containers, procreations and evaluations, and job names none of them.

**How to apply:** In replies, issues, commits, the README and help text, name the thing: a procreation, an evaluation, or the container brought up for one. A time limit is the limit of a procreation or of an evaluation, given with `--procreation-timeout` or `--evaluation-timeout`, and `--workers` is the most containers running at once. Read job in an older issue as a procreation or an evaluation, and `src/job` as `src/containers`, and correct both when the issue is brought up to date, per [[an-issue-is-brought-up-to-date-before-work]].

Mine, not the user's: where the code needs one word for a procreation or an evaluation, it is action, as in `policy::Action` and `next_action`, and that word stays out of what a person reads. Step is not that word: the help text uses it for a part of an evaluation. A job of a GitHub workflow keeps its name, since that is GitHub's word and not something `cladenc` does. Related: [[a-flag-is-not-called-an-option]], [[a-harness-is-not-called-a-coding-agent]].
