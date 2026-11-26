---
name: autopilot
description: Start, restart or stop the autopilot reminders, which keep the session solving the open issues, or only the open issues that carry one label, without being asked and end themselves when no issue is left to take, or schedule only the reminders that hold during any work. Use when the user says "start autopilot", "go autonomous on the open issues", "go autonomous on the issues labelled <label>", "restart autopilot", "reminders only", "switch to reminders only", "stop autopilot" or "reminders off", or asks to clear the reminders. Takes "start", "start bylabel <label>", "start reminders-only", the same three after "restart", or "stop"; every "start" and "restart" form but "reminders-only" also takes `--skip-label <label>`, repeatable.
---

# Autopilot

The autopilot is a set of scheduled reminders. Together they keep the session solving the open issues, or with `bylabel` only the open issues that carry one label, per "By label", in the order their `blocked by` edges set, until none is left that it can take. Then the autopilot ends itself, per "Ending itself".

The session works on one issue or several at a time: the first listed issue that no open issue blocks, and any other listed issues it chooses to add, per `.claude/memories/several-issues-can-be-worked-on-together.md`.

Starting the autopilot is the user's request to commit and push what it solves: its work goes to `main` without a further go-ahead, per `.claude/memories/push-to-main.md`. A push carries one commit or several and goes when the session chooses, per `.claude/memories/commits-in-a-push-are-the-sessions-choice.md`, and each push closes at least one issue, per `.claude/memories/a-push-closes-at-least-one-issue.md`.

Every part of the work, reading, searching and filing as well as code, is done in the session itself: no Agent or Workflow call takes any part of it, per `.claude/memories/working-without-subagents.md`.

Fetch `CronCreate`, `CronList` and `CronDelete` with `ToolSearch` first, and with them `TaskCreate` and `TaskUpdate`, which the task list is kept with.

## Reminders

A rule is a reminder only when it tells the session what to do next at the moment it fires: to wait while a CI run is going, which work to take, to bring the task list up to date, to keep going, how to hand an issue over instead of stopping, and to end the autopilot when nothing is left. Every other rule the work is done under holds whether or not the autopilot is running, so it is a memory, listed under "Rules that are memories". Three rules are both: the wait for CI, that no test runs locally, and that the task list holds what is left to do. Each is a memory, since it holds with the autopilot stopped. The first two are reminders since without them the other reminders would draw the session against them: back to work while a run is going, or to running a test locally to check the work it took. The third is a reminder since the user asked for it as one. They are the three reminders of the form `reminders-only`, per "Reminders only".

Before a reminder is added, the test is whether its rule would still hold with the autopilot stopped. When it would, the rule is written as a memory and given a row in "Rules that are memories", and it is a reminder as well only when a reminder that fires would otherwise lead the session against it, as with the wait for CI and with the tests that run in CI alone, or when the user asks for it as a reminder, as with the task list.

Each reminder is indivisible: it states one rule and names one action, so no rule can be quietly dropped from a merged block of text. A rule that names two actions is written as two reminders. The reminder that takes work is one rule, taking the issues in turn: listing them, taking the first, and listing them again once those close are its steps, so the same words hold at the first start and in the middle of work. The reminder about the task list is one rule as well, that the list holds what is left to do: adding a task, marking it, rewording it and deleting it are the steps that bring the list up to date.

Each reminder fires every fifteen minutes, on a minute of its own at least two minutes from the next, and none falls on the hour or the half hour. A reminder fires only while the session is idle, which is why every command that can run long goes in the background, per `.claude/memories/long-running-commands-in-the-background.md`.

There are seven. The first, on `1,16,31,46 * * * *`, is the wait for CI:

```text
REMINDER: While a CI run that a push of this session started is queued or in progress, only wait: no reading, edits, issue writing, commits or pushes, except what the user instructs, which is done in full at once.
```

The second, on `3,18,33,48 * * * *`, is the reminder that takes work:

