---
name: a-blocked-by-edge-states-a-real-block
description: "A blocked_by edge is written only where the issue that waits cannot be finished until the other is closed; it is read and written through gh api, the blocker's numeric id sent with -F"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:08:25.546Z
---

# A blocked_by edge states a real block

A `blocked_by` edge is written only where the block is real: the issue that waits cannot be finished until the other is closed.

**Why:** The autopilot takes only issues that no open issue blocks, so a false edge holds back work that could be done. The sibling repositories record the same: an ordering is not a dependency, and an edge written only to give an issue a place in the queue says something untrue about the work. On 2026-11-14 the user asked for `blocked by` edges to enforce the order of the ten issues that build `cladenc`; there each block is real, because each issue builds on the code of the one before. This was a reminder of the autopilot's and moved here the same day, per [[reminders-drive-the-loop]].

**How to apply:** An issue is blocked while `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state'` prints `open`; `gh api` fills in `{owner}` and `{repo}`. To write an edge, run `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by -F issue_id=<id>`, where `<number>` is the issue that waits and `<id>` is the blocker's numeric id from `gh api repos/{owner}/{repo}/issues/<n> --jq .id`, not the node id `gh issue view --json id` returns. `-F` sends the id as a number; sent with `-f`, as a string, it is rejected with HTTP 422 and no edge is added.
