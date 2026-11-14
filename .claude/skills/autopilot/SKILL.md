---
name: autopilot
description: Start, restart or stop the autopilot reminders, which keep the session solving the open issues one at a time without being asked. Use when the user says "start autopilot", "go autonomous on the open issues", "restart autopilot" or "stop autopilot", or asks to clear the reminders. Takes "start", "restart" or "stop"; "start" and "restart" also take `--skip-label <label>`, repeatable.
---

# Autopilot

The autopilot is a set of scheduled reminders. Each repeats one standing rule into the session, and together they keep it solving the open issues one at a time, in the order their `blocked by` edges set, until none is left that it can take.

Starting the autopilot is the user's request to commit and push what it solves: each issue's commit goes to `main` without a further go-ahead, per `.claude/memories/push-to-main.md`.

Every part of the work, reading, searching and filing as well as code, is done in the session itself: no Agent or Workflow call takes any part of it, per `.claude/memories/working-without-subagents.md`.

Fetch `CronCreate`, `CronList` and `CronDelete` with `ToolSearch` first.

## Reminders

Each reminder is indivisible: it states one rule and names one action, so no rule can be quietly dropped from a merged block of text. A rule that names two actions is written as two reminders.

Each reminder has its own minute of a twenty-minute cycle, so they arrive one at a time. Minutes 0 and 10 stay empty, which keeps every reminder off the hour and the half hour. A reminder fires only while the session is idle, which is why every wait runs in the background, per `.claude/memories/a-wait-runs-in-the-background.md`.

There are eighteen. The first, on `1,21,41 * * * *`, is the listing:

```text
REMINDER: When no issue is in hand, run gh issue list --state open --limit 10000 --json number,title,labels --jq 'map(select(all(.labels[]; .name != "needs decision"))) | sort_by(.number) | map({number, title, labels: [.labels[].name]})' to list the open issues no label holds back, lowest number first.
```

The other seventeen:

| Cron | Prompt |
| --- | --- |
| `2,22,42 * * * *` | `REMINDER: Take one issue at a time: the lowest-numbered issue of the listing for which gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state' prints no open.` |
| `3,23,43 * * * *` | `REMINDER: Continue autonomously: while an open issue can be taken, do not stop to ask or to report.` |
| `4,24,44 * * * *` | `REMINDER: When the issue in hand cannot go on without a person's answer, hand it to a person by labelling it 'needs decision'.` |
| `5,25,45 * * * *` | `REMINDER: Before labelling an issue 'needs decision', write into its body what a person has to decide.` |
| `6,26,46 * * * *` | `REMINDER: Before labelling an issue 'needs decision', assess it against the memories in .claude/memories/ and the code to determine whether it truly needs a person.` |
| `7,27,47 * * * *` | `REMINDER: Before working on an issue, bring its title and body up to date with the code as it stands.` |
| `8,28,48 * * * *` | `REMINDER: An issue documents one indivisible problem: split one that documents more into one issue per problem, reusing the issue itself as one of the splits.` |
| `9,29,49 * * * *` | `REMINDER: File a problem found outside the issue in hand as an issue of its own.` |
| `11,31,51 * * * *` | `REMINDER: Solve a problem found along the way now only if the issue in hand cannot move forward without it.` |
| `12,32,52 * * * *` | `REMINDER: Write a blocked_by edge only where the block is real: the issue that waits cannot be finished until the other is closed.` |
| `13,33,53 * * * *` | `REMINDER: Keep the work clean-room: never read the authors' code at github.com/metauto-ai/HGM, the Darwin Gödel Machine code, or a port or derivative of either.` |
| `14,34,54 * * * *` | `REMINDER: Write the tests of a change before its code.` |
| `15,35,55 * * * *` | `REMINDER: Let every push carry exactly one commit.` |
| `16,36,56 * * * *` | `REMINDER: Give each issue a commit solves a line of its own reading Closes #N.` |
| `17,37,57 * * * *` | `REMINDER: While any CI run for a commit pushed to this repository is in progress, only wait, in the background: no diagnosis, edits or commits.` |
| `18,38,58 * * * *` | `REMINDER: When a run that this session's push started fails, fix it in a push of its own before taking another issue.` |
| `19,39,59 * * * *` | `REMINDER: Once a push's runs are clean, make sure every issue its commit closes is closed.` |

In a prompt, `<number>` stands for the issue being checked, and `gh api` fills in `{owner}` and `{repo}` from the repository the session runs in.

## Start

Each `--skip-label <label>` puts `map(select(all(.labels[]; .name != "<label>"))) |` and a space at the very start of the listing's `--jq` program, and appends `An issue labelled '<label>' is left to a person, whatever else it carries.` to the listing's reminder. The filter stays in the `--jq` program and never moves to `--label` or `--search`, which the sibling repositories found to send the listing through GitHub's search and its limit of 1000 issues.

An issue is blocked while `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by` lists an open issue.

1. Say which model this session runs on, as its system prompt names it. The issues that build `cladenc` ask, under "Working rules", for the latest Fable model at Extra High effort, and only the user can set either. When the model is another, say so and stop here.
2. Make sure the label the autopilot leaves to a person exists: when `gh label list --search 'needs decision' --json name --jq '.[].name'` does not print `needs decision`, create it with `gh label create 'needs decision' --description 'Left to a person' --color d876e3`.
3. Run the listing's command once, its `--skip-label` filters in place. When it names no issue, say so and stop: there is nothing to schedule reminders for.
4. Call `CronList`, and `CronDelete` each job on one of the eighteen crons whose prompt differs from that cron's reminder.
5. `CronCreate` with `recurring: true` each reminder that is not already scheduled.
6. Tell the user how many reminders are running, and the two limits that come with them: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after seven days.
7. Take the lowest-numbered listed issue that no open issue blocks, and solve it. When no issue can be taken, say which label or blocking issue holds back each open issue and stop.

## Restart

Run "Stop", then "Start" with the `--skip-label` flags that `restart` was given. That leaves exactly the eighteen reminders, each with seven fresh days, and it is how a changed set of `--skip-label` flags takes effect.

## Stop

`CronDelete` every job `CronList` returns, then call `CronList` again to confirm it is empty and report how many jobs were deleted. A `CronList` that returns nothing is not a failure; say the schedule was already empty.

## Writing a blocked_by edge

`gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by -F issue_id=<id>`, where `<number>` is the issue that waits and `<id>` is the blocker's numeric id from `gh api repos/{owner}/{repo}/issues/<n> --jq .id`, not the node id `gh issue view --json id` returns. `-F` sends the id as a number; sent with `-f`, as a string, it is rejected with HTTP 422.
