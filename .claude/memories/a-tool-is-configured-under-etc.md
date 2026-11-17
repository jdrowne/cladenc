---
name: a-tool-is-configured-under-etc
description: "A tool's configuration file is etc/<tool>/<tool>.<extension>, such as etc/clippy/clippy.toml and etc/nfpm/nfpm.yaml, and the workflow points the tool at it; a linter gets one only for what it takes no other way"
metadata:
  node_type: memory
  type: feedback
  originSessionId: c13f2c90-63b4-4656-9ced-ca7a27003b42
  modified: 2026-11-17T04:24:48.050Z
---

# A tool is configured under etc

The configuration file of a tool is `etc/<tool>/<tool>.<extension>`: `etc/clippy/clippy.toml` for clippy, `etc/nfpm/nfpm.yaml` for nfpm. Nothing a tool reads sits at the top of the repository.

**Why:** It is the user's convention for where such a file goes.

**How to apply:**

- The workflow names the file to the tool, since the tool does not look under `etc/` by itself: `CLIPPY_CONF_DIR` set to `${{ github.workspace }}/etc/clippy`, an absolute path because clippy runs from the directory of each package, and `--config etc/nfpm/nfpm.yaml` for nfpm.
- `etc/**` is in the paths that start `.github/workflows/release.yml`, so a change to such a file starts a run.
- A linter's file is refused by `assert-no-linter-config-files`, per [[the-checks-the-code-faces]], so the workflow passes `--exclude` with the one path of the file, and nothing wider.
- Mine, not the user's: a linter gets a file only for what it takes no other way. clippy takes a lint's level on the command line, so `-D clippy::cognitive_complexity` stays in the workflow, and takes a threshold only from `clippy.toml`, so the thresholds are in the file. A file that turns a lint off is not what the exception is for.
