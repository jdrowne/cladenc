---
name: cargo-is-not-on-the-shell-path
description: "On the user's Mac the Rust toolchain is in ~/.cargo/bin, which the Bash tool's shell does not have on PATH; put it on PATH before calling cargo, and do not install Rust again"
metadata:
  node_type: memory
  type: reference
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# cargo is not on the shell's PATH

On the user's Mac, rustup's toolchain is in `~/.cargo/bin`: `cargo`, `rustc`, `cargo-fmt`, `cargo-clippy` and `cargo-llvm-cov` among others. The shell the Bash tool starts does not have that directory on `PATH`, so `command -v cargo` finds nothing even though Rust is installed. Start a command that needs the toolchain with `export PATH="$HOME/.cargo/bin:$PATH"`, and do not install Rust again.

The toolchain is the stable one, with `rustfmt`, `clippy` and `llvm-tools` among its components. A look that stops at `command -v cargo` wrongly concludes that no toolchain is installed.
