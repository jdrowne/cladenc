---
name: assessing-is-not-scoring
description: "To assess is to try a harness on a task or a benchmark; to score is to set it against success and failure — an assessment never succeeds, fails or has an outcome, the harness is scored a success or a failure, and what comes of it is a score"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 39898ed4-3db1-47de-9121-2bc5e7d15159
  modified: 2026-11-16T13:08:11.199Z
---

# Assessing is not scoring

`cladenc` does two things with a harness and a task, and each has a word of its own.

- To assess is to try a harness on something: a task, or the tasks of a benchmark. Assessment, assess, assessed and assessing are right for that, much as evaluating would be.
- To score is to set the harness against success and failure. What comes of it is a score.

**Why:** The user's rule. Both words are valid, and each says what is being done and against what: an assessment is against a task or a benchmark, and scoring is against a success or a failure. Saying assessment for the second is the wrong word.

**How to apply:** In replies, issues, commits, the README, the help, what `cladenc` prints and the names in the code:

- An assessment starts, runs, ends, is carried out again and runs past its time limit. It does not succeed or fail, and it has no outcome.
- A harness is scored a success or a failure on a task. Where what matters is that its success or failure is known, say that the harness has been scored, per [[an-explanation-says-what-happens]].
- What names the trying of a harness on a task keeps the word: the flag `--assessment-timeout`, the directories named `assessments`, and the records `assessment_started` and `assessment_finished` of the journal.

Mine, not the user's: the names that follow from the rule. The type is `Score`, and its words are `success`, `failure` and `timed_out`. The file an assessment leaves and the key of `assessment_finished` are both named `score`. An assessment that runs past its time limit scores the harness a failure, which is kept as `timed_out`. A procreation is not scored: how it ended is its outcome, `ProcreationOutcome`. Related: [[the-paper-is-a-starting-point]], which holds the other words.
