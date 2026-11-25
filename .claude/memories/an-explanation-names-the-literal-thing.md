---
name: an-explanation-names-the-literal-thing
description: "when explaining code or an issue to the user, name the file, the command, the path and what happens to it; no abstract stand-in such as setup, inputs, build or the machine; one fact per reply when the user asks to go one at a time"
metadata:
  node_type: memory
  type: feedback
  originSessionId: df0bf027-c554-4f41-8bfe-5eb116dd45bf
  modified: 2026-11-25T03:39:34.035Z
---

# An explanation names the literal thing

An explanation given to the user names the literal thing: the file and line, the command that runs, the path that is created, the computer it runs on (for CI, the GitHub virtual machine of the job, by its label). An abstract stand-in is not used in its place. On 2026-11-24, while being told what issue 170 was about, the user stopped the session at each of these words and asked what it stood for: setup, check (of a test), carried out, shared, inputs, build (where nothing was compiled), the machine. The user said the looseness of the words made the conversation very hard to follow.

When the user asks to go one thing at a time, a reply holds one fact and nothing else, and the next fact waits for the user.

**Why:** a word that stands for several concrete things makes the user ask what it stands for, once per word, before the point is reached.

**How to apply:** before sending an explanation, replace each general noun or verb with what it is in the code: "creates the directory `/tmp/.tmpAbC123` and runs `cladenc start` on it", not "carries out a setup". This is about replies and explanations, and an issue's body is written the same way: the wording of issue 170 is where the loose words came from. See [[the-local-machine-stays-out-of-the-record]] for what is still not written down about the local machine.
