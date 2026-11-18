---
name: more-tasks-completed-is-not-shown-to-be-an-improvement
description: "The user's argument: a harness chosen by the tasks its progenitors read about is expected to overfit to the benchmark, not to improve; nothing is written as fact that says `cladenc`'s harnesses improve themselves; issue 62 holds the analysis, and the user's design holds a random part of each benchmark out from the progenitors, the holdout tasks"
metadata:
  node_type: memory
  type: project
  originSessionId: cbdff9f0-8a8b-4526-8929-b4403e305f2d
  modified: 2026-11-17T13:28:52.346Z
---

# More tasks completed is not shown to be an improvement

That a harness of a directed evolution completes more tasks than its progenitor does not show that it improved. Every task `cladenc` counts is a task that progenitors read about, so what the count shows is what overfitting to the benchmark produces too. As `cladenc` stands, it is not said to lead to recursive self-improvement.

**Why:** It is the user's argument, and to the user it is the heart of the matter: the user expects the harnesses to overfit to the benchmarks, not to improve, and holds a sentence that says they improve themselves to be without support. My reasoning, which agrees: a progenitor reads the task specification, its own work and what the grader printed for each task, a change fitted to one task is a surer gain than a change to how the child works in general, and `cladenc` then chooses by the same tasks.

**How to apply:**

- Say that a child completes more tasks, or completes more of the tasks its progenitor read about. Say improved, better or self-improvement only for what is shown on tasks that no progenitor of the line read about, and write the rest as what is meant or expected, per [[an-issue-says-what-is-expected-apart-from-what-happens]].
- Do not put into the README, the help or an issue, as a fact, that `cladenc` leads to recursive self-improvement or that its harnesses improve themselves. That `cladenc` is there for that, and that a progenitor is asked to improve the progenitor's copy, can be said: each is a purpose or an instruction.
- Issue 62 holds the analysis and names the issues that follow from it, each with what a person has to decide. The tasks that no progenitor reads about are the holdout tasks, which is the user's word, per [[the-words-about-benchmarks]]. `cladenc` draws them at random out of each benchmark a person brings, and a person brings no benchmark of them: the design is the user's. Mine, not the user's: the draw is made once for a directed evolution, every harness is still benchmarked in full, on the holdout tasks as on the others, per [[every-harness-is-benchmarked-in-full]], and `cladenc` chooses the progenitor and names the best harness by the holdout tasks only.
- When the user states a doubt about what `cladenc` achieves, weigh it on its merits and say plainly where it holds, before defending what an issue says.

Related: [[the-words-about-benchmarks]], [[the-paper-is-a-starting-point]], [[cladenc-takes-a-primogenitor-and-a-model]], [[a-procreation-has-a-progenitor-a-copy-and-a-child]].
