---
name: the-work-in-hand-can-be-several-issues
description: "The work the autopilot takes on is the lowest-numbered unblocked issue, alone or with other listed issues it chooses; an issue with open blockers joins only when all of them are in the work"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# The work in hand can be several issues

The work the autopilot takes on is not limited to one issue. It begins with the lowest-numbered listed issue that no open issue blocks, and other listed issues may be added to it.

**Why:** Issues which change the same code share one design, and taken singly each is designed without the others in view. That the work may be more than one issue is the user's rule, and one of their other repositories works this way too.

**How to apply:** Which issues to add is the session's choice. An issue that an open issue blocks joins only when every open issue blocking it is already in the work, and it is closed no earlier than they are, in the same commit or a later one; that keeps the order the `blocked_by` edges set. An issue carrying `needs decision`, or a label that `--skip-label` names, never joins. Each commit closes at least one of the work's issues, per [[a-commit-closes-at-least-one-issue]], and is pushed as soon as it is made, per [[the-work-does-not-wait-for-ci]]. When an issue of the work turns out to need a person, it is labelled per [[before-an-issue-is-labelled-needs-decision]] and leaves the work, its edits and the issues it blocks with it, and the rest goes on. My own reading, not something the user said: every issue is written to stand alone, per [[issues-are-sized-to-one-context]], so each issue added costs the context its whole body, and that cost is the thing to weigh when choosing.
