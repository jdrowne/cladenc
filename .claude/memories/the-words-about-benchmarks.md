---
name: the-words-about-benchmarks
description: "The user's ten words about benchmarks, each chosen from the industry's names: benchmark, task, task specification, workspace, grader, attempt, completed and not completed, result, to benchmark, score; a harness is said to be benchmarked, not to attempt every task; a new one is chosen by the user, asked one at a time in the text of a reply"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 39898ed4-3db1-47de-9121-2bc5e7d15159
  modified: 2026-11-17T07:05:40.738Z
---

# The words about benchmarks

The words `cladenc` uses about benchmarks are the user's choice, each taken from the names the industry has for the thing.

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

**Why:** The user's rule. The source of truth for these words is the benchmarks themselves and what the industry has adopted de facto, and among the industry's names the user chooses. My reading of the reason, not the user's: `cladenc` is run against benchmarks that people already use, and those come with words a reader already knows.

**How to apply:**

- Say these words in replies, issues, commits, the README, the help, what `cladenc` prints and the names in the code. Assessment, assess, statement, check, success and failure are not said for them, and score is not said for how one attempt ended. An older issue or commit that says them means the words above.
- What a harness goes through is said with the verb: the primogenitor is benchmarked, a child is benchmarked, `cladenc` benchmarks a harness. It is not said as a harness attempting every task: the user reads a sentence of that kind as wrong, not as another way to put it. Mine, not the user's: that each task takes one attempt is said once, where the word is explained, and attempt stays the word wherever one attempt is what the sentence is about, such as which attempt starts next.
- An example is told in these words too. What a harness works on is a task, with its task specification and its workspace. Project is not said for it: the user's point is that `cladenc` runs benchmarks, and a project names nothing in one. Mine, not the user's: the same holds for repository and code base, and the source code of a harness is called that.
- Completed is said only where the grader accepted the work. An attempt that has ended, whatever came of it, has finished.
- An attempt that ran past its time limit is not completed, and is recorded as `timed_out`.
- A word that is not in the table is chosen by the user. Put the things to them one at a time, one question for each: say what `cladenc` does, what `cladenc` calls it today, and every name the industry has for it, with who uses which. Do not put several words into one choice, do not choose for them, and say plainly where a word they bring is not the industry's. Ask in the text of a reply and not with the question dialog, which holds too few options to list every name.
- What the user says about one of these words in passing describes the idea. It is not a definition: do not take it to the letter.
- No word is coined for something that has a name, per [[the-simplest-design-is-offered-first]].

Mine, not the user's: the names that follow from the table. The flag is `--attempt-timeout`, the directories are named `attempts`, and the records of the journal are `attempt_started` and `attempt_finished`. The words a result is written with are `completed`, `not_completed` and `timed_out`. The columns of `cladenc report` are completed, not completed, clade completed, clade not completed and clade score. A harness has been benchmarked in full once it has attempted every task of every benchmark. `/var/lib/cladenc/work` keeps its name, as the directory a harness works in, for a procreation as for an attempt. In the code the outputs directory is not called a workspace, so that the word names one thing.

Related: [[the-paper-is-a-starting-point]], which holds the biological words, [[a-harness-procreates-only-once-benchmarked-in-full]] and [[an-explanation-says-what-happens]].
