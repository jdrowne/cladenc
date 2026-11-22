use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of_start() -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    start.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_that_a_procreation_finds_its_attempts_by_benchmark() {
    assert!(long_help_of_start().contains(
        "one directory\nfor each benchmark, and in it one for each task, holding what the \
         attempt\nof that task kept."
    ));
}

#[test]
fn the_long_help_of_start_says_where_a_procreation_finds_its_attempts() {
    assert!(long_help_of_start().contains(
        "/var/log/cladenc/attempts holds the finished attempts\nof the harness at the \
         non-holdout tasks, to read:"
    ));
}

#[test]
fn the_long_help_of_start_says_of_which_harnesses_a_harness_that_procreates_is_given_more() {
    assert!(long_help_of_start().contains(
        "A harness that procreates is given more to read, and nothing more to\nchange: what \
         was kept of each harness it descends from, back to the\nprimogenitor, and of each \
         harness it made in an earlier procreation that\nhas ended, whether that one was \
         created or not. It is given nothing of\nany other harness."
    ));
}

#[test]
fn the_long_help_of_start_names_where_the_line_and_the_children_of_a_harness_are() {
    let help = long_help_of_start();
    let given = [
        "  /var/log/cladenc/line/<ID>      a harness it descends from\n",
        "  /var/log/cladenc/children/<ID>  a harness it made before\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_names_what_is_given_of_a_harness_of_the_line_or_a_child() {
    let help = long_help_of_start();
    let given = [
        "  harness      the source code of that harness\n",
        "  attempts     its attempts at the non-holdout tasks, laid out\n",
        "  procreation  what the harness that made it printed, in stdout and\n",
        "               progenitor and one word in the file named outcome:\n",
        "               created, unchanged, failed, build_failed or timed_out\n",
        "  build        what its build printed, in stdout and stderr\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_which_directory_of_such_a_harness_is_not_there() {
    assert!(long_help_of_start().contains(
        "A directory that would hold nothing is not there: the primogenitor has no\nprocreation, \
         a harness that was not created has no attempts, and a harness\nthat was not built has \
         no build. Nothing of a holdout task is in any of\nthem."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_outputs_directory_keeps_of_a_procreation() {
    assert!(long_help_of_start().contains(
        "procreations in the\noutputs directory keeps, in a directory named after the child, \
         the\nstandard output and standard error of the harness that made it, the id of\nthat \
         harness in the file named progenitor, and how the procreation ended\nin the file \
         named outcome."
    ));
}

#[test]
fn the_long_help_of_start_says_what_kind_of_change_the_instruction_asks_for() {
    assert!(long_help_of_start().contains(
        "The instruction asks the harness that procreates for a change to how its\nchild works \
         on any task: one that answers something it saw in several of\nits attempts, not in \
         one, and that leaves in the source code nothing that\nnames a task, a file of a task \
         or the answer to one."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_instruction_counts_taking_out_as_an_improvement() {
    assert!(
        long_help_of_start()
            .contains("It says that taking\nout what does not help is an improvement as well,")
    );
}

#[test]
fn the_long_help_of_start_says_that_the_instruction_tells_of_the_child_procreating_in_its_turn() {
    assert!(long_help_of_start().contains(
        "that the child may be\nasked in its turn to improve a copy of its own source code, so \
         that making\nthe child better at that counts,"
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_instruction_tells_how_the_child_is_judged() {
    assert!(long_help_of_start().contains(
        "and that the child is judged by tasks\nthe harness that procreates is not shown."
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_instruction_is_a_request_cladenc_does_not_enforce() {
    assert!(
        long_help_of_start().contains("An instruction is a request:\ncladenc enforces none of it.")
    );
}

#[test]
fn the_long_help_of_start_says_that_a_harness_procreates_only_once_benchmarked_in_full() {
    assert!(long_help_of_start().contains(
        "A harness procreates only once it has been benchmarked in full, on every\ntask of \
         every benchmark of the directed evolution, so that it finds every\nnon-holdout task \
         in /var/log/cladenc/attempts."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_primogenitor_is_benchmarked_before_the_first_procreation() {
    assert!(
        long_help_of_start()
            .contains("The primogenitor is\nbenchmarked in full before the first procreation.")
    );
}

#[test]
fn the_long_help_of_start_says_that_every_harness_is_benchmarked_in_full() {
    assert!(long_help_of_start().contains(
        "Every harness is benchmarked in full once it is created, whether it\nprocreates or not."
    ));
}

#[test]
fn the_long_help_of_start_says_which_harness_attempts_next() {
    assert!(long_help_of_start().contains(
        "It does not choose which harness attempts next: the created harness\nwith the lowest \
         id that has a task left attempts it."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_most_procreations_counts() {
    assert!(long_help_of_start().contains(
        "--max-procreations gives the most procreations a directed evolution\nstarts. A \
         procreation counts once it is started, whether or not it creates\nits child."
    ));
}

#[test]
fn the_long_help_of_start_says_when_a_directed_evolution_ends() {
    assert!(long_help_of_start().contains(
        "The directed evolution ends once that many have been started,\neach of them has ended, \
         and every created harness has been benchmarked in\nfull."
    ));
}

#[test]
fn the_long_help_of_start_says_when_a_procreation_starts() {
    assert!(long_help_of_start().contains(
        "A procreation starts only once no attempt is left to start, no attempt is\nrunning and \
         no other procreation is under way: every harness created so far\nhas been benchmarked \
         in full, the child of the procreation before among\nthem."
    ));
}

#[test]
fn the_long_help_of_start_says_what_each_harness_that_procreates_is_chosen_with() {
    assert!(long_help_of_start().contains(
        "So cladenc chooses each harness that procreates with what every\ncreated harness \
         completed in hand."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_singleton_control_group_harness_is_not_waited_for() {
    assert!(long_help_of_start().contains(
        "A singleton control group harness is not\nwaited for, since it never procreates: its \
         creation and its attempts go on\nbeside a procreation."
    ));
}

#[test]
fn the_long_help_of_start_says_that_fewer_containers_run_while_a_procreation_runs() {
    assert!(long_help_of_start().contains(
        "While a procreation runs, no container of another\nchild is started, so fewer \
         containers run than --max-concurrent-containers\nallows."
    ));
}

#[test]
fn the_long_help_of_start_names_no_growth() {
    assert!(!long_help_of_start().contains("growth"));
}

#[test]
fn the_long_help_of_start_says_that_the_best_of_harnesses_that_are_alike_is_ahead_by_luck() {
    assert!(long_help_of_start().contains(
        "The best harness is named among many, each benchmarked once, and a model\ndoes not answer \
         the same way twice. So the best of many harnesses that are\nall alike has still completed \
         more selection tasks than the primogenitor,\nby luck."
    ));
}

#[test]
fn the_long_help_of_start_says_how_a_directed_evolution_of_harnesses_that_work_alike_is_carried_out()
 {
    assert!(long_help_of_start().contains(
        "How many more selection tasks the best of many harnesses that are all alike\ncompletes is \
         measured by a directed evolution in which every harness works\nas the primogenitor \
         does: one carried out with the values of the directed\nevolution it is set beside, and \
         with a run command that, where the harness\nis started for a procreation, calls no \
         model and adds one line to a file\nthat no harness reads, as"
    ));
}

#[test]
fn the_long_help_of_start_gives_a_run_command_that_adds_one_line_in_a_procreation() {
    assert!(long_help_of_start().contains(
        "\n    --harness-run-command 'if [ -d /var/log/cladenc/attempts ]\n        then echo >> \
         procreated; else <COMMAND>; fi'\n\nwhere <COMMAND> is the run command of the other, \
         and procreated is a name\nthat no file of the primogenitor has."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_child_made_with_that_run_command_is() {
    assert!(long_help_of_start().contains(
        "/var/log/cladenc/attempts is there for\na procreation and not for an attempt, so every \
         procreation calls no model,\nand creates a child that is the source code of its \
         progenitor with one line\nmore in the file named procreated."
    ));
}

#[test]
fn the_long_help_of_start_says_why_that_run_command_adds_a_line() {
    assert!(long_help_of_start().contains(
        "The line is added because a procreation\nthat changes nothing creates no child."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_directed_evolution_of_harnesses_that_work_alike_holds() {
    assert!(long_help_of_start().contains(
        "Such a directed evolution holds as\nmany children as --max-procreations gives, each \
         of them benchmarked in full."
    ));
}

#[test]
fn the_long_help_of_start_says_how_the_report_of_such_a_directed_evolution_is_read() {
    assert!(long_help_of_start().contains(
        "In `cladenc report` of it, how far apart the counts of the harnesses lie is\nwhat luck \
         does to the count of one harness, and how many more selection\ntasks the best harness \
         completed than the primogenitor is what luck and the\nchoosing give together, with \
         nothing changed in how a harness works."
    ));
}

#[test]
fn the_long_help_of_start_says_how_far_a_best_harness_is_ahead_by_a_change() {
    assert!(long_help_of_start().contains(
        "The\nbest harness of the other directed evolution is ahead of its primogenitor \
         on\nthe selection tasks by a change only as far as it is ahead by more than\nthat."
    ));
}

#[test]
fn the_long_help_of_start_says_that_report_counts_the_tasks_one_harness_at_least_completed() {
    assert!(long_help_of_start().contains(
        "`cladenc report` also says how many tasks at least one harness\ncompleted, of the \
         selection tasks, of the corroboration tasks and of the\nnon-holdout tasks, and \
         `cladenc status` says how many so far."
    ));
}

#[test]
fn the_long_help_of_start_says_how_that_count_is_read_where_every_harness_works_alike() {
    assert!(long_help_of_start().contains(
        "Where every\nharness works as the primogenitor does, that is what one more attempt \
         at\neach task for each child gives the primogenitor, and it is set beside \
         the\ncount of the best harness of the other directed evolution, which had \
         one\nattempt at each task."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_procreation_that_changes_nothing_creates_no_child() {
    assert!(long_help_of_start().contains(
        "A procreation whose harness leaves /var/opt/cladenc/work as it was creates\nno child, \
         and is recorded as unchanged."
    ));
}

#[test]
fn the_long_help_of_start_says_what_is_compared_to_find_a_procreation_unchanged() {
    assert!(long_help_of_start().contains(
        "cladenc compares what the harness\nleft with the source code of that harness, as it \
         is kept under harnesses in\nthe outputs directory. The two are the same where they \
         hold the same files,\ndirectories and symbolic links, each file with the same \
         contents and the\nsame permission bits, and each symbolic link with the same target."
    ));
}

#[test]
fn the_long_help_of_start_says_why_an_unchanged_child_is_not_benchmarked() {
    assert!(long_help_of_start().contains(
        "Such a\nchild would be its progenitor over again, and a model does not answer \
         the\nsame way twice, so what the child completed would differ from what \
         its\nprogenitor completed by luck alone. It is not built and not benchmarked, and\nit \
         is never chosen to procreate."
    ));
}

#[test]
fn the_long_help_of_start_says_that_luck_in_the_choosing_does_not_reach_the_corroboration_tasks() {
    assert!(long_help_of_start().contains(
        "That luck does not reach the corroboration tasks: a harness that\nwas lucky on the \
         selection tasks is no luckier on other tasks, and a change\nthat makes a harness complete \
         more tasks shows on both sets."
    ));
}

#[test]
fn the_long_help_of_start_says_where_the_lead_of_the_best_harness_is_read() {
    assert!(long_help_of_start().contains(
        "So how far the\nbest harness is ahead of the primogenitor is read in the columns of \
         the\ncorroboration tasks of `cladenc report`."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_lead_that_is_corroborated_is_not_proven() {
    assert!(long_help_of_start().contains(
        "A lead there corroborates the lead\non the selection tasks: it came through a test that \
         could have refuted it,\nwhich is not to say that it is proven."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_harness_is_still_ahead_on_the_corroboration_tasks_by_luck() {
    assert!(long_help_of_start().contains(
        "One of two harnesses that are alike\nis still ahead on the corroboration tasks by a few \
         of them, and the fewer\nthe corroboration tasks, the more a few of them weigh."
    ));
}

#[test]
fn the_long_help_of_start_says_which_three_things_report_says_of_the_best_harness() {
    assert!(long_help_of_start().contains(
        "`cladenc report` says three things of the best harness and the\nprimogenitor, counted on \
         the corroboration tasks: how many of them only the\nbest harness completed, how many only \
         the primogenitor completed, and how\nlikely luck alone gives a lead at least as large."
    ));
}

#[test]
fn the_long_help_of_start_says_which_corroboration_tasks_are_left_out_of_the_lead() {
    assert!(long_help_of_start().contains(
        "A corroboration task that\nboth harnesses completed, or that neither completed, says \
         nothing about\nwhich of the two is ahead, and is left out."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_task_only_one_harness_completed_is_like_the_toss_of_a_coin() {
    assert!(long_help_of_start().contains(
        "Where two harnesses are equally\ngood, a task that only one of them completed is as \
         likely the one's as the\nother's, like the toss of a fair coin."
    ));
}

#[test]
fn the_long_help_of_start_says_how_the_likelihood_of_a_lead_is_counted() {
    assert!(long_help_of_start().contains(
        "So the likelihood is that of as many\nheads or more in as many tosses as there are tasks \
         that tell the two apart:\nwith 14 tasks that only the best harness completed and 6 that \
         only the\nprimogenitor completed, 14 heads or more in 20 tosses, which is 0.058,\nabout 6 \
         times in a hundred."
    ));
}

#[test]
fn the_long_help_of_start_says_what_report_says_where_no_lead_is_counted() {
    assert!(long_help_of_start().contains(
        "Where no task tells the two apart the\nlikelihood is 1, and where the best harness is the \
         primogenitor there are\nno two harnesses to compare."
    ));
}

#[test]
fn the_long_help_of_start_says_that_cladenc_draws_no_line_through_the_likelihood() {
    assert!(long_help_of_start().contains(
        "cladenc draws no line through the likelihood:\nwhat counts as more than luck is for the \
         person who reads it to say."
    ));
}

#[test]
fn the_long_help_of_start_says_why_the_likelihood_is_shown_for_the_best_harness_alone() {
    assert!(long_help_of_start().contains(
        "The\nlikelihood is shown for the best harness alone, since among many harnesses\none or \
         two show a low likelihood by luck."
    ));
}
