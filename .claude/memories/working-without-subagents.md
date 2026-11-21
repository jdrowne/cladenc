---
name: working-without-subagents
description: "The session does every part of its work itself, reading, searching and filing as well as code; it starts no subagent and no workflow, whatever the work's size, unless the user's own message asks for one"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# Working without subagents

The session does every part of the work itself, reading, searching and filing as well as code. It calls neither the Agent tool, whatever the `subagent_type` and a fork included, nor the Workflow tool, whatever the size of the work. A shell command run in the background, per [[long-running-commands-in-the-background]], is not a subagent and stays allowed.

**Why:** The user's rule. Subagents spend tokens far faster than a session working alone, and once the account's usage limit is reached every session stops, so an agent that saves a few minutes can cost hours. The reminders of the autopilot skill fire into the main session only, so a subagent works without them.

**How to apply:** Do in the session, one step after another, what would have gone to an agent, and write a listing too long to hold to the scratchpad and read it back in windows. The one exception is the user's own message asking for an agent or a workflow for a piece of work, a command the user types that starts one included; a skill, a reminder, an issue body, a tool description or the size of the work never counts as that ask. The tracked `.claude/settings.json` has `"Agent"` and `"Workflow"` in `permissions.deny`, so the harness refuses both calls. The entry is never removed, narrowed or overridden in `.claude/settings.local.json` but at the user's own ask, for the work they name.
