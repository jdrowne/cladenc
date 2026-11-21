---
name: a-commit-closes-at-least-one-issue
description: "Every commit closes at least one issue and may close more, with a Closes #N line of its own for each; only the fix for a failed run of the session's own push and a change the user asks for directly close none, and the fix writes Refs #N for each issue the failed commit closed"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.164Z
---

# A commit closes at least one issue

Every commit closes at least one issue. It may close more than one, as many as the work has finished, and it never closes none.

**Why:** A string of commits that each touch an issue and none closes it leaves the issue's work half done on `main`, and the list of open issues stops saying what is still to do.

**How to apply:** Commit when the work finishes at least one issue; until then it stays in the working tree. Give each issue the commit finishes a line of its own reading `Closes #N`, ahead of the attribution line: GitHub binds a closing keyword to one number, so `Closes #1, #2` closes the first and leaves the second open. Write `Refs #N` for an issue the commit only mentions, and keep the subject free of a closing word followed by `#N`, since `Fix the journal (#3)` closes issue 3 on push. A memory, or anything else changed along the way, rides in the next commit, per [[one-commit-per-push]]. An issue whose work cannot land in one commit is split before its work starts, per [[issues-are-sized-to-one-context]]. A commit closes nothing only when all it holds is the fix for a failed run of the session's own push, a change the user asks for directly, or both. A commit holding the fix writes a `Refs #N` line for each issue the failed commit closed, per [[a-failed-run-is-fixed-before-other-work]]; a change the user asks for has no issue to close, per [[push-to-main]]. A commit that holds an issue's finished work as well closes that issue. After the push, see [[a-push-can-leave-its-issues-open]].
