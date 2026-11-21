---
name: a-failed-run-is-fixed-before-other-work
description: "When a run that the session's own push started is found to have failed, the fix is the next commit the session pushes, ahead of any other work not yet pushed"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.164Z
---

# A failed run is fixed before other work

When a run that the session's own push started is found to have failed, the fix goes in the next commit the session pushes, and no new work is begun until that commit is pushed.

**Why:** A change is unverified until every job that checks it has passed, and a failure left standing hides how each later change fares. The user lets the fix share its commit with other finished changes, so long as the push is that one commit, per [[one-commit-per-push]]. Every run is waited for, per [[the-work-waits-for-ci]], so a failure comes to light before any new work is begun.

**How to apply:** When a watch kept per [[every-run-is-watched]] ends on a failed run, read `gh run view <id> --log-failed`, and fix the failure whether the push caused it or inherited it from an earlier commit. A change the user asked for while the run was going, or anything else finished in the working tree, goes in the same commit as the fix. A commit that holds only the fix, or the fix and changes the user asked for, closes no issue, per [[a-commit-closes-at-least-one-issue]], and writes `Refs #N` for each issue the failed commit closed. The issues the failed commit closed were closed on push, whatever the run said; they stay closed while the fix goes in, and are reopened by hand with `gh issue reopen <number>` only if the commit is taken back instead.
