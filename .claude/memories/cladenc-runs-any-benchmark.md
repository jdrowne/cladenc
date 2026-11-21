---
name: cladenc-runs-any-benchmark
description: "The user's rule: cladenc runs a benchmark in the form it is published in, such as SWE-bench or Polyglot, and a person brings a free harness such as OpenCode; the repository writes no harness, no benchmark and no converter of its own, and example/ holds a Dockerfile and nothing else"
metadata:
  node_type: memory
  type: project
  originSessionId: a2ae831f-272a-4a47-b46b-d83e0c3cb77c
  modified: 2026-11-21T12:50:18.285Z
---

# cladenc runs any benchmark

`cladenc` is expected to run a benchmark as the benchmark is published, with nothing written for that one benchmark. A person brings a harness and benchmarks that already exist as free software: OpenCode as a harness, SWE-bench and Polyglot as benchmarks, for example.

**Why:** It is the user's rule. Free harnesses and benchmarks exist, so a harness or a benchmark written for `cladenc` alone is work nobody needs, and a converter written for one benchmark means `cladenc` does not in fact run any benchmark.

**How to apply:**

- Do not propose a harness, a benchmark, or a script that converts one benchmark into `cladenc`'s own task layout, under `example/` or anywhere else in the repository.
- `example/` holds a Dockerfile and nothing else.
- Where a published benchmark does not run in `cladenc`, the fault is in `cladenc`: the issue is written against how `cladenc` reads a benchmark, not as a converter.
- The person still brings each benchmark as files on the machine, per [[cladenc-works-offline-and-self-hosted]].

Related: [[cladenc-takes-a-primogenitor-and-a-model]], [[the-simplest-design-is-offered-first]].
