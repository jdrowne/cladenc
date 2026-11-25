use test_inputs::{manual, manual_on_one_line};

fn run_command_the_manual_gives(command: &str) -> String {
    manual("cladenc-start")
        .split_once("--harness-run-command '")
        .and_then(|(_, after)| after.split_once('\''))
        .map(|(given, _)| given.replace("<COMMAND>", command))
        .unwrap()
}

#[test]
fn the_manual_of_start_says_that_a_procreation_finds_its_trials_by_benchmark() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "one directory for each benchmark, and in it one for each task, holding what the trial on \
         that task kept."
    ));
}

#[test]
fn the_manual_of_start_says_where_a_procreation_finds_its_trials() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "/var/log/cladenc/trials holds the finished trials of the harness on the non-holdout \
         tasks, to read:"
    ));
}

#[test]
fn the_manual_of_start_says_of_which_harnesses_a_harness_that_procreates_is_given_more() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A harness that procreates is given more to read, and nothing more to change: what was \
         kept of each harness it descends from, back to the primogenitor, and of each harness it \
         made in an earlier procreation that has ended, whether that one was created or not. It is \
         given nothing of any other harness."
    ));
}

#[test]
fn the_manual_of_start_names_where_the_line_and_the_children_of_a_harness_are() {
    let help = manual_on_one_line("cladenc-start");
    let given = [
        "/var/log/cladenc/line/<ID> a harness it descends from",
        "/var/log/cladenc/children/<ID> a harness it made before",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_names_what_is_given_of_a_harness_of_the_line_or_a_child() {
    let help = manual_on_one_line("cladenc-start");
    let given = [
        "harness the source code of that harness",
        "trials its trials on the non-holdout tasks, laid out",
        "procreation what the harness that made it printed, in stdout and",
        "progenitor and one word in the file named outcome:",
        "created, unchanged, failed, build_failed or timed_out",
        "build what its build printed, in stdout and stderr",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_which_directory_of_such_a_harness_is_not_there() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A directory that would hold nothing is not there: the primogenitor has no procreation, a \
         harness that was not created has no trials, and a harness that was not built has no \
         build. Nothing of a holdout task is in any of them."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_outputs_directory_keeps_of_a_procreation() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "procreations in the outputs directory keeps, in a directory named after the child, the \
         standard output and standard error of the harness that made it, the id of that harness in \
         the file named progenitor, and how the procreation ended in the file named outcome."
    ));
}

#[test]
fn the_manual_of_start_says_what_kind_of_change_the_instruction_asks_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The instruction asks the harness that procreates for a change to how its child works on \
         any task: one that answers something it saw in several of its trials, not in one, and \
         that leaves in the source code nothing that names a task, a file of a task or the answer \
         to one."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_instruction_counts_taking_out_as_an_improvement() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("It says that taking out what does not help is an improvement as well,")
    );
}

#[test]
fn the_manual_of_start_says_that_the_instruction_tells_of_the_child_procreating_in_its_turn() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "that the child may be asked in its turn to improve a copy of its own source code, so that \
         making the child better at that counts,"
    ));
}

#[test]
fn the_manual_of_start_says_that_the_instruction_tells_how_the_child_is_judged() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "and that the child is judged by tasks the harness that procreates is not shown."
    ));
}

#[test]
fn the_manual_of_start_says_that_an_instruction_is_a_request_cladenc_does_not_enforce() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("An instruction is a request: cladenc enforces none of it.")
    );
}

#[test]
fn the_manual_of_start_says_that_a_harness_procreates_only_once_benchmarked_in_full() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A harness procreates only once it has been benchmarked in full, on every task of every \
         benchmark of the directed evolution, so that it finds every non-holdout task in \
         /var/log/cladenc/trials."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_primogenitor_is_benchmarked_before_the_first_procreation() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("The primogenitor is benchmarked in full before the first procreation.")
    );
}

#[test]
fn the_manual_of_start_says_that_every_harness_is_benchmarked_in_full() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Every harness is benchmarked in full once it is created, whether it procreates or not."
    ));
}

