---
name: before-an-issue-is-labelled-needs-decision
description: "Before an issue is labelled needs decision it is assessed against the memories and the code to see whether it truly needs a person, and what a person has to decide is written into its body"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:08:07.177Z
---

# Before an issue is labelled needs decision

Two things are done before an issue is labelled `needs decision`. It is assessed against the memories in `.claude/memories/` and the code, to determine whether it truly needs a person. Then what a person has to decide is written into its body.

**Why:** The label takes the issue out of the autopilot's listing until a person acts, and here each of the issues that build `cladenc` blocks the one after it, so a label on one stops everything behind it. An issue labelled where a memory or the code already answers the question stalls the chain for nothing, and one labelled without the question written down leaves the person to work out what is being asked. Both were reminders of the autopilot's and moved here on 2026-11-14, per [[reminders-drive-the-loop]].

**How to apply:** Read the memories for a rule that settles the question, and the code and the other issues for a fact that does. Only when neither does, rewrite the body so that it states what is to be decided and what the work needs from the answer, with a guarded edit such as `[ -s body.md ] && gh issue edit <number> --body-file body.md`, and then add the label. The label is the reminder's part; these two steps come first.
