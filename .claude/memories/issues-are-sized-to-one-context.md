---
name: issues-are-sized-to-one-context
description: "A plan too large to solve inside one context is split into issues that each stand alone, the original issue re-used as one of them, and blocked_by edges enforce their order"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:09:04.690Z
---

# Issues are sized to one context

A plan too large to be solved before the session's context fills is not kept in one issue. It is split into issues that can each be solved alone, the original issue re-used as one of the splits, and each issue is made blocked by the one that has to land before it.

**Why:** On 2026-11-14 the plan to build `cladenc` sat in issue 1 as one body of 325 lines with nine steps. The user's concern was that a session working through it would run out of context and be compacted, which loses detail the work depends on. They had it split, asked that the existing issue be one of the splits, and asked for `blocked by` edges to enforce the sequence. The result is issues 1 to 10.

**How to apply:** Write each issue so that a session which has just been compacted, or has never seen the conversation, can act on it from the issue and the code alone. It states its own rules, defines the terms it uses, repeats the short sections every issue needs (the clean-room rule, the rules for the code, the working rules), and names the earlier issue where a rule it relies on is stated instead of restating it. Keep each fact in one issue, the one that builds it. Order the issues with `blocked_by` edges, written as [[a-blocked-by-edge-states-a-real-block]] describes; in a chain of steps where each builds on the code of the one before, that block is real. My own reading, not something the user said: the size to aim for is one that leaves most of the context for the work itself, and each of the ten issues here is about a third of the original body. See [[working-without-subagents]], which rules out the other way of saving context.
