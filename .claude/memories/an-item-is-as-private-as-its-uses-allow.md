---
name: an-item-is-as-private-as-its-uses-allow
description: "In src/, a function, type, constant, field, method or re-export is visible only as far as something uses it: no keyword, then pub(super), then pub(crate), and pub only for what src/main.rs or a tier under tests/ uses; a new item starts with no keyword; holdout::HoldoutError is the one thing left pub that nothing outside uses"
metadata:
  node_type: memory
  type: project
  originSessionId: e7a2e4e2-1507-4386-b337-6f713c0c9a4d
  modified: 2026-11-18T05:50:35.234Z
---

# An item is as private as its uses allow

Nothing in the library is `pub` for the unit tests. They are inside the library, per [[the-layout-follows-rust-conventions]], and reach it as `crate::…`. So each function, type, constant, field, method, module and re-export of `src/` is visible only as far as something uses it.

There are four levels, and an item has the narrowest one that compiles:

- **no keyword**: used only in its own file, and by the unit tests of the module when the file is `src/<module>/mod.rs`;
- **`pub(super)`**: in a file such as `src/<module>/<file>.rs`, used by the rest of `<module>`, its unit tests among them;
- **`pub(crate)`**: used by another module of the library;
- **`pub`**: used by `src/main.rs` or by a tier under `tests/`, which are programs of their own and see the library from outside.

**Why:** The user asked for what was `pub` only so that the unit tests could reach it to stop being `pub`. Mine, not the user's: the four levels, and taking the narrowest for every kind of item, fields and re-exports among them, are how I carried that out.

**How to apply:**

- A new item starts with no keyword and is widened one level at a time, only when a use needs it. The compiler says when an item is too private. Nothing says when one is more public than it needs to be, so an item is lowered by hand when its last wider use goes.
- In `src/<module>/mod.rs`, `pub(super)` would mean the whole library, so `pub(crate)` is written there.
- A unit test names what a file of its module holds by the file, as `crate::cli::checks::Refused`. No re-export is kept in a `mod.rs` for the tests alone: the compiler reports one as unused.
- A struct can have fields at different levels. Each field follows its own uses.
- A type that a `pub` item shows, in a signature, a field or a variant, has to be `pub` as well, or the compiler warns (`private_interfaces`). Its re-export is `pub use` too, so that it can be named from outside: `genealogy::GenealogyError` is one, shown by `journal::JournalError`.
- `clippy::enum_variant_names` leaves alone an enum that another crate can reach and judges every other one: it objects when all the variants of an enum of three or more start or end with the same word. So lowering such an enum is a moment to look at its variants' names.