#[test]
fn the_manual_of_start_says_which_harness_trials_next() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "It does not choose which trial starts next: the created harness with the lowest id \
         that has a task left starts a trial on it."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_most_procreations_counts() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "--max-procreations gives the most procreations a directed evolution starts. A procreation \
         counts once it is started, whether or not it creates its child."
    ));
}

#[test]
fn the_manual_of_start_says_when_a_directed_evolution_ends() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The directed evolution ends once that many have been started, each of them has ended, and \
         every created harness has been benchmarked in full."
    ));
}

#[test]
fn the_manual_of_start_says_when_a_procreation_starts() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A procreation starts only once no trial is left to start, no trial is running and no \
         other procreation is under way: every harness created so far has been benchmarked in \
         full, the child of the procreation before among them."
    ));
}

#[test]
fn the_manual_of_start_says_what_each_harness_that_procreates_is_chosen_with() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "So cladenc chooses each harness that procreates with what every created harness completed \
         in hand."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_singleton_control_group_harness_is_not_waited_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A singleton control group harness is not waited for, since it never procreates: its \
         creation and its trials go on beside a procreation."
    ));
}

#[test]
fn the_manual_of_start_says_that_fewer_containers_run_while_a_procreation_runs() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "While a procreation runs, no container of another child is started, so fewer containers \
         run than --max-concurrent-containers allows."
    ));
}

#[test]
fn the_manual_of_start_names_no_growth() {
    assert!(!manual_on_one_line("cladenc-start").contains("growth"));
}

#[test]
fn the_manual_of_start_says_that_the_best_of_harnesses_that_are_alike_is_ahead_by_luck() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The best harness is named among many, each benchmarked once, and a model does not answer \
         the same way twice. So the best of many harnesses that are all alike has still completed \
         more selection tasks than the primogenitor, by luck."
    ));
}

#[test]
fn the_manual_of_start_says_how_a_directed_evolution_of_harnesses_that_work_alike_is_carried_out() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "How many more selection tasks the best of many harnesses that are all alike completes is \
         measured by a directed evolution in which every harness works as the primogenitor does: \
         one carried out with the values of the directed evolution it is set beside, and with a \
         run command that, where the harness is started for a procreation, calls no model and adds \
         one line to a file that no harness reads, as"
    ));
}

#[test]
fn the_manual_of_start_gives_a_run_command_that_adds_one_line_in_a_procreation() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "--harness-run-command 'if [ -d /var/log/cladenc/trials ] then echo >> procreated; else \
         <COMMAND>; fi' where <COMMAND> is the run command of the other, and procreated is a name \
         that no file of the primogenitor has."
    ));
}

#[test]
fn the_run_command_the_manual_gives_starts_the_command_of_the_other_only_for_a_trial() {
    assert_eq!(
        run_command_the_manual_gives("sh harness.sh"),
        "if [ -d /var/log/cladenc/trials ]\n    then echo >> procreated; else sh harness.sh; fi"
    );
}

#[test]
fn the_manual_of_start_says_what_a_child_made_with_that_run_command_is() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "/var/log/cladenc/trials is there for a procreation and not for a trial, so every \
         procreation calls no model, and creates a child that is the source code of its progenitor \
         with one line more in the file named procreated."
    ));
}

#[test]
fn the_manual_of_start_says_why_that_run_command_adds_a_line() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The line is added because a procreation that changes nothing creates no child."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_directed_evolution_of_harnesses_that_work_alike_holds() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Such a directed evolution holds as many children as --max-procreations gives, each of \
         them benchmarked in full."
    ));
}

#[test]
fn the_manual_of_start_says_how_the_report_of_such_a_directed_evolution_is_read() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "In cladenc report of it, how far apart the counts of the harnesses lie is what luck does \
         to the count of one harness, and how many more selection tasks the best harness completed \
         than the primogenitor is what luck and the choosing give together, with nothing changed \
         in how a harness works."
    ));
}

#[test]
fn the_manual_of_start_says_how_far_a_best_harness_is_ahead_by_a_change() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The best harness of the other directed evolution is ahead of its primogenitor on the \
         selection tasks by a change only as far as it is ahead by more than that."
    ));
}

