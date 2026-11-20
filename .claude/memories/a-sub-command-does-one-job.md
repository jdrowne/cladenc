---
name: a-sub-command-does-one-job
description: "the user holds the command line of cladenc to the Unix principle; a sub-command does one job on one directed evolution, and what goes across several directed evolutions is left to another program that reads `--json`"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 49f37d44-126e-4710-90c8-e4226a477263
  modified: 2026-11-20T03:58:05.872Z
---

# A sub-command does one job

The user holds the command line of `cladenc` to the Unix principle: each program does one thing, and programs work together through text. A design for a sub-command is tested against the principle before it is offered.

- A sub-command reads one directed evolution, which has one outputs directory. No sub-command takes several outputs directories, and none compares directed evolutions. A person who wants a comparison calls the sub-command with `--json` once for each directed evolution, and another program, such as `jq`, does the comparing.
- Where one sub-command would serve two kinds of case, two sub-commands are offered, each for one kind: `cladenc status` for a directed evolution that has not ended, and `cladenc report` for a directed evolution that has ended. Each refuses the other kind and names the sub-command to call.
- A sub-command prints one form. A flag or a count of arguments that makes a sub-command print another kind of thing gives the sub-command a second job.

**Why:** It is the user's rule. My reading of the reason, not the user's statement: a sub-command with two jobs has to be explained twice, and the user cannot tell from its name what it will print.

**How to apply:** Before proposing a flag, a mode or a repeated argument, ask whether it gives the sub-command a second job or a second form of output. Where another program can do the second job from what the sub-command already prints, the proposal is not made. Where the second job belongs in `cladenc`, a sub-command of its own is offered. Related: [[the-simplest-design-is-offered-first]], [[no-flag-for-what-cladenc-already-holds]], [[a-flag-is-not-called-an-option]].
