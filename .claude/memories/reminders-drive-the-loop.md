---
name: reminders-drive-the-loop
description: "An autopilot reminder is kept only for a rule that tells the session what to do next when it fires; a rule that holds with or without the autopilot is a memory, the wait for CI is both, and every reminder states one rule and one action"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-16T00:41:30.605Z
---

# Reminders drive the loop

The autopilot skill schedules a reminder only for a rule that tells the session what to do next at the moment it fires. A rule that holds whether or not the autopilot is running is a memory, and the skill lists it under "Rules that are memories". The wait for CI, per [[the-work-waits-for-ci]], is both: the user wants it as a reminder as well as a memory, as in the user's other repositories, since the other reminders would otherwise draw the session back to work while a run is going. It is the one reminder that `start reminders-only` schedules, for work done without the loop. Every reminder is indivisible: it states one rule and names one action.

**Why:** A reminder for every rule the work is done under is more than is needed. The user's view is that every reminder is indivisible, that some rules are better kept as memories, and that the reminder to continue autonomously is the kind that is wanted. Which rule is a reminder and which a memory, and the test that sorts them, are my own call.

**How to apply:** Before adding a reminder to `.claude/skills/autopilot/SKILL.md`, ask whether the rule would still hold with the autopilot stopped. If it would, write a memory and add a row to the skill's table; it is a reminder as well only when a reminder that fires would otherwise lead the session against it, as with the wait for CI. The six are the wait for CI, the listing, which work to take, ending the autopilot when no issue is left that it can take, continuing autonomously and handing an issue to a person. Ending is a reminder because it has no meaning with the autopilot stopped.
