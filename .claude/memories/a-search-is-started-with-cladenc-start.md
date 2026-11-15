---
name: a-search-is-started-with-cladenc-start
description: "The sub-command that starts a new search is `cladenc start`; older issues call it `cladenc run` and mean the same sub-command"
metadata:
  node_type: memory
  type: project
  originSessionId: 4255197e-36ce-457f-a870-d07a0b8628ef
  modified: 2026-11-15T15:40:19.165Z
---

# A search is started with cladenc start

The sub-command that starts a new search is `cladenc start`. Older issues call it `cladenc run`, which is the same sub-command under its earlier name.

**Why:** The name pairs with `resume`, and the sub-command's own help text says that it starts a search. The user agreed to it.

**How to apply:** Write `cladenc start` in replies, issues, commits and the README. Read `cladenc run` in an older issue as the same sub-command, and correct it when the issue is brought up to date, per [[an-issue-is-brought-up-to-date-before-work]]. My own choice, not something the user said: the noun stays, so one search is still a run, kept in the run directory that `--run-dir` names and opened in the journal by the run record. There is no sub-command that stops a search: Ctrl-C or a signal does, and the README says so. Related: [[a-flag-is-not-called-an-option]].
