---
name: the-simplest-design-is-offered-first
description: "When a decision is put to the user, lead with the simplest design that works, such as an option on the command line, and explain it in short concrete steps with every word defined"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 2c25b762-5edb-4922-82ce-34177297057c
  modified: 2026-11-15T05:04:31.144Z
---

# The simplest design is offered first

When a design decision is put to the user, the first option offered is the simplest one that solves the problem, and for `cladenc` that is usually one more option on the command line. A heavier design, such as a convention every harness has to follow or a tool `cladenc` has to drive, is offered only where the simple one fails, and the reply says where it fails.

The explanation is a few short steps about one concrete case. Each word that carries meaning is either the user's own or defined where it first appears.

A reply carries one point. The other points wait for later replies, one each, even when they are all ready. The user asked for this on 2026-11-15, after the reply that recorded #28's decision also listed three choices of mine, a removed option, uncommitted memory files and three other open issues, and left them lost.

**Why:** On 2026-11-14 and 2026-11-15 it took about ten messages to get issue #28's one decision made. The user said several times that they did not follow. The explanations used words nobody had defined for them, parent, run and picked among them, and loose wording they had to correct: what the harness is called, that the directory holds source code, and what exactly gets improved. When the choice finally reached them as a file required in the source code against an image and a command, they asked why it was so complicated and settled it at once with two plain options, one to build a harness and one to run it. The earlier session had the same trouble: an explanation that confused them, and four decisions asked at once.

**How to apply:** Before asking, find the smallest thing that would work and offer that. Walk through the first step of a search with the progenitor alone, not the general case. Keep the trade-offs to what changes for the user. My own interpretation, not something the user said: they would sooner pass one more option than have `cladenc` impose a rule on harnesses or depend on another tool. Related: [[cladenc-takes-a-progenitor-and-a-model]], [[a-harness-is-not-called-a-coding-agent]], [[before-an-issue-is-labelled-needs-decision]]. Recorded under [[never-quote-the-user-verbatim]].