```text
REMINDER: Run gh issue list --state open --limit 10000 --json number,title,labels --jq 'map(select(all(.labels[]; .name != "needs decision"))) | sort_by((if any(.labels[]; .name == "memories" or .name == "autopilot") then 0 else 1 end), .number) | map({number, title, labels: [.labels[].name]})' to list the open issues no label holds back, those labelled 'memories' or 'autopilot' first and then the rest, each part lowest number first; take the first issue it lists for which gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state' prints no open, alone or with other listed issues you choose to solve with it, and run the same command again when they close.
```

An issue labelled `memories` or `autopilot` changes a rule the session works under, so it is taken before any other: every issue solved after it is solved under the changed rule. Such an issue that an open issue blocks still waits for its blocker, so the first issue taken can be one of the others.

The `bylabel` form puts a reminder of its own that takes work on the same cron, in place of the one above, per "By label".

The other five:

| Cron | Prompt |
| --- | --- |
| `5,20,35,50 * * * *` | `REMINDER: Bring the task list up to date with TaskCreate and TaskUpdate: a piece of work that has come up is a task on it, a task is marked in progress when its work starts and completed when it ends, a task whose work changed is reworded, a task no longer needed is deleted, and so is each completed one, so that the list holds what is left to do.` |
| `7,22,37,52 * * * *` | `REMINDER: When the listing names no issue that can be taken, end the autopilot: CronDelete every job that CronList returns.` |
| `9,24,39,54 * * * *` | `REMINDER: Continue autonomously: while an open issue can be taken, do not stop to ask or to report.` |
| `11,26,41,56 * * * *` | `REMINDER: When an issue you are working on cannot go on without a person's answer, hand it to a person by labelling it 'needs decision'.` |
| `13,28,43,58 * * * *` | `REMINDER: Run no test locally, of the unit, integration or end-to-end tier: push the tests with their code and let CI run them.` |

In a prompt, `<number>` stands for the issue being checked, and `gh api` fills in `{owner}` and `{repo}` from the repository the session runs in.

An issue can be taken when the listing names it and no open issue blocks it. A listing that names no issue has none that can be taken.

After a push the session does nothing else until every run the push started has ended, per `.claude/memories/the-work-waits-for-ci.md`, while a background command watches each run, per `.claude/memories/every-run-is-watched.md`. What the user instructs is the one exception, per `.claude/memories/what-the-user-instructs-is-done.md`: the session does it in full during the wait, and a change it makes is committed and pushed, with the runs of that push watched as well. The reminder on `1,16,31,46 * * * *` says so. Another reminder that fires during that wait, to take work or to go on, does not end it: the session ends its turn again. Nor does the reminder about the task list: the session brings the list up to date, which touches nothing in the repository, and ends its turn again.

## Ending itself

