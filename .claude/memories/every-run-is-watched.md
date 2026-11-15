---
name: every-run-is-watched
description: "Every workflow run that a push of the session's starts is watched to its end by a background command, so that its result is known and a failure is fixed, while the work goes on"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 7cdb412c-4502-4be0-9040-8f5b755aa863
  modified: 2026-11-15T04:30:49.428Z
---

# Every run is watched

Every workflow run that a push of the session's starts is watched to its end. The watch is a background command, and the work goes on while it runs.

**Why:** On 2026-11-14 the session ran out of work while the release run of 2f8a6cc was still in progress, and stopped. The user asked why that run was not being watched. The rule of the time had the session look at the runs once, when it took new work, and keep no watch; with no new work to take, a failure would have gone unseen. Offered a watch for that one case, the user ruled more widely that CI is to be watched. Reading that as a watch in the background, which holds no work up, is my own interpretation: earlier the same day the user had dropped every wait for a run, per [[the-work-does-not-wait-for-ci]].

**How to apply:**

- Right after a push, list the runs it started: `gh run list --commit <sha> --json databaseId,workflowName,status`. A run takes a few seconds to appear, so when the listing shows fewer runs than the `on:` section of each workflow says the push starts, list again.
- Start one watch for each run as a background `Bash` call, per [[long-running-commands-in-the-background]]: `perl -e 'alarm 21600; exec @ARGV' gh run watch <id> --exit-status --interval 60 --compact`, with its output sent to a file in the scratchpad. The user's other repositories found that the default of one poll every 3 seconds trips GitHub's secondary rate limit over a long run.
- Go on with the work. When nothing but the watches is left, end the turn; the notification of a watch that ends resumes the session.
- A watch that ends with status 0 says the run passed. For any other status, read `gh run view <id> --json conclusion --jq .conclusion`. A failure is fixed per [[a-failed-run-is-fixed-before-other-work]]. A run cancelled because a later push landed is left alone, since that push's runs are watched in its place. A watch that its time limit ended, with the run still queued or in progress, is started again.
- A session that starts or resumes while a run of an earlier push is still queued or in progress watches that run too.
- Tell the user how each run ended.
