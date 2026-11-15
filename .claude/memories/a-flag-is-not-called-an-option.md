---
name: a-flag-is-not-called-an-option
description: "A `--name` argument of cladenc is called a flag, not an option; the README documents `cladenc` itself and each sub-command as a command block followed by a table of its flags in alphabetical order, and what applies to some sub-commands is written under them"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 4255197e-36ce-457f-a870-d07a0b8628ef
  modified: 2026-11-15T12:58:17.891Z
---

# A flag is not called an option

A `--name` argument of `cladenc` is called a flag. The README does not call it an option.

**Why:** On 2026-11-15 the user went through the README's section on the sub-commands and asked for five changes: the column that names an argument is headed Flag, the columns that explain are headed Description, a flag is marked Required or Not Required in place of yes or no, the flags are sorted alphabetically, and each sub-command shows a plain-text command with every flag on a line of its own.

Later the same day the user found two things missing: `cladenc` with no sub-command had no section of its own, and the paragraph on how long a search goes on and how it is stopped sat in the general part, where it belongs under `start` and `resume`, the two sub-commands it is about.

**How to apply:** `cladenc` itself has a section in the same layout as a sub-command's, ahead of the sub-commands. Text that is about some sub-commands goes in the section of each of them, not in the general part. `test/unit/cli/readme.rs` holds the README to that layout, so a new flag or sub-command fails the tests until the README has it. Mine, not the user's: the paragraph is written out in both sections, after the table, in words fitted to each; the section of `cladenc` says what it does with nothing after it; the command block sits right under the sub-command's heading, ahead of the table, with a backslash ending each line but the last; the flags of `--help` are in the same alphabetical order, which a test in `test/unit/cli/help.rs` holds; and the word flag is used in replies, issues and commits as well as in the README. Help is asked for with the flag `--help` alone, and `cladenc` does not support a `help` sub-command: since 2026-11-15 `cladenc help` is refused like any other word that is not a sub-command. The user saw the sub-command clap adds by itself named in the README and said it should not be supported, help being a flag and not a sub-command. Say that it is not supported; calling it turned off reads as if it were still there, and the user objected to that wording. Left as they were: the names in the code, such as `StartOptions`, and the heading `Options:` that clap prints in `--help`. Related: [[a-search-is-started-with-cladenc-start]]. Recorded under [[never-quote-the-user-verbatim]].
