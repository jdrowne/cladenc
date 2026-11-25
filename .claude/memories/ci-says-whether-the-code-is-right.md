---
name: ci-says-whether-the-code-is-right
description: "The goal is a cladenc that is right; a first push that passes CI was never asked for and is not worked toward, and neither are fewer runs or less time; CI is one of the tools that say whether the code is right, used as often as it takes"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-11-23T03:30:23.086Z
  originSessionId: 43d1a223-9777-4a32-b6c8-9e9dd430d745
---

# CI says whether the code is right

What the session works toward is a `cladenc` that is right: one that does what its issues ask. Whether the code is right is found out with every tool there is, and CI is one of them: it compiles the code and runs every test and every check. A run that fails has done what it is for, and the code is not expected to be right the first time it is pushed.

Three things are not goals, and none of them is weighed against getting the code right:

- a first push that passes;
- few runs;
- little time.

**Why:** The user said so on 2026-11-22, while #151 and #153 were being solved, in three steps:

- The session had spent two hours reading and tracing code that nothing local compiles, so as to push only what would pass, and had pushed nothing. The user asked why it did not push and let CI show what was wrong. The push that followed compiled at once, and its run found one block nested too deep for clippy and 4 failing tests out of 2,612.
- The session then gave, as the reason a memory of its own listed every check of the workflow, that code is to pass the checks from the moment it is written. The user answered that they had never asked for that, that it is not what the session is to work toward, that every tool at hand is to be used to reach the goal, CI among them, and that they do not expect code to be written right in one go.
- The session had also counted the 13 minutes of a failed run as a cost. The user answered that how long it takes does not matter: what matters most is that `cladenc` is accurate.

That code is written to pass the checks from the start was a session's wording in that memory, never the user's. The memory itself is gone, per [[what-the-repository-says-is-read-there]].

**How to apply:**

- Push once the change is complete as far as reading it shows and the checks of [[only-quick-checks-run-before-a-push]] pass. The run says what is wrong, and what it finds is fixed in the next commit, per [[a-failed-run-is-fixed-before-other-work]].
- Do not trace by hand what a compiler or a test would say in order to be spared a run.
- Time and the count of runs are never given as a reason: not for a test left out, not for a narrower fix, not for doing other than a rule such as [[the-work-waits-for-ci]] says.
- What the checks are is read in `.github/workflows/release.yml` and `etc/clippy/clippy.toml`, not in a memory, per [[what-the-repository-says-is-read-there]].
- This does not make a careless push fine. What is known to be wrong is fixed before the push, and what is not known is left to CI.
- A test that fails because it expected the wrong thing is corrected to what `cladenc` is meant to do, which is read in the code and the issue, and never to whatever makes it pass.

How long a directed evolution itself takes is another matter with the same answer, per [[how-long-a-directed-evolution-takes-does-not-matter]].
