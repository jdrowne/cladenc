---
name: cladenc-runs-any-dataset
description: "cladenc runs a dataset in the form it is published in, such as SWE-bench or Polyglot, and a person brings a free harness such as OpenCode; what a Harbor task's own configuration says is obeyed and no flag overrides it; the repository writes no harness, no dataset and no converter of its own, and example/ holds a Dockerfile and nothing else"
metadata:
  node_type: memory
  type: project
  originSessionId: a2ae831f-272a-4a47-b46b-d83e0c3cb77c
  modified: 2026-11-21T12:50:18.285Z
---

# cladenc runs any dataset

`cladenc` is expected to run a dataset as the dataset is published, with nothing written for that one dataset. A person brings a harness and datasets that already exist as free software: OpenCode as a harness, SWE-bench and Polyglot as datasets, for example.

**Why:** Free harnesses and datasets exist, so a harness or a dataset written for `cladenc` alone is work nobody needs, and a converter written for one dataset means `cladenc` does not in fact run any dataset.

**How to apply:**

- Do not propose a harness, a dataset, or a script that converts one dataset into `cladenc`'s own task layout, under `example/` or anywhere else in the repository.
- `example/` holds a Dockerfile and nothing else.
- Where a published dataset does not run in `cladenc`, the fault is in `cladenc`: the issue is written against how `cladenc` reads a dataset, not as a converter.
- Every Harbor task carries its own configuration file, and `cladenc` obeys all of it: time limits, resources, network, environment, verifier. No flag overrides a setting the configuration holds, and a flag that does is removed. Where a task leaves a setting out, the setting means what Harbor's documentation says it means when left out. A task without the configuration file is refused, as Harbor refuses it. A setting `cladenc` has no way to obey makes the task refused, never run otherwise than as published. The reason is that the task's author chose those values, and a value the task holds is one `cladenc` reads itself, per [[no-flag-for-what-cladenc-already-holds]].
- The person still brings each dataset as files on the machine, per [[cladenc-works-offline-and-self-hosted]].

Related: [[cladenc-takes-a-primogenitor-and-a-model]], [[the-simplest-design-is-offered-first]].