The autopilot ends itself when nothing is left for it to do: no issue can be taken. That is so when no issue is open, and when each open issue carries a label that holds it back or is blocked by an open issue. In the `bylabel` form, only the open issues that carry its label count, per "By label".

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
| The session works on one issue or several at a time. | `several-issues-can-be-worked-on-together.md` |
| An issue is handed to a person only after a check that it needs one, and its body says what is to be decided. | `before-an-issue-is-labelled-needs-decision.md` |
| An issue is brought up to date before it is worked on, but for the body of one the session reopened after a run that passed. | `an-issue-is-brought-up-to-date-before-work.md` |
| An issue documents one indivisible problem, which alone sets its size. | `one-indivisible-problem-per-issue.md` |
| An issue states a functional requirement, seen from outside `cladenc`. | `an-issue-states-a-functional-requirement.md` |
| An issue about a memory, a reminder or the skill carries the label `memories` or `autopilot`. | `issues-about-the-sessions-rules-are-labelled.md` |
| An issue has no fixed form and owes no section that other issues carry. | `issues-have-no-fixed-form.md` |
| An order the session recommends between issues is written as `blocked_by` edges. | `a-blocked-by-edge-sets-the-order-of-the-work.md` |
| The work is clean-room. | `the-work-is-clean-room.md` |
| The tests of a change are written before its code. | `tests-are-written-before-code.md` |
| The goal is a `cladenc` that is right, and CI is a tool that says whether it is: a first push that passes, few runs and little time are not goals. | `ci-says-whether-the-code-is-right.md` |
| No test runs locally and no change is compiled locally to check it: before a push, the eleven checks of the workflows that compile nothing run locally: `assert-no-comments`, `assert-no-inline-directives` and `assert-no-linter-config-files` before every push; `cargo fmt --all`, `assert-no-empty-test-files`, `assert-no-oversized-functions`, `assert-no-oversized-source-files`, `assert-one-assert-per-test` and `assert-one-directory-per-module` for a commit that changes a Rust file; `markdownlint` for a commit that changes a Markdown file; `yamllint` for a commit that changes a workflow or an action. Whether the code compiles, every test, `clippy`, coverage and the package are left to CI. | `only-quick-checks-run-before-a-push.md` |
| A command that can take more than about a minute runs in the background. | `long-running-commands-in-the-background.md` |
| A push closes at least one issue, with a `Closes #N` line for each in the commit that finishes it. | `a-push-closes-at-least-one-issue.md` |
| A push carries one commit or several and goes when the session chooses. | `commits-in-a-push-are-the-sessions-choice.md` |
| After a push, nothing else is done until every run it started has ended, except what the user instructs. | `the-work-waits-for-ci.md` |
| Every run a push starts is watched to its end in the background. | `every-run-is-watched.md` |
| A run found to have failed is fixed in the next push, before new work begins. | `a-failed-run-is-fixed-before-other-work.md` |
| After a push, the issues its commits close are checked to be closed. | `a-push-can-leave-its-issues-open.md` |
| When a run that passed shows that an issue its push closed is not solved, the issue is reopened with a comment that says why, and its body is left as it is. | `a-passed-run-can-show-a-closed-issue-unsolved.md` |
| The task list holds what is left to do, and is brought up to date as the work goes. | `the-task-list-holds-what-is-left-to-do.md` |

## Start

Each `--skip-label <label>` puts `map(select(all(.labels[]; .name != "<label>"))) |` and a space at the very start of the listing's `--jq` program, and appends `An issue labelled '<label>' is left to a person, whatever else it carries.` to the reminder that takes work. The filter stays in the `--jq` program and never moves to `--label` or `--search`, which the sibling repositories found to send the listing through GitHub's search and its limit of 1000 issues. A `--skip-label` that names `needs decision` changes nothing, since the listing leaves that label out as it stands.

An issue is blocked while `gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by` lists an open issue.

1. Make sure the label the autopilot leaves to a person exists: when `gh label list --search 'needs decision' --json name --jq '.[].name'` does not print `needs decision`, create it with `gh label create 'needs decision' --description 'Left to a person' --color d876e3`.
2. Run the listing's command once, its `--skip-label` filters in place. When it names no issue that can be taken, say so, say which label or blocking issue holds back each open issue, and stop: there is nothing to schedule reminders for.
3. Call `CronList`, and `CronDelete` each job whose prompt begins `REMINDER:` and is not one of the form's seven reminders on its own cron. That clears a reminder left from before one was changed or moved to another minute, and a reminder that takes work for another form.
4. `CronCreate` with `recurring: true` each of the form's reminders that is not already scheduled.
5. Tell the user how many reminders are running, that the autopilot ends itself when no issue is left that it can take, and the two limits that come with the reminders: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after thirty days.
6. Read in full each memory that "Rules that are memories" names.
7. Take the first listed issue that no open issue blocks, alone or with other listed issues chosen to be solved with it, and solve them. When they close, run the listing's command again. When it names no issue that can be taken, end the autopilot, per "Ending itself".

