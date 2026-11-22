---
name: the-outputs-directory
description: "The directory a directed evolution writes into is the outputs directory, given with `--outputs-dir`, and it is not called the run directory"
metadata:
  node_type: memory
  type: feedback
  originSessionId: cbdff9f0-8a8b-4526-8929-b4403e305f2d
  modified: 2026-11-17T06:41:19.656Z
---

# The outputs directory

The directory that `cladenc start` creates and a directed evolution writes into is the outputs directory. The flag that names it is `--outputs-dir`. It is not called the run directory.

**Why:** A name such as run directory tells the user nothing: it can be read as the place `cladenc` runs in, or as a directory inside a container, and the directory is neither. What the directory holds is what a directed evolution puts out, so the name says what a person finds there.

**How to apply:**

- Say outputs directory in replies, issues, commits, the README and the help text, and write the flag as `--outputs-dir <DIR>`.
- In the code the module is `src/outputs_directory/mod.rs`, its type is `OutputsDirectory`, and a value that holds its path is `outputs_directory`, or `outputs_dir` where it is the field of a flag. A test names its example directory `outputs`.
- What it is: a directory on the person's own machine, `cladenc start` creates it, and `cladenc` puts there the source code of every harness, what each attempt and each procreation left, and the journal. A container sees only the parts of it that `cladenc` mounts into it, under the paths of [[paths-follow-the-linux-filesystem-convention]].
- The first record of the journal keeps the name run record.

Related: [[a-flag-is-not-called-an-option]].
