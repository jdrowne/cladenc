---
name: a-push-can-leave-its-issues-open
description: "GitHub can accept a commit on main and act on none of its Closes #N lines, so after a push each issue its commit closes is read back and closed by hand if still open"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# A push can leave its issues open

After a push, check that every issue its commit closes is closed.

**Why:** GitHub can accept a commit on `main` and act on none of its `Closes #N` lines. The user's other repositories see it with a long message that names many issues; whether the count or the size sets it off is not known, and it has not been seen in this repository. An issue left open after its work landed reads as unsolved, and here it would also keep every issue it blocks out of the autopilot's reach.

**How to apply:** For each `Closes #N` line, read `gh issue view <N> --json state --jq .state`. Close any still open with `gh issue close <N> --reason completed` and a comment naming the commit that solved it. The lines themselves are written per [[a-commit-closes-at-least-one-issue]].
