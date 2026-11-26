use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_what_the_harness_of_a_trial_is_given_of_its_task() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("CLADENC_INSTRUCTION holds the instruction of the task. The")
    );
}

#[test]
fn the_manual_of_start_says_that_a_trial_is_carried_out_in_the_environment_of_its_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "and cladenc carries it out in two steps, in one sandbox of the environment of the task, \
         started as"
    ));
}

#[test]
fn the_manual_of_start_says_where_the_source_code_of_the_harness_of_a_trial_is() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "First the source code of the harness is copied into that sandbox, at \
         /var/opt/cladenc/harness,"
    ));
}

#[test]
fn the_manual_of_start_says_as_whom_and_where_the_harness_of_a_trial_runs() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "in the working directory of the environment and as its user: the user and the working \
         directory its Dockerfile names, and root and / where it names none."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_verifier_runs_where_and_as_whom_the_harness_ran() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "its verifier is run there, in the same working directory and as the same user,"
    ));
}

#[test]
fn the_manual_of_start_says_that_the_harness_can_neither_read_nor_change_the_verifier() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("so it can neither read nor change the verifier.")
    );
}

#[test]
fn the_manual_of_start_shows_how_the_verifier_of_a_task_is_called() {
    assert!(manual_on_one_line("cladenc-start").contains("sh -c <TIMER> /tests/test.sh <SECONDS>"));
}

#[test]
fn the_manual_of_start_names_what_a_verifier_is_given() {
    let help = manual_on_one_line("cladenc-start");
    let given = [
        "/tests a symbolic link to /var/opt/cladenc/harbor/tests, which",
        "holds a copy of the tests directory of the task, and",
        "nothing that the harness left there",
        "/logs/verifier an empty directory, for the verdict of the verifier",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_names_what_cladenc_makes_before_the_harness_of_a_trial_is_started() {
    let help = manual_on_one_line("cladenc-start");
    let made = [
        "change the verifier. Before the harness is started, cladenc makes",
        "/logs a symbolic link to /var/log/cladenc/harbor",
        "/logs/artifacts an empty directory, for what the harness saves to be",
        "kept with the trial",
    ];
    assert!(made.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_that_cladenc_makes_no_directory_at_the_top_of_the_filesystem() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "/logs and /tests are the two names that Harbor's task layout fixes at the top of the \
         filesystem. cladenc makes no directory there, as the Filesystem Hierarchy Standard asks \
         of an application: each of the two is a symbolic link."
    ));
}

#[test]
fn the_manual_of_start_says_where_the_two_links_point_and_why() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "/logs is a link to /var/log/cladenc/harbor, since /var/log/cladenc holds the records \
         cladenc keeps, and /tests is a link to /var/opt/cladenc/harbor/tests, since \
         /var/opt/cladenc holds what cladenc gives a sandbox to work with."
    ));
}

#[test]
fn the_manual_of_start_says_where_logs_artifacts_and_logs_verifier_are() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "So /logs/artifacts is /var/log/cladenc/harbor/artifacts, and /logs/verifier is \
         /var/log/cladenc/harbor/verifier."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_harness_finds_nothing_of_the_tests() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "While the harness runs, the sandbox holds neither /tests nor the directory that /tests \
         is a link to."
    ));
}

#[test]
fn the_manual_of_start_says_what_cladenc_does_where_the_environment_holds_logs() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the image of the environment already holds something at /logs, cladenc makes no \
         link and leaves what is there, and makes /logs/artifacts and /logs/verifier inside it."
    ));
}

#[test]
fn the_manual_of_start_says_that_what_was_left_at_tests_is_removed_before_the_link_is_made() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "What the image or the harness left at /tests is removed before the link is made."
    ));
}

#[test]
fn the_manual_of_start_says_where_the_verifier_writes_its_verdict() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The verifier writes its verdict in /logs/verifier/reward.txt, a single number, or in \
         /logs/verifier/reward.json, an object of labelled numbers, which is read in place of \
         reward.txt where both are there."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_exit_status_of_the_verifier_decides_nothing_more() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("The exit status of the verifier decides nothing more.")
    );
}

#[test]
fn the_manual_of_start_says_that_no_sandbox_is_given_the_solution_of_a_task() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("such as solution, which no sandbox is ever given.")
    );
}

#[test]
fn the_manual_of_start_says_that_how_a_trial_ends_decides_the_result() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("How a trial ends decides its result, which is pass, fail or inconclusive:")
    );
}

