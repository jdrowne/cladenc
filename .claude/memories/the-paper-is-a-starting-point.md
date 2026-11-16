---
name: the-paper-is-a-starting-point
description: "The HGM paper is where cladenc starts, not a standard it is held to — keep its ideas, name things plainly and with biological words, and let cladenc leave the paper where a change improves it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# The paper is a starting point

`cladenc` takes its ideas from the HGM paper (arXiv:2510.21614) and is not bound to the paper's vocabulary, notation or details. Where a plainer, more natural name exists than the paper's, use it: in flag names, journal keys, code identifiers, the issues and the README. `cladenc` may also depart from the paper's method wherever a change makes it better.

**Why:** In the user's view the paper's terms are not needed where a more natural one exists, and the paper is followed in its ideas, not to the letter. `cladenc` is intellectual property owned by Jose Drowne (`LICENSE.txt` reserves all rights), so it is free to move away from the paper as it improves. And `cladenc` is named after the clade, so the words of its search are biological ones, kept simple: the paper's tree-search words mean nothing to the user.

**How to apply:** Name things for what they are. The user's own words: `progenitor` for the harness a search starts from, `genealogy` for what the paper calls the tree, and `procreation` for the step that makes a new harness. Artificial selection, `ArtificialSelection` in the code, is the rule that chooses what a search does next, which harness procreates and which harness is tried on which task, and which harness is best at the end: the two words are the user's choice, always written in full and never shortened to selection, and policy, the reinforcement-learning word, is not used. What the paper calls an agent is a `harness` here: the user thinks of `cladenc` as improving harnesses and finds the paper's word unclear. Mention the paper's name for an idea once, where the README maps ideas to the paper, so a reader can still find it there. Do not write point-by-point reconciliations with the paper's text, and do not reject an improvement because the paper does it differently. The clean-room rule is separate and unchanged: the paper is the only source, and the authors' code is never read, per [[the-work-is-clean-room]].

When a thing needs a name, look first for a simple word from biology or family descent that says what the thing is, in keeping with progenitor, parent, child, genealogy, clade and procreation; a tree-search word or a word of the paper's is not a reason by itself. The word has to be true of what happens. A parent is never changed, and it is its child that is changed, so no word that says the parent changes fits: mutation does not, since to mutate is to change oneself.

My own readings, not the user's words: the rule covers the paper's Greek-letter symbols too, which is why the flags are `--genealogy-growth` and `--confidence`; a new word that people will type or read is put to the user, with what it would look like, before anything is renamed; and `evaluation` stays because it is plain and biology offers nothing simpler.
