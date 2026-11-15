---
name: a-commit-closes-at-least-one-issue
description: "Every commit closes at least one issue and may close more, with a Closes #N line of its own for each; only the fix for a failed run of the session's own push and a change the user asks for directly close none"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:07:55.225Z
---

# A commit closes at least one issue

Every commit closes at least one issue. It may close more than one, as many as the work in hand has finished, and it never closes none.

**Why:** On 2026-11-14 the user asked for this rule, pointing at the sibling repositories that already hold it; the wording here follows `deltahdl`'s memory on the same subject. The reason that memory records: a string of commits that each touch an issue and none closes it leaves the issue's work half done on `main`, and the list of open issues stops saying what is still to do.

**How to apply:** Commit when the work finishes at least one issue; until then it stays in the working tree. Give each issue the commit finishes a line of its own reading `Closes #N`, ahead of the attribution line: GitHub binds a closing keyword to one number, so `Closes #1, #2` closes the first and leaves the second open. Write `Refs #N` for an issue the commit only mentions, and keep the subject free of a closing word followed by `#N`, since `Fix the journal (#3)` closes issue 3 on push. A memory, or anything else changed along the way, rides in a commit that closes an issue. An issue whose work cannot land in one commit is split before its work starts, re-using the issue as one of the splits, so that each part closes with the commit that finishes it. Two kinds of commit close nothing, as in `deltahdl`: the fix for a failed run of the session's own push, per [[a-failed-run-is-fixed-before-other-work]], and a change the user asks for directly. Carrying those two exceptions over is my own reading; the user stated the rule without them. After the push, see [[a-push-can-leave-its-issues-open]].
