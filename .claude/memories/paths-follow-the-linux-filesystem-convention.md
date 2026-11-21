---
name: paths-follow-the-linux-filesystem-convention
description: "Every path cladenc uses on a Linux system, in a container or on the machine, follows the Filesystem Hierarchy Standard: nothing at the top of the filesystem, what a container works with under /var/lib/cladenc, records of what happened under /var/log/cladenc"
metadata:
  node_type: memory
  type: feedback
  originSessionId: ef2131eb-746a-4fc0-939f-1dbeaa241ced
  modified: 2026-11-16T12:35:34.158Z
---

# Paths follow the Linux filesystem convention

Every fixed path `cladenc` uses on a Linux system follows the Filesystem Hierarchy Standard 3.0, in a container as much as on the machine `cladenc` runs on. A path inside a container is not exempt because containers commonly ignore the standard.

- Nothing `cladenc` needs sits at the top of the filesystem: the standard gives no program a directory of its own in the root directory (section 3.1).
- What `cladenc` gives a container to work with is under `/var/lib/cladenc`, the place the standard gives a program for its data (section 5.8).
- A record of what happened, kept to be read afterwards, is under `/var/log/cladenc` (section 5.10), as what a harness's attempts left is, at `/var/log/cladenc/attempts`.
- On the machine, the program is `/usr/bin/cladenc`, and everything a directed evolution makes is in the outputs directory, which the person names with `--outputs-dir`, per [[the-outputs-directory]].

**Why:** A person who knows the standard finds each of `cladenc`'s files where the standard puts it. The directory under `/var/log/cladenc` is named `attempts` for the word attempt, per [[the-words-about-benchmarks]].

**How to apply:** Before `cladenc` gives a container a new directory, or the package installs a new file, find its place in the standard and put it there, in a directory named after `cladenc`. Never propose a path at the top of the filesystem, and do not defend one by what is usual in containers. The standard is readable at <https://www.debian.org/doc/packaging-manuals/fhs/fhs-3.0.html>. Related: [[the-only-package-is-a-deb]], [[the-simplest-design-is-offered-first]].
