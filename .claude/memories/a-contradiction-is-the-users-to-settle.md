---
name: a-contradiction-is-the-users-to-settle
description: "Where two memories, a memory and a reminder or the skill, or a memory and the code say opposite things about a rule or a design, the user decides which holds; each contradiction is put to them on its own, never settled by choosing a side; once the user answers, every memory the answer touches is rewritten in part so that no two memories disagree"
metadata:
  node_type: memory
  type: feedback
  originSessionId: b1d622db-0e04-473f-b926-900805d77634
  modified: 2026-11-21T15:54:41.625Z
---

# A contradiction is the user's to settle

Where two memories say opposite things, or a memory and an autopilot reminder or `.claude/skills/autopilot/SKILL.md` do, or a memory and the code disagree on what `cladenc` is meant to do, the user decides which one holds. The session does not choose a side, even where one side looks newer or better supported, and does not count the fix as one that needs no decision.

**Why:** Each side of a contradiction is a rule or a design someone set, and picking one drops the other without the user knowing.

**How to apply:** Put each contradiction to the user on its own: as an issue of its own, labelled `needs decision` per [[before-an-issue-is-labelled-needs-decision]] and labelled per [[issues-about-the-sessions-rules-are-labelled]], or in a reply, one per reply. Say what each side tells the session to do, which files hold each, and what changes under each answer. A recommendation may come with it, marked as mine. Rewrite the memories, the skill or the code only after the user answers. Then leave no two memories saying opposite things: each memory the answer touches is rewritten in the same commit, and only in the part the answer reaches, so that the rest of it holds as before. On 2026-11-24 the user held that memories may not disagree with one another, and had [[an-issue-is-brought-up-to-date-before-work]] rewritten in part to fit a decision recorded in [[a-passed-run-can-show-a-closed-issue-unsolved]]. A count or a name in a memory that the code has moved past, with no rule or design behind it, is no contradiction and is corrected. Nor is a rule set against an instruction from the user: the instruction is carried out, per [[what-the-user-instructs-is-done]].
