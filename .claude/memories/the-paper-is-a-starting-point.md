---
name: the-paper-is-a-starting-point
description: "The HGM paper is where cladenc starts, not a standard it is held to — keep its ideas, name things plainly and with biological words, and let cladenc leave the paper where a change improves it"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-17T02:52:12.802Z
---

# The paper is a starting point

`cladenc` takes its ideas from the HGM paper (arXiv:2510.21614) and is not bound to the paper's vocabulary, notation or details. Where a plainer, more natural name exists than the paper's, use it: in flag names, journal keys, code identifiers, the issues and the README. `cladenc` may also depart from the paper's method wherever a change makes it better.

**Why:** In the user's view the paper's terms are not needed where a more natural one exists, and the paper is followed in its ideas, not to the letter. `cladenc` is intellectual property owned by Jose Drowne (`LICENSE.txt` reserves all rights), so it is free to move away from the paper as it improves. And `cladenc` is named after the clade, so its words are biological ones, kept simple: the paper's tree-search words mean nothing to the user.

**How to apply:** Name things for what they are. The user's own words: `primogenitor` for the harness a directed evolution starts from, and `progenitor` for the harness that procreates, the two told apart per [[the-first-harness-is-the-primogenitor]], `genealogy` for what the paper calls the tree, and `procreation` for the step that makes a new harness. Three more are the user's choice. A `directed evolution` is one run of `cladenc` from the primogenitor to the best harness, `directed_evolution` in the code and `cladenc.directed-evolution` as the label of a container. `Artificial selection`, `ArtificialSelection` in the code, is the rule that chooses what a directed evolution does next, which harness procreates and which harness attempts which task, and which harness is best at the end. Where the paper says evaluate and evaluation, the words are those of [[the-words-about-benchmarks]]: a harness attempts a task, and is benchmarked. Search, policy, evaluation and evaluate are not used for them: an older issue says those words and means these, and is corrected when it is brought up to date, per [[an-issue-is-brought-up-to-date-before-work]]. What the paper calls an agent is a `harness` here: the user thinks of `cladenc` as improving harnesses and finds the paper's word unclear. Mention the paper's name for an idea once, where the README maps ideas to the paper, so a reader can still find it there. Do not write point-by-point reconciliations with the paper's text, and do not reject an improvement because the paper does it differently. The clean-room rule is separate and unchanged: the paper is the only source, and the authors' code is never read, per [[the-work-is-clean-room]].

When a thing needs a name, look first for a simple word from biology or family descent that says what the thing is, in keeping with primogenitor, progenitor, child, genealogy, clade and procreation; a tree-search word or a word of the paper's is not a reason by itself. The word has to be true of what happens. A progenitor is never changed, and it is its child that is changed, so no word that says the progenitor changes fits: mutation does not, since to mutate is to change oneself.

My own readings, not the user's words: the rule covers the paper's Greek-letter symbols too, so a setting is named in words and never by a symbol; and a new word that people will type or read is put to the user, with what it would look like, before anything is renamed; and directed evolution and artificial selection are written in full, two words each, and not shortened to evolution or selection.
