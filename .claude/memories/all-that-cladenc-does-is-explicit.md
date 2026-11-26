---
name: all-that-cladenc-does-is-explicit
description: "Everything cladenc does is explicit: what it hands to a program it starts is a flag with its value on that program's command line, never an environment variable that cladenc sets"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 1aed5bc3-ee66-48b9-a0d7-7bd822cb7b45
  modified: 2026-11-26T03:55:37.000Z
---

# All that cladenc does is explicit

Everything `cladenc` does is explicit. What `cladenc` hands to a program it starts, a harness above all, is a flag with its value on the command line of that program. Nothing depends on an environment variable that `cladenc` sets, since nothing on the command shows where such a variable comes from.

**Why:** On 2026-11-25 `cladenc` handed a harness its instruction, the place of its source code, the model's name, the model's URL and the model's key in five environment variables that it set itself, and the command that starts a harness referred to them by name. To whoever reads that command, the variables come from nowhere. A long explanation of how they travel did not make the design acceptable: the rule is simpler than any such explanation.

**How to apply:**

- Never propose an environment variable as the way `cladenc` hands a value to a sandbox, a harness or any other program. Propose a flag with its value, per [[the-simplest-design-is-offered-first]].
- A secret is still kept off every command line: it goes in a file, and a flag names the file, per [[a-key-is-given-in-a-file]].
- Where the code depends on something that is not written on a command line or in a file that a flag names, that is a problem to solve, not a design to defend. Say what it is and how it becomes explicit, in a few lines.
- The variables that a task's own `task.toml` names are the task's and are obeyed, per [[cladenc-runs-any-benchmark]]: they are written in a file of the benchmark.

Related: [[no-flag-has-a-default]], [[an-explanation-names-the-literal-thing]], [[cladenc-is-run-by-a-session-the-user-or-a-program]].
