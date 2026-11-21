---
name: one-commit-per-push
description: "Every push carries exactly one commit, and that one commit may hold several things at once: the work of several issues, a failed run's fix, changes the user asked for, and memory or skill edits"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 730d98e5-80ec-4091-88f3-cbbe76d1d0ae
  modified: 2026-11-21T15:29:50.022Z
---

# One commit per push

Every push carries exactly one commit. Local `main` is never more than one commit ahead of `origin/main`: a commit is pushed as soon as it is made, before another is made. That one commit may hold several things at once. It may hold the work of several issues, the fix for a failed run, changes the user asked for, and memory or skill edits: whatever is finished in the working tree when it is made.

**Why:** The user's rule, as in their other repositories. CI checks a push as one run against its head, so with one commit per push each run says how exactly one commit fared. Within that limit, the user wants finished things committed and pushed together, not queued for a push each, since every push waits out its own runs per [[the-work-waits-for-ci]].

**How to apply:**

- Before `git commit`, check that `git rev-list --count origin/main..HEAD` prints 0. If a commit is still unpushed, push it, or amend it while it is unpushed, instead of making a second.
- What is finished in the working tree joins the commit: a failed run's fix, per [[a-failed-run-is-fixed-before-other-work]]; a change the user asked for, per [[push-to-main]]; and the work of each issue that is done, per [[a-commit-closes-at-least-one-issue]]. Nothing finished is held back for a push of its own.
- Work that is not finished, such as an issue only part done, is left out of the commit and goes in a later one.
