---
name: crates-stores-shared-code
description: "Code that more than one consumer uses is a package under crates/; here that means code shared by the test tiers; a package under crates/ has tests of its own, in its own tests/ directory, that are held to full coverage of it, even one that only tests use"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-18T03:36:47.804Z
---

# crates/ stores shared code

Code that more than one consumer uses is a package of its own under `crates/`. In `cladenc` that is the code more than one test tier uses, `crates/test_inputs`, which the `cladenc` package names as a path dev-dependency. The unit tests inside `src/` and the two test programs under `tests/` are three separate programs, so what they share has to be a package.

A package under `crates/` has tests of its own, and those tests alone are held to every line, region and function of the package. That nothing but tests uses the package changes nothing. The tests of `crates/test_inputs` are in `crates/test_inputs/tests/`, each file a test program of its own that Cargo finds by itself. The job `assert-coverage-of-test-inputs` runs them and nothing else, and fails unless they reach all of the package, per [[the-checks-the-code-faces]].

**Why:** Shared code is kept in one place, with tests of its own, under the name a Rust reader looks for, per [[the-layout-follows-rust-conventions]].

**How to apply:** Before putting a helper inside one test tier, ask whether another tier needs it; if so it goes in a package under `crates/`. A function added to `crates/test_inputs` has its tests written first, in `crates/test_inputs/tests/`, per [[tests-are-written-before-code]]: a tier that calls the function does not stand in for them. A test there tests a function of the package and nothing of `cladenc`. Keep `crates/**` in the paths that start `.github/workflows/release.yml`, and hold code under `crates/` to the same checks as `src/` and `tests/`.

- The tests of the package stay outside its library, in its `tests/` directory, and none is written inside `crates/test_inputs/src/`. They use only what the package makes public, and with no test inside the library Cargo builds the package once, so the percentages of the coverage tool hold as they are printed.
- Another package under `crates/` gets its tests in its own `tests/` directory and a job of its own like `assert-coverage-of-test-inputs`, per [[the-checks-the-code-faces]].
- The product's library, `src/lib.rs` and its modules, stays in `src/`, because it is the product and not a helper.

Related: [[the-paper-is-a-starting-point]].
