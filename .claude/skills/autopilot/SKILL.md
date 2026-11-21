---
name: autopilot
description: Start, restart or stop the autopilot reminders, which keep the session solving the open issues without being asked and end themselves when no issue is left to take, or schedule only the reminder that holds during any work. Use when the user says "start autopilot", "go autonomous on the open issues", "restart autopilot", "reminders only", "switch to reminders only", "stop autopilot" or "reminders off", or asks to clear the reminders. Takes "start", "start reminders-only", "restart", "restart reminders-only" or "stop"; "start" and "restart" without "reminders-only" also take `--skip-label <label>`, repeatable.
---

# Autopilot

The autopilot is a set of scheduled reminders. Together they keep the session solving the open issues, in the order their `blocked by` edges set, until none is left that it can take. Then the autopilot ends itself, per "Ending itself".

The work in hand is one issue or several: it begins with the lowest-numbered listed issue that no open issue blocks, and other listed issues may join it, per `.claude/memories/the-work-in-hand-can-be-several-issues.md`.

Starting the autopilot is the user's request to commit and push what it solves: each commit goes to `main` without a further go-ahead, per `.claude/memories/push-to-main.md`, and each closes at least one issue, per `.claude/memories/a-commit-closes-at-least-one-issue.md`. Each push is one commit, which may hold several things at once, per `.claude/memories/one-commit-per-push.md`.

Every part of the work, reading, searching and filing as well as code, is done in the session itself: no Agent or Workflow call takes any part of it, per `.claude/memories/working-without-subagents.md`.

Fetch `CronCreate`, `CronList` and `CronDelete` with `ToolSearch` first.

## Reminders

A rule is a reminder only when it tells the session what to do next at the moment it fires: to wait while a CI run is going, which work to take, to keep going, how to hand an issue over instead of stopping, and to end the autopilot when nothing is left. Every other rule the work is done under holds whether or not the autopilot is running, so it is a memory, listed under "Rules that are memories". The wait for CI is both. It is a memory, since it holds with the autopilot stopped, and a reminder, since without it the other reminders would draw the session back to work while a run is going. It is the one reminder of the form `reminders-only`, per "Reminders only".

Before a reminder is added, the test is whether its rule would still hold with the autopilot stopped. When it would, the rule is written as a memory and given a row in "Rules that are memories", and it is a reminder as well only when a reminder that fires would otherwise lead the session against it, as with the wait for CI.

Each reminder is indivisible: it states one rule and names one action, so no rule can be quietly dropped from a merged block of text. A rule that names two actions is written as two reminders.

Each reminder fires every fifteen minutes, on a minute of its own at least two minutes from the next, and none falls on the hour or the half hour. A reminder fires only while the session is idle, which is why every command that can run long goes in the background, per `.claude/memories/long-running-commands-in-the-background.md`.

There are six. The first, on `1,16,31,46 * * * *`, is the wait for CI:

```text
REMINDER: While a CI run that a push of this session started is queued or in progress, only wait: no reading, edits, issue writing, commits or pushes, except what the user instructs, which is done in full at once.
```

The second, on `3,18,33,48 * * * *`, is the listing:

```text
REMINDER: When no work is in hand, run gh issue list --state open --limit 10000 --json number,title,labels --jq 'map(select(all(.labels[]; .name != "needs decision"))) | sort_by(.number) | map({number, title, labels: [.labels[].name]})' to list the open issues no label holds back, lowest number first.
```

The other four:

| Cron | Prompt |
| --- | --- |
| `5,20,35,50 * * * *` | `REMINDER: Take as the work in hand the lowest-numbered issue of the listing for which gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state' prints no open, alone or with other issues of the listing that you choose to solve with it.` |
| `7,22,37,52 * * * *` | `REMINDER: When no work is in hand and no issue of the listing can be taken, end the autopilot: CronDelete every job that CronList returns.` |
| `9,24,39,54 * * * *` | `REMINDER: Continue autonomously: while an open issue can be taken, do not stop to ask or to report.` |
| `11,26,41,56 * * * *` | `REMINDER: When an issue in hand cannot go on without a person's answer, hand it to a person by labelling it 'needs decision'.` |

In a prompt, `<number>` stands for the issue being checked, and `gh api` fills in `{owner}` and `{repo}` from the repository the session runs in.

An issue can be taken when the listing names it and no open issue blocks it. A listing that names no issue has none that can be taken.

After a push the session does nothing else until every run the push started has ended, per `.claude/memories/the-work-waits-for-ci.md`, while a background command watches each run, per `.claude/memories/every-run-is-watched.md`. What the user instructs is the one exception, per `.claude/memories/what-the-user-instructs-is-done.md`: the session does it in full during the wait, and a change it makes is committed and pushed, with the runs of that push watched as well. The reminder on `1,16,31,46 * * * *` says so. Another reminder that fires during that wait, to take work or to go on, does not end it: the session ends its turn again.

## Ending itself

The autopilot ends itself when nothing is left for it to do: no work is in hand, and no issue can be taken. That is so when no issue is open, and when each open issue carries a label that holds it back or is blocked by an open issue.

Left running, the reminders would wake the session every few minutes for nothing, until the session ends or their thirty days are up.

The reminder on `7,22,37,52 * * * *` says so every fifteen minutes, and the session does not wait for it: the moment it finds nothing left, it ends the autopilot.

1. Run the listing's command, so that what is found is how the issues stand now. When it names an issue that can be taken after all, take it and do not end.
2. Run "Stop".
3. Tell the user that the autopilot has ended and why: that no issue is open, or which label or blocking issue holds back each open one.

