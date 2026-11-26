---
name: cladenc-is-run-by-a-session-the-user-or-a-program
description: "Whoever runs cladenc can be a session of Claude, the user, or another program; never say that the user is the one who runs it or who gave a flag its value"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 1aed5bc3-ee66-48b9-a0d7-7bd822cb7b45
  modified: 2026-11-26T01:19:16.298Z
---

# cladenc is run by a session, the user or a program

`cladenc` is run by a session of Claude, by the user, or by another program. None of the three is assumed. The user is not said to be the one who runs it, and the value of a flag is not said to be something the user typed.

**Why:** On 2026-11-25 an explanation of `--harness-run-command` called the flag's value the line of a person, without saying who that was, and then said that in practice the user gives it. Both were wrong and left the explanation unclear: the user does not run `cladenc` as a rule, and in the local check of issue 95 a session wrote the run command and started `cladenc`.

**How to apply:**

- In an explanation, name who does each thing: `cladenc`, the session, the user, or the program that runs `cladenc`. Where it can be any of them, say whoever runs `cladenc`.
- Do not address whoever runs `cladenc` as you.
- Before saying who gave a flag its value in a directed evolution that was carried out, read who did, per [[what-the-code-answers-is-checked-before-it-is-raised]].

Related: [[an-explanation-names-the-literal-thing]], [[cladenc-takes-a-primogenitor-and-a-model]].
