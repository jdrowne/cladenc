---
name: the-simplest-design-is-offered-first
description: "When a decision is put to the user, lead with the simplest design that works, such as an option on the command line, and explain it in short concrete steps with every word defined"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 2c25b762-5edb-4922-82ce-34177297057c
  modified: 2026-11-15T15:40:19.165Z
---

# The simplest design is offered first

When a design decision is put to the user, the first option offered is the simplest one that solves the problem, and for `cladenc` that is usually one more option on the command line. A heavier design, such as a convention every harness has to follow or a tool `cladenc` has to drive, is offered only where the simple one fails, and the reply says where it fails.

The explanation is a few short steps about one concrete case. Each word that carries meaning is either the user's own or defined where it first appears.

A reply carries one point. The other points wait for later replies, one each, even when they are all ready. A reply that also lists choices of mine, uncommitted files and other open issues leaves the user lost.

**Why:** The user stops following an explanation that uses words nobody has defined for them, such as parent, run, picked, task and session, that is loose about what a thing is, or that asks several decisions at once. A choice between heavy designs strikes them as overcomplicated, and they settle it at once when it reaches them as plain flags.

**How to apply:** Before asking, find the smallest thing that would work and offer that. Walk through the first step of a directed evolution with the progenitor alone, not the general case. Keep the trade-offs to what changes for the user. My own interpretation, not something the user said: they would sooner pass one more option than have `cladenc` impose a rule on harnesses or depend on another tool. Related: [[cladenc-takes-a-progenitor-and-a-model]], [[a-harness-is-not-called-a-coding-agent]], [[before-an-issue-is-labelled-needs-decision]].
