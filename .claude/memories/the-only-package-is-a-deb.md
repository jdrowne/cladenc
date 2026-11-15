---
name: the-only-package-is-a-deb
description: "cladenc is packaged as a .deb alone: no RPM is made, tested or released, and nothing in CI runs on Fedora"
metadata:
  node_type: memory
  type: project
  originSessionId: 7cdb412c-4502-4be0-9040-8f5b755aa863
  modified: 2026-11-15T04:21:54.656Z
---

# The only package is a .deb

`cladenc` is packaged as a `.deb` alone. No RPM is made, tested or released, and nothing in CI runs on Fedora.

**Why:** On 2026-11-14 the user settled that Fedora is not needed and that `.deb` packages are enough. The workflow had been making an RPM beside the `.deb` and installing it on `fedora:latest`. My own reading of what prompted it, not something the user said: the release run of d7a4687 had failed there, since rpm on Fedora installs only a package whose signature it has verified and the packages are not signed, and I had just pushed a fix that passed `--nosignature`.

**How to apply:** `package` in `.github/workflows/release.yml` makes `cladenc-linux-arm64.deb` with nfpm, and `test-package` installs it with `dpkg --install` on a clean `ubuntu:latest` and runs `cladenc --version`. Do not add another package format, or a system to test one on, unless the user asks. The platform is set by [[every-build-is-for-linux-arm64]]. Recorded under [[never-quote-the-user-verbatim]].
