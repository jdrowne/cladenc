---
name: an-issue-is-brought-up-to-date-before-work
description: "Before an issue is worked on, its title and body are brought up to date with the code as it stands, without adding work the issue does not ask for"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# An issue is brought up to date before work

Before working on an issue, bring its title and body up to date with the code as it stands.

**Why:** An issue is written ahead of its work. It relies on names, files and behaviour that other issues build, and those can come out differently from what was planned. An issue is read as the current statement of the work, so a stale one is corrected before the work starts instead of being worked around.

**How to apply:** Read the issue, then the code it names and relies on. Where they disagree and the code is right, rewrite the issue with a guarded edit such as `[ -s body.md ] && gh issue edit <number> --body-file body.md`. Where the disagreement is a question only a person can settle, see [[before-an-issue-is-labelled-needs-decision]]. Check at the same time that it documents one problem, per [[one-indivisible-problem-per-issue]].

Bringing an issue up to date does not widen it. An issue asks for what its title asks and nothing more, and a body that plans past the title confuses the user. The body of an issue the user opened says what its title asks, in more detail. An idea that goes past it is never written into that body; the session carries the idea out alongside the issue's work, per [[room-to-be-creative]].
