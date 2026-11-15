---
name: an-issue-is-brought-up-to-date-before-work
description: "Before an issue is worked on, its title and body are brought up to date with the code as it stands, without adding work the issue does not ask for"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T00:08:11.608Z
---

# An issue is brought up to date before work

Before working on an issue, bring its title and body up to date with the code as it stands.

**Why:** An issue is written ahead of its work, and the ten that build `cladenc` were written all at once, before any code existed. Each relies on names, files and behaviour that the issues before it build, and those can come out differently from what was planned. An issue is read as the current statement of the work, so a stale one is corrected before the work starts instead of being worked around. This was a reminder of the autopilot's and moved here on 2026-11-14, per [[reminders-drive-the-loop]].

**How to apply:** Read the issue, then the code it names and relies on. Where they disagree and the code is right, rewrite the issue with a guarded edit such as `[ -s body.md ] && gh issue edit <number> --body-file body.md`. Where the disagreement is a question only a person can settle, see [[before-an-issue-is-labelled-needs-decision]]. Check at the same time that it documents one problem, per [[one-indivisible-problem-per-issue]].

Bringing an issue up to date does not widen it. On 2026-11-15 the user opened issue #38 with a title asking for the sub-commands and flags of `cladenc` to be documented in `README.md`, and no body. I wrote it a body that also planned how a harness is started and built, how the evaluator is called, the run directory, exit statuses and installing, and wrote that README. The user found this confusing and pointed out that the issue asks for the commands and flags and nothing more. I cut the body and the README back before anything was committed. My own reading, not something the user said: the body of an issue the user opened says what its title asks, in more detail, and an idea that goes past it is offered to the user or filed as an issue of its own, per [[room-to-be-creative]], not folded into theirs. Recorded under [[never-quote-the-user-verbatim]].
