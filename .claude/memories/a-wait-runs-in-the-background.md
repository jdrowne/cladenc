---
name: a-wait-runs-in-the-background
description: "A wait for CI, or for anything that takes minutes, is a Bash run_in_background or a Monitor, never a foreground sleep or gh run watch, so the session stays idle and the autopilot reminders can fire"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:09:06.262Z
---

# A wait runs in the background

Waiting for a workflow run, or for anything else that takes minutes, is done with `Bash` and `run_in_background: true`, which notifies once when the command exits, or with `Monitor`, which notifies once per event; never with a foreground `sleep`, `gh run watch` or polling loop in the session's own shell.

**Why:** A scheduled reminder fires only while the session is idle, as the `CronCreate` tool's own description says. A foreground wait holds the session busy for the whole run, so the standing rules of the autopilot skill stop arriving exactly while a push is in flight, and the user cannot type either. The user's other repositories hold the same rule for the same reason. It was adopted here on 2026-11-14, when the autopilot skill was added; the user asked for the skill and its reminders, and bringing this rule along with them was my own call.

**How to apply:** Start the wait in the background, end the turn, and act on the notification. `gh run watch <id> --exit-status --interval 60` is fine inside the background command. The interval is there because the sibling repositories found that the default of one poll every 3 seconds trips GitHub's secondary rate limit over a long run; that has not been seen in this repository yet. Nothing but the wait goes on while the run is in progress. Any other command that can run past a minute follows [[long-running-commands-in-the-background]]. See [[working-without-subagents]].
