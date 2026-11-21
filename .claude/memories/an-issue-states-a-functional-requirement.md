---
name: an-issue-states-a-functional-requirement
description: "An issue describes a problem or states a functional requirement, as a person running cladenc would see it, and leaves out how to build it: no functions, files, Docker commands or mechanisms to use"
metadata:
  node_type: memory
  type: feedback
  originSessionId: a2ae831f-272a-4a47-b46b-d83e0c3cb77c
  modified: 2026-11-21T13:05:38.600Z
---

# An issue states a functional requirement

An issue describes a problem, or states a functional requirement: what `cladenc` does wrong, or what it is expected to do, seen from outside: what a person gives it, what it does with that, and what comes out. It does not say how to build it: no functions or files to change, no Docker commands, no mechanism to choose between.

**Why:** Steps for carrying it out make the issue long, tie the session that does the work to one way of doing it, and hide what is being asked.

**How to apply:**

- State each behaviour so that it can be checked from the command line, the outputs directory or the README.
- Leave the design to the session that works on the issue; that session reads the code itself, per [[an-issue-is-brought-up-to-date-before-work]].
- A constraint the work has to keep, such as [[cladenc-works-offline-and-self-hosted]], may be stated as a requirement; how to keep it is still left out.

Related: [[issues-have-no-fixed-form]], [[one-indivisible-problem-per-issue]].
