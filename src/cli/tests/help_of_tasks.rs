use crate::cli::Cli;
use clap::CommandFactory;

fn help_of_start(long: bool) -> String {
    let mut cladenc = Cli::command();
    let start = cladenc.find_subcommand_mut("start").unwrap();
    let help = if long {
        start.render_long_help()
    } else {
        start.render_help()
    };
    help.to_string()
}

#[test]
fn the_long_help_of_start_says_what_the_harness_of_an_attempt_is_given_of_its_task() {
    assert!(help_of_start(true).contains(
        "/var/lib/cladenc/work holds a copy of the workspace of the task, \
         and\nCLADENC_INSTRUCTION holds the task specification."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_harness_can_neither_read_nor_change_the_grader() {
    assert!(help_of_start(true).contains("so it can neither read nor change the grader."));
}

#[test]
fn the_long_help_of_start_shows_how_the_grader_of_a_task_is_called() {
    assert!(
        help_of_start(true)
            .contains("\n    sh -c <TIMER> /var/lib/cladenc/task/grader <SECONDS>\n")
    );
}

#[test]
fn the_long_help_of_start_names_what_a_grader_is_given() {
    let help = help_of_start(true);
    let given = [
        "  /var/lib/cladenc/work  holds what the harness left, and is the working\n",
        "  /var/lib/cladenc/task  holds the directory of the task, to read and not\n",
        "  CLADENC_TASK           /var/lib/cladenc/task\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_how_an_attempt_ends_decides_the_result() {
    assert!(help_of_start(true).contains("\nHow an attempt ends decides its result:\n"));
}

#[test]
fn the_long_help_of_start_says_what_each_way_an_attempt_ends_gives() {
    let help = help_of_start(true);
    let endings = [
        "  the grader exits with status 0   the harness completed the task\n",
        "  the grader exits with another    the harness did not complete the task\n",
        "  exit status 125 of either step   Docker did not start the container",
        "  the time limit                   the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_long_help_of_start_says_that_the_time_limit_gives_not_completed_recorded_as_timed_out() {
    assert!(help_of_start(true).contains(
        "harness did not complete the task,\n                                   which is \
         recorded as timed_out\n"
    ));
}

#[test]
fn the_long_help_of_start_names_what_the_outputs_directory_keeps_of_an_attempt() {
    let help = help_of_start(true);
    let kept = [
        "  result         one word: completed, not_completed or timed_out\n",
        "  specification  the task specification\n",
        "  work           the workspace of the task as the harness left it\n",
        "  stdout         the standard output of the harness\n",
        "  stderr         the standard error of the harness\n",
        "  grader         the standard output and the standard error of the grader",
    ];
    assert!(kept.iter().all(|line| help.contains(line)));
}

#[test]
fn the_short_help_of_start_names_the_flag_that_names_the_benchmark() {
    assert!(help_of_start(false).contains("--benchmark <DIR>"));
}

#[test]
fn the_long_help_of_start_says_that_a_benchmark_holds_a_directory_for_each_task() {
    assert!(
        help_of_start(true)
            .contains("A benchmark is a directory that holds one directory for each task.")
    );
}

#[test]
fn the_long_help_of_start_names_what_the_directory_of_a_task_holds() {
    let help = help_of_start(true);
    let held = [
        "  specification  a file of text, the task specification, which says what\n",
        "  workspace      a directory of the files the work starts from",
        "  grader         an executable file, the program that says by its exit\n",
    ];
    assert!(held.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_the_tasks_are_copied_into_the_outputs_directory() {
    assert!(help_of_start(true).contains(
        "The tasks are copied into\nbenchmarks in the outputs directory when a directed evolution \
         is started,\neach into a directory named after its benchmark"
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_id_of_a_task_is_the_name_of_its_directory() {
    assert!(help_of_start(true).contains(
        "The name\nof a task's directory is the id of the task, made of letters, digits, dots,\n\
         underscores and hyphens"
    ));
}

#[test]
fn the_short_help_of_start_says_that_the_benchmark_flag_is_given_once_for_each_benchmark() {
    assert!(help_of_start(false).contains("given once for each benchmark"));
}

#[test]
fn the_long_help_of_start_says_what_a_directed_evolution_given_several_benchmarks_looks_for() {
    assert!(help_of_start(true).contains(
        "A\ndirected evolution that is given several has each harness benchmarked on\nall of \
         them, and looks for the one harness that completes the most of\ntheir holdout tasks."
    ));
}

#[test]
fn the_long_help_of_start_says_that_every_holdout_task_counts_the_same() {
    assert!(help_of_start(true).contains(
        "Every holdout task counts the same, so a benchmark\nwith more of them weighs more."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_holdout_tasks_are_counted_and_no_others() {
    assert!(help_of_start(true).contains("cladenc counts the holdout tasks and no others."));
}

#[test]
fn the_long_help_of_start_says_by_which_tasks_the_harness_that_procreates_is_chosen() {
    assert!(help_of_start(true).contains(
        "It chooses which harness\nprocreates by the holdout tasks that the harness and its \
         descendants\ncompleted and did not complete,"
    ));
}

#[test]
fn the_long_help_of_start_says_which_harness_is_named_the_best() {
    assert!(help_of_start(true).contains(
        "it names as the best harness the\ncreated harness, benchmarked in full, that \
         completed the most holdout\ntasks: of several that completed as many, the one created \
         first."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_tasks_that_are_not_held_out_are_for() {
    assert!(help_of_start(true).contains(
        "The\ntasks that are not held out are what a harness reads to find what to\nchange, \
         and a harness that completes more of those alone is not chosen\nfor it."
    ));
}

#[test]
fn the_long_help_of_start_says_how_a_task_of_a_directed_evolution_is_known() {
    assert!(help_of_start(true).contains(
        "a task\nof a directed evolution is known by the two together, as\n\n    \
         <BENCHMARK>/<TASK>\n\nso two benchmarks may each hold a task of the same id."
    ));
}

#[test]
fn the_long_help_of_start_says_that_two_benchmarks_of_the_same_name_are_refused() {
    assert!(
        help_of_start(true)
            .contains("Two benchmarks whose\ndirectories have the same name are refused.")
    );
}

#[test]
fn the_short_help_of_start_names_the_flag_that_gives_the_holdout_percentage() {
    assert!(help_of_start(false).contains("--holdout-percentage <NUMBER>"));
}

#[test]
fn the_short_help_of_start_says_which_numbers_the_holdout_percentage_takes() {
    assert!(help_of_start(false).contains("above 0 and below 100"));
}

#[test]
fn the_long_help_of_start_says_what_the_holdout_percentage_gives() {
    assert!(help_of_start(true).contains(
        "--holdout-percentage gives the part of the tasks of each benchmark that is\nheld out, \
         as so many in a hundred: a number above 0 and below 100, such as\n60 or 12.5."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_holdout_tasks_are() {
    assert!(
        help_of_start(true)
            .contains("The holdout tasks are the tasks that no progenitor reads\nabout.")
    );
}

#[test]
fn the_long_help_of_start_says_that_the_holdout_tasks_are_drawn_out_of_each_benchmark_by_itself() {
    assert!(help_of_start(true).contains(
        "cladenc draws them at random when a directed evolution is started,\nout of each \
         benchmark by itself:"
    ));
}

#[test]
fn the_long_help_of_start_says_how_many_tasks_of_a_benchmark_are_held_out() {
    assert!(help_of_start(true).contains(
        "the number of its tasks times the\npercentage, divided by 100 and rounded to the \
         nearest whole number, a half\nrounded up. At 60, a benchmark of 500 tasks has 300 \
         holdout tasks."
    ));
}

#[test]
fn the_long_help_of_start_says_which_holdout_percentage_is_refused() {
    assert!(help_of_start(true).contains(
        "A value\nthat leaves a benchmark no holdout task, or no task that is not held \
         out,\nis refused, and so is a benchmark of one task, which cannot be divided."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_holdout_tasks_are_drawn_once_and_kept_in_the_journal() {
    assert!(help_of_start(true).contains(
        "The draw is made once for a directed evolution and written in its journal,\nand \
         `cladenc resume` reads it there and draws nothing."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_harness_is_benchmarked_on_the_holdout_tasks_too() {
    assert!(help_of_start(true).contains(
        "Every harness is benchmarked on the holdout tasks as on the others, and\nnothing a \
         harness is given in an attempt says whether the task is held\nout."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_harness_that_procreates_is_given_no_holdout_task() {
    assert!(help_of_start(true).contains(
        "A harness that procreates is given its attempts at the tasks that are\nnot held out, \
         and nothing of the holdout tasks: no task specification, no\nworkspace, no result, \
         and no count of them."
    ));
}

#[test]
fn the_long_help_of_start_says_which_attempts_the_directory_named_attempts_keeps() {
    assert!(help_of_start(true).contains(
        "attempts in the outputs directory keeps every attempt at a task that is not\nheld out,"
    ));
}

#[test]
fn the_long_help_of_start_says_where_the_attempts_at_the_holdout_tasks_are_kept() {
    assert!(help_of_start(true).contains(
        "holdout in the outputs directory keeps the attempts at the holdout tasks in\nthe same \
         way, apart from the others, so that the directory a harness that\nprocreates is given \
         holds none of them."
    ));
}

#[test]
fn the_long_help_of_start_says_where_the_directory_of_an_attempt_is() {
    assert!(help_of_start(true).contains(
        "in a directory named after the harness, in it one named after the\nbenchmark, and in \
         that one named after the task, which holds"
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_completed_task_means_the_work_only_as_far_as_three_things() {
    assert!(help_of_start(true).contains(
        "A task that is recorded as completed means that the harness did the work\nonly as far \
         as three things hold, and cladenc sees to none of them."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_harness_reaches_the_network() {
    assert!(help_of_start(true).contains(
        "A harness reaches the network. Every container is started on the usual\nnetwork of \
         Docker, and reaches the machine cladenc runs on and whatever\nthat machine reaches."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_harness_that_reaches_the_network_can_do() {
    assert!(help_of_start(true).contains(
        "Where a task was made from something public, its\nanswer may be one fetch away, and a \
         harness can leave something outside\nits container for a later harness to find, which \
         holding tasks out does\nnot guard against."
    ));
}

#[test]
fn the_long_help_of_start_says_how_a_harness_is_kept_from_reaching_further_than_the_model() {
    assert!(help_of_start(true).contains(
        "To keep every harness from reaching further than the\nmodel, carry out the directed \
         evolution on a machine that has no other\nconnection, with the model served on that \
         machine."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_grader_may_trust_what_the_harness_left() {
    assert!(help_of_start(true).contains(
        "A grader may trust what the harness left. The grader is started in the\nworkspace as \
         the harness left it, and everything there was the harness's\nto change: tests, build \
         files, any program the grader starts from there."
    ));
}

#[test]
fn the_long_help_of_start_says_which_grader_a_harness_that_changed_the_tests_passes() {
    assert!(help_of_start(true).contains(
        "A grader that runs the tests it finds in the workspace is passed by a\nharness that \
         changed those tests, on a holdout task as on any other."
    ));
}

#[test]
fn the_long_help_of_start_says_which_grader_cannot_be_passed_that_way() {
    assert!(help_of_start(true).contains(
        "The\ndirectory of the task is never given to a harness: a grader that keeps\nwhat it \
         relies on there, its tests among them, and takes nothing from the\nworkspace on \
         trust, cannot be passed that way."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_model_may_know_the_task() {
    assert!(help_of_start(true).contains(
        "The model may know the task. A task taken from a public source may have\nbeen among \
         what the model was trained on. A harness then completes it from\nwhat the model \
         remembers, and a child that completes more of such tasks\nmay only be better at \
         drawing that out."
    ));
}
