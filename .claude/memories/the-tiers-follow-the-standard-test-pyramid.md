---
name: the-tiers-follow-the-standard-test-pyramid
description: "The unit, integration and end-to-end tiers are what the standard test pyramid defines: a unit test replaces what is outside the process with a stand-in, an integration test sets the code against a real outside thing, an end-to-end test runs the whole program with nothing stood in for; the higher the tier, the fewer its tests"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 61956d23-4c1b-40cc-a69e-f7b691101931
  modified: 2026-11-22T22:45:12.341Z
---

# The tiers follow the standard test pyramid

The three test tiers of `cladenc` are what the standard test pyramid defines, and no line the repository draws for itself replaces those definitions:

- **Unit.** One piece of code, called directly. Whatever is outside the process is replaced with a stand-in: the disk, the network, another program such as `docker`.
- **Integration.** The code set against a real thing outside the process, one such thing at a time: the real disk, a real Docker. The code is called below the command line.
- **End-to-end.** The whole `cladenc` program, run as a person runs it and completely integrated: nothing stands in for anything.

The pyramid sets the proportions too. The higher the tier, the fewer its tests, and a test goes as far down as it can: what a lower tier can show is not tested again in a higher one.

**Why:** The user decided on 2026-11-22 that these three tiers must follow the definitions of the standard test pyramid. It came up while deciding what a check should hold each tier to, which is #153. Until then the repository drew its own line, at Docker alone: a test that wrote to the disk counted as a unit test so long as it started no `docker`, and the integration tier was whatever needed a container. The user named no source. The wording above is mine, read from "The Practical Test Pyramid" on martinfowler.com, which sets out Mike Cohn's pyramid, and it is what the user was shown before deciding.

**How to apply:**

- A new test goes in the lowest tier that can show what it checks.
- A unit test reads and writes no file, opens no connection and starts no program. Where the code under test needs one of these, the test gives it a stand-in.
- A test that needs the real disk or a real Docker is an integration test, and one that needs the program run is an end-to-end test.
- An end-to-end test puts nothing in the place of Docker. What `cladenc` does when Docker fails in a chosen way is checked in the unit tier, with a stand-in.
- Where the code, a workflow or another memory draws the tiers another way, the pyramid's definitions hold, and the other is corrected. [[the-checks-the-code-faces]] says what CI checks of a unit test today, which is less than this: only that it starts no `docker`.

Where each tier's tests are kept is per [[the-layout-follows-rust-conventions]], and what the tiers share is per [[crates-stores-shared-code]].
