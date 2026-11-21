---
name: an-issue-says-what-is-expected-apart-from-what-happens
description: "Where an issue states a problem, what is expected is written as expected and never as though it were so; a plain present tense is kept for what really happens"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 6aa2e45b-78e0-4dfe-9188-c84b1471c3fe
  modified: 2026-11-17T01:57:02.205Z
---

# An issue says what is expected apart from what happens

Where an issue states a problem, it tells what is expected apart from what happens. What is expected is written as expected. A plain present tense is kept for what is really so.

The directory a progenitor is given to read is expected to store the tasks the progenitor has attempted. Where the problem is that it stores none, a sentence saying that the directory stores them states the opposite of the problem.

**Why:** A reader takes a plain present tense for a fact about the code. Where that fact is the very thing that fails, the sentence hides the problem it was written to state, and the issue misleads its reader.

**How to apply:** Write is expected to, or is meant to, for what should be so, and say who or what counts on it, such as the instruction a progenitor is started with. Then say what happens, in the plain present. Before sending an issue, read each present-tense sentence of the problem and ask whether it is true in the failing case; where it is not, it is an expectation and is written as one.

The problem is laid out as two parts, what is expected and then what happens, each under a heading of its own; each failing case says what the harness in it really finds; and the rule reaches a reply or a commit that states a problem. Related: [[an-explanation-says-what-happens]], [[a-procreation-has-a-progenitor-a-copy-and-a-child]], [[issues-have-no-fixed-form]].
