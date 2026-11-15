---
name: a-push-carries-one-commit
description: "Every push carries exactly one commit, and its runs are read before the next push; both workflows cancel a run in progress when another push lands on main"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:08:38.970Z
---

# A push carries one commit

Every push carries exactly one commit, and its runs are read before the next commit is pushed.

**Why:** Both workflows under `.github/workflows/` set `cancel-in-progress: true` for the ref, so a second push to `main` cancels the run the first one started, and a run checks only the head of what was pushed. With two commits in one push, or two pushes in a row, the earlier commit is never checked by itself and a failed run cannot say which of them broke it. The sibling repositories hold the same rule. It was a reminder of the autopilot's and moved here on 2026-11-14, per [[reminders-drive-the-loop]].

**How to apply:** Commit, push, start the wait per [[a-wait-runs-in-the-background]], and read every run the push started before committing again. A failed run is handled per [[a-failed-run-is-fixed-before-other-work]]. Work that will close several issues may still go out as several commits, each in a push of its own, per [[the-work-in-hand-can-be-several-issues]].
