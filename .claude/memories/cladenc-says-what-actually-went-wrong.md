---
name: cladenc-says-what-actually-went-wrong
description: "cladenc is held to precision about failures: what it prints and records says what actually went wrong; giving no reason, or recording a result that reads as something else having happened, is incorrect behaviour and is never offered as an option"
metadata:
  type: project
---

# cladenc says what actually went wrong

`cladenc` is held to precision. When something goes wrong, what `cladenc` prints and what it records say what actually went wrong. Giving no reason is incorrect behaviour, and so is recording an outcome that reads as something else having happened, such as a harness not completing a task when in fact the grader never ran.

**Why:** A person and a progenitor act on what `cladenc` reports, and artificial selection counts the recorded results. A vague or wrong record misleads all three.

**How to apply:** Treat a silent or mislabelled failure as a bug, not as a design choice. When laying out the options for a decision about a failure, leaving the failure unexplained is not one of them. Where a failure can be transient, retrying with exponential backoff belongs among the options, ahead of recording any outcome. Related: [[cladenc-runs-any-benchmark]], [[more-tasks-completed-is-not-shown-to-be-an-improvement]].
