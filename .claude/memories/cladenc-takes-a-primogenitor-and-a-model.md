---
name: cladenc-takes-a-primogenitor-and-a-model
description: "A person brings the source code of the primogenitor, a model, remote or self-hosted, and one or more benchmarks, and cladenc does the self-improvement itself, starting every harness in Docker and calling no model itself"
metadata:
  node_type: memory
  type: project
  originSessionId: a00ea0c7-d6bd-4711-a4cb-3f7fb2827078
  modified: 2026-11-15T15:40:19.165Z
---

# cladenc takes a primogenitor and a model

`cladenc` does recursive self-improvement over coding harnesses, after the HGM paper, and it does all of it itself. A person brings three things to a directed evolution:

- **The primogenitor**, as a directory that stores the harness's source code. The directory stores source code, not a harness; say it that way in replies, issues and commits.
- **A model**, which may be remote or self-hosted. `cladenc` supports both. A self-hosted model needs the address it is served at as well as its name, so the model is not designed as a name alone, or as a hosted provider's alone. Ollama is an example of how a self-hosted model can be served, not a requirement.
- **One or more benchmarks**, each given with the flag `--benchmark`, which names the whole benchmark and not one task. A benchmark is a set of tasks, and a task is one item of it. The person brings each benchmark and `cladenc` fetches none, per [[cladenc-works-offline-and-self-hosted]]. A directed evolution that is given several looks for one harness that does well across all of them, where the paper gives each benchmark a run of its own.

What `cladenc` does with them:

- It starts every harness itself, in a Docker container. `cladenc` knows about Docker, and Docker is a dependency of the `.deb` package.
- How a harness is built and run is given in plain flags: `--diversification-docker-image` for the image the containers that make harnesses start from, `--harness-run-command` for how a harness is run, and a flag for how one is built, which is given only where a harness has to be built.
- The source code of every child is kept, whether or not it builds, all in one folder, and `cladenc`'s own record says which built.
- It hands the model to the harnesses it starts and calls no model itself. Work that calls for `cladenc` to call a model would be an issue of its own.
- A key reaches it in a file, per [[a-key-is-given-in-a-file]].
- No flag has a default, per [[no-flag-has-a-default]].

**Why:** `cladenc` is meant to be handed a harness, a model and benchmarks, and to improve the harness. Having to write a program that carries out part of the self-improvement leaves the main work to the person.

**How to apply:** Treat the three inputs as the intended interface. A program that a person writes to carry out part of a directed evolution, as one that makes a child or one that benchmarks a harness would be, is not part of the design, and `cladenc` has none: do not propose one. Say these words plainly to the user: benchmark first, and task only as one item of a benchmark; directed evolution, which is one run of `cladenc` from start to end; and session, which is one conversation with me.

Related: [[cladenc-works-offline-and-self-hosted]], [[the-simplest-design-is-offered-first]], [[the-paper-is-a-starting-point]], [[one-indivisible-problem-per-issue]], [[before-an-issue-is-labelled-needs-decision]].
