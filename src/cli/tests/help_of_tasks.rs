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
    assert!(
        help_of_start(true).contains(" CLADENC_INSTRUCTION holds the task specification. The\n")
    );
}

#[test]
fn the_long_help_of_start_says_that_an_attempt_is_carried_out_in_the_environment_of_its_task() {
    assert!(help_of_start(true).contains(
        "and cladenc carries it\nout in two steps, in one container of the environment of the \
         task, started\nas\n"
    ));
}

#[test]
fn the_long_help_of_start_says_where_the_source_code_of_the_harness_of_an_attempt_is() {
    assert!(help_of_start(true).contains(
        "First the source code of the harness is copied into that container, at\n\
         /var/lib/cladenc/harness,"
    ));
}

#[test]
fn the_long_help_of_start_says_as_whom_and_where_the_harness_of_an_attempt_runs() {
    assert!(help_of_start(true).contains(
        "in the working directory of the environment and as its user:\nthe user and the \
         working directory its Dockerfile names, and root and /\nwhere it names none."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_grader_runs_where_and_as_whom_the_harness_ran() {
    assert!(
        help_of_start(true)
            .contains("grader is run there, in the same working directory and as the same user,\n")
    );
}

#[test]
fn the_long_help_of_start_says_that_the_harness_can_neither_read_nor_change_the_grader() {
    assert!(help_of_start(true).contains("so it can neither read nor\nchange the grader."));
}

#[test]
fn the_long_help_of_start_shows_how_the_grader_of_a_task_is_called() {
    assert!(help_of_start(true).contains("\n    sh -c <TIMER> /tests/test.sh <SECONDS>\n"));
}

#[test]
fn the_long_help_of_start_names_what_a_grader_is_given() {
    let help = help_of_start(true);
    let given = [
        "  /tests          holds a copy of the tests directory of the task, and\n",
        "                  nothing that the harness left there\n",
        "  /logs/verifier  an empty directory, for the score of the grader\n",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_where_the_grader_writes_its_score() {
    assert!(help_of_start(true).contains(
        "The grader writes its score in /logs/verifier/reward.txt, a single number,\nor in \
         /logs/verifier/reward.json, an object of labelled numbers, which is\nread in place of \
         reward.txt where both are there."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_exit_status_of_the_grader_decides_nothing_more() {
    assert!(
        help_of_start(true).contains("\nThe exit status of the grader decides nothing more.\n")
    );
}

#[test]
fn the_long_help_of_start_says_that_no_container_is_given_the_solution_of_a_task() {
    assert!(help_of_start(true).contains("such as solution, which no\ncontainer is ever given."));
}

#[test]
fn the_long_help_of_start_says_that_how_an_attempt_ends_decides_the_result() {
    assert!(help_of_start(true).contains("\nHow an attempt ends decides its result:\n"));
}

#[test]
fn the_long_help_of_start_says_what_each_way_an_attempt_ends_gives() {
    let help = help_of_start(true);
    let endings = [
        "  a score of 1, or 1 for every     the harness completed the task\n",
        "  any other score, a file that     the harness did not complete the task\n",
        "  exit status 125 of either step   Docker did not start the container",
        "  the time limit of the grader     the container is removed",
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
        "  work           what the harness left in the working directory of the\n",
        "                 environment, not kept where that directory is /\n",
        "  changes        every path the harness added, changed or removed in the\n",
        "                 container, one to a line after A, C or D\n",
        "  stdout         the standard output of the harness\n",
        "  stderr         the standard error of the harness\n",
        "  grader         the standard output and the standard error of the grader,\n",
        "                 in stdout and stderr, and in logs/verifier what the grader\n",
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
        "  instruction.md  a file of text, the task specification, which says what\n",
        "  task.toml       a file of text, the task configuration of Harbor, which\n",
        "  tests           a directory that holds test.sh, an executable file, the\n",
        "  environment     a directory that holds a file named Dockerfile, which\n",
    ];
    assert!(held.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_the_environment_of_a_task_holds_the_files_the_work_starts_from()
{
    assert!(help_of_start(true).contains(
        "describes the environment of the task, the image its\n                  attempts are \
         carried out in with the files the work\n                  starts from,"
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_task_that_lacks_one_of_the_four_is_refused() {
    assert!(help_of_start(true).contains("a task that lacks one of the four,"));
}

#[test]
fn the_long_help_of_start_says_what_the_task_toml_of_a_task_is_for() {
    assert!(
        help_of_start(true)
            .contains("which\n                  says how the task is to be attempted\n  tests  ")
    );
}

#[test]
fn the_long_help_of_start_says_that_a_task_whose_environment_holds_a_compose_file_is_refused() {
    assert!(help_of_start(true).contains(
        "or whose environment directory holds a\nCompose file, such as docker-compose.yaml: an \
         environment of more than one\ncontainer cannot be built."
    ));
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
        "A\ndirected evolution that is given several has each harness benchmarked on\nall of them, \
         and looks for the one harness that completes the most of their\nselection tasks."
    ));
}

#[test]
fn the_long_help_of_start_says_that_every_selection_task_counts_the_same() {
    assert!(help_of_start(true).contains(
        "Every selection task counts the same, so a benchmark with\nmore of them weighs more."
    ));
}

#[test]
fn the_long_help_of_start_says_that_artificial_selection_goes_by_the_selection_tasks_alone() {
    assert!(
        help_of_start(true)
            .contains("Artificial selection goes by the selection tasks and by no others.")
    );
}

#[test]
fn the_long_help_of_start_says_by_which_tasks_the_harness_that_procreates_is_chosen() {
    assert!(help_of_start(true).contains(
        "cladenc\nchooses which harness procreates by the selection tasks that the harness\nand \
         its descendants completed and did not complete,"
    ));
}

#[test]
fn the_long_help_of_start_says_which_harness_is_named_the_best() {
    assert!(help_of_start(true).contains(
        "it names as the\nbest harness the created harness, benchmarked in full, that completed \
         the\nmost selection tasks: of several that completed as many, the one created\nfirst."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_non_holdout_tasks_are_for() {
    assert!(help_of_start(true).contains(
        "The non-holdout tasks are what a harness reads to find\nwhat to change, and a \
         harness that completes more of those alone is not\nchosen for it."
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
fn the_short_help_of_start_names_the_flag_that_gives_the_selection_tasks_percentage() {
    assert!(help_of_start(false).contains("--selection-tasks-percentage <NUMBER>"));
}

#[test]
fn the_short_help_of_start_says_which_numbers_the_selection_tasks_percentage_takes() {
    assert!(help_of_start(false).contains("above 0 and below 100"));
}

#[test]
fn the_long_help_of_start_says_what_the_selection_tasks_percentage_gives() {
    assert!(help_of_start(true).contains(
        "--selection-tasks-percentage gives the part of the tasks of each benchmark\nthat are \
         selection tasks, as so many in a hundred: a number above 0 and\nbelow 100, such as 45 or \
         12.5."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_holdout_tasks_are() {
    assert!(
        help_of_start(true)
            .contains("The holdout tasks\nare the tasks that no progenitor reads about")
    );
}

#[test]
fn the_long_help_of_start_says_that_the_holdout_tasks_are_drawn_out_of_each_benchmark_by_itself() {
    assert!(help_of_start(true).contains(
        "cladenc draws the selection tasks and\nthe corroboration tasks at random when a directed \
         evolution is started, out\nof each benchmark by itself:"
    ));
}

#[test]
fn the_long_help_of_start_says_how_many_tasks_of_a_benchmark_are_held_out() {
    assert!(help_of_start(true).contains(
        "the number of its tasks times the percentage,\ndivided by 100 and rounded to the nearest \
         whole number, a half rounded up.\nAt 45 and 15, a benchmark of 500 tasks has 225 \
         selection tasks, 75\ncorroboration tasks and 200 non-holdout tasks."
    ));
}

#[test]
fn the_long_help_of_start_says_which_selection_tasks_percentage_is_refused() {
    assert!(help_of_start(true).contains(
        "A --selection-tasks-percentage that leaves a benchmark no selection task is\nrefused,"
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_holdout_tasks_are_drawn_once_and_kept_in_the_journal() {
    assert!(help_of_start(true).contains(
        "The draw is made once for a\ndirected evolution and written in its journal, and `cladenc \
         resume` reads\nit there and draws nothing."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_harness_is_benchmarked_on_the_holdout_tasks_too() {
    assert!(help_of_start(true).contains(
        "Every harness is benchmarked on the holdout tasks as on the non-holdout\ntasks, \
         selection tasks and corroboration tasks alike, and nothing a harness\nis given in an \
         attempt says which of the sets its task is in."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_harness_that_procreates_is_given_no_holdout_task() {
    assert!(help_of_start(true).contains(
        "A harness\nthat procreates is given its attempts at the non-holdout tasks, and \
         nothing\nof the holdout tasks: no task specification, no work, no result, and no\ncount \
         of them."
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
        "holdout in the outputs directory keeps the attempts at the holdout tasks,\nselection \
         tasks and corroboration tasks together, in the same way, apart\nfrom the others, so that \
         the directory a harness that procreates is given\nholds none of them."
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
        "A grader may trust what the harness left. The grader is started in the\ncontainer as \
         the harness left it, and everything there was the harness's\nto change: tests, build \
         files, any program the grader starts from the\nworking directory, and the processes \
         the harness left running."
    ));
}

#[test]
fn the_long_help_of_start_says_which_grader_a_harness_that_changed_the_tests_passes() {
    assert!(help_of_start(true).contains(
        "A grader\nthat runs the tests it finds in the working directory is passed by a\nharness \
         that changed those tests, on a holdout task as on any other."
    ));
}

#[test]
fn the_long_help_of_start_says_which_harness_can_pass_a_grader_that_trusts_nothing_it_left() {
    assert!(help_of_start(true).contains(
        "The\ntests directory of the task is never given to a harness: a grader that\nkeeps what \
         it relies on there, and takes nothing from the working\ndirectory on trust, can be \
         passed that way only by a harness that left a\nprocess running to change the copy of \
         that directory, or the score, once\nthey are made."
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

#[test]
fn the_long_help_of_start_says_what_the_corroboration_tasks_percentage_gives() {
    assert!(help_of_start(true).contains(
        "--corroboration-tasks-percentage gives the\npart that are corroboration tasks, in the \
         same way."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_selection_and_corroboration_tasks_are_the_holdout_tasks() {
    assert!(help_of_start(true).contains(
        "The selection tasks and\nthe corroboration tasks together are the holdout tasks."
    ));
}

#[test]
fn the_long_help_of_start_says_what_the_non_holdout_tasks_are() {
    assert!(
        help_of_start(true).contains("the tasks that are not\nheld out are the non-holdout tasks.")
    );
}

#[test]
fn the_long_help_of_start_says_which_corroboration_tasks_percentage_is_refused() {
    assert!(help_of_start(true).contains(
        "and so is a --corroboration-tasks-percentage that leaves a\nbenchmark no corroboration \
         task"
    ));
}

#[test]
fn the_long_help_of_start_says_which_two_percentages_together_are_refused() {
    assert!(
        help_of_start(true).contains(
            "and so are two values that together leave\na benchmark no non-holdout task."
        )
    );
}

#[test]
fn the_long_help_of_start_says_that_a_benchmark_of_fewer_than_three_tasks_is_refused() {
    assert!(help_of_start(true).contains(
        "So a benchmark holds three tasks at the\nleast, and a benchmark of fewer is refused."
    ));
}

#[test]
fn the_long_help_of_start_says_that_nothing_is_chosen_by_the_corroboration_tasks() {
    assert!(help_of_start(true).contains(
        "cladenc counts for every harness the corroboration tasks it completed, and\nchooses \
         nothing by them: not the harness that procreates, and never the\nbest harness."
    ));
}

#[test]
fn the_long_help_of_start_says_when_a_set_of_tasks_gives_a_fair_count() {
    assert!(
        help_of_start(true)
            .contains("A set of tasks gives a fair count only while nothing is\nchosen by it.")
    );
}

#[test]
fn the_short_help_of_start_names_the_flag_that_gives_the_corroboration_tasks_percentage() {
    assert!(help_of_start(false).contains("--corroboration-tasks-percentage <NUMBER>"));
}

#[test]
fn the_short_help_of_start_says_what_the_corroboration_tasks_percentage_gives() {
    assert!(
        help_of_start(false)
            .contains("The percentage of each benchmark's tasks that are corroboration tasks")
    );
}

#[test]
fn the_short_help_of_start_says_what_the_selection_tasks_percentage_gives() {
    assert!(
        help_of_start(false)
            .contains("The percentage of each benchmark's tasks that are selection tasks")
    );
}
