---
name: a-push-can-leave-its-issues-open
description: "GitHub can accept a commit on main and act on none of its Closes #N lines, so after a push each issue its commit closes is read back and closed by hand if still open"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:57:23.161Z
---

# A push can leave its issues open

After a push, check that every issue its commit closes is closed.

**Why:** The user's other repositories found that GitHub can accept a commit on `main` and act on none of its `Closes #N` lines; a long message naming many issues is where they saw it, and whether the count or the size sets it off is not known. It has not been seen in this repository. An issue left open after its work landed reads as unsolved, and here it would also keep every issue it blocks out of the autopilot's reach. This was a reminder of the autopilot's and moved here on 2026-11-14, per [[reminders-drive-the-loop]].

**How to apply:** For each `Closes #N` line, read `gh issue view <N> --json state --jq .state`. Close any still open with `gh issue close <N> --reason completed` and a comment naming the commit that solved it. The lines themselves are written per [[a-commit-closes-at-least-one-issue]].
