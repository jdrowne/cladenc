---
name: lib-stores-shared-code
description: "Code that more than one consumer uses goes under lib/, as in the user's other repositories; here that means code shared by the test tiers; a package under lib/ has tests of its own, under test/lib/<package>/, that are held to full coverage of it, even one that only tests use"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-18T03:36:47.804Z
---

# lib/ stores shared code

Code that more than one consumer uses lives under `lib/`, the convention of the user's other repositories. In `cladenc` that is the code more than one test tier uses, kept as a package of its own, `lib/test_inputs`, that the `cladenc` package names as a path dev-dependency.

A package under `lib/` has tests of its own, and those tests alone are held to every line, region and function of the package. That nothing but tests uses the package changes nothing. It is the user's rule, and it is how two of their other repositories hold each package under their `lib/`, with the tests under `test/lib/`. The tests of `lib/test_inputs` are in `test/lib/test_inputs/`: a test target named `test_inputs`, which `lib/test_inputs/Cargo.toml` names by its path. The job `assert-coverage-of-test-inputs` runs them and nothing else, and fails unless they reach all of the package, per [[the-checks-the-code-faces]].

**Why:** The user's other repositories keep their shared code there, and the user wants the same here. In those repos `lib/` stores helper packages shared by the test tiers or modules shared by several deployable parts, each tier keeps its own `samples/` under `test/`, and the pipeline's path filter lists `lib/**`.

**How to apply:** Before putting a helper inside one test tier, ask whether another tier needs it; if so it goes under `lib/`. A function added to `lib/test_inputs` has its tests written first, in `test/lib/test_inputs/`, per [[tests-are-written-before-code]]: a tier that calls the function does not stand in for them. A test there tests a function of the package and nothing of `cladenc`. Mine, not the user's: another package under `lib/` gets its tests in `test/lib/<package>/` and a job of its own like that one, named in the `needs` of `release`. Keep `lib/**` in the paths that start `.github/workflows/release.yml`, and hold code under `lib/` to the same checks as `src/` and `test/`. My own reading, not something the user confirmed: the product's library, `src/lib.rs` and its modules, stays in `src/`, because it is the product and not a helper, as `src/<package>/` is in the user's other Python repositories. Related: [[the-paper-is-a-starting-point]].
