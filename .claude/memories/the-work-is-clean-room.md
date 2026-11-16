---
name: the-work-is-clean-room
description: "The paper is the only source for how a directed evolution behaves; the authors' code, the Darwin Gödel Machine code and any port or derivative of either are never read"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 3c2b70de-42b2-4496-8bfc-c8f6aecc6bad
  modified: 2026-11-15T15:40:19.165Z
---

# The work is clean-room

`cladenc` is a clean-room implementation. The HGM paper (arXiv:2510.21614) is the only source for how a directed evolution behaves. The authors' code, the Darwin Gödel Machine code, and any port or derivative of either are never read, in a browser, a clone, a search result or a quotation.

**Why:** `cladenc` is to be owned by Jose Drowne, with all rights reserved in `LICENSE.txt`. Code written after reading another implementation cannot be shown to be independent of it, and that cannot be undone afterwards. Every issue that builds `cladenc` restates the rule under "Sources".

**How to apply:** Take behaviour from the paper and from the issues, and take the use of a crate from that crate's own documentation. When a search or a page turns out to hold code from one of those sources, leave it unread. The code of any other implementation of the same paper's method is left unread in the same way, whoever wrote it; that is my own caution, not something the user said. See [[the-paper-is-a-starting-point]] for how far `cladenc` may move away from the paper itself.
