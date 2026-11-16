---
name: cladenc-does-not-run-jobs
description: "What cladenc does is not called a job: it brings up a container, for a procreation or for an assessment; an older issue that says job means one of those; it has no workers either, only containers running at the same time"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 1363dd2e-d43d-4955-a13c-888d1b3e9bc9
  modified: 2026-11-16T02:12:02.486Z
---

# cladenc does not run jobs

`cladenc` does not run jobs, and the word job is not used for what it does. It brings up a container, for a procreation or for an assessment. It has no workers either: what a directed evolution limits is the number of containers running at the same time.

**Why:** The user's rule, and the phrase about bringing up a container is the user's own. That the flag which gives that limit is named `--max-concurrent-containers` is the user's decision too: `cladenc` starts Docker containers, and worker names nothing a person can find. My reading of the reason, not the user's: a person who uses `cladenc` gives it and sees containers, procreations and assessments, and neither job nor worker names one of them.

**How to apply:** In replies, issues, commits, the README and help text, name the thing: a procreation, an assessment, or the container brought up for one. A time limit is the limit of a procreation or of an assessment, given with `--procreation-timeout` or `--assessment-timeout`, and the most containers running at the same time is given with `--max-concurrent-containers`, which the settings and the run record of the journal keep as `max_concurrent_containers`. Worker is not said: say the containers running at the same time. The user's point on the name: a flag that gives a limit is named so that it cannot be read as a total, so max-concurrent and not max alone. Read job in an older issue as a procreation or an assessment, `src/job` as `src/containers`, `--workers` as `--max-concurrent-containers` and `src/search/worker.rs` as `src/directed_evolution/action.rs`, and correct each when the issue is brought up to date, per [[an-issue-is-brought-up-to-date-before-work]].

Mine, not the user's: where the code needs one word for a procreation or an assessment, it is action, as in `artificial_selection::Action` and `next_action`, and that word stays out of what a person reads. Step is not that word: the help text uses it for a part of an assessment. A job of a GitHub workflow keeps its name, since that is GitHub's word and not something `cladenc` does. Related: [[a-flag-is-not-called-an-option]], [[a-harness-is-not-called-a-coding-agent]].
