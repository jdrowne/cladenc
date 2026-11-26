---
name: a-flag-name-says-what-it-holds
description: "A flag's name says what kind of thing the value is and what it is for, such as --diversification-docker-image, never a bare word like image"
metadata:
  node_type: memory
  type: feedback
  originSessionId: a2ae831f-272a-4a47-b46b-d83e0c3cb77c
  modified: 2026-11-21T13:26:51.341Z
---

# A flag's name says what it holds

A flag's name says what kind of thing its value is, and what `cladenc` uses it for. The kind is said in full: `docker-image`, not `image`. The use comes first: `--diversification-docker-image`, `--diversification-dockerfile`, `--diversification-docker-image-build-timeout`.

**Why:** A short word such as `image` does not tell a person what the flag takes. Once `cladenc` uses more than one Docker image, a bare `--docker-image` does not say which one.

**How to apply:**

- When proposing a flag, name the kind of value in full and put its use in front.
- When a change gives an existing flag a second meaning, or narrows it to one use, propose renaming it so the name says which.

Related: [[a-flag-is-not-called-an-option]], [[no-flag-has-a-default]], [[no-flag-for-what-cladenc-already-holds]].