#[test]
fn the_manual_of_start_says_what_each_way_a_trial_ends_gives() {
    let help = manual_on_one_line("cladenc-start");
    let endings = [
        "a verdict of 1, or 1 for every label of reward.json the result is pass: the harness \
         completed the task",
        "any other verdict, or a reward.json in which a verdict is not 1 the result is fail: the \
         harness did not complete the task",
        "no verdict, or a file that cannot be read as one the result is inconclusive, and its \
         cause is recorded as no_verdict",
        "exit status 125 of either step or of the health check Docker did not start the sandbox: \
         the directed evolution stops",
        "a health check of the task, or of the verifier sandbox, that passes in none of its \
         tries no result is recorded: the directed evolution stops, and cladenc resume carries out \
         the trial again",
        "/logs/verifier, where it cannot be copied out of the sandbox the verdict is not known, \
         and no result is recorded: the directed evolution stops, and cladenc resume carries out \
         the trial again",
        "the time limit of the verifier the sandbox is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_manual_of_start_says_that_the_time_limit_of_the_verifier_gives_an_inconclusive_trial() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "the time limit of the verifier the sandbox is removed, the result is inconclusive, and \
         its cause is recorded as timed_out"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_trial_has_one_of_three_results() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("A trial so has one of three results: pass, fail or inconclusive.")
    );
}

#[test]
fn the_manual_of_start_says_that_a_trial_fails_only_for_a_verdict_that_is_not_1() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A trial fails only where the verifier judged what the harness left and wrote a verdict \
         that is not 1."
    ));
}

#[test]
fn the_manual_of_start_says_what_is_not_known_of_an_inconclusive_trial() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where a trial is inconclusive, nothing judged what the harness left, and whether the \
         harness completed the task is not known."
    ));
}

#[test]
fn the_manual_of_start_says_which_files_cannot_be_read_as_a_verdict() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A file cannot be read as a verdict where reward.txt holds anything but one number, \
         where reward.json holds anything but an object of one labelled number or more, and \
         where either is not a regular file."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_cause_of_no_verdict_leaves_unknown() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the cause of an inconclusive trial is no_verdict, the verifier ran to its end \
         and wrote no verdict that can be read: the verifier may have broken by itself, or what \
         the harness left may have broken it, and cladenc has no way to tell which."
    ));
}

#[test]
fn the_manual_of_start_says_where_a_person_reads_why_a_verifier_left_no_verdict() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("What the verifier printed and left is kept with the trial.")
    );
}

#[test]
fn the_manual_of_start_says_what_cladenc_does_where_the_copy_of_logs_verifier_fails() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Once the verifier has ended, cladenc copies /logs/verifier out of the sandbox and reads \
         the verdict in the copy. Where the copy fails, the verdict is not known, so cladenc does \
         not record that the verifier left no verdict: the directed evolution stops with exit \
         status 1, and no result is recorded for the trial. cladenc then names the harness and \
         the task, and says what Docker said of the copy."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_that_removed_logs_verifier_left_no_verdict() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the verifier removed /logs/verifier, or put something that is not a directory in \
         its place, there is nothing to copy: the verifier left no verdict, and the cause of the \
         inconclusive trial is recorded as no_verdict."
    ));
}

#[test]
fn the_manual_of_start_says_that_start_says_which_result_a_trial_had() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "At the end of a trial, cladenc start says on standard error whether the trial passed, \
         failed or was inconclusive, and of an inconclusive trial whether the verifier left no \
         verdict that can be read or ran past its time limit."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_three_results_are_counted_apart() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc report and cladenc status count the trials that passed, those that failed and \
         those that were inconclusive apart, each in a column of its own."
    ));
}

#[test]
fn the_manual_of_start_says_that_an_inconclusive_trial_is_a_finished_trial() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "An inconclusive trial is a finished trial all the same: a completion rate is the part \
         of the finished trials that passed, and artificial selection goes by the finished \
         trials of a clade and by how many of them passed, so a harness gains nothing by \
         leaving what breaks a verifier."
    ));
}

#[test]
fn the_manual_of_start_names_what_the_outputs_directory_keeps_of_a_trial() {
    let help = manual_on_one_line("cladenc-start");
    let kept = [
        "result one word: pass, fail or inconclusive",
        "cause one word, there only where the result is inconclusive: no_verdict where the \
         verifier left no verdict that can be read, or timed_out where it ran past its time \
         limit",
        "instruction the instruction of the task",
        "work what the harness left in the working directory of the",
        "environment, not kept where that directory is /",
        "changes every path the harness added, changed or removed in the",
        "sandbox, one to a line after A, C or D",
        "stdout the standard output of the harness",
        "stderr the standard error of the harness",
        "verifier the standard output and the standard error of the verifier,",
        "in stdout and stderr, and in logs/verifier what the verifier",
    ];
    assert!(kept.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_that_a_benchmark_holds_a_directory_for_each_task() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("A benchmark is a directory that holds one directory for each task.")
    );
}

