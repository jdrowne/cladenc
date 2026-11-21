---
name: what-the-user-instructs-is-done
description: "An instruction from the user is carried out in full and at once; no memory, autopilot reminder or wait for CI holds it back, and it is never turned into a decision put back to the user"
metadata:
  node_type: memory
  type: feedback
  originSessionId: b1d622db-0e04-473f-b926-900805d77634
  modified: 2026-11-21T16:14:43.949Z
---

# What the user instructs is done

When the user tells the session to do something, the session does it, in full and at once. No memory, autopilot reminder or wait for CI holds it back, and the session does not turn it into a question or a `needs decision` issue.

**Why:** An instruction from the user is to be carried out, never left half done.

**How to apply:**

- A question is answered with whatever reading it takes, and a change is made, committed and pushed, per [[push-to-main]], even while a CI run is going, per [[the-work-waits-for-ci]].
- A memory or a reminder that seems to forbid what the user instructs is no contradiction for [[a-contradiction-is-the-users-to-settle]]: the instruction is carried out, and nothing is put to the user.
