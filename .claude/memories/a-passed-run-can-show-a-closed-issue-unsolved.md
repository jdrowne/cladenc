---
name: a-passed-run-can-show-a-closed-issue-unsolved
description: "When a commit of the session's closed an issue, the run of its push passed, and the run shows the issue's problem is still there, the session reopens the issue and says why in a comment; the body of the issue is left as it was written, on reopening and when the issue is next worked on"
metadata:
  node_type: memory
  type: feedback
  originSessionId: f5dd30ea-0a59-45fc-80f8-cd36982ae19a
  modified: 2026-11-25T04:04:45.028Z
---

# A passed run can show a closed issue unsolved

A commit of the session's closes an issue, the run of its push passes, and what the run shows is that the issue's problem is still there. The session then reopens the issue and says why in a comment. It does not edit the body of the issue, on reopening or when it next works on the issue.

**Why:** The user decided it on 2026-11-24, in two answers to #169. The case had come up with #162, about how long a tier of tests takes in CI: the first commit closed it on push, and only the times of its run could say that nothing had been gained. That session reopened the issue, said why in a comment, and wrote the times of both runs into the body. The user accepted the first two and refused the third. Asked then about the edit that brings an issue up to date before work, the user answered that the body of a reopened issue is left alone at that point too. No reason came with either answer.

**How to apply:**

- The case is one where only a run can say whether the work did what the issue asks, as with anything about how long CI takes. A run that failed is another case, per [[a-failed-run-is-fixed-before-other-work]].
- Reopen with `gh issue reopen <number>`, then `gh issue comment <number>` with a comment that names the commit, what the run of its push showed, with the numbers where there are any, and what is still to do.
- Leave the body as it is. What the run showed is said in the comment, not written into the body.
- Go on working on the issue with its body still as it is. This is the one exception to [[an-issue-is-brought-up-to-date-before-work]], which says so too.
- The commit that finishes the work closes the issue again with its `Closes #N` line, per [[a-push-closes-at-least-one-issue]].
- No new issue is filed for what is left, and the first commit still closes its issue on push.
