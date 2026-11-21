---
name: long-running-commands-in-the-background
description: "Any command that can take more than about a minute, a cargo build or test run included, goes in a background Bash call or under Monitor, with a time limit and its output in a scratchpad file, and is never followed by a foreground wait"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# Long-running commands in the background

Run every command that can take more than about a minute as a background `Bash` call (`run_in_background: true`) or under `Monitor`, then end the turn and let its notification resume the work. In this repository that covers a `cargo` build or test run that compiles the dependencies, a sweep of `gh` calls over many issues or runs, and a long `grep` or `find` over another of the user's repositories, as well as the watch of each workflow run, per [[every-run-is-watched]].

**Why:** The user's rule. A foreground command holds the turn for as long as it runs, so every scheduled reminder due in that time waits behind it and the user cannot type. A background command frees the turn as soon as it starts.

**How to apply:**

- Use a background `Bash` call when the command's end is the news, and `Monitor` when its events along the way are.
- Give a command that might not end a time limit as well. macOS has no `timeout`, so use `perl -e 'alarm <seconds>; exec @ARGV' <command>`.
- Send the output to a file in the scratchpad and read that file once the notification arrives.
- Never follow a background command with a foreground wait: no `sleep`, no `until` loop, and no rereading of its output file until it fills. Any of them blocks the session again.
- Other work that does not depend on the result goes on meanwhile. Once only the result is left to wait for, end the turn.
- A turn made of many short foreground calls also holds the reminders back until it ends. Split long work at its natural boundaries, such as after a batch of edits.

On going on with other work meanwhile, a CI run is the exception: nothing else is done while it is queued or in progress, per [[the-work-waits-for-ci]]. A `cargo` build or test reads the working tree until it ends, so an edit to the code it is building depends on its result and waits for it.
