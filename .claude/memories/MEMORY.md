# Memory Index

- [Push to main](push-to-main.md) — commit/push directly to main, no feature branches or PRs; starting the autopilot counts as the ask
- [Never quote the user verbatim](never-quote-the-user-verbatim.md) — restate what the user says in fresh words in memories, issues, commits, comments and summaries
- [lib/ holds shared code](lib-holds-shared-code.md) — code more than one consumer uses goes under lib/, as in the sibling repos; here, code shared by the test tiers
- [The paper is a starting point](the-paper-is-a-starting-point.md) — keep the HGM paper's ideas, name things plainly, and let cladenc leave the paper where a change improves it
- [Working without subagents](working-without-subagents.md) — no Agent or Workflow call, whatever the work's size; denied in the tracked settings.json, lifted only by the user's own ask
- [A wait runs in the background](a-wait-runs-in-the-background.md) — wait for CI with run_in_background or Monitor, never in the foreground, so the autopilot reminders can fire
- [cargo is not on the shell's PATH](cargo-is-not-on-the-shell-path.md) — Rust is installed in ~/.cargo/bin; put it on PATH before calling cargo, and do not install it again
