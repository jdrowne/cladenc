---
name: cladenc-works-offline-and-self-hosted
description: "The user's rule: cladenc has to work with no connection to the internet, with everything a directed evolution uses hosted by the person who runs it, so cladenc fetches nothing and the person brings each thing a directed evolution needs, the whole benchmark among them"
metadata:
  node_type: memory
  type: project
  originSessionId: 97022597-fda2-406a-9e2f-9509cdef782d
  modified: 2026-11-15T15:47:48.287Z
---

# cladenc works offline and self-hosted

`cladenc` has to work on a machine with no connection to the internet, with everything a directed evolution uses hosted by the person who runs it.

**Why:** It is the user's rule. The user has given no reason for it.

**How to apply:**

- Never propose that `cladenc` fetches, downloads or looks up a thing: not a benchmark, not an image, not a model, not a harness. The person brings it, as a directory or a file on the machine that a flag names.
- Do not put a design that needs the internet to the user as one of the ways to decide between. The rule has decided against it, and offering it costs the user a reply.
- A benchmark is brought by the person, and `--benchmark` names the whole of it, every task it holds, never one task.
- When a benchmark is shown by example, show several tasks in it. An example that holds one task reads as though the flag named a task.

Mine, not the user's: that a model served somewhere else stays something a person may give, since the rule says what `cladenc` has to be able to do and not what a person may point it at; that a benchmark is a directory which holds a directory for each task; and that the rule reaches Docker, so every `docker run` is told `--pull never`, the image of a directed evolution is on the machine before the directed evolution, and `cladenc` has no flag for a registry, since the name of an image says which registry it is of. `docker build` has no flag that forbids fetching a base image the machine does not hold, and the README says so.

Related: [[cladenc-takes-a-progenitor-and-a-model]], [[the-simplest-design-is-offered-first]], [[no-flag-has-a-default]].
