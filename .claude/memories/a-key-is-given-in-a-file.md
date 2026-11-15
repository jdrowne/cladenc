---
name: a-key-is-given-in-a-file
description: "A secret such as a model's key reaches cladenc in a file that a flag names; it is never the value of a flag, and an environment variable was turned down as the way to give it"
metadata:
  type: feedback
---

# A key is given in a file

A secret that `cladenc` needs, such as the key a remote model is called with, is kept in a file, and a flag names the file. The secret is not the value of a flag.

**Why:** On 2026-11-15, for issue #27, I proposed a flag naming an environment variable of `cladenc`'s own that holds the key. The user answered that the key is not to be handed over in a flag and that the flag is to point to a file holding it. My own reading, not something the user said: they may have taken my flag to carry the key itself, which it did not; either way the file is what they asked for, so the environment variable is not offered again.

**How to apply:** For any secret, propose a flag whose value is the path of a file, such as `--model-key-file` in #27, a name of mine that the user has not confirmed. Keep the secret off every command line, that of `docker run` included, and out of the journal and of anything `cladenc` prints; the journal records the path. Such a flag is given only where a secret is needed, which assumes no value in its place, per [[no-flag-has-a-default]]. Related: [[cladenc-takes-a-progenitor-and-a-model]], [[the-simplest-design-is-offered-first]]. Recorded under [[never-quote-the-user-verbatim]].
