---
name: every-harness-is-benchmarked-in-full
description: "Every harness is benchmarked in full once it is made, and the one number that bounds a directed evolution is the most diversifications, given with --max-diversifications; a count of trials is never offered as what a person gives; a harness is cloned only once it is benchmarked in full"
metadata:
  node_type: memory
  type: project
  originSessionId: 3ac128cd-9432-4e14-bf1c-720fe0e77c5a
  modified: 2026-11-21T18:46:58.989Z
---

# Every harness is benchmarked in full

Every harness of a directed evolution is benchmarked in full once it is made, on every task of every benchmark. It is said that way, per [[the-words-about-benchmarks]], and not as a harness that has a trial on every task. The one number a person gives to bound a directed evolution is the most diversifications it starts, with the flag `--max-diversifications`. It does not count trials, and there is no `--budget`. The flag takes 1 or more, and 0 is refused: at 1, the primogenitor and one child are benchmarked.

`cladenc` chooses which harness is cloned. It does not choose which trial starts next, and no harness is left benchmarked in part.

A harness can be the progenitor of a diversification only once it has been benchmarked in full: on every task of every benchmark the directed evolution was given, its trial has finished. Where a diversification is due and no harness has been benchmarked in full, a trial is started in its place.

**Why:** A progenitor learns from all of its benchmarks, each as a whole, so a harness that has had a trial on part of its tasks is not cloned. What is measured is a harness against the benchmarks, each as a whole, and a trial is not a unit a person thinks in or gives a number of. That a child which is plainly worse is still benchmarked in full is accepted. `cladenc` is there for harnesses to improve themselves, so a directed evolution in which no harness is cloned goes against what the tool is for. Every score is then over the same tasks, and no value of the flag pays for something other than what it says.

**How to apply:**

- Do not offer a count of trials as the number a person gives, and do not propose leaving a harness after some of its tasks to save trials.
- Do not propose a value, a flag or a mode that only benchmarks a harness and starts no diversification, as a way to learn a score or otherwise.
- Do not propose one task, another number of tasks, or a flag that gives the number, as what a harness needs before it is cloned. Where a harness that has had a trial on some of the tasks is meant, say how many tasks it has had a trial on.
- Being benchmarked in full is not being run in one container.
- Code, help, README or an issue that has `--budget`, that counts in trials how long a directed evolution goes on, or that has artificial selection choose which harness has a trial on which task, is older than this design and is brought to it, per [[an-issue-is-brought-up-to-date-before-work]].

Related: [[the-words-about-benchmarks]], [[the-paper-is-a-starting-point]], [[no-flag-has-a-default]].
