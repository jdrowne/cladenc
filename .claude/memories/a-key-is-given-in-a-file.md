---
name: a-key-is-given-in-a-file
description: "A secret such as a model's key reaches cladenc in a file that a flag names; it is never the value of a flag and never read from an environment variable of cladenc's own"
metadata:
  type: feedback
---

# A key is given in a file

A secret that `cladenc` needs, such as the key a remote model is called with, is kept in a file, and a flag names the file. The secret is not the value of a flag, and it is not read from an environment variable of `cladenc`'s own.

**Why:** It is the user's rule: a key is not handed over in a flag, and the flag points to a file that holds it. A flag that names an environment variable holding the key is not what they want either, so it is not offered.

**How to apply:** For any secret, propose a flag whose value is the path of a file, as `--model-key-file` is, a name of mine. Keep the secret off every command line, that of `docker run` included, and out of the journal and of anything `cladenc` prints; the journal records the path. Such a flag is given only where a secret is needed, which assumes no value in its place, per [[no-flag-has-a-default]]. Related: [[cladenc-takes-a-progenitor-and-a-model]], [[the-simplest-design-is-offered-first]].
