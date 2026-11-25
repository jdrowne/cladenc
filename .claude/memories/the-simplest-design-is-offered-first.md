---
name: the-simplest-design-is-offered-first
description: "When a design decision is put to the user, lead with the simplest design that works, such as a flag on the command line; a heavier design is offered only where the simple one fails"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 2c25b762-5edb-4922-82ce-34177297057c
  modified: 2026-11-15T15:40:19.165Z
---

# The simplest design is offered first

When a design decision is put to the user, the first option offered is the simplest one that solves the problem, and for `cladenc` that is usually one more flag on the command line. A heavier design, such as a convention every harness has to follow or a tool `cladenc` has to drive, is offered only where the simple one fails, and the reply says where it fails.

**Why:** A choice between heavy designs strikes the user as overcomplicated, and they settle it at once when it reaches them as plain flags.

**How to apply:** Before asking, find the smallest thing that would work and offer that. Related: [[cladenc-takes-a-primogenitor-and-a-model]], [[before-an-issue-is-labelled-needs-decision]].
