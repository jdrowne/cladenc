---
name: what-the-code-answers-is-checked-before-it-is-raised
description: a question the repository can answer is answered by reading the repository before it is mentioned to the user; never told to the user as something not yet checked
metadata:
  node_type: memory
  type: feedback
  originSessionId: df0bf027-c554-4f41-8bfe-5eb116dd45bf
  modified: 2026-11-25T03:39:35.307Z
---

# What the code answers is checked before it is raised

A question that reading the repository can answer is checked before it is mentioned to the user, and the reply gives the result. It is not put to the user as an open question that has not been looked at.

On 2026-11-24 the session told the user that it had not checked whether the tests that read one directory on purpose can interfere with each other, and the user asked why it had not. The session then read every such test and reported the result. That the user wants the check made first is the session's reading of that question, not something the user stated as a rule.

**Why:** an unchecked question handed to the user is work the session could have done, and it leaves the user unsure whether there is a problem.

**How to apply:** when a neighbouring question comes up while answering, read the code that settles it, then say what was found and where. Related: [[before-an-issue-is-labelled-needs-decision]], [[an-explanation-names-the-literal-thing]].
