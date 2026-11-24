---
name: the-task-list-holds-what-is-left-to-do
description: "The session's task list is brought up to date as the work goes: a task for each piece of work that comes up, marked when it starts and when it ends, reworded when its work changes, deleted once completed or no longer needed; the autopilot has a reminder of its own for it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 43d1a223-9777-4a32-b6c8-9e9dd430d745
  modified: 2026-11-24T01:45:14.000Z
---

# The task list holds what is left to do

The session brings its task list up to date as the work goes, so that at any moment the list holds what is left to do and nothing else.

**Why:** On 2026-11-23 the user asked for the autopilot to carry a reminder that keeps the tasks up to date. No reason came with the request.

**How to apply:**

- A piece of work that comes up becomes a task when it comes up, before it is started.
- A task is marked in progress when its work starts, and completed when it ends.
- A task whose work changed is reworded, and a task no longer needed is deleted.
- A completed task is deleted, so that finished work does not pile up above what is left.
- The autopilot has a reminder of its own for this rule, per "Reminders" in `.claude/skills/autopilot/SKILL.md`.

What is mine, and not the user's:

- That the rule holds with the autopilot stopped. The user asked for a reminder. The skill's test for a new reminder asks whether its rule would hold without the autopilot, and I took this one to.
- That the list is brought up to date during the wait of [[the-work-waits-for-ci]] as well. Marking a task reads no code and changes nothing in the repository, and those are what the wait holds back.
