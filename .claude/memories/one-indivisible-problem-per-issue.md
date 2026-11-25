---
name: one-indivisible-problem-per-issue
description: "An issue documents one indivisible problem, and that problem alone sets its size; one that documents more is split into one issue per problem, re-using the issue itself as one of the splits"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-25T01:05:29.439Z
---

# One indivisible problem per issue

An issue documents one indivisible problem. An issue that documents more is split into one issue per problem, and the issue itself is re-used as one of the splits. The problem alone sets an issue's size: an issue that documents one problem stays one issue, however much work the problem takes.

**Why:** An issue holding two problems never closes cleanly, since it sits open while half of it is done and its title says less than its body. It also follows from [[a-commit-closes-at-least-one-issue]]: a commit can only close an issue whose whole problem it solves. An indivisible problem has no part that is a problem of its own, so cutting it into several issues by the amount of work leaves issues that each solve nothing a person can check.

**How to apply:** The issue's number stays with the problem its title leads with, and each other problem moves to a new issue written so that a session which never saw the conversation can act on it, per [[issues-have-no-fixed-form]]. A split divides problems, not file lists: two symptoms of one defect stay together. An issue is never split for the amount of work it takes or for the room it needs in the session's context. Order the splits with `blocked_by` edges, per [[a-blocked-by-edge-sets-the-order-of-the-work]].

A title that needs an and is a sign of two problems, and so is an issue the user cannot follow in one go.
