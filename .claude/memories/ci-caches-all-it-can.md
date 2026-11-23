---
name: ci-caches-all-it-can
description: "Every job of the workflows takes from the GitHub Actions cache whatever it would otherwise fetch or compile again: Rust builds, pip and npm downloads, the Docker images and what apt fetches; a job added later is cached the same way"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c0ccaee5-b418-4970-9ebc-74bd1acf04de
  modified: 2026-11-21T19:54:11.224Z
---

# CI caches all it can

Each job in `.github/workflows/` restores from the GitHub Actions cache whatever it would otherwise fetch or compile anew, and a job added later is given the same caching in the commit that adds it.

**Why:** GitHub starts each job on a fresh machine, so a job with no cache downloads and compiles the same things again on every push.

**How to apply:**

- A job that compiles Rust runs `Swatinem/rust-cache` right after the toolchain step.
- A job that installs with pip or npm restores `~/.cache/pip` or `~/.npm`, keyed on the job's name and the hash of its workflow file, with the job's name alone as the fallback key.
- Every Docker image a job of `release.yml` runs is listed in `IMAGES` at the top of the file. `cache-images` saves them to the cache under a key made from their digests, and a job that runs one waits for `cache-images` and loads `images.tar` instead of pulling.
- Key a cache on what it holds, such as a digest or a file's hash, rather than on a commit, so a run that changes nothing reuses it. GitHub keeps 10 GB of caches for the repository and evicts the least recently used past that.
- A job that waits for no other job and is not a build is named in the `needs` of `release`, `cache-images` among them.
