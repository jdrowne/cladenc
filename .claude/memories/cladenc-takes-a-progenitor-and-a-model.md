---
name: cladenc-takes-a-progenitor-and-a-model
description: "The user's design is that a search is given two things, the progenitor harness's directory and a model, and cladenc does the self-improvement itself; the code of 2026-11-14 asks for a procreator and an evaluator program instead"
metadata:
  node_type: memory
  type: project
  originSessionId: a00ea0c7-d6bd-4711-a4cb-3f7fb2827078
  modified: 2026-11-15T04:17:51.702Z
---

# cladenc takes a progenitor and a model

The user's design for `cladenc`: it does recursive self-improvement over coding harnesses after the HGM paper, and a person gives it two things to do so, a directory holding the copy of the progenitor harness and a model.

The user said this on 2026-11-14 while asking what issue #14 left to decide. They did not recognise a `cladenc` that starts programs its user writes, and the explanation of those programs confused them.

The code as it stood that day does not match. `cladenc run` takes no model and never runs a harness. It requires a procreator program and an evaluator program, a file of task ids and nine more options with no defaults, and does only the bookkeeping of the search. The plan of issues #1 to #9 says that `cladenc` never starts a harness and never calls a model. My own reading, not something the user said: that split was an earlier session's generalisation, not a thing the user asked for.

**Why:** The user wants a tool that is handed a harness and a model and improves the harness. Having to write the two programs that carry out the self-improvement leaves the main work to the person.

**How to apply:** Treat the two inputs as the intended interface. Do not describe the procreator and evaluator split to the user as settled, and do not deepen that contract, issue #14 included, before the mismatch is resolved with them. Open on 2026-11-14, and mine to ask, not things the user said: how `cladenc` starts a harness for a procreation and for an evaluation, where the tasks come from, what naming a model means, and whether the other options get defaults. Issue #27 first held the four. The user found that too much to be asked at once, so the same day it was split under [[one-indivisible-problem-per-issue]] into one decision to an issue, each labelled `needs decision`: #28 how a harness is started, #29 where the tasks come from, #27 what the model is, #30 whether the other options get defaults. #28 comes first, since #27 and #29 are blocked by it. My own reading, not something the user said: #28 and #29 state that `--procreator` and `--evaluator` are removed, not kept as a second way to carry out a job. Related: [[the-paper-is-a-starting-point]], [[before-an-issue-is-labelled-needs-decision]]. Recorded under [[never-quote-the-user-verbatim]].
