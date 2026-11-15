---
name: no-flag-has-a-default
description: "No flag of cladenc gets a default and nothing about a search is assumed; the user decided this on 2026-11-15 and issue #30 was closed as not planned, so a default is never proposed and the count of required flags is not a problem"
metadata:
  node_type: memory
  type: feedback
  originSessionId: ab45aa6f-2cc3-4e53-b3f0-eae6aa421c43
  modified: 2026-11-15T13:35:39.927Z
---

# No flag has a default

`cladenc` has no defaults. A flag that takes a value has to be given, and nothing is assumed in the place of one left out. A search does what a person stated on its command line and nothing they did not state.

**Why:** On 2026-11-15 the user asked why issue #30 was a problem at all. The issue held that nine flags of `cladenc start`, among them the budget, the seed and the numbers of the policy, are a burden because a person has to choose every one, and the session that wrote it took the view that each should get a default. I answered that it was a small problem of convenience. The user did not see a problem in it at all: `cladenc` is to assume no value, and is to do nothing that was not spelled out. #30 was closed as not planned. The rule had stood in the plan of issue #7 with nothing on record about where it came from; since that day it is the user's.

**How to apply:** Do not propose a default for a flag, in an issue, in a decision put to the user or in a design, and do not file the number of required flags as a problem. A new flag that takes a value is required, `--model` of #27 included. The two inputs of [[cladenc-takes-a-progenitor-and-a-model]] are what a person brings to a search, not a count of flags, so a long command line does not go against that design.

Mine, not the user's: reading the rule as covering every flag added later and not only the nine of #30; reading it past flags, so that where a design could either assume something or have the person state it, having it stated is the one to offer; and leaving `--harness-build-command` as it is, since the user decided in #28 that it is given only where a harness has to be built, and leaving it out assumes no value in its place. Related: [[the-simplest-design-is-offered-first]], [[a-flag-is-not-called-an-option]], [[before-an-issue-is-labelled-needs-decision]]. Recorded under [[never-quote-the-user-verbatim]].
