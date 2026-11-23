---
name: what-the-repository-says-is-read-there
description: "No memory describes how the code, a workflow or a tool works: the file that says it is read instead, where it is always right; a memory holds what no file of the repository says, such as a decision of the user's and its reason"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 43d1a223-9777-4a32-b6c8-9e9dd430d745
  modified: 2026-11-23T03:39:24.164Z
---

# What the repository says is read there

How the code, a workflow or a tool works is read in the file that says it. No memory restates it. A memory holds what no file of the repository says: a decision of the user's, the reason for it, a word they chose, how they want the work done.

**Why:** The user said so twice on 2026-11-22, about a memory of the session's that listed the checks of `.github/workflows/release.yml`:

- First they asked why memories describe how the software works when the source of truth, here the workflow, can be read. The session cut the memory down to what it took the workflow not to say, and kept it.
- Then they said they still did not see why it existed, and that the session's own example of what the workflow does not say, how clippy counts its limits, is in the workflow. It is: the lints are in the workflow's `clippy` command, the limits in `etc/clippy/clippy.toml`, and a failed run names the line and the count. The memory was deleted.

Such a memory goes stale each time the file changes, and a session that trusts it over the file is wrong where a session that reads the file is right.

**How to apply:**

- Before a memory is saved, ask whether a file of the repository says it, or a run of CI would. When one does, nothing is saved.
- What a check asks is read in the workflow and in the configuration it points at. What a function does is read in the function.

What is mine, and not the user's:

- What a failed run teaches about a tool is not saved either. The run says it again the next time, per [[ci-says-whether-the-code-is-right]], and the deleted memory grew out of such notes.
- The user named one memory. Another memory found to restate a file has the same fault.
