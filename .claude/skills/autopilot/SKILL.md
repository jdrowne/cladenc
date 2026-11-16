---
name: autopilot
description: Start, restart or stop the autopilot reminders, which keep the session solving the open issues without being asked and end themselves when no issue is left to take. Use when the user says "start autopilot", "go autonomous on the open issues", "restart autopilot" or "stop autopilot", or asks to clear the reminders. Takes "start", "restart" or "stop"; "start" and "restart" also take `--skip-label <label>`, repeatable.
---

# Autopilot

The autopilot is a set of scheduled reminders. Together they keep the session solving the open issues, in the order their `blocked by` edges set, until none is left that it can take. Then the autopilot ends itself, per "Ending itself".

The work in hand is one issue or several: it begins with the lowest-numbered listed issue that no open issue blocks, and other listed issues may join it, per `.claude/memories/the-work-in-hand-can-be-several-issues.md`.

Starting the autopilot is the user's request to commit and push what it solves: each commit goes to `main` without a further go-ahead, per `.claude/memories/push-to-main.md`, and each closes at least one issue, per `.claude/memories/a-commit-closes-at-least-one-issue.md`.

Every part of the work, reading, searching and filing as well as code, is done in the session itself: no Agent or Workflow call takes any part of it, per `.claude/memories/working-without-subagents.md`.

Fetch `CronCreate`, `CronList` and `CronDelete` with `ToolSearch` first.

## Reminders

A rule is a reminder only when it tells the session what to do next at the moment it fires: which work to take, to keep going, how to hand an issue over instead of stopping, and to end the autopilot when nothing is left. Every other rule the work is done under holds whether or not the autopilot is running, so it is a memory, listed under "Rules that are memories".

Each reminder is indivisible: it states one rule and names one action, so no rule can be quietly dropped from a merged block of text. A rule that names two actions is written as two reminders.

Each reminder fires every ten minutes, on a minute of its own that is two minutes from the next, and none falls on the hour or the half hour. A reminder fires only while the session is idle, which is why every command that can run long goes in the background, per `.claude/memories/long-running-commands-in-the-background.md`.

There are five. The first, on `1,11,21,31,41,51 * * * *`, is the listing:

```text
REMINDER: When no work is in hand, run gh issue list --state open --limit 10000 --json number,title,labels --jq 'map(select(all(.labels[]; .name != "needs decision"))) | sort_by(.number) | map({number, title, labels: [.labels[].name]})' to list the open issues no label holds back, lowest number first.
```

The other four:

| Cron | Prompt |
| --- | --- |
| `3,13,23,33,43,53 * * * *` | `REMINDER: Take as the work in hand the lowest-numbered issue of the listing for which gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state' prints no open, alone or with other issues of the listing that you choose to solve with it.` |
| `5,15,25,35,45,55 * * * *` | `REMINDER: When no work is in hand and no issue of the listing can be taken, end the autopilot: CronDelete every job that CronList returns.` |
| `7,17,27,37,47,57 * * * *` | `REMINDER: Continue autonomously: while an open issue can be taken, do not stop to ask or to report.` |
| `9,19,29,39,49,59 * * * *` | `REMINDER: When an issue in hand cannot go on without a person's answer, hand it to a person by labelling it 'needs decision'.` |

In a prompt, `<number>` stands for the issue being checked, and `gh api` fills in `{owner}` and `{repo}` from the repository the session runs in.

An issue can be taken when the listing names it and no open issue blocks it. A listing that names no issue has none that can be taken.

No reminder tells the session to wait for a run: after a push the work goes straight on, per `.claude/memories/the-work-does-not-wait-for-ci.md`, while a background command watches each run the push started, per `.claude/memories/every-run-is-watched.md`.

## Ending itself

The autopilot ends itself when nothing is left for it to do: no work is in hand, and no issue can be taken. That is so when no issue is open, and when each open issue carries a label that holds it back or is blocked by an open issue.

Left running, the reminders would wake the session every ten minutes for nothing, until the session ends or their seven days are up.

The reminder on `5,15,25,35,45,55 * * * *` says so every ten minutes, and the session does not wait for it: the moment it finds nothing left, it ends the autopilot.

