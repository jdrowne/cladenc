---
name: a-push-closes-at-least-one-issue
description: "Every push closes at least one issue and may close more; a commit inside it may finish an issue or only be a step toward one, with a Closes #N line in the commit that finishes issue N and Refs #N in each earlier commit that works on it; only a push holding nothing but the fix for a failed run of the session's own push or a change the user asks for directly closes none, and the fix writes Refs #N for each issue the failed push closed"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 9f1bac3d-c3fb-4ecc-865f-f3474baaafda
  modified: 2026-11-26T15:39:24.678Z
---

# A push closes at least one issue

Every push closes at least one issue. It may close more than one, as many as the work has finished, and it never closes none. A push carries one commit or several, per [[commits-in-a-push-are-the-sessions-choice]]: a commit inside it may finish an issue or only be a step toward one, and it is the push, taken as a whole, that has to finish at least one. This says what a push holds, not when it goes.

**Why:** CI checks a push at its head, so the push is what lands on `main` checked. A string of pushes that each touch an issue and none closes it leaves the issue's work half done on `main`, and the list of open issues stops saying what is still to do. Inside one push that does not arise, since its commits land and are checked together.

**How to apply:** A push goes only once its work finishes at least one issue; until then the work stays local, committed or in the working tree. Give each issue a line of its own reading `Closes #N`, ahead of the attribution line, in the commit that finishes it: GitHub binds a closing keyword to one number, so `Closes #1, #2` closes the first and leaves the second open. Write `Refs #N` in each earlier commit of the push that works on issue N without finishing it, and for an issue a commit only mentions, and keep the subject free of a closing word followed by `#N`, since `Fix the journal (#3)` closes issue 3 on push. A memory, or anything else changed along the way, rides in the next push, in a commit of its own or with other work. The whole work of an issue lands in the one push that closes it, however much work that is; an issue is split only where it documents more than one problem, per [[one-indivisible-problem-per-issue]]. A push closes nothing only when all it holds is the fix for a failed run of the session's own push, a change the user asks for directly, or both. The commit that holds the fix writes a `Refs #N` line for each issue the failed push closed, per [[a-failed-run-is-fixed-before-other-work]]; a change the user asks for has no issue to close, per [[push-to-main]]. A push that holds an issue's finished work as well closes that issue. After the push, see [[a-push-can-leave-its-issues-open]].
