---
name: a-flag-is-not-called-an-option
description: "A `--name` argument of cladenc is called a flag, not an option, and the README documents each sub-command as a command block followed by a table of its flags in alphabetical order"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 4255197e-36ce-457f-a870-d07a0b8628ef
  modified: 2026-11-15T12:58:17.891Z
---

# A flag is not called an option

A `--name` argument of `cladenc` is called a flag. The README does not call it an option.

**Why:** On 2026-11-15 the user went through the README's section on the sub-commands and asked for five changes: the column that names an argument is headed Flag, the columns that explain are headed Description, a flag is marked Required or Not Required in place of yes or no, the flags are sorted alphabetically, and each sub-command shows a plain-text command with every flag on a line of its own.

**How to apply:** `test/unit/cli/readme.rs` holds the README to that layout, so a new flag or sub-command fails the tests until the README has it. Mine, not the user's: the command block sits right under the sub-command's heading, ahead of the table, with a backslash ending each line but the last; the flags of `--help` are in the same alphabetical order, which a test in `test/unit/cli/help.rs` holds; and the word flag is used in replies, issues and commits as well as in the README. Left as they were: the names in the code, such as `StartOptions`, and the heading `Options:` that clap prints in `--help`. Related: [[a-search-is-started-with-cladenc-start]]. Recorded under [[never-quote-the-user-verbatim]].
