---
name: the-words-about-benchmarks
description: "The words about benchmarks, eleven chosen from the industry's names: benchmark, task, task specification, workspace, grader, attempt, completed and not completed, result, to benchmark, score, holdout; and two that are not the industry's, non-holdout and corroboration; a harness is said to be benchmarked, not to attempt every task; a new one is chosen by the user, asked one at a time in the text of a reply"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 39898ed4-3db1-47de-9121-2bc5e7d15159
  modified: 2026-11-21T17:04:52.675Z
---

# The words about benchmarks

Each word `cladenc` uses about benchmarks is taken from the names the industry has for the thing, but for the last two of the table, non-holdout and corroboration, which stand in place of the industry's.

| What `cladenc` does | The word |
| --- | --- |
| The set of tasks a person brings with `--benchmark` | benchmark |
| One item of a benchmark | task |
| The text in a task that says what is to be done | task specification, in the file `specification` |
| The files in a task that the work starts from | workspace, in the directory `workspace` |
| The program in a task that says whether the work passes | grader, in the file `grader` |
| One harness working on one task, once | attempt |
| The two ways an attempt can end | completed, not completed |
| The name for how an attempt ended | result |
| Putting a harness through a whole benchmark | to benchmark, as a verb |
| The number that says how a harness did | score |
| The tasks of a directed evolution that no progenitor reads about | holdout, as in holdout tasks |
| The tasks that are not held out | non-holdout, as in non-holdout tasks |
| The holdout tasks that `cladenc` only scores, and chooses nothing by | corroboration, as in corroboration tasks |

**Why:** The source of truth for these words is the benchmarks themselves and what the industry has adopted de facto, and among the industry's names the user chooses. `cladenc` is run against benchmarks that people already use, and those come with words a reader already knows.

**How to apply:**

- Say these words in replies, issues, commits, the README, the help, what `cladenc` prints and the names in the code. Assessment, assess, statement, check, success and failure are not said for them, and score is not said for how one attempt ended.
- What a harness goes through is said with the verb: the primogenitor is benchmarked, a child is benchmarked, `cladenc` benchmarks a harness. It is not said as a harness attempting every task: a sentence of that kind is wrong, not another way to put it. That each task takes one attempt is said once, where the word is explained, and attempt stays the word wherever one attempt is what the sentence is about, such as which attempt starts next.
- An example is told in these words too. What a harness works on is a task, with its task specification and its workspace. Project is not said for it: `cladenc` runs benchmarks, and a project names nothing in one. The same holds for repository and code base, and the source code of a harness is called that.
- Completed is said only where the grader accepted the work. An attempt that has ended, whatever came of it, has finished.
- An attempt that ran past its time limit is not completed.
- A word that is not in the table is chosen by the user. Put the things to them one at a time, one question for each: say what `cladenc` does, what `cladenc` calls it today, and every name the industry has for it, with who uses which. Do not put several words into one choice, do not choose for them, and say plainly where a word they bring is not the industry's. Ask in the text of a reply and not with the question dialog, which holds too few options to list every name.
- What the user says about one of these words in passing describes the idea. It is not a definition: do not take it to the letter.
- The flag that gives how large a part of each benchmark's tasks is held out is `--holdout-percentage`.
- The flag that divides the holdout tasks is `--holdout-corroboration-percentage`. It gives the percentage of the holdout tasks of each benchmark that are corroboration tasks, not a percentage of the benchmark. `cladenc` only scores the corroboration tasks, and chooses nothing by them. Corroboration is taken from science and not from the industry, whose word is test: a claim is corroborated when it survives a test that could have refuted it, and corroborated does not mean proven. The holdout tasks that are left are the selection tasks, which artificial selection goes by: selection is the word for them, over pick and validation. Issue 67 holds the design.
- The tasks that are not held out are the non-holdout tasks, and other tasks is not said for them.
- No word is coined for something that has a name, per [[the-simplest-design-is-offered-first]].

The names in the code, the journal and the outputs directory follow the table, as `--attempt-timeout`, the record `attempt_started` and the result `timed_out` do. A harness has been benchmarked in full once it has attempted every task of every benchmark. A task that is one of the holdout tasks is said to be held out. `/var/lib/cladenc/work` keeps its name, as the directory a harness works in, for a procreation as for an attempt, and in the code the outputs directory is not called a workspace, so that the word names one thing.

Related: [[the-paper-is-a-starting-point]], which holds the biological words, [[every-harness-is-benchmarked-in-full]] and [[an-explanation-says-what-happens]].
