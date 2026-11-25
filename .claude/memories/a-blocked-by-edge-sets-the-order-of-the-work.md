---
name: a-blocked-by-edge-sets-the-order-of-the-work
description: "An order the session recommends between issues is written as blocked_by edges, so that the autopilot keeps to it; an edge is not kept for a real block alone; it is read and written through gh api, the blocker's numeric id sent with -F"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 4709d643-13b7-4647-9fd8-fbd97a9d875d
  modified: 2026-11-25T01:05:26.143Z
---

# A blocked_by edge sets the order of the work

Where the session recommends an order between issues, it writes that order as `blocked_by` edges: the issue that comes later is blocked by the one that comes first. An edge is not kept for the case where the issue that waits cannot be finished until the other is closed.

**Why:** On 2026-11-24 the session filed #161 to #165 and recommended an order for them, in a reply and in their bodies, with no edge between them. The user asked why the order was not enforced with `blocked_by` edges. The reason was a rule of 2026-11-15, by which an edge was written only for a real block and an ordering was no dependency. The user had that rule removed.

What is mine, and not the user's:

- That an order the session recommends is from now on written as edges. The user removed the old rule and asked the question above. They stated no new rule in its place.
- That no edge is written where the session has no order to recommend. The autopilot takes only issues that no open issue blocks, so an edge with no order behind it holds back work that could be done.

**How to apply:**

- When a reply or the body of an issue says which issue to do first, write the edge at once, and tell the user which edges were written.
- To write an edge, run `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by -F issue_id=<id>`, where `gh api` fills in `{owner}` and `{repo}`, `<number>` is the issue that waits and `<id>` is the blocker's numeric id from `gh api repos/{owner}/{repo}/issues/<n> --jq .id`, not the node id `gh issue view --json id` returns. `-F` sends the id as a number; sent with `-f`, as a string, it is rejected with HTTP 422 and no edge is added.
- How the autopilot keeps the order the edges set is per [[several-issues-can-be-worked-on-together]].
