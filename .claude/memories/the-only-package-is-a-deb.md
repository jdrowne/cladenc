---
name: the-only-package-is-a-deb
description: "cladenc is packaged as a .deb alone: no RPM is made, tested or released, and nothing in CI runs on Fedora"
metadata:
  node_type: memory
  type: project
  originSessionId: 7cdb412c-4502-4be0-9040-8f5b755aa863
  modified: 2026-11-21T19:44:58.792Z
---

# The only package is a .deb

`cladenc` is packaged as a `.deb` alone. No RPM is made, tested or released, and nothing in CI runs on Fedora.

**Why:** Fedora is not needed, and `.deb` packages are enough.

**How to apply:** `package` in `.github/workflows/release.yml` makes `cladenc-linux-arm64.deb` with nfpm, and `test-package` installs it with `apt-get install` on a clean `debian:stable-slim`, the image every container of CI starts from, and runs `cladenc --version`, `docker --version` and `timeout --version`. Do not add another package format, or a system to test one on, unless the user asks. The platform is set by [[every-build-is-for-linux-arm64]].