## By label

`start bylabel <label>` is "Start" held to the open issues that carry `<label>`. The autopilot takes those issues the way "Start" takes every open issue, but lowest number first, with no place ahead for an issue labelled `memories` or `autopilot`, and an open issue without the label is not its work. The form takes `--skip-label` as "Start" does.

Its seven reminders are those of "Start", but for the reminder that takes work on `3,18,33,48 * * * *`, which is this one, with the label itself put in place of each `<label>` when it is scheduled:

```text
REMINDER: Run gh issue list --state open --limit 10000 --json number,title,labels --jq 'map(select(any(.labels[]; .name == "<label>"))) | map(select(all(.labels[]; .name != "needs decision"))) | sort_by(.number) | map({number, title, labels: [.labels[].name]})' to list the open issues labelled '<label>' that no other label holds back, lowest number first; take the lowest-numbered issue it lists for which gh api repos/{owner}/{repo}/issues/<number>/dependencies/blocked_by --jq '.[].state' prints no open, alone or with other listed issues you choose to solve with it, and run the same command again when they close. An open issue without the label '<label>' is not this loop's work.
```

`<number>` stays as written, for the session to fill with the issue it checks.

The filter that keeps the label, `map(select(any(.labels[]; .name == "<label>"))) |`, stands in the `--jq` program ahead of the filter that leaves out `needs decision`, and never moves to `--label` or `--search`, for the reason "Start" gives for `--skip-label`. Each `--skip-label` still goes at the very start of the program, ahead of the filter that keeps the label.

An open issue that blocks a labelled issue holds it back whether it carries the label or not. The autopilot does not take a blocker without the label: the labelled issue waits until a person, or another form of the autopilot, closes it.

A `bylabel` that names `needs decision`, or a label that one of its `--skip-label` flags names too, lists no issue, so step 2 of "Start" finds nothing to take and schedules nothing.

The steps are those of "Start", with this form's listing in steps 2 and 7 and this form's seven reminders in steps 3 and 4. Step 3 deletes the reminder that takes work of "Start", or of `bylabel` with another label, since it is not one of this form's seven.

The autopilot ends itself, per "Ending itself", once no open issue that carries the label can be taken. In step 3 of "Ending itself" it tells the user that no open issue carries the label, or which label or blocking issue holds back each open issue that does.

## Reminders only

`start reminders-only` schedules only the reminders whose rules hold during any work, with the loop or without it: the wait for CI, on `1,16,31,46 * * * *`, the reminder about the task list, on `5,20,35,50 * * * *`, and the reminder that no test runs locally, on `13,28,43,58 * * * *`. The other four drive the loop through the issues, so they are left out. The form takes no `--skip-label`, runs no listing, takes no issue, and never ends itself, since the reminder that ends the autopilot is one of the four left out: "Stop" clears it.

1. Call `CronList`, and `CronDelete` each job whose prompt begins `REMINDER:` and is neither one of the seven reminders of "Start" nor the reminder that takes work of "By label", whatever its label, each on its own cron.
2. `CronCreate` each of the three reminders with `recurring: true` unless it is already scheduled.
3. Tell the user how many reminders are running, that they last until "Stop" is run, and the two limits that come with them: the jobs live in this session only and are gone when it ends, and recurring jobs auto-expire after thirty days.

## Restart

Run "Stop", then the form of start that `restart` names: "Start" with the `--skip-label` flags that `restart` was given, "By label" with its label and `--skip-label` flags for `restart bylabel <label>`, or "Reminders only" for `restart reminders-only`. That leaves exactly the reminders of the form, seven or three, each with thirty fresh days. It is how a changed set of `--skip-label` flags or a changed label takes effect, and how the session switches from one form to another.

## Stop

`CronDelete` every job `CronList` returns, then call `CronList` again to confirm it is empty and report how many jobs were deleted. A `CronList` that returns nothing is not a failure; say the schedule was already empty.
