---
name: a-passed-run-can-show-a-closed-issue-unsolved
description: "When a commit of the session's closed an issue, its run passed, and the run shows the issue's problem is still there, the session reopens the issue and says why in a comment; the body of the issue is left as it was written"
metadata:
  node_type: memory
  type: feedback
  originSessionId: f5dd30ea-0a59-45fc-80f8-cd36982ae19a
  modified: 2026-11-25T03:39:08.127Z
---

# A passed run can show a closed issue unsolved

A commit of the session's closes an issue, its run passes, and what the run shows is that the issue's problem is still there. The session then reopens the issue and says why in a comment. It does not edit the body of the issue.

**Why:** The user decided it on 2026-11-24, in answer to #169. The case had come up with #162, about how long a tier of tests takes in CI: the first commit closed it on push, and only the times of its run could say that nothing had been gained. That session reopened the issue, said why in a comment, and wrote the times of both runs into the body. The user accepted the first two and refused the third. No reason came with the answer.

**How to apply:**

- The case is one where only a run can say whether the work did what the issue asks, as with anything about how long CI takes. A run that failed is another case, per [[a-failed-run-is-fixed-before-other-work]].
- Reopen with `gh issue reopen <number>`, then `gh issue comment <number>` with a comment that names the commit, what its run showed, with the numbers where there are any, and what is still to do.
- Leave the body as it is. This holds against [[an-issue-is-brought-up-to-date-before-work]] too: what the closed commit changed in the code is said in the comment, not written into the body.
- Go on working on the issue. The commit that finishes the work closes it again with its `Closes #N` line, per [[a-commit-closes-at-least-one-issue]].
- No new issue is filed for what is left, and the first commit still closes its issue on push.

What is mine, and not the user's:

- That the comment carries the numbers of the run. The user accepted a comment that says why, and in the case of #162 the numbers were the reason.
- That the body stays untouched for as long as the issue is open again, the edit that brings an issue up to date before work among the edits left out. The user refused the edit made on reopening and said nothing of a later one.
