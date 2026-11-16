---
name: cladenc-takes-a-progenitor-and-a-model
description: "The user's design: a person brings the source code of the progenitor, a model, remote or self-hosted, and one or more benchmarks, and cladenc does the self-improvement itself, starting every harness in Docker and calling no model itself"
metadata:
  node_type: memory
  type: project
  originSessionId: a00ea0c7-d6bd-4711-a4cb-3f7fb2827078
  modified: 2026-11-15T15:40:19.165Z
---

# cladenc takes a progenitor and a model

The user's design for `cladenc`: it does recursive self-improvement over coding harnesses, after the HGM paper, and it does all of it itself. A person brings three things to a search:

- **The progenitor**, as a directory that holds the harness's source code. The directory holds source code, not a harness; say it that way in replies, issues and commits.
- **A model**, which may be remote or self-hosted. `cladenc` supports both. A self-hosted model needs the address it is served at as well as its name, so the model is not designed as a name alone, or as a hosted provider's alone. Ollama is an example of how a self-hosted model can be served, not a requirement.
- **One or more benchmarks**, each given with the flag `--benchmark`, which names the whole benchmark and not one task. A benchmark is a set of tasks, and a task is one item of it. The person brings each benchmark and `cladenc` fetches none, per [[cladenc-works-offline-and-self-hosted]]. A search that is given several looks for one harness that does well across all of them, where the paper runs one search to a benchmark.

What `cladenc` does with them. Each of these is the user's decision:

- It starts every harness itself, in a Docker container. `cladenc` knows about Docker, and Docker is a dependency of the `.deb` package.
- How a harness is built and run is given in plain flags: `--docker-image` for the image a container starts from, `--harness-run-command` for how a harness is run, and a flag for how one is built, which is given only where a harness has to be built.
- The source code of every child is kept, whether or not it builds, all in one folder, and `cladenc`'s own record says which built.
- It hands the model to the harnesses it starts and calls no model itself. Work that calls for `cladenc` to call a model would be an issue of its own.
- A key reaches it in a file, per [[a-key-is-given-in-a-file]].
- No flag has a default, per [[no-flag-has-a-default]]. The three inputs are what a person brings to a search, not a count of flags.

**Why:** The user wants a tool that is handed a harness, a model and benchmarks, and improves the harness. Having to write a program that carries out part of the self-improvement leaves the main work to the person.

**How to apply:** Treat the three inputs as the intended interface. A program that a person writes to carry out part of a search, as one that makes a child or one that assesses a harness would be, is not part of the design, and `cladenc` has none: do not propose one. Say these words plainly to the user: benchmark first, and task only as one item of a benchmark; search, which is one run of `cladenc` from start to end; and session, which is one conversation with me.

Mine, not the user's, and open to the session that does the work: the names `--harness-build-command` and `--model-key-file`; the variables `CLADENC_MODEL`, `CLADENC_MODEL_URL` and `CLADENC_MODEL_KEY` that a container is given; that every container can reach a model served on the machine `cladenc` runs on; that a container's variables reach Docker through the environment of the `docker` command and not on its command line; that `cladenc report` shows the model's name and address; that the source code sits on the machine and is mounted into the container; that the source code of a child whose making fails in any other way is kept too; that a program a person had to write is removed and not kept as a second way to carry out a job; that an assessment starts the harness on a copy of the task's files, with the statement as its instruction, and then runs the task's check in a second container of the same image; that a harness which ends with a status other than 0 is still checked; that the tasks of several benchmarks count alike, so a larger benchmark weighs more; that a search has one image, which has to suit every benchmark; that a benchmark is a directory which holds a directory for each task; that the name of a benchmark is the name of its directory; and that a task of a search is known by the two together, as `<benchmark>/<task>`, in the journal, in what `cladenc` says and in the directories of the run directory.

Related: [[cladenc-works-offline-and-self-hosted]], [[the-simplest-design-is-offered-first]], [[the-paper-is-a-starting-point]], [[one-indivisible-problem-per-issue]], [[before-an-issue-is-labelled-needs-decision]].
