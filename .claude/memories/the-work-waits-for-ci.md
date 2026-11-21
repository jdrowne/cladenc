---
name: the-work-waits-for-ci
description: "While a run that a push of the session's started is queued or in progress, the session does nothing else: no reading, editing, filing, committing or pushing; it ends the turn until the watch reports the run's end"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 730d98e5-80ec-4091-88f3-cbbe76d1d0ae
  modified: 2026-11-21T15:22:17.194Z
---

# The work waits for CI

While a workflow run that a push of the session's started is queued or in progress, the session does nothing else. That means no reading of code, no edits, no issue filed or changed, no commit and no push. It ends its turn, and the notification of the background watch kept per [[every-run-is-watched]] resumes it when the run ends.

**Why:** The user's rule. Each push's result is known before the next piece of work begins, and a failure is fixed per [[a-failed-run-is-fixed-before-other-work]] with nothing built on top of it. The organization's free-tier runners are shared by all of its repositories, so a run can stay queued for hours, and the work waits for that long.

**How to apply:**

- The push itself finishes first: list the runs it started, start a watch on each, and read back the issues its commit closes, per [[a-push-can-leave-its-issues-open]]. Then end the turn.
- The autopilot has a reminder of its own for this rule, per [[reminders-drive-the-loop]]. Another reminder that fires during the wait, to take work or to go on autonomously, does not override it. End the turn again.
- Once every run of the push has ended: if all passed, the next work begins. If one failed, its fix comes first.
- A session that starts or resumes while a run of an earlier push is queued or in progress waits for that run too.
- Replies to the user are not held back. A change the user asks for during the wait is made in the working tree, and it is committed and pushed once the runs have ended.
