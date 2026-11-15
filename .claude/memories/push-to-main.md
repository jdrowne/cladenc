---
name: push-to-main
description: "In this repo, commit and push directly to main, with no feature branches or PRs; a change the user asks for is committed and pushed once it is done and checked, without asking again"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 92d16771-d8b4-4ee4-8f39-ae96be264e22
  modified: 2026-11-15T15:40:19.165Z
---

# Push to main

Commit and push directly to `main`; don't create feature branches or open PRs.

**Why:** The user's stated workflow for this repo is to push to `main`.

**How to apply:** When asked to commit/push, stay on `main` and `git push origin main`. This overrides the default "branch first if on the default branch" guidance. Commit and push only when asked, and two things count as the ask. Starting the autopilot skill, `.claude/skills/autopilot/SKILL.md`, is one: what it solves is committed and pushed without a further go-ahead. Asking for a change to the repository is the other: once the change is made and the local checks pass, commit and push it, and do not stop to ask whether to.

A change the user asks for is not left in the working tree behind a question about whether to commit it: asking for the change is the ask. Such a commit closes no issue, per [[a-commit-closes-at-least-one-issue]], and its runs are watched, per [[every-run-is-watched]]. A question, or a request to explain or to look into something, is not an ask for a change.
