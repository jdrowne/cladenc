---
name: a-problem-found-along-the-way-is-filed
description: "A problem found outside the work in hand is filed as an issue of its own, and solved now only if the work in hand cannot move forward without it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:08:20.210Z
---

# A problem found along the way is filed

A problem found outside the work in hand is filed as an issue of its own. It is solved now only if the work in hand cannot move forward without it.

**Why:** Solving everything met along the way grows a commit past the issues it closes and hides unrelated changes inside it, and passing a problem by loses the finding. Filing keeps both the commit and the finding. These were two reminders of the autopilot's, modeled on the sibling repositories', and moved here on 2026-11-14, per [[reminders-drive-the-loop]].

**How to apply:** File the problem so that it stands alone, per [[issues-are-sized-to-one-context]] and [[one-indivisible-problem-per-issue]], with a `blocked_by` edge only where [[a-blocked-by-edge-states-a-real-block]] allows, and go back to the work. When the work in hand cannot go on without it, file it all the same, add it to the work, and close it with a `Closes #N` line of its own, per [[a-commit-closes-at-least-one-issue]].
