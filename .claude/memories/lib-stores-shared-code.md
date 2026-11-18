---
name: lib-stores-shared-code
description: "Code that more than one consumer uses goes under lib/, as in the user's other repositories; here that means code shared by the test tiers; a package under lib/ has tests of its own that are held to full coverage of it, even one that only tests use"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-18T03:36:47.804Z
---

# lib/ stores shared code

Code that more than one consumer uses lives under `lib/`, the convention of the user's other repositories. In `cladenc` that is the code more than one test tier uses, kept as a package of its own, `lib/test_inputs`, that the `cladenc` package names as a path dev-dependency.

A package under `lib/` has tests of its own, and those tests alone are held to every line, region and function of the package. That nothing but tests uses the package changes nothing. It is the user's rule, and it is how two of their other repositories hold each package under their `lib/`, with the tests under `test/lib/`. Issue 79 holds the work that brings `lib/test_inputs` to it.

**Why:** The user's other repositories keep their shared code there, and the user wants the same here. In those repos `lib/` stores helper packages shared by the test tiers or modules shared by several deployable parts, each tier keeps its own `samples/` under `test/`, and the pipeline's path filter lists `lib/**`.

**How to apply:** Before putting a helper inside one test tier, ask whether another tier needs it; if so it goes under `lib/`. Keep `lib/**` in the paths that start `.github/workflows/release.yml`, and hold code under `lib/` to the same checks as `src/` and `test/`. My own reading, not something the user confirmed: the product's library, `src/lib.rs` and its modules, stays in `src/`, because it is the product and not a helper, as `src/<package>/` is in the user's other Python repositories. Related: [[the-paper-is-a-starting-point]].
