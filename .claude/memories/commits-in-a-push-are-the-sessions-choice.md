---
name: commits-in-a-push-are-the-sessions-choice
description: "How many commits a push carries, one or several, and when the push goes are both the session's choice, outside a wait for CI; the job summarize-commits of each workflow lists the commits of a push on the summary page of its run"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 9f1bac3d-c3fb-4ecc-865f-f3474baaafda
  modified: 2026-11-26T15:39:18.020Z
---

# Commits in a push are the session's choice

A push carries one commit or several, as the session judges best for the work, and the session chooses when to push. Nothing asks for a single commit and nothing asks for several: one commit that holds a whole change is as right as several commits that each take a step of it.

**Why:** On 2026-11-26 the user had the rule of exactly one commit in every push replaced by this one, which leaves to the session both how the work is cut into commits and when it is pushed. The page of a run names only the last commit of its push, so each workflow has a job `summarize-commits`, which lists every commit of the push on the summary page of the run, and the user reads the commits there.

**How to apply:**

- A commit may hold several things at once: the work of several issues, the fix for a failed run, changes the user asked for, and memory or skill edits. It may also hold one step of the work of an issue and finish nothing. The push as a whole closes at least one issue, per [[a-push-closes-at-least-one-issue]].
- Each commit brings the tests of the code it holds, per [[tests-are-written-before-code]], and is checked before it is made, per [[only-quick-checks-run-before-a-push]].
- The choice of when holds outside a wait for CI: while a run of an earlier push is queued or in progress, nothing is committed or pushed but what the user instructs, per [[the-work-waits-for-ci]].
- A change the user asks for is still committed and pushed once it is done, per [[push-to-main]], and the fix for a failed run still goes in the next push, per [[a-failed-run-is-fixed-before-other-work]]: the choice of when is no reason to hold either back.
- CI checks a push at its head, and its runs are listed under its last commit, per [[every-run-is-watched]]. A failed run is read against the push as a whole.
- A new workflow gets the job `summarize-commits` and names the action among its paths.
