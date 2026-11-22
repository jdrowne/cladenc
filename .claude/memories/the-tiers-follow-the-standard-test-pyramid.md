---
name: the-tiers-follow-the-standard-test-pyramid
description: "The unit, integration and end-to-end tiers are what the standard test pyramid defines: a unit test replaces what is outside the process with a stand-in, an integration test sets the code against a real outside thing, an end-to-end test runs the whole program with nothing stood in for; the higher the tier, the fewer its tests; alone, the unit tier reaches all a unit test can and the integration tier all the code that reaches outside the process"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 61956d23-4c1b-40cc-a69e-f7b691101931
  modified: 2026-11-22T22:57:02.220Z
---

# The tiers follow the standard test pyramid

The three test tiers of `cladenc` are what the standard test pyramid defines, and no line the repository draws for itself replaces those definitions:

- **Unit.** One piece of code, called directly. Whatever is outside the process is replaced with a stand-in: the disk, the network, another program such as `docker`.
- **Integration.** The code set against a real thing outside the process, one such thing at a time: the real disk, a real Docker. The code is called below the command line.
- **End-to-end.** The whole `cladenc` program, run as a person runs it and completely integrated: nothing stands in for anything.

The pyramid sets the proportions too. The higher the tier, the fewer its tests, and a test goes as far down as it can: what a lower tier can show is not tested again in a higher one.

**Why:** The user decided on 2026-11-22 that these three tiers must follow the definitions of the standard test pyramid. It came up while deciding what a check should hold each tier to, which is #153. Until then the repository drew its own line, at Docker alone: a test that wrote to the disk counted as a unit test so long as it started no `docker`, and the integration tier was whatever needed a container. The user named no source. The wording above is mine, read from "The Practical Test Pyramid" on martinfowler.com, which sets out Mike Cohn's pyramid, and it is what the user was shown before deciding.

**What each tier reaches by itself.** The user decided this on 2026-11-22 as well, in two answers:

- The unit tier, taken alone, reaches every line of `src/` that a unit test can reach. The user's answer was that the unit tier alone reaches every line of the library, given before the rule on the pyramid. Narrowing it to what a unit test can reach is my reading: all of `src/` but `src/main.rs`, which only the program runs, and the code a stand-in takes the place of, which is the code that itself reads or writes the disk or starts a program. The user was told of the reading and answered the next question on it.
- The integration tier, taken alone, reaches all of the code that reaches outside the process, which is the part the unit tier leaves out, but `src/main.rs`.
- No tier reaches all of `src/` by itself, and nothing is asked of the end-to-end tier alone. The three tiers together reach all of `src/`.

The checks that hold the tiers to this are asked for in #153 and #157. Until they exist, CI holds only the three tiers together to all of `src/`, per [[the-checks-the-code-faces]].

**How to apply:**

- A new test goes in the lowest tier that can show what it checks.
- New code that reaches outside the process is reached in full by integration tests, and all other new code in full by unit tests, whether or not a check asks for it yet.
- A unit test reads and writes no file, opens no connection and starts no program. Where the code under test needs one of these, the test gives it a stand-in.
- A test that needs the real disk or a real Docker is an integration test, and one that needs the program run is an end-to-end test.
- An end-to-end test puts nothing in the place of Docker. What `cladenc` does when Docker fails in a chosen way is checked in the unit tier, with a stand-in.
- Where the code, a workflow or another memory draws the tiers another way, the pyramid's definitions hold, and the other is corrected. [[the-checks-the-code-faces]] says what CI checks of a unit test today, which is less than this: only that it starts no `docker`.

Where each tier's tests are kept is per [[the-layout-follows-rust-conventions]], and what the tiers share is per [[crates-stores-shared-code]].
