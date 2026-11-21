---
name: issues-about-the-sessions-rules-are-labelled
description: "An issue about a memory carries the label memories, and one about an autopilot reminder or the autopilot skill carries the label autopilot; an issue about both carries both"
metadata:
  type: feedback
---

# Issues about the session's rules are labelled

An issue about a memory under `.claude/memories/` carries the label `memories`. An issue about an autopilot reminder, or about `.claude/skills/autopilot/SKILL.md`, carries the label `autopilot`. An issue about both carries both, and an issue that needs a person's answer carries `needs decision` as well, per [[before-an-issue-is-labelled-needs-decision]].

**Why:** The user made the two labels, so that the issues about how the session works stand apart from the issues about `cladenc`.

**How to apply:** Put the label on when the issue is filed, with `gh issue create --label`. An issue about `cladenc` that also changes a memory, as most work does along the way, carries neither label; the labels are for an issue whose problem is in a memory, a reminder or the skill. Related: [[a-contradiction-is-the-users-to-settle]].
