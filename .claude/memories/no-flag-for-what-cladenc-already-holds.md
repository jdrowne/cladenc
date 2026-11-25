---
name: no-flag-for-what-cladenc-already-holds
description: "A flag is for what only a person can tell cladenc; what cladenc holds already, such as the source code of the primogenitor and of every harness in the outputs directory, is read by cladenc and never asked for by a flag"
metadata:
  node_type: memory
  type: feedback
  originSessionId: e6fd5ee3-e0eb-4639-9e67-bfbb3c1ed6de
  modified: 2026-11-20T01:16:03.746Z
---

# No flag for what cladenc already holds

A flag gives `cladenc` something only a person can tell it. What `cladenc` holds already is read by `cladenc` and is not asked for by a flag. At every moment of a directed evolution the outputs directory keeps the source code of the primogenitor and of every other harness, the trials of every harness, and the journal, so `cladenc` knows where the source code of the primogenitor and of a progenitor is without being told.

**Why:** A flag for a value `cladenc` has at hand is one more thing a person can give wrongly. A design that has a person name, by a flag, a harness or a value that `cladenc` has at hand is turned down, and so is a second flag put in the place of the first.

**How to apply:** Before a flag is proposed, in an issue, a design or a decision put to the user, check whether `cladenc` holds the value or can read it from the outputs directory and the journal. Where it can, the design has `cladenc` read it. A design that works inside one directed evolution is offered before one that has a person carry a harness from the outputs directory of one directed evolution into another. Where a design seems to need a count of its own, look for a way to do the thing at each procreation, so that `--max-procreations` already gives the count.

The rule reaches past source code, to every value `cladenc` can read or work out. Related: [[no-flag-has-a-default]], [[the-simplest-design-is-offered-first]], [[the-outputs-directory]], [[cladenc-takes-a-primogenitor-and-a-model]].
