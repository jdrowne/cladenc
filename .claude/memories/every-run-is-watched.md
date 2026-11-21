---
name: every-run-is-watched
description: "Every workflow run that a push of the session's starts is watched to its end by a background command, so that its result is known and a failure is fixed; the session does nothing else until the run ends"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7cdb412c-4502-4be0-9040-8f5b755aa863
  modified: 2026-11-15T15:40:19.165Z
---

# Every run is watched

Every workflow run that a push of the session's starts is watched to its end. The watch is a background command, and the session does nothing else until the run ends, per [[the-work-waits-for-ci]].

**Why:** A run that nobody watches can fail unseen, most of all when the session has no new work to take and stops. The watch runs in the background so that the turn stays free for the user and for the reminders while the session waits.

**How to apply:**

- Right after a push, list the runs it started: `gh run list --commit <sha> --json databaseId,workflowName,status`. A run takes a few seconds to appear, so when the listing shows fewer runs than the `on:` section of each workflow says the push starts, list again.
- Start one watch for each run as a background `Bash` call, per [[long-running-commands-in-the-background]]: `perl -e 'alarm 21600; exec @ARGV' gh run watch <id> --exit-status --interval 60 --compact > /dev/null 2>&1; ended=$?; echo "<workflow> run <id> ended with $ended"; exit $ended`. The default of one poll every 3 seconds trips GitHub's secondary rate limit over a long run. The command exits with the watch's status, so the notification's exit code is the run's. A command that ends with the `echo` exits 0 whatever the run did, and its notification says nothing about the run.
- End the turn; the notification of a watch that ends resumes the session.
- A watch that ends with status 0 says the run passed. Never take a run to have passed without reading that status or the line the watch printed. For any other status, read `gh run view <id> --json conclusion --jq .conclusion`. A failure is fixed per [[a-failed-run-is-fixed-before-other-work]]. A run cancelled because a later push landed is left alone, since that push's runs are watched in its place. A watch that its time limit ended, with the run still queued or in progress, is started again.
- A session that starts or resumes while a run of an earlier push is still queued or in progress watches that run too.
- Tell the user how each run ended.
