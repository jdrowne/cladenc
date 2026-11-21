---
name: no-flag-has-a-default
description: "No flag of cladenc gets a default and nothing about a directed evolution is assumed; a default is never proposed and the count of required flags is not a problem; the seed of the random draws is no flag, cladenc draws it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: ab45aa6f-2cc3-4e53-b3f0-eae6aa421c43
  modified: 2026-11-15T15:40:19.165Z
---

# No flag has a default

`cladenc` has no defaults. A flag that takes a value has to be given, and nothing is assumed in the place of one left out. A directed evolution does what a person stated on its command line and nothing they did not state.

The seed of the random draws is not a flag. `cladenc` draws it.

**Why:** `cladenc` assumes no value and does nothing that was not spelled out. Having to choose every value of a directed evolution, the most procreations and the numbers the artificial selection works from among them, is no problem, not even a small one of convenience. The seed is drawn: no value of it is better than another, and a directed evolution cannot be repeated, so a person has no reason to choose one.

**How to apply:** Do not propose a default for a flag, in an issue, in a decision put to the user or in a design, and do not file the number of required flags as a problem. A new flag that takes a value is required. Do not propose a flag for the seed. The inputs of [[cladenc-takes-a-primogenitor-and-a-model]] are what a person brings to a directed evolution, not a count of flags, so a long command line does not go against that design.

The rule covers every flag added later, and reaches past flags: where a design could either assume something or have the person state it, having it stated is the one to offer. The seed is the one number of a directed evolution that a person has no reason to choose, so it is no ground for having `cladenc` draw or assume any other value. The flag that says how a harness is built is given only where a harness has to be built, and leaving it out assumes no value in its place. Related: [[the-simplest-design-is-offered-first]], [[a-flag-is-not-called-an-option]], [[before-an-issue-is-labelled-needs-decision]].
