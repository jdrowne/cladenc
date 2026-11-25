---
name: paths-follow-the-linux-filesystem-convention
description: "Every path cladenc uses on a Linux system, in a container or on the machine, follows the Filesystem Hierarchy Standard, which counts cladenc as an add-on package: the program at /opt/cladenc/bin/cladenc, the manual pages under /opt/cladenc/share/man, what a container works with under /var/opt/cladenc, records of what happened under /var/log/cladenc, no directory at the top of the filesystem, where the names the Harbor format fixes are symbolic links, /logs to /var/log/cladenc/harbor and /tests to /var/opt/cladenc/harbor/tests, and nothing under /usr or /var/lib"
metadata:
  node_type: memory
  type: feedback
  originSessionId: ef2131eb-746a-4fc0-939f-1dbeaa241ced
  modified: 2026-11-22T15:16:22.000Z
---

# Paths follow the Linux filesystem convention

Every fixed path `cladenc` uses on a Linux system follows the Filesystem Hierarchy Standard 3.0, in a container as much as on the machine `cladenc` runs on. A path inside a container is not exempt because containers commonly ignore the standard.

In the words of the standard, `cladenc` is an add-on application software package: no distribution includes it. Its places are the ones the standard gives such a package, `/opt`, `/var/opt` and `/etc/opt` (section 3.13), and not the ones the standard keeps for the software of a distribution.

- On the machine, the program is `/opt/cladenc/bin/cladenc`, where the standard puts a program that a person calls (section 3.13). The package puts no file outside `/opt/cladenc`: no link in `/usr/bin`, and none in `/opt/bin`, which the standard keeps for the administrator. A person calls the program by its full path or adds `/opt/cladenc/bin` to `PATH`.
- The manual pages are under `/opt/cladenc/share/man`, in the same layout as `/usr/share/man`, where the standard puts the manual pages of an add-on package (section 3.13). `man` finds them where `/opt/cladenc/bin` is on `PATH`, and is given the directory with `-M` otherwise. In the repository they are under `docs/man/man1`.
- What `cladenc` gives a container to work with is under `/var/opt/cladenc`, the place for the files of an add-on package that change (sections 3.13 and 5.12): the source code of the harness at `/var/opt/cladenc/harness`, and the directory a harness changes at `/var/opt/cladenc/work`.
- A record of what happened, kept to be read afterwards, is under `/var/log/cladenc`, as what a harness's attempts left is, at `/var/log/cladenc/attempts`. Section 5.10 gives logs their place whatever program writes them, and section 3.13 lets an add-on package keep a file where another section of the standard fixes its place. A record is never under `/var/opt/cladenc`.
- Nothing is under `/usr` or under `/var/lib/cladenc`: section 4.9 keeps software that is installed locally out of `/usr`, and section 5.8 keeps a directory under `/var/lib` for an application that the distribution includes.
- `cladenc` reads no configuration file. One that it came to read would be under `/etc/opt/cladenc` (section 3.7.4).
- Nothing `cladenc` makes is a directory at the top of the filesystem: the standard has an application create no subdirectory in the root directory (section 3.1).
- A name that the Harbor format fixes at the top of the filesystem is a symbolic link that `cladenc` makes, to a directory named `harbor` in the place the standard gives the files behind the name. `/logs` is a link to `/var/log/cladenc/harbor`, which holds `artifacts` for the harness and, once the harness has ended, `verifier` for the verifier. `/tests` is a link to `/var/opt/cladenc/harbor/tests`, which holds the tests of the task for the verifier: tests are worked with and are no record, so they are not under `/var/log/cladenc`. The directory named `harbor` keeps what the Harbor format names apart from what `cladenc` names itself, such as `harness` and `work`.
- A link is no subdirectory, and it answers the two reasons section 3.1 gives: it takes no space on the partition of the root directory, and the files are under `/var`, inside the plan of the administrator for which directory is on which disk. A link named `/logs/artifacts` would need a real directory `/logs` to be in, so the link is `/logs` itself.
- Everything a directed evolution makes is in the outputs directory, which the person names with `--outputs-dir`, per [[the-outputs-directory]].

**Why:** A person who knows the standard finds each of `cladenc`'s files where the standard puts it, and a record where anyone looks for a log, under `/var/log`. The directory under `/var/log/cladenc` is named `attempts` for the word attempt, per [[the-words-about-benchmarks]].

**How to apply:** Before `cladenc` gives a container a new directory, or the package installs a new file, find its place in the standard as a file of an add-on package and put it there, in a directory named after `cladenc`. Read the section of the standard itself before naming a place, at <https://www.debian.org/doc/packaging-manuals/fhs/fhs-3.0.html>: a sentence that keeps the files of a package in one hierarchy can carry an exception, which another section fills, as section 5.10 does for logs. Never propose a directory at the top of the filesystem, or a path under `/usr` or `/var/lib`, and do not defend one by what is usual in containers or in other packages. Where the format of a benchmark fixes a name at the top of the filesystem, propose a symbolic link of that name to a directory named after the format, under `/var/log/cladenc` for a record and under `/var/opt/cladenc` for anything else. Related: [[the-only-package-is-a-deb]], [[the-simplest-design-is-offered-first]].
