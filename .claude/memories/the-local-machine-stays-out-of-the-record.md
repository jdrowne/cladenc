---
name: the-local-machine-stays-out-of-the-record
description: "No memory and no issue says what kind of machine the session works on or what running the work does to that machine; say locally for where the session works, as opposed to CI"
metadata:
  node_type: memory
  type: feedback
  modified: 2026-11-22T13:23:02.000Z
---

# The local machine stays out of the record

No memory and no issue says what kind of machine the session works on: not its maker, not its operating system, not whose it is. None says what running the work does to that machine either. Where the place matters, the word is locally: where the session works, as opposed to CI.

**Why:** No reason is recorded. My own reading, not the user's words: the repository is public, and the machine is nobody's business but its owner's.

**How to apply:**

- Say that a thing runs locally, or was seen by hand, and stop there. A fact about Docker names the Docker product and its version, such as Docker Desktop 29.9.3, and not the machine under it.
- Give the reason of a rule about local work as what the work is, such as that compiling is the heavy part of the checks, and never as what the machine did.
- The scope is the memories and the issues, titles and comments among them. The autopilot skill is held to the same words, since each row of "Rules that are memories" says what its memory says.
- My own reading, which the user has not confirmed: a sentence about which platforms `cladenc` runs on is not about the local machine, and names each platform as before, per [[every-build-is-for-linux-arm64]].
- An edit of an issue leaves the earlier text in the edit history of the issue, and a changed title in the timeline of the issue, where anyone can read both. Tell the user so when an edit takes such a mention out.

Related: [[repositories-outside-jdrowne-stay-out-of-the-record]], [[never-quote-the-user-verbatim]], [[only-quick-checks-run-before-a-push]].