#[test]
fn the_manual_of_start_names_what_the_directory_of_a_task_holds() {
    let help = manual_on_one_line("cladenc-start");
    let held = [
        "instruction.md a file of text, the instruction of the task, which says",
        "task.toml a file of text, the task configuration of Harbor, which",
        "tests a directory that holds test.sh, an executable file, the",
        "environment a directory that holds a file named Dockerfile, which",
    ];
    assert!(held.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_that_the_environment_of_a_task_holds_the_files_the_work_starts_from() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "describes the environment of the task, the image its trials are carried out in with the \
         files the work starts from,"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_task_that_lacks_one_of_the_four_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains("a task that lacks one of the four,"));
}

#[test]
fn the_manual_of_start_says_what_the_task_toml_of_a_task_is_for() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("which says how a trial on the task is carried out tests")
    );
}

#[test]
fn the_manual_of_start_says_that_a_task_whose_environment_holds_a_compose_file_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "or whose environment directory holds a Compose file, such as docker-compose.yaml: an \
         environment of more than one container cannot be built."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_tasks_are_copied_into_the_outputs_directory() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The tasks are copied into benchmarks in the outputs directory when a directed evolution \
         is started, each into a directory named after its benchmark"
    ));
}

#[test]
fn the_manual_of_start_says_that_the_id_of_a_task_is_the_name_of_its_directory() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The name of a task's directory is the id of the task, made of letters, digits, dots, \
         underscores and hyphens"
    ));
}

#[test]
fn the_manual_of_start_says_what_a_directed_evolution_given_several_benchmarks_looks_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A directed evolution that is given several has each harness benchmarked on all of them, \
         and looks for the one harness that completes the most of their selection tasks."
    ));
}

#[test]
fn the_manual_of_start_says_that_every_selection_task_counts_the_same() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Every selection task counts the same, so a benchmark with more of them weighs more."
    ));
}

#[test]
fn the_manual_of_start_says_that_artificial_selection_goes_by_the_selection_tasks_alone() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("Artificial selection goes by the selection tasks and by no others.")
    );
}

#[test]
fn the_manual_of_start_says_by_which_tasks_the_harness_that_is_cloned_is_chosen() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc chooses which harness is cloned by the finished trials of the harness and its \
         descendants on the selection tasks, and by how many of those trials passed,"
    ));
}

#[test]
fn the_manual_of_start_says_which_harness_is_named_the_best() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "it names as the best harness the created harness, benchmarked in full, that completed the \
         most selection tasks: of several that completed as many, the one created first."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_non_holdout_tasks_are_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The non-holdout tasks are what a harness reads to find what to change, and a harness that \
         completes more of those alone is not chosen for it."
    ));
}

#[test]
fn the_manual_of_start_says_how_a_task_of_a_directed_evolution_is_known() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "a task of a directed evolution is known by the two together, as <BENCHMARK>/<TASK> so two \
         benchmarks may each hold a task of the same id."
    ));
}

#[test]
fn the_manual_of_start_says_that_two_benchmarks_of_the_same_name_are_refused() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("Two benchmarks whose directories have the same name are refused.")
    );
}

#[test]
fn the_manual_of_start_says_what_the_selection_tasks_percentage_gives() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "--selection-tasks-percentage gives the part of the tasks of each benchmark that are \
         selection tasks, as so many in a hundred: a number above 0 and below 100, such as 45 or \
         12.5."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_holdout_tasks_are() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("The holdout tasks are the tasks that no progenitor reads about")
    );
}

#[test]
fn the_manual_of_start_says_that_the_holdout_tasks_are_drawn_out_of_each_benchmark_by_itself() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc draws the selection tasks and the corroboration tasks at random when a directed \
         evolution is started, out of each benchmark by itself:"
    ));
}

#[test]
fn the_manual_of_start_says_how_many_tasks_of_a_benchmark_are_held_out() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "the number of its tasks times the percentage, divided by 100 and rounded to the nearest \
         whole number, a half rounded up. At 45 and 15, a benchmark of 500 tasks has 225 selection \
         tasks, 75 corroboration tasks and 200 non-holdout tasks."
    ));
}

#[test]
fn the_manual_of_start_says_which_selection_tasks_percentage_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A --selection-tasks-percentage that leaves a benchmark no selection task is refused,"
    ));
}

