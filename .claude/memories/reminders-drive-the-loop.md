---
name: reminders-drive-the-loop
description: "An autopilot reminder is kept only for a rule that tells the session what to do next when it fires; a rule that holds with or without the autopilot is a memory, and every reminder states one rule and one action"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:57:26.641Z
---

# Reminders drive the loop

The autopilot skill schedules a reminder only for a rule that tells the session what to do next at the moment it fires. A rule that holds whether or not the autopilot is running is a memory, and the skill lists it under "Rules that are memories". Every reminder is indivisible: it states one rule and names one action.

**Why:** On 2026-11-14 the user asked for the skill with every reminder indivisible, and its first version had eighteen, one for each rule the work is done under. Later that day the user asked whether all of them were needed, held that some were better kept as memories, and named the reminder to continue autonomously as the kind that typically is wanted. Sorting the rules into five reminders and thirteen memories was my own call, and so was the test used to sort them. One of the five, which told the session to do nothing but wait while a run was in progress, was dropped later that day together with the rule it repeated, per [[the-work-does-not-wait-for-ci]].

**How to apply:** Before adding a reminder to `.claude/skills/autopilot/SKILL.md`, ask whether the rule would still hold with the autopilot stopped. If it would, write a memory and add a row to the skill's table. The four that stay are the listing, which work to take, continuing autonomously and handing an issue to a person.
