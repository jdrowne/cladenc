---
name: more-tasks-completed-is-not-shown-to-be-an-improvement
description: "A harness chosen by the tasks its progenitors read about is expected to overfit to the dataset, not to improve; nothing is written as fact that says `cladenc`'s harnesses improve themselves; issue 62 holds the analysis, and a random part of each dataset is held out from the progenitors, the holdout tasks"
metadata:
  node_type: memory
  type: project
  originSessionId: cbdff9f0-8a8b-4526-8929-b4403e305f2d
  modified: 2026-11-26T05:37:35.817Z
---

# More tasks completed is not shown to be an improvement

That a harness of a directed evolution completes more tasks than its progenitor does not show that it improved. Every task `cladenc` counts is a task that progenitors read about, so what the count shows is what overfitting to the dataset produces too. As `cladenc` stands, it is not said to lead to recursive self-improvement.

**Why:** The harnesses are expected to overfit to the datasets, not to improve, and a sentence that says they improve themselves is without support. A progenitor reads the instruction, its own work and what the verifier printed for each task, a change fitted to one task is a surer gain than a change to how the child works in general, and `cladenc` then chooses by the same tasks.

**How to apply:**

- Say that a child completes more tasks, or completes more of the tasks its progenitor read about. Say improved, better or self-improvement only for what is shown on tasks that no progenitor of the line read about, and write the rest as what is meant or expected.
- Do not put into the README, the help or an issue, as a fact, that `cladenc` leads to recursive self-improvement or that its harnesses improve themselves. That `cladenc` is there for that, and that a progenitor is asked to improve the progenitor's clone, can be said: each is a purpose or a request.
- Issue 62 holds the analysis. The tasks that no progenitor reads about are the holdout tasks, per [[the-words-about-datasets]]. `cladenc` draws them at random out of each dataset a person brings, and a person brings no dataset of them.
- A person has to be able to tell whether the lead of the best harness is luck, as the scientific method asks, though luck in the choosing does not stop a directed evolution. So a part of the holdout tasks, the corroboration tasks of [[the-words-about-datasets]], is kept from artificial selection. `cladenc` chooses nothing by that part, and never the best harness.
- When the user states a doubt about what `cladenc` achieves, weigh it on its merits and say plainly where it holds, before defending what an issue says.

Related: [[the-words-about-datasets]], [[the-paper-is-a-starting-point]], [[cladenc-takes-a-primogenitor-and-a-model]], [[a-diversification-has-a-progenitor-a-clone-and-a-child]].
