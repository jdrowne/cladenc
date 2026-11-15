---
name: the-work-does-not-wait-for-ci
description: "After a push the session goes straight on with the next work, commits and pushes included; the organization's free-tier runners are shared by all its repositories, so a run here can stay queued for hours"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:56:47.155Z
---

# The work does not wait for CI

After a push, the session goes straight on with the next work: edits, commits and further pushes included. No run is waited for, and what the workflows check is checked locally before the push.

**Why:** On 2026-11-14 the session sat idle behind a queued run, and the user asked why it had to. Told that the waiting rules were my own carry-over from the user's other repositories, the user answered that they did not recall setting any such restriction and did not care about them. They gave the reason a wait here is pointless: the organization is on GitHub's free tier, its allotment of runners is shared by all of its repositories, and a long render in another of its repositories was holding every one of them, so a run here stays queued for as long as that render goes on. Three things were dropped that day: the rule that nothing but the wait goes on while a run is in progress, the rule that a push's runs are read before the next push, and the autopilot reminder that repeated the first. Reading the answer as dropping the gate on pushing as well as the one on local work is my own interpretation.

**How to apply:**

- Before a push, run locally what the workflows run on the files the push changes: `yamllint` with the workflow's own rules for a workflow file, `npx --yes markdownlint-cli --dot --disable MD013 --ignore 'test/*/samples/**' -- '**/*.md'` for Markdown, and the `cargo` and `assert-*` checks for code.
- Push, then take the next work at once. Both workflows cancel their own run for the ref when another push lands, queued or in progress, and that is accepted: the latest push is the one that gets checked.
- When taking new work, look once at `gh run list --limit 10 --json databaseId,headSha,workflowName,status,conclusion`. A run that completed as a failure is handled per [[a-failed-run-is-fixed-before-other-work]]; a run still queued or cancelled is left alone. No loop and no watch.
- A watch of a run, should one ever be wanted, goes in the background per [[long-running-commands-in-the-background]], as `gh run watch <id> --exit-status --interval 60`; the user's other repositories found that the default of one poll every 3 seconds trips GitHub's secondary rate limit over a long run.
