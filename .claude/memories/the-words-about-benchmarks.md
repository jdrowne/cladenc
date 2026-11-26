---
name: the-words-about-benchmarks
description: "The words about benchmarks follow Harbor's terminology: a thing Harbor names takes Harbor's name, read from its documentation, with no question; verifier, never grader, and trial, never attempt; the README and the manual page of `cladenc start` define the words, and no definition or file name is kept here; the words never said; a harness is benchmarked, never said to have a trial on every task; an older word that Harbor names otherwise, and a thing Harbor has no name for, is the user's to choose, asked one question per word in the text of a reply"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 39898ed4-3db1-47de-9121-2bc5e7d15159
  modified: 2026-11-26T03:33:36.376Z
---

# The words about benchmarks

`cladenc` uses Harbor's terminology. The words it uses about benchmarks are benchmark, task, instruction, diversification prompt, workspace, verifier, verifier environment, verifier container, trial, completed and not completed, result, to benchmark, score, holdout, non-holdout, selection and corroboration. The README and the manual page of `cladenc start`, `docs/man/man1/cladenc-start.1`, say what each word means, and the code says which file of a task holds what. No definition and no file name is written in a memory.

**Why:** On 2026-11-25 the user decided that `cladenc` uses Harbor's terminology from then on. Grader had been chosen as one of the industry's names five days before `cladenc` took up Harbor's task format, nothing had weighed it against Harbor's verifier, and the industry has more than one name for most of these things. `cladenc` reads Harbor's tasks, so a person who brings them already knows Harbor's words. A definition or a file name written both in a memory and in the repository drifts apart from the one in the repository when the code changes.

**How to apply:**

- Say the words in replies, issues, commits, the README, the help, the manual pages, what `cladenc` prints and the names in the code.
- Harbor's names are read from Harbor's documentation, which the README links, and not recalled.
- A thing that Harbor names and the list at the top does not takes Harbor's name. No question is put to the user for it, and the name is added to the list.
- Verifier replaced grader on 2026-11-25, in the repository as well. With it came the verifier environment and the verifier container, and a verifier that has a container of its own runs separately, after `separate` in `task.toml`. Grader, grade, graded and grading are never said.
- Trial replaced attempt on 2026-11-25 as well, in the repository too: one harness working on one task, once, is a trial of the harness on the task. Attempt, attempted and attempting are never said for it. Trial is no verb: a harness has a trial on a task, a trial is started, carried out and ended, and a task is given a trial.
- Assessment, assess, statement, check, success and failure are never said for the words of the list. Score is never said for how one trial ended: result is.
- Instruction replaced task specification, decided on 2026-11-25: it is the text that a task brings and that a harness is given in a trial. It takes no word in front. A task has one instruction and every trial on the task is given the same one, so trial instruction is not said. Task specification and specification are never said for it. Issue 172 carries the word into the repository.
- The diversification prompt is the text that `cladenc` holds itself and gives a harness in a diversification, the one that asks the harness to improve its clone. Harbor has no name for it. On 2026-11-25 the user chose prompt for it, over instruction and self-modification instruction, and later that day named the step it belongs to a diversification, per [[a-diversification-has-a-progenitor-a-clone-and-a-child]]. It is never called an instruction, and the instruction of a task is never called a prompt. The primogenitor is given the diversification prompt as well in the diversification of an experiment, where it creates the singleton control group harness, per [[the-singleton-control-group-harness]]. It is the name in the repository as well.
- Four older words name things that Harbor names otherwise: benchmark is Harbor's dataset, score its reward, harness its agent, and container its sandbox. Each stays as it is until the user says that it changes, and [[a-harness-is-not-called-a-coding-agent]] and [[cladenc-does-not-run-jobs]] hold as written. Put each to the user on its own, one per reply, with what changes in the repository under Harbor's name.
- What a harness works on is a task, with its instruction and its workspace. Project, repository and code base are never said for it, and the source code of a harness is called that. Workspace names the files of a task and nothing else: neither the outputs directory nor the directory a harness works in is called a workspace.
- A harness is benchmarked: the primogenitor is benchmarked, a child is benchmarked, `cladenc` benchmarks a harness. A sentence that has a harness having a trial on every task is wrong, not another way to say the same. That each task takes one trial is said once, where the word trial is explained, and trial stays the word wherever one trial is what a sentence is about, such as which trial starts next.
- Completed is said only where the verifier accepted the work, and a trial that ran past its time limit is not completed. A trial that has ended, whatever came of it, has finished.
- The tasks that are not held out are the non-holdout tasks, never the other tasks.
- The holdout tasks that `cladenc` only scores, and chooses nothing by, are the corroboration tasks, never test tasks, though test is the industry's word. Corroboration is taken from science: a claim is corroborated when it survives a test that could have refuted it, and corroborated does not mean proven. The holdout tasks that are left are the selection tasks, which artificial selection goes by, never pick or validation tasks.
- No word is coined for something that has a name.
- A thing that Harbor has no name for, and that the list does not hold, is named by the user, and the name is added to the list once chosen. Put the things to them one at a time, one question for each: say what `cladenc` does, what `cladenc` calls it today, and every name the industry has for it, with who uses which. Do not put several words into one choice, do not choose for them, and say plainly where a word they bring is not the industry's. Ask in the text of a reply and not with the question dialog, which holds too few options to list every name.
- What the user says about one of the words in passing describes the idea. It is not a definition: do not take it to the letter.

Related: [[the-paper-is-a-starting-point]], which holds the biological words, and [[every-harness-is-benchmarked-in-full]].
