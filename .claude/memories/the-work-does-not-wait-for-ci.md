---
name: the-work-does-not-wait-for-ci
description: "After a push the session goes straight on with the next work, commits and pushes included; the organization's free-tier runners are shared by all its repositories, so a run here can stay queued for hours"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T04:30:53.222Z
---

# The work does not wait for CI

After a push, the session goes straight on with the next work: edits, commits and further pushes included. No run is waited for, and what the workflows check is checked locally before the push.

**Why:** On 2026-11-14 the session sat idle behind a queued run, and the user asked why it had to. Told that the waiting rules were my own carry-over from the user's other repositories, the user answered that they did not recall setting any such restriction and did not care about them. They gave the reason a wait here is pointless: the organization is on GitHub's free tier, its allotment of runners is shared by all of its repositories, and a long render in another of its repositories was holding every one of them, so a run here stays queued for as long as that render goes on. Three things were dropped that day: the rule that nothing but the wait goes on while a run is in progress, the rule that a push's runs are read before the next push, and the autopilot reminder that repeated the first. Reading the answer as dropping the gate on pushing as well as the one on local work is my own interpretation.

**How to apply:**

- Before a push, run locally what the workflows run on the files the push changes: `yamllint` with the workflow's own rules for a workflow file, `npx --yes markdownlint-cli --dot --disable MD013 --ignore 'test/*/samples/**' -- '**/*.md'` for Markdown, and the `cargo` and `assert-*` checks for code.
- Push, then take the next work at once. Both workflows cancel their own run for the ref when another push lands, queued or in progress, and that is accepted: the latest push is the one that gets checked.
- Not waiting is not the same as not looking. Each run the push started is watched to its end in the background, per [[every-run-is-watched]], and the watch holds no work up. Until later on 2026-11-14 this memory had the session look at the runs once, when it took new work, and keep no watch; the user replaced that.
- A run the watch reports as failed is handled per [[a-failed-run-is-fixed-before-other-work]].