#[test]
fn the_manual_of_start_says_that_report_counts_the_tasks_one_harness_at_least_completed() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc report also says how many tasks at least one harness completed, of the selection \
         tasks, of the corroboration tasks and of the non-holdout tasks, and cladenc status says \
         how many so far."
    ));
}

#[test]
fn the_manual_of_start_says_how_that_count_is_read_where_every_harness_works_alike() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where every harness works as the primogenitor does, that is what one more trial on each \
         task for each child gives the primogenitor, and it is set beside the count of the best \
         harness of the other directed evolution, which had one trial on each task."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_procreation_that_changes_nothing_creates_no_child() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A procreation whose harness leaves /var/opt/cladenc/work as it was creates no child, and \
         is recorded as unchanged."
    ));
}

#[test]
fn the_manual_of_start_says_what_is_compared_to_find_a_procreation_unchanged() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc compares what the harness left with the source code of that harness, as it is \
         kept under harnesses in the outputs directory. The two are the same where they hold the \
         same files, directories and symbolic links, each file with the same contents and the same \
         permission bits, and each symbolic link with the same target."
    ));
}

#[test]
fn the_manual_of_start_says_why_an_unchanged_child_is_not_benchmarked() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Such a child would be its progenitor over again, and a model does not answer the same way \
         twice, so what the child completed would differ from what its progenitor completed by \
         luck alone. It is not built and not benchmarked, and it is never chosen to procreate."
    ));
}

#[test]
fn the_manual_of_start_says_that_luck_in_the_choosing_does_not_reach_the_corroboration_tasks() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "That luck does not reach the corroboration tasks: a harness that was lucky on the \
         selection tasks is no luckier on other tasks, and a change that makes a harness complete \
         more tasks shows on both sets."
    ));
}

#[test]
fn the_manual_of_start_says_where_the_lead_of_the_best_harness_is_read() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "So how far the best harness is ahead of the primogenitor is read in the columns of the \
         corroboration tasks of cladenc report."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_lead_that_is_corroborated_is_not_proven() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A lead there corroborates the lead on the selection tasks: it came through a test that \
         could have refuted it, which is not to say that it is proven."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_harness_is_still_ahead_on_the_corroboration_tasks_by_luck() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "One of two harnesses that are alike is still ahead on the corroboration tasks by a few of \
         them, and the fewer the corroboration tasks, the more a few of them weigh."
    ));
}

#[test]
fn the_manual_of_report_says_which_three_things_report_says_of_the_best_harness() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "cladenc report says three things of the best harness and the primogenitor, counted on the \
         corroboration tasks: how many of them only the best harness completed, how many only the \
         primogenitor completed, and how likely luck alone gives a lead at least as large."
    ));
}

#[test]
fn the_manual_of_report_says_which_corroboration_tasks_are_left_out_of_the_lead() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "A corroboration task that both harnesses completed, or that neither completed, says \
         nothing about which of the two is ahead, and is left out."
    ));
}

#[test]
fn the_manual_of_report_says_that_a_task_only_one_harness_completed_is_like_the_toss_of_a_coin() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "Where two harnesses are equally good, a task that only one of them completed is as likely \
         the one's as the other's, like the toss of a fair coin."
    ));
}

#[test]
fn the_manual_of_report_says_how_the_likelihood_of_a_lead_is_counted() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "So the likelihood is that of as many heads or more in as many tosses as there are tasks \
         that tell the two apart: with 14 tasks that only the best harness completed and 6 that \
         only the primogenitor completed, 14 heads or more in 20 tosses, which is 0.058, about 6 \
         times in a hundred."
    ));
}

#[test]
fn the_manual_of_report_says_what_report_says_where_no_lead_is_counted() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "Where no task tells the two apart the likelihood is 1, and where the best harness is the \
         primogenitor there are no two harnesses to compare."
    ));
}

#[test]
fn the_manual_of_report_says_that_cladenc_draws_no_line_through_the_likelihood() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "cladenc draws no line through the likelihood: what counts as more than luck is for the \
         person who reads it to say."
    ));
}

#[test]
fn the_manual_of_report_says_why_the_likelihood_is_shown_for_the_best_harness_alone() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "The likelihood is shown for the best harness alone, since among many harnesses one or two \
         show a low likelihood by luck."
    ));
}
