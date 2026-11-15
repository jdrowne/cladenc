---
name: a-search-is-started-with-cladenc-start
description: "The sub-command that starts a new search is `cladenc start`; it was `cladenc run` until 2026-11-15, so older issues and memories that say `cladenc run` mean it"
metadata:
  node_type: memory
  type: project
  originSessionId: 4255197e-36ce-457f-a870-d07a0b8628ef
  modified: 2026-11-15T12:58:12.204Z
---

# A search is started with cladenc start

The sub-command that starts a new search is `cladenc start`. Until 2026-11-15 it was `cladenc run`, and the issues and memories written before then still use that name.

**Why:** On 2026-11-15 the user asked why the sub-command was not named after starting, given that there is a `resume`. No reason for the old name was on record: it came from the plan of issues #7 to #9. I recommended the new name because it pairs with `resume` and because the sub-command's own help text already said it starts a search, and the user agreed.

**How to apply:** Write `cladenc start` in replies, issues, commits and the README. Read `cladenc run` in anything older as the same sub-command, and correct it when an issue is brought up to date, per [[an-issue-is-brought-up-to-date-before-work]]. My own choice, not something the user said: the noun stays, so one search is still a run, kept in the run directory that `--run-dir` names and opened in the journal by the run record. There is no sub-command that stops a search: Ctrl-C or a signal does, and the README says so. Related: [[a-flag-is-not-called-an-option]].
