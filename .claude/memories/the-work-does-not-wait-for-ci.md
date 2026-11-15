---
name: the-work-does-not-wait-for-ci
description: "After a push the session goes straight on with the next work, commits and pushes included; the organization's free-tier runners are shared by all its repositories, so a run here can stay queued for hours"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# The work does not wait for CI

After a push, the session goes straight on with the next work: edits, commits and further pushes included. No run is waited for, and nothing the workflows check is run locally first, per [[what-ci-can-do-is-done-in-ci]].

**Why:** The organization is on GitHub's free tier, and its allotment of runners is shared by all of its repositories, so a long job in another of them keeps a run here queued for as long as it goes on. A wait here is pointless, and the user sets no rule to wait. Reading that as covering the gate on pushing as well as the one on local work is my own interpretation.

**How to apply:**

- Push, then take the next work at once. Both workflows cancel their own run for the ref when another push lands, queued or in progress, and that is accepted: the latest push is the one that gets checked.
- Not waiting is not the same as not looking. Each run the push started is watched to its end in the background, per [[every-run-is-watched]], and the watch holds no work up.
- A run the watch reports as failed is handled per [[a-failed-run-is-fixed-before-other-work]].