The watches of the runs a push started are background commands and not reminders, so they go on. A watch that ends still resumes the session, and a run found to have failed is still fixed, per `.claude/memories/a-failed-run-is-fixed-before-other-work.md`.

The autopilot does not start again by itself. An issue opened later, or a label taken off, is picked up when the user starts the autopilot again.

## Rules that are memories

Each file is under `.claude/memories/`.

| Rule | Memory |
| --- | --- |
| What the user instructs is done in full and at once, whatever a memory or reminder says. | `what-the-user-instructs-is-done.md` |
| The work in hand is one issue or several. | `the-work-in-hand-can-be-several-issues.md` |
| An issue is handed to a person only after a check that it needs one, and its body says what is to be decided. | `before-an-issue-is-labelled-needs-decision.md` |
| An issue is brought up to date before it is worked on. | `an-issue-is-brought-up-to-date-before-work.md` |
| An issue documents one indivisible problem. | `one-indivisible-problem-per-issue.md` |
| An issue too large for one context is split into issues that each stand alone. | `issues-are-sized-to-one-context.md` |
| An issue states a functional requirement, seen from outside `cladenc`. | `an-issue-states-a-functional-requirement.md` |
| An issue about a memory, a reminder or the skill carries the label `memories` or `autopilot`. | `issues-about-the-sessions-rules-are-labelled.md` |
| An issue has no fixed form and owes no section that other issues carry. | `issues-have-no-fixed-form.md` |
| A `blocked_by` edge is written only where the block is real. | `a-blocked-by-edge-states-a-real-block.md` |
| The work is clean-room. | `the-work-is-clean-room.md` |
| The tests of a change are written before its code. | `tests-are-written-before-code.md` |
| The code is written to pass the checks of the release workflow. | `the-checks-the-code-faces.md` |
| A command that can take more than about a minute runs in the background. | `long-running-commands-in-the-background.md` |
| A commit closes at least one issue, with a `Closes #N` line for each. | `a-commit-closes-at-least-one-issue.md` |
| A push carries exactly one commit, which may hold several things at once. | `one-commit-per-push.md` |
| After a push, nothing else is done until every run it started has ended, except what the user instructs. | `the-work-waits-for-ci.md` |
| Every run a push starts is watched to its end in the background. | `every-run-is-watched.md` |
| A run found to have failed is fixed in the next commit pushed, before new work begins. | `a-failed-run-is-fixed-before-other-work.md` |
| After a push, the issues its commit closes are checked to be closed. | `a-push-can-leave-its-issues-open.md` |

## Start

Each `--skip-label <label>` puts `map(select(all(.labels[]; .name != "<label>"))) |` and a space at the very start of the listing's `--jq` program, and appends `An issue labelled '<label>' is left to a person, whatever else it carries.` to the listing's reminder. The filter stays in the `--jq` program and never moves to `--label` or `--search`, which the sibling repositories found to send the listing through GitHub's search and its limit of 1000 issues. A `--skip-label` that names `needs decision` changes nothing, since the listing leaves that label out as it stands.

An issue is blocked while `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by` lists an open issue.

1. Make sure the label the autopilot leaves to a person exists: when `gh label list --search 'needs decision' --json name --jq '.[].name'` does not print `needs decision`, create it with `gh label create 'needs decision' --description 'Left to a person' --color d876e3`.
2. Run the listing's command once, its `--skip-label` filters in place. When it names no issue that can be taken, say so, say which label or blocking issue holds back each open issue, and stop: there is nothing to schedule reminders for.
3. Call `CronList`, and `CronDelete` each job whose prompt begins `REMINDER:` and is not one of the six reminders on its own cron. That clears a reminder left from before one was changed or moved to another minute.
4. `CronCreate` with `recurring: true` each reminder that is not already scheduled.
5. Tell the user how many reminders are running, that the autopilot ends itself when no issue is left that it can take, and the two limits that come with the reminders: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after thirty days.
6. Read in full each memory that "Rules that are memories" names.
7. Take the lowest-numbered listed issue that no open issue blocks, alone or with other listed issues chosen to be solved with it, and solve that work. When no work is in hand and no issue can be taken, end the autopilot, per "Ending itself".

## Reminders only

`start reminders-only` schedules only the reminder whose rule holds during any work, with the loop or without it: the wait for CI, on `1,16,31,46 * * * *`. The other five drive the loop through the issues, so they are left out. The form takes no `--skip-label`, runs no listing, takes no issue, and never ends itself, since the reminder that ends the autopilot is one of the five left out: "Stop" clears it.

1. Call `CronList`, and `CronDelete` each job whose prompt begins `REMINDER:` and is not one of the six reminders on its own cron.
2. `CronCreate` the wait for CI with `recurring: true` unless it is already scheduled.
3. Tell the user how many reminders are running, that they last until "Stop" is run, and the two limits that come with them: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after thirty days.

## Restart

Run "Stop", then the form of start that `restart` names: "Start" with the `--skip-label` flags that `restart` was given, or "Reminders only" for `restart reminders-only`. That leaves exactly the reminders of the form, six or one, each with thirty fresh days. It is how a changed set of `--skip-label` flags takes effect, and how the session switches from one form to the other.

## Stop

`CronDelete` every job `CronList` returns, then call `CronList` again to confirm it is empty and report how many jobs were deleted. A `CronList` that returns nothing is not a failure; say the schedule was already empty.
