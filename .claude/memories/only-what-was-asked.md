---
name: only-what-was-asked
description: "Plan and file only the work the user asked for, and adopt a sibling repository's rule only when the user points at it; a README issue and the CI waiting rules were both taken back on 2026-11-14"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T01:01:07.007Z
---

# Only what was asked

Plan and file only the work the user asked for, and adopt from the sibling repositories only the rules the user named.

**Why:** Twice on 2026-11-14 the user took back something I had added on my own. They closed issue #10, which I had filed for writing `README.md` as the last step of the plan, with a comment saying they had not requested it. Minutes earlier they had said they did not recall setting the rules that kept the session waiting on CI, which I had carried over from the siblings, per [[the-work-does-not-wait-for-ci]]. Taking the two together as one rule is my own reading.

**How to apply:** When a plan is split into issues, each issue traces back to something the user asked for. Documentation, tooling and housekeeping that nobody asked for are left out, or named to the user in one line and left unfiled. An issue the user closed as not requested stays closed and is not filed again under another name. A rule seen in a sibling repository is brought here only when the user points at it, as they did for [[working-without-subagents]] and [[long-running-commands-in-the-background]]. A problem met during the work is still filed, per [[a-problem-found-along-the-way-is-filed]]: that is a defect in what was asked for, not new scope.
