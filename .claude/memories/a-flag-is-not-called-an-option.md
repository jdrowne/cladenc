---
name: a-flag-is-not-called-an-option
description: "A `--name` argument of cladenc is called a flag, not an option; the README documents `cladenc` itself and each sub-command as a command block followed by a table of its flags in alphabetical order, and what applies to some sub-commands is written under them"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 4255197e-36ce-457f-a870-d07a0b8628ef
  modified: 2026-11-15T15:40:19.165Z
---

# A flag is not called an option

A `--name` argument of `cladenc` is called a flag. The README does not call it an option.

Help is asked for with the flag `--help` alone. `cladenc` has no `help` sub-command, and `cladenc help` is refused like any other word that is not a sub-command. Say that it is not supported; calling it turned off reads as if it were still there.

**Why:** The word and the layout of the README are the user's choices. Help is a flag and not a sub-command, so the sub-command that clap adds by itself is not wanted.

**How to apply:** `test/unit/cli/readme.rs` holds the README to this layout, so a new flag or sub-command fails the tests until the README has it.

- `cladenc` itself has a section in the same layout as a sub-command's, ahead of the sub-commands.
- A section shows a plain-text command with every flag on a line of its own, and a table headed Flag, Required and Description.
- A flag is marked Required or Not Required, and the flags are in alphabetical order.
- Text that is about some sub-commands, such as how long a directed evolution goes on and how it is stopped, goes in the section of each of them, not in the general part.

Mine, not the user's: such text is written out in each section, after the table, in words fitted to it; the section of `cladenc` says what it does with nothing after it; the command block sits right under the heading, ahead of the table, with a backslash ending each line but the last; the flags of `--help` are in the same alphabetical order, which a test in `test/unit/cli/help.rs` holds; and the word flag is used in replies, issues and commits as well as in the README. Left as they are: the names in the code, such as `StartOptions`, and the heading `Options:` that clap prints in `--help`. Related: [[a-directed-evolution-is-started-with-cladenc-start]].
