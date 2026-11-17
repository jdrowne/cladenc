---
name: lib-stores-shared-code
description: "Code that more than one consumer uses goes under lib/, as in the user's other repositories; here that means code shared by the test tiers"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# lib/ stores shared code

Code that more than one consumer uses lives under `lib/`, the convention of the user's other repositories. In `cladenc` that is the code more than one test tier uses, kept as a package of its own, `lib/test_inputs`, that the `cladenc` package names as a path dev-dependency.

**Why:** The user's other repositories keep their shared code there, and the user wants the same here. In those repos `lib/` stores helper packages shared by the test tiers or modules shared by several deployable parts, each tier keeps its own `samples/` under `test/`, and the pipeline's path filter lists `lib/**`.

**How to apply:** Before putting a helper inside one test tier, ask whether another tier needs it; if so it goes under `lib/`. Keep `lib/**` in the paths that start `.github/workflows/release.yml`, and hold code under `lib/` to the same checks as `src/` and `test/`. My own reading, not something the user confirmed: the product's library, `src/lib.rs` and its modules, stays in `src/`, because it is the product and not a helper, as `src/<package>/` is in the user's other Python repositories. Related: [[the-paper-is-a-starting-point]].
