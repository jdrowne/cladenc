---
name: one-indivisible-problem-per-issue
description: "An issue documents one indivisible problem; one that documents more is split into one issue per problem, re-using the issue itself as one of the splits"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:08:15.812Z
---

# One indivisible problem per issue

An issue documents one indivisible problem. An issue that documents more is split into one issue per problem, and the issue itself is re-used as one of the splits.

**Why:** The reason the user's other repositories record: an issue holding two problems never closes cleanly, since it sits open while half of it is done and its title says less than its body. Here it also follows from [[a-commit-closes-at-least-one-issue]]: a commit can only close an issue whose whole problem it solves. This was a reminder of the autopilot's, modeled on theirs, and moved here on 2026-11-14, per [[reminders-drive-the-loop]].

**How to apply:** The issue's number stays with the problem its title leads with, and each other problem moves to a new issue written to stand alone, per [[issues-are-sized-to-one-context]]. A split divides problems, not file lists: two symptoms of one defect stay together. Order the splits with `blocked_by` edges only where [[a-blocked-by-edge-states-a-real-block]] allows.
