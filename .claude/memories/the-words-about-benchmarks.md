---
name: the-words-about-benchmarks
description: "The words about benchmarks are the industry's but for non-holdout and corroboration; the README and the manual page of `cladenc start` define them, and no definition or file name is kept here; the words never said; a harness is benchmarked, never said to attempt every task; a new word is the user's to choose, asked one question per word in the text of a reply"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 39898ed4-3db1-47de-9121-2bc5e7d15159
  modified: 2026-11-21T17:53:50.306Z
---

# The words about benchmarks

The words `cladenc` uses about benchmarks are benchmark, task, task specification, workspace, grader, attempt, completed and not completed, result, to benchmark, score, holdout, non-holdout, selection and corroboration. Each but non-holdout and corroboration is taken from the names the industry has for the thing. The README and the manual page of `cladenc start`, `docs/man/man1/cladenc-start.1`, say what each word means, and the code says which file of a task holds what. No definition and no file name is written in a memory.

**Why:** The benchmarks people already run, and the names the industry has adopted for what is in them, are the source of truth, so a reader already knows the words. A definition or a file name written both in a memory and in the repository drifts apart from the one in the repository when the code changes.

**How to apply:**

- Say the words in replies, issues, commits, the README, the help, the manual pages, what `cladenc` prints and the names in the code.
- Assessment, assess, statement, check, success and failure are never said for them. Score is never said for how one attempt ended: result is.
- What a harness works on is a task, with its task specification and its workspace. Project, repository and code base are never said for it, and the source code of a harness is called that. Workspace names the files of a task and nothing else: neither the outputs directory nor the directory a harness works in is called a workspace.
- A harness is benchmarked: the primogenitor is benchmarked, a child is benchmarked, `cladenc` benchmarks a harness. A sentence that has a harness attempting every task is wrong, not another way to say the same. That each task takes one attempt is said once, where the word attempt is explained, and attempt stays the word wherever one attempt is what a sentence is about, such as which attempt starts next.
- Completed is said only where the grader accepted the work, and an attempt that ran past its time limit is not completed. An attempt that has ended, whatever came of it, has finished.
- The tasks that are not held out are the non-holdout tasks, never the other tasks.
- The holdout tasks that `cladenc` only scores, and chooses nothing by, are the corroboration tasks, never test tasks, though test is the industry's word. Corroboration is taken from science: a claim is corroborated when it survives a test that could have refuted it, and corroborated does not mean proven. The holdout tasks that are left are the selection tasks, which artificial selection goes by, never pick or validation tasks.
- No word is coined for something that has a name.
- A word not in the list at the top is chosen by the user, and is added to the list once chosen. Put the things to them one at a time, one question for each: say what `cladenc` does, what `cladenc` calls it today, and every name the industry has for it, with who uses which. Do not put several words into one choice, do not choose for them, and say plainly where a word they bring is not the industry's. Ask in the text of a reply and not with the question dialog, which holds too few options to list every name.
- What the user says about one of the words in passing describes the idea. It is not a definition: do not take it to the letter.

Related: [[the-paper-is-a-starting-point]], which holds the biological words, and [[every-harness-is-benchmarked-in-full]].