#[test]
fn the_manual_of_start_says_that_the_holdout_tasks_are_drawn_once_and_kept_in_the_journal() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The draw is made once for a directed evolution and written in its journal, and cladenc \
         resume reads it there and draws nothing."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_harness_is_benchmarked_on_the_holdout_tasks_too() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Every harness is benchmarked on the holdout tasks as on the non-holdout tasks, selection \
         tasks and corroboration tasks alike, and nothing a harness is given in a trial says \
         which of the sets its task is in."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_harness_that_diversifies_a_clone_is_given_no_holdout_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A harness that diversifies a clone is given the trials of the progenitor on the \
         non-holdout tasks, and nothing of the holdout tasks: no instruction, no work, no result, \
         and no count of them."
    ));
}

#[test]
fn the_manual_of_start_says_which_trials_the_directory_named_trials_keeps() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "trials in the outputs directory keeps every trial on a task that is not held out,"
    ));
}

#[test]
fn the_manual_of_start_says_where_the_trials_on_the_holdout_tasks_are_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "holdout in the outputs directory keeps the trials on the holdout tasks, selection tasks \
         and corroboration tasks together, in the same way, apart from the others, so that the \
         directory a harness that diversifies a clone is given holds none of them."
    ));
}

#[test]
fn the_manual_of_start_says_where_the_directory_of_a_trial_is() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "in a directory named after the harness, in it one named after the benchmark, and in that \
         one named after the task, which holds"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_pass_means_the_work_only_as_far_as_three_things() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A trial that is recorded as pass means that the harness did the work only as far as \
         three things hold, and cladenc sees to none of them."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_harness_reaches_the_network() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A harness reaches the network. Every sandbox is started on the usual network of Docker, \
         and reaches the machine cladenc runs on and whatever that machine reaches."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_harness_that_reaches_the_network_can_do() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where a task was made from something public, its answer may be one fetch away, and a \
         harness can leave something outside its sandbox for a later harness to find, which \
         holding tasks out does not guard against."
    ));
}

#[test]
fn the_manual_of_start_says_how_a_harness_is_kept_from_reaching_further_than_the_model() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "To keep every harness from reaching further than the model, carry out the directed \
         evolution on a machine that has no other connection, with the model served on that \
         machine."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_may_trust_what_the_harness_left() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A verifier may trust what the harness left. The verifier is started in the sandbox as \
         the harness left it, and everything there was the harness's to change: tests, build \
         files, any program the verifier starts from the working directory, and the processes the \
         harness left running."
    ));
}

#[test]
fn the_manual_of_start_says_which_verifier_a_harness_that_changed_the_tests_passes() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A verifier that runs the tests it finds in the working directory is passed by a harness \
         that changed those tests, on a holdout task as on any other."
    ));
}

#[test]
fn the_manual_of_start_says_which_harness_can_pass_a_verifier_that_trusts_nothing_it_left() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The tests directory of the task is never given to a harness: a verifier that keeps what \
         it relies on there, and takes nothing from the working directory on trust, can be passed \
         that way only by a harness that left a process running to change the copy of that \
         directory, or the verdict, once they are made."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_model_may_know_the_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The model may know the task. A task taken from a public source may have been among what \
         the model was trained on. A harness then completes it from what the model remembers, and \
         a child that completes more of such tasks may only be better at drawing that out."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_corroboration_tasks_percentage_gives() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "--corroboration-tasks-percentage gives the part that are corroboration tasks, in the same \
         way."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_selection_and_corroboration_tasks_are_the_holdout_tasks() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The selection tasks and the corroboration tasks together are the holdout tasks."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_non_holdout_tasks_are() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("the tasks that are not held out are the non-holdout tasks.")
    );
}

#[test]
fn the_manual_of_start_says_which_corroboration_tasks_percentage_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "and so is a --corroboration-tasks-percentage that leaves a benchmark no corroboration task"
    ));
}

#[test]
fn the_manual_of_start_says_which_two_percentages_together_are_refused() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("and so are two values that together leave a benchmark no non-holdout task.")
    );
}

#[test]
fn the_manual_of_start_says_that_a_benchmark_of_fewer_than_three_tasks_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "So a benchmark holds three tasks at the least, and a benchmark of fewer is refused."
    ));
}

#[test]
fn the_manual_of_start_says_that_nothing_is_chosen_by_the_corroboration_tasks() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc counts for every harness the corroboration tasks it completed, and chooses \
         nothing by them: not the harness that is cloned, and never the best harness."
    ));
}

#[test]
fn the_manual_of_start_says_when_a_set_of_tasks_gives_a_fair_count() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("A set of tasks gives a fair count only while nothing is chosen by it.")
    );
}
