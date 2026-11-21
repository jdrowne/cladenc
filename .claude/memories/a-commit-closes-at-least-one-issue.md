---
name: a-commit-closes-at-least-one-issue
description: "Every commit closes at least one issue and may close more, with a Closes #N line of its own for each; only the fix for a failed run of the session's own push and a change the user asks for directly close none"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.164Z
---

# A commit closes at least one issue

Every commit closes at least one issue. It may close more than one, as many as the work in hand has finished, and it never closes none.

**Why:** The rule is the user's. A string of commits that each touch an issue and none closes it leaves the issue's work half done on `main`, and the list of open issues stops saying what is still to do.

**How to apply:** Commit when the work finishes at least one issue; until then it stays in the working tree. Give each issue the commit finishes a line of its own reading `Closes #N`, ahead of the attribution line: GitHub binds a closing keyword to one number, so `Closes #1, #2` closes the first and leaves the second open. Write `Refs #N` for an issue the commit only mentions, and keep the subject free of a closing word followed by `#N`, since `Fix the journal (#3)` closes issue 3 on push. A memory, or anything else changed along the way, rides in the next commit, per [[one-commit-per-push]]. An issue whose work cannot land in one commit is split before its work starts, re-using the issue as one of the splits, so that each part closes with the commit that finishes it. A commit closes nothing only when all it holds is the fix for a failed run of the session's own push, per [[a-failed-run-is-fixed-before-other-work]], a change the user asks for directly, or both. A commit that holds an issue's finished work as well closes that issue. The two exceptions are my own reading; the user's rule has none. After the push, see [[a-push-can-leave-its-issues-open]].
