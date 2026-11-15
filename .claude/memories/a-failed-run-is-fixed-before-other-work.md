---
name: a-failed-run-is-fixed-before-other-work
description: "When a run that the session's own push started is found to have failed, the fix is the next commit the session pushes, ahead of any other work not yet pushed"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:57:20.855Z
---

# A failed run is fixed before other work

When a run that the session's own push started is found to have failed, the fix is the next commit the session pushes, ahead of any other work not yet pushed.

**Why:** The reason the sibling repositories record: a change is unverified until every job that checks it has passed, and if the fix were pushed together with new work, a second failed run could not say whether the fix or the new work broke it. This was a reminder of the autopilot's, modeled on the siblings', and moved here on 2026-11-14, per [[reminders-drive-the-loop]]. No run is waited for, per [[the-work-does-not-wait-for-ci]], so a failure can come to light several pushes after the one that caused it; what was pushed in between stays.

**How to apply:** When the look at the runs that [[the-work-does-not-wait-for-ci]] asks for shows a failure, read `gh run view <id> --log-failed`, and fix the failure whether the push caused it or inherited it from an earlier commit. Work in the tree that is not part of the fix is set aside with `git stash` and brought back after the push. The fix is a commit of its own, and it is one of the two kinds of commit that close no issue, per [[a-commit-closes-at-least-one-issue]]. The issues the failed commit closed were closed on push, whatever the run said; they stay closed while the fix goes in, and are reopened by hand with `gh issue reopen <number>` only if the commit is taken back instead.
