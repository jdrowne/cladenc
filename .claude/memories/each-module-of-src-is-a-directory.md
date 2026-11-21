---
name: each-module-of-src-is-a-directory
description: "The layout of src/: every module is a directory with its own code in mod.rs, a module of one file among them; lib.rs and main.rs are the only files directly in src/, and no Rust file sits beside a directory of its name"
metadata:
  node_type: memory
  type: project
  originSessionId: cbdff9f0-8a8b-4526-8929-b4403e305f2d
  modified: 2026-11-17T06:23:22.011Z
---

# Each module of src is a directory

Every module of `src/` is a directory, and the module's own code is in the `mod.rs` of that directory: `src/cli/mod.rs`, with `src/cli/checks.rs` and `src/cli/contract.rs` beside it. A module of one file is a directory of one file, as `src/report/mod.rs`. The only files directly in `src/` are `lib.rs` and `main.rs`, which Rust needs to be files.

**Why:** A listing of `src/` shows each module once, everything of a module is in one directory, its unit tests among them.

**How to apply:**

- A new module is `src/<name>/mod.rs` from its first line, with `pub mod <name>;` in `src/lib.rs`. It is not started as `src/<name>.rs` to be moved later.
- A part of a module is a file in the module's directory, as `src/containers/program.rs`. Once that part has files of its own under it, it becomes a directory with a `mod.rs` too: `src/containers/program/mod.rs`.
- A file is named `mod.rs` when a memory, an issue or a reply names a module's own code: `src/directed_evolution/mod.rs`, not `src/directed_evolution.rs`.
- Two checks of `release.yml` hold the layout, per [[the-checks-the-code-faces]]: the job `assert-one-directory-per-module`, and `clippy::self_named_module_files` on the `cargo clippy` line.
- The other two layouts are not proposed again, and `clippy::mod_module_files`, which refuses a `mod.rs`, is never turned on.
- The unit tests and the tiers under `tests/` are laid out per [[the-layout-follows-rust-conventions]], and `crates/` per [[crates-stores-shared-code]].
