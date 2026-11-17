---
name: the-outputs-directory
description: "The directory a directed evolution writes into is the outputs directory, given with `--outputs-dir`: the user's name for it; run directory, `--run-dir` and `src/run_directory` in an older issue mean it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: cbdff9f0-8a8b-4526-8929-b4403e305f2d
  modified: 2026-11-17T06:41:19.656Z
---

# The outputs directory

The directory that `cladenc start` creates and a directed evolution writes into is the outputs directory. The flag that names it is `--outputs-dir`, on `start`, `resume` and `report` alike. It is not called the run directory.

**Why:** The name of the flag is the user's. The name before it told the user nothing: it could be read as the place `cladenc` runs in, or as a directory inside a container, and it is neither. My reading, not the user's: what the directory holds is what a directed evolution puts out, so the name says what a person finds there.

**How to apply:**

- Say outputs directory in replies, issues, commits, the README and the help text, and write the flag as `--outputs-dir <DIR>`.
- In the code the module is `src/outputs_directory/mod.rs`, its type is `OutputsDirectory`, and a value that holds its path is `outputs_directory`, or `outputs_dir` where it is the field of a flag. A test names its example directory `outputs`.
- Read run directory, `--run-dir`, `src/run_directory` and `RunDirectory` in an older issue as these, and correct each when the issue is brought up to date, per [[an-issue-is-brought-up-to-date-before-work]].
- When the user asks what it is, say it by what happens, per [[an-explanation-says-what-happens]]: it is a directory on the person's own machine, `cladenc start` creates it, and `cladenc` puts there the source code of every harness, what each attempt and each procreation left, and the journal. A container sees only the parts of it that `cladenc` mounts into it, under the paths of [[paths-follow-the-linux-filesystem-convention]].
- Mine, not the user's: the first record of the journal is still the run record, and no one has asked for another name.

Related: [[a-flag-is-not-called-an-option]], [[a-directed-evolution-is-started-with-cladenc-start]].
