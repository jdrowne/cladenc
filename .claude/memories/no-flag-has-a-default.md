---
name: no-flag-has-a-default
description: "No flag of cladenc gets a default and nothing about a search is assumed; a default is never proposed and the count of required flags is not a problem; the seed of the random draws is no flag, cladenc draws it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: ab45aa6f-2cc3-4e53-b3f0-eae6aa421c43
  modified: 2026-11-15T15:40:19.165Z
---

# No flag has a default

`cladenc` has no defaults. A flag that takes a value has to be given, and nothing is assumed in the place of one left out. A search does what a person stated on its command line and nothing they did not state.

The seed of the random draws is not a flag. `cladenc` draws it when a search is started and writes it in the journal, and `cladenc resume` reads it from there.

**Why:** It is the user's rule: `cladenc` assumes no value and does nothing that was not spelled out. Having to choose every value of a search, the budget and the numbers of the policy among them, is no problem to them, not even a small one of convenience. That the seed is drawn is the user's decision too: no value of it is better than another, and a search cannot be repeated, so a person has no reason to choose one.

**How to apply:** Do not propose a default for a flag, in an issue, in a decision put to the user or in a design, and do not file the number of required flags as a problem. A new flag that takes a value is required. Do not propose a flag for the seed. The inputs of [[cladenc-takes-a-progenitor-and-a-model]] are what a person brings to a search, not a count of flags, so a long command line does not go against that design.

Mine, not the user's: reading the rule as covering every flag added later; and reading it past flags, so that where a design could either assume something or have the person state it, having it stated is the one to offer. Also mine: the seed is the one number of a search that a person has no reason to choose, so it is no ground for having `cladenc` draw or assume any other value. The flag that says how a harness is built is given only where a harness has to be built, which is the user's decision, and leaving it out assumes no value in its place. Related: [[the-simplest-design-is-offered-first]], [[a-flag-is-not-called-an-option]], [[before-an-issue-is-labelled-needs-decision]].
