---
name: autopilot
description: Start, restart or stop the autopilot reminders, which keep the session solving the open issues without being asked. Use when the user says "start autopilot", "go autonomous on the open issues", "restart autopilot" or "stop autopilot", or asks to clear the reminders. Takes "start", "restart" or "stop"; "start" and "restart" also take `--skip-label <label>`, repeatable.
---

# Autopilot

The autopilot is a set of scheduled reminders. Together they keep the session solving the open issues, in the order their `blocked by` edges set, until none is left that it can take.

The work in hand is one issue or several: it begins with the lowest-numbered listed issue that no open issue blocks, and other listed issues may join it, per `.claude/memories/the-work-in-hand-can-be-several-issues.md`.

Starting the autopilot is the user's request to commit and push what it solves: each commit goes to `main` without a further go-ahead, per `.claude/memories/push-to-main.md`, and each closes at least one issue, per `.claude/memories/a-commit-closes-at-least-one-issue.md`.

Every part of the work, reading, searching and filing as well as code, is done in the session itself: no Agent or Workflow call takes any part of it, per `.claude/memories/working-without-subagents.md`.

Fetch `CronCreate`, `CronList` and `CronDelete` with `ToolSearch` first.

## Reminders

A rule is a reminder only when it tells the session what to do next at the moment it fires: which work to take, to keep going, and how to hand an issue over instead of stopping. Every other rule the work is done under holds whether or not the autopilot is running, so it is a memory, listed under "Rules that are memories".

Each reminder is indivisible: it states one rule and names one action, so no rule can be quietly dropped from a merged block of text. A rule that names two actions is written as two reminders.

Each reminder has its own minute of a twenty-minute cycle, four minutes from the next, and none falls on the hour or the half hour. A reminder fires only while the session is idle, which is why every command that can run long goes in the background, per `.claude/memories/long-running-commands-in-the-background.md`.

There are four. The first, on `1,21,41 * * * *`, is the listing:

```text
REMINDER: When no work is in hand, run gh issue list --state open --limit 10000 --json number,title,labels --jq 'map(select(all(.labels[]; .name != "needs decision"))) | sort_by(.number) | map({number, title, labels: [.labels[].name]})' to list the open issues no label holds back, lowest number first.
```

The other three:

| Cron | Prompt |
| --- | --- |
| `5,25,45 * * * *` | `REMINDER: Take as the work in hand the lowest-numbered issue of the listing for which gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state' prints no open, alone or with other issues of the listing that you choose to solve with it.` |
| `9,29,49 * * * *` | `REMINDER: Continue autonomously: while an open issue can be taken, do not stop to ask or to report.` |
| `13,33,53 * * * *` | `REMINDER: When an issue in hand cannot go on without a person's answer, hand it to a person by labelling it 'needs decision'.` |

In a prompt, `<number>` stands for the issue being checked, and `gh api` fills in `{owner}` and `{repo}` from the repository the session runs in.

No reminder tells the session to wait for a run: after a push the work goes straight on, per `.claude/memories/the-work-does-not-wait-for-ci.md`.

## Rules that are memories

Each file is under `.claude/memories/`.

| Rule | Memory |
| --- | --- |
| The work in hand is one issue or several. | `the-work-in-hand-can-be-several-issues.md` |
| An issue is handed to a person only after a check that it needs one, and its body says what is to be decided. | `before-an-issue-is-labelled-needs-decision.md` |
| An issue is brought up to date before it is worked on. | `an-issue-is-brought-up-to-date-before-work.md` |
| An issue documents one indivisible problem. | `one-indivisible-problem-per-issue.md` |
| A problem found along the way is filed, and solved now only when the work cannot go on without it. | `a-problem-found-along-the-way-is-filed.md` |
| A `blocked_by` edge is written only where the block is real. | `a-blocked-by-edge-states-a-real-block.md` |
| The work is clean-room. | `the-work-is-clean-room.md` |
| The tests of a change are written before its code. | `tests-are-written-before-code.md` |
| A command that can take more than about a minute runs in the background. | `long-running-commands-in-the-background.md` |
| A commit closes at least one issue, with a `Closes #N` line for each. | `a-commit-closes-at-least-one-issue.md` |
| After a push the work goes straight on, and no run is waited for. | `the-work-does-not-wait-for-ci.md` |
| A run found to have failed is fixed in the next commit pushed, ahead of other work. | `a-failed-run-is-fixed-before-other-work.md` |
| After a push, the issues its commit closes are checked to be closed. | `a-push-can-leave-its-issues-open.md` |

## Start

Each `--skip-label <label>` puts `map(select(all(.labels[]; .name != "<label>"))) |` and a space at the very start of the listing's `--jq` program, and appends `An issue labelled '<label>' is left to a person, whatever else it carries.` to the listing's reminder. The filter stays in the `--jq` program and never moves to `--label` or `--search`, which the sibling repositories found to send the listing through GitHub's search and its limit of 1000 issues. A `--skip-label` that names `needs decision` changes nothing, since the listing leaves that label out as it stands.

An issue is blocked while `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by` lists an open issue.

1. Say which model this session runs on, as its system prompt names it. The issues that build `cladenc` ask, under "Working rules", for the latest Fable model at Extra High effort, and only the user can set either. When the model is another, say so and stop here.
2. Make sure the label the autopilot leaves to a person exists: when `gh label list --search 'needs decision' --json name --jq '.[].name'` does not print `needs decision`, create it with `gh label create 'needs decision' --description 'Left to a person' --color d876e3`.
3. Run the listing's command once, its `--skip-label` filters in place. When it names no issue, say so and stop: there is nothing to schedule reminders for.
4. Call `CronList`, and `CronDelete` each job on one of the four crons whose prompt differs from that cron's reminder.
5. `CronCreate` with `recurring: true` each reminder that is not already scheduled.
6. Tell the user how many reminders are running, and the two limits that come with them: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after seven days.
7. Read in full each memory that "Rules that are memories" names.
8. Take the lowest-numbered listed issue that no open issue blocks, alone or with other listed issues chosen to be solved with it, and solve that work. When no issue can be taken, say which label or blocking issue holds back each open issue and stop.

## Restart

Run "Stop", then "Start" with the `--skip-label` flags that `restart` was given. That leaves exactly the four reminders, each with seven fresh days, and it is how a changed set of `--skip-label` flags takes effect.

## Stop

`CronDelete` every job `CronList` returns, then call `CronList` again to confirm it is empty and report how many jobs were deleted. A `CronList` that returns nothing is not a failure; say the schedule was already empty.
