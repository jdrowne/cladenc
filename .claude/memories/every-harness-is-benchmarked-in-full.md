---
name: every-harness-is-benchmarked-in-full
description: "Every harness is benchmarked in full once it is made, and the one number that bounds a directed evolution is the most procreations, given with --max-procreations; a count of attempts is never offered as what a person gives; a harness procreates only once it is benchmarked in full"
metadata:
  node_type: memory
  type: project
  originSessionId: 3ac128cd-9432-4e14-bf1c-720fe0e77c5a
  modified: 2026-11-21T18:46:58.989Z
---

# Every harness is benchmarked in full

Every harness of a directed evolution is benchmarked in full once it is made, on every task of every benchmark. It is said that way, per [[the-words-about-benchmarks]], and not as a harness attempting every task. The one number a person gives to bound a directed evolution is the most procreations it starts, with the flag `--max-procreations`. It does not count attempts, and there is no `--budget`. The flag takes 1 or more, and 0 is refused: at 1, the primogenitor and one child are benchmarked.

`cladenc` chooses which harness procreates. It does not choose which harness attempts next, and no harness is left benchmarked in part.

A harness can be the progenitor of a procreation only once it has been benchmarked in full: on every task of every benchmark the directed evolution was given, its attempt has finished. Where a procreation is due and no harness has been benchmarked in full, an attempt is started in its place.

**Why:** A progenitor learns from all of its benchmarks, each as a whole, so a harness that has attempted part of its tasks does not procreate. What is measured is a harness against the benchmarks, each as a whole, and an attempt is not a unit a person thinks in or gives a number of. That a child which is plainly worse is still benchmarked in full is accepted. `cladenc` is there for harnesses to improve themselves, so a directed evolution in which nothing procreates goes against what the tool is for. Every score is then over the same tasks, and no value of the flag pays for something other than what it says.

**How to apply:**

- Do not offer a count of attempts as the number a person gives, and do not propose leaving a harness after some of its tasks to save attempts.
- Do not propose a value, a flag or a mode that only benchmarks a harness and starts no procreation, as a way to learn a score or otherwise.
- Do not propose one task, another number of tasks, or a flag that gives the number, as what a harness needs before it procreates. Where a harness that has attempted some of the tasks is meant, say how many tasks it has attempted.
- Being benchmarked in full is not being run in one container.
- Code, help, README or an issue that has `--budget`, that counts in attempts how long a directed evolution goes on, or that has artificial selection choose which harness attempts which task, is older than this design and is brought to it, per [[an-issue-is-brought-up-to-date-before-work]].

Related: [[the-words-about-benchmarks]], [[the-paper-is-a-starting-point]], [[no-flag-has-a-default]].
