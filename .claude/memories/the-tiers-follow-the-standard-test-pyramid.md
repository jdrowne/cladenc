---
name: the-tiers-follow-the-standard-test-pyramid
description: "The unit, integration and end-to-end tiers are what the standard test pyramid defines, told apart by three things, the file system, other programs and the network: a unit test uses none of them and gives the code a test double, an integration test calls the library and uses them for real, an end-to-end test starts the program with nothing replaced; the higher the tier, the fewer its tests; alone, the integration tier reaches every function that itself calls the file system or a program, and the unit tier every other function outside src/main.rs"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 61956d23-4c1b-40cc-a69e-f7b691101931
  modified: 2026-11-23T04:00:00.000Z
---

# The tiers follow the standard test pyramid

The three test tiers of `cladenc` are what the standard test pyramid defines, and no line the repository draws for itself replaces those definitions.

## The three things that tell the tiers apart

A tier is told by what a test of it lets the code use for real, out of three things:

- **the file system**: a file, a directory or a symbolic link on disk, with its owner and its mode;
- **another program**: `cladenc` starts `docker`, `sh` and `timeout`;
- **the network**: `cladenc` opens no connection today.

Nothing else counts. The clock, an environment variable and what the program itself reads and prints are not among the three.

A **test double** is what a test puts in the place of one of the three: an object the test makes, held in memory, that the code calls where it would otherwise call the file system or start the program. Test double is the pyramid's own word. A fake, a stub and a mock are kinds of test double, and a mock-up is something else, a model of a design.

## The three tiers

| Tier | What a test calls | The file system, other programs, the network |
| --- | --- | --- |
| Unit | a function of the library | None is used. From its first line to its last the test opens, makes, reads, writes, lists and removes no file and no directory, starts no program and opens no connection. Where the code needs one of the three, the test gives it a test double |
| Integration | a function of the library | Used for real, and the test is about one of them: a directory on the real file system, or a real Docker daemon |
| End-to-end | the `cladenc` program, started as a person starts it | All real. There is no test double: nothing is put in the place of `docker` or of any other program on `PATH` |

The higher the tier, the fewer its tests. A test goes in the lowest tier that can show what it checks, and what a lower tier shows is not tested again in a higher one.

## What each tier reaches by itself

Each function of `src/` is in one of three groups. The group is told by the statements written in the function itself, not by what the functions it calls do:

1. A function of `src/main.rs`. Only the program runs it, so only the end-to-end tier reaches it.
2. A function that itself calls the file system or a program: a call to `std::fs`, a call to a method of a path or of an open file that asks the file system, or a call that starts, waits for or stops a program. A call that opens a connection would put a function here too.
3. Every other function.

The integration tier, taken alone, reaches every line, region and function of the second group. The unit tier, taken alone, reaches every line, region and function of the third group: where a function of the third group calls one of the second, a unit test gives it a test double. The three tiers together reach all of `src/`. Nothing else is asked of a tier alone.

CI holds the unit tier to this with `assert-coverage-of-the-unit-tier`, and the three tiers together to all of `src/` with `assert-coverage`, per [[the-checks-the-code-faces]]. The check that holds the integration tier to the second group is asked for in #157, and until it exists nothing holds that tier alone.

**Why:** The user decided all of this on 2026-11-22:

- The three tiers must follow the definitions of the standard test pyramid. It came up while deciding what a check should hold each tier to, which is #153. Until then the repository drew its own line, at Docker alone: a test that wrote to the disk counted as a unit test so long as it started no `docker`, and the integration tier was whatever needed a container.
- The unit tier, taken alone, reaches every line of the library. That answer came before the rule on the pyramid, by which no unit test can run a function of the second group.
- The integration tier, taken alone, is held to what the unit tier cannot be held to.
- Each term of the rule is defined by something that can be looked up in the code. The user turned down an earlier wording that said the unit tier reaches what a unit test can reach and the integration tier the code that reaches outside the process, since it defined neither.
- The word is test double. The user asked what the session's own word for it, stand-in, meant.

What is mine, and not the user's:

- The user named no source. The three things are read from "The Practical Test Pyramid" on martinfowler.com, which sets out Mike Cohn's pyramid and names databases, the file system and other applications reached over the network, and they are applied to what `cladenc` has.
- Leaving out the clock, the environment and the program's own input and output.
- Telling the second group from the third by the statements of a function itself. It needs no list kept by hand, which the user turned down in #153.

**How to apply:**

- A new test goes in the lowest tier that can show what it checks.
- A unit test uses none of the three: no `std::fs`, no temporary directory, no started program.
- A new function of the second group gets integration tests that reach all of it, though no check asks for it yet, and a new function of the third group gets unit tests that reach all of it.
- A function of the second group holds as little as it can besides its call: what decides and what computes goes in a function of its own, so that the unit tier holds it.
- An end-to-end test puts nothing in the place of `docker`. What `cladenc` does when Docker fails in a chosen way is checked in the unit tier, with a test double.
- Where the code, a workflow or another memory draws the tiers another way, the pyramid's definitions hold, and the other is corrected.

Where each tier's tests are kept is per [[the-layout-follows-rust-conventions]], and what the tiers share is per [[crates-stores-shared-code]].