1. Run the listing's command, so that what is found is how the issues stand now. When it names an issue that can be taken after all, take it and do not end.
2. Run "Stop".
3. Tell the user that the autopilot has ended and why: that no issue is open, or which label or blocking issue holds back each open one.

The watches of the runs a push started are background commands and not reminders, so they go on. A watch that ends still resumes the session, and a run found to have failed is still fixed, per `.claude/memories/a-failed-run-is-fixed-before-other-work.md`.

The autopilot does not start again by itself. An issue opened later, or a label taken off, is picked up when the user starts the autopilot again.

## Rules that are memories

Each file is under `.claude/memories/`.

| Rule | Memory |
| --- | --- |
| The work in hand is one issue or several. | `the-work-in-hand-can-be-several-issues.md` |
| An issue is handed to a person only after a check that it needs one, and its body says what is to be decided. | `before-an-issue-is-labelled-needs-decision.md` |
| An issue is brought up to date before it is worked on. | `an-issue-is-brought-up-to-date-before-work.md` |
| An issue documents one indivisible problem. | `one-indivisible-problem-per-issue.md` |
| A `blocked_by` edge is written only where the block is real. | `a-blocked-by-edge-states-a-real-block.md` |
| The work is clean-room. | `the-work-is-clean-room.md` |
| The tests of a change are written before its code. | `tests-are-written-before-code.md` |
| A command that can take more than about a minute runs in the background. | `long-running-commands-in-the-background.md` |
| A commit closes at least one issue, with a `Closes #N` line for each. | `a-commit-closes-at-least-one-issue.md` |
| After a push the work goes straight on, and no run is waited for. | `the-work-does-not-wait-for-ci.md` |
| Every run a push starts is watched to its end in the background. | `every-run-is-watched.md` |
| A run found to have failed is fixed in the next commit pushed, ahead of other work. | `a-failed-run-is-fixed-before-other-work.md` |
| After a push, the issues its commit closes are checked to be closed. | `a-push-can-leave-its-issues-open.md` |

## Start

Each `--skip-label <label>` puts `map(select(all(.labels[]; .name != "<label>"))) |` and a space at the very start of the listing's `--jq` program, and appends `An issue labelled '<label>' is left to a person, whatever else it carries.` to the listing's reminder. The filter stays in the `--jq` program and never moves to `--label` or `--search`, which the sibling repositories found to send the listing through GitHub's search and its limit of 1000 issues. A `--skip-label` that names `needs decision` changes nothing, since the listing leaves that label out as it stands.

An issue is blocked while `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by` lists an open issue.

1. Say which model this session runs on, as its system prompt names it. The issues that build `cladenc` ask, under "Working rules", for the latest Fable model at Extra High effort, and only the user can set either. When the model is another, say so and stop here.
2. Make sure the label the autopilot leaves to a person exists: when `gh label list --search 'needs decision' --json name --jq '.[].name'` does not print `needs decision`, create it with `gh label create 'needs decision' --description 'Left to a person' --color d876e3`.
3. Run the listing's command once, its `--skip-label` filters in place. When it names no issue that can be taken, say so, say which label or blocking issue holds back each open issue, and stop: there is nothing to schedule reminders for.
4. Call `CronList`, and `CronDelete` each job whose prompt begins `REMINDER:` and is not one of the five reminders on its own cron. That clears a reminder left from before one was changed or moved to another minute.
5. `CronCreate` with `recurring: true` each reminder that is not already scheduled.
6. Tell the user how many reminders are running, that the autopilot ends itself when no issue is left that it can take, and the two limits that come with the reminders: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after seven days.
7. Read in full each memory that "Rules that are memories" names.
8. Take the lowest-numbered listed issue that no open issue blocks, alone or with other listed issues chosen to be solved with it, and solve that work. When no work is in hand and no issue can be taken, end the autopilot, per "Ending itself".

## Restart

Run "Stop", then "Start" with the `--skip-label` flags that `restart` was given. That leaves exactly the five reminders, each with seven fresh days, and it is how a changed set of `--skip-label` flags takes effect.

## Stop

`CronDelete` every job `CronList` returns, then call `CronList` again to confirm it is empty and report how many jobs were deleted. A `CronList` that returns nothing is not a failure; say the schedule was already empty.
