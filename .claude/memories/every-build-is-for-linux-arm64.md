---
name: every-build-is-for-linux-arm64
description: "Every build in this repository is for Linux arm64 and no other platform: what CI compiles, tests, packages and releases"
metadata:
  node_type: memory
  type: project
  originSessionId: 7cdb412c-4502-4be0-9040-8f5b755aa863
  modified: 2026-11-15T04:21:57.910Z
---

# Every build is for Linux arm64

Every build in this repository is for Linux arm64, and for no other platform.

**Why:** On 2026-11-14 the user set this as a rule for all of the repository's builds, while `.github/workflows/release.yml` was building, testing, packaging and releasing `cladenc` for Linux x86-64 as well. They gave no reason.

**How to apply:** A CI job that compiles Rust or runs what was compiled runs on `ubuntu-24.04-arm`, and no job has a second platform in a matrix. The released program is `cladenc-linux-arm64`, its package is `cladenc-linux-arm64.deb`, per [[the-only-package-is-a-deb]], and `etc/nfpm/nfpm.yaml` names `arm64` outright. Anything added later that is built, such as a container image for a task, is for Linux arm64 too. My own readings, not things the user said: a job that compiles nothing, such as a linter or the job that wraps the built program into its package, may stay on `ubuntu-latest`; and a build made on the user's Mac to run the tests while working is not one of the repository's builds. Recorded under [[never-quote-the-user-verbatim]].
