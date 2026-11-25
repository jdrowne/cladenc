use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_that_the_task_toml_of_a_task_is_the_task_configuration_of_harbor() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The task.toml of a task is the task configuration of Harbor. cladenc obeys what it says \
         of the environment of the task:"
    ));
}

#[test]
fn the_manual_of_start_names_each_setting_of_a_task_toml_that_cladenc_obeys() {
    let help = manual_on_one_line("cladenc-start");
    let settings = [
        "docker_image under environment the image that is the environment, in",
        "workdir under environment the working directory of the harness",
        "env under environment variables that the container is given,",
        "user under agent the user the harness runs as, a name or",
        "user under verifier the user the verifier runs as, in the",
        "env under verifier variables that the verifier alone is",
    ];
    assert!(settings.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_that_cladenc_obeys_nothing_else_of_a_task_toml() {
    assert!(
        manual_on_one_line("cladenc-start").contains("cladenc obeys nothing else of a task.toml.")
    );
}

#[test]
fn the_manual_of_start_says_what_a_task_may_ask_for_that_cladenc_has_no_way_to_give() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task is refused as well where its task.toml asks for what cladenc has no way to give: \
         several steps, as [[steps]]; a GPU, as gpus under environment above 0, or a TPU, as tpu; \
         Windows, as os; MCP servers, as mcp_servers; a skills directory, as skills_dir; a path or \
         a command of a container other than that of the harness, as service of an entry under \
         artifacts or under verifier.collect that is not main; or the whole container of the \
         harness in the verifier container, as an entry under artifacts whose path names no \
         file or directory, where the verifier runs separately. The refusal names the task and \
         what it asks for."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_setting_that_is_left_out_changes_nothing() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A setting that is left out, or written with nothing between its quotation marks, changes \
         nothing. The"
    ));
}

#[test]
fn the_manual_of_start_says_to_whom_what_cladenc_copies_into_the_container_belongs() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The source code of the harness and /logs/artifacts belong to the user the harness runs \
         as, and the copy of the tests directory and /logs/verifier belong to the user the \
         verifier runs as."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_name_in_braces_stands_for_in_the_value_of_a_variable() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "In the value of a variable, ${NAME} stands for the value that the variable NAME has where \
         cladenc runs, and ${NAME:-TEXT} for that value, or for TEXT where NAME is not set there."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_variable_cladenc_sets_keeps_its_value() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A variable that cladenc sets for a harness, such as CLADENC_MODEL, keeps the value \
         cladenc gives it."
    ));
}

#[test]
fn the_manual_of_start_says_when_the_values_are_read_and_that_the_journal_holds_none() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc reads the values when the directed evolution is started, and again each time it \
         is resumed, and writes none of them in the journal."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_values_are_on_no_command_line() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "It keeps them off every command line: Docker reads them from its standard input, one line \
         for each variable, so the name of a variable is made of letters, digits, dots, \
         underscores and hyphens, and its value is one line of text."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_task_without_a_valid_task_toml_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task is refused when the directed evolution is started where it holds no task.toml, or \
         where its task.toml is not valid as the task configuration of Harbor."
    ));
}

#[test]
fn the_manual_of_start_says_what_makes_a_task_toml_not_valid() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task.toml is not valid where it is not TOML, where it gives a setting a value of \
         another kind than the setting takes, or a value that is not one of those the setting \
         takes, where a table lacks a key it has to hold, such as command under \
         environment.healthcheck, or where it says under verifier that the verifier shares the \
         environment of the harness, as environment_mode, and gives the verifier an environment of \
         its own, as environment."
    ));
}

#[test]
fn the_manual_of_start_says_which_settings_of_a_task_toml_cladenc_checks() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc checks every setting that Harbor's documentation of the task configuration names, \
         those it does not obey among them, and lets through a setting that the documentation does \
         not name."
    ));
}

#[test]
fn the_manual_of_start_says_which_variable_of_a_task_toml_has_its_task_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task is also refused where its task.toml names a variable otherwise, or takes a value \
         from a variable that is not set where cladenc runs and gives no text in its place."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_verifier_runs_as_the_user_the_task_toml_names_for_it() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "as the same user, unless the task.toml of the task names a user for the verifier, as"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_task_that_names_an_image_has_it_as_its_environment() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task whose task.toml names an image, as docker_image under environment, has that image \
         as its environment: cladenc builds nothing for the task, and the task needs no \
         environment directory."
    ));
}

#[test]
fn the_manual_of_start_shows_how_docker_is_asked_for_the_image_a_task_names() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("docker image inspect --format {{.Id}} <IMAGE>")
    );
}

#[test]
fn the_manual_of_start_says_that_an_image_that_is_not_on_the_machine_stops_with_exit_status_2() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "and a task whose image is not on the machine stops the directed evolution with exit \
         status 2. cladenc start then leaves it no journal."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_task_that_names_an_image_needs_no_dockerfile() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task whose task.toml names an image needs neither the Dockerfile nor the environment \
         directory."
    ));
}

#[test]
fn the_manual_of_resume_says_that_it_reads_the_task_toml_of_each_task_again() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "cladenc resume reads the task.toml of the copy of each task again, and the variables it \
         names as they are set where cladenc resume runs."
    ));
}

#[test]
fn the_manual_of_resume_says_what_of_a_task_toml_stops_it_with_exit_status_2() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "A task whose image is not there, or which takes a value from a variable that is not set, \
         stops cladenc resume with exit status 2."
    ));
}

#[test]
fn the_manual_of_start_names_the_time_limits_of_a_task_toml_that_cladenc_obeys() {
    let help = manual_on_one_line("cladenc-start");
    let settings = [
        "timeout_sec under agent the time limit of the harness, in seconds",
        "timeout_sec under verifier the time limit of the verifier, in seconds",
        "build_timeout_sec under environment the time limit of the build of the environment, in \
         seconds",
    ];
    assert!(settings.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_what_time_limit_the_harness_of_a_trial_has() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The harness has the time limit that the task.toml of the task gives, as timeout_sec under \
         agent, and no time limit where it gives none, as in Harbor."
    ));
}

#[test]
fn the_manual_of_start_says_that_what_a_stopped_harness_left_is_judged_all_the_same() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A harness that runs past its time limit is stopped, and every process it left running is \
         ended, as by kill -9 -1. What it left is judged all the same."
    ));
}

#[test]
fn the_manual_of_start_says_what_time_limit_the_verifier_has_and_from_when_it_is_counted() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The verifier has a time limit of its own, timeout_sec under verifier, or 600 seconds \
         where the task gives none, counted from the moment the harness ended or was stopped."
    ));
}

#[test]
fn the_manual_of_start_says_that_no_flag_gives_a_time_limit_of_a_trial() {
    assert!(manual_on_one_line("cladenc-start").contains("No flag gives a time limit of a trial."));
}

#[test]
fn the_manual_of_start_says_that_a_container_with_no_time_limit_does_not_end_itself() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where a task gives its harness no time limit, the container of a trial does not end \
         itself: a cladenc that was killed outright leaves it running until cladenc resume removes \
         it, or a person does."
    ));
}

#[test]
fn the_manual_of_start_says_how_long_the_container_of_a_trial_sleeps() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<SECONDS> the time limit of the harness and the time limit of the verifier together, the \
         longest the health check of the task can take, and ten minutes more, for what cladenc \
         copies in and out"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_time_limit_that_is_not_above_0_is_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A time limit that is not a number of seconds above 0 is refused in the same way, and"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_trial_keeps_whether_its_harness_was_stopped() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "ending one word: ended, or stopped where the harness ran past its time limit specification"
    ));
}

#[test]
fn the_manual_of_start_names_the_cpus_and_the_memory_of_a_task_toml_that_cladenc_obeys() {
    let help = manual_on_one_line("cladenc-start");
    let settings = [
        "cpus under environment the most CPUs the container of an",
        "memory_mb under environment the most memory that container is",
    ];
    assert!(settings.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_that_a_trial_is_given_no_more_than_its_task_asks_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The container of a trial is given no more CPUs and no more memory than its task asks \
         for, as --cpus and --memory of docker run, so the harness and the verifier have them \
         between them."
    ));
}

#[test]
fn the_manual_of_start_says_how_the_older_spelling_of_the_memory_is_read() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The older spelling of the memory, memory under environment, a number and one of G, M and \
         K, such as 4G, is read as Harbor reads it, and a task that gives both spellings with \
         sizes that differ is refused."
    ));
}

#[test]
fn the_manual_of_start_shows_how_docker_is_asked_what_the_machine_has() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("docker info --format '{{.NCPU}} {{.MemTotal}}'")
    );
}

#[test]
fn the_manual_of_start_says_that_a_task_that_asks_for_more_than_the_machine_has_stops_it() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "and a task that asks for more CPUs or more memory than that stops the directed evolution \
         with exit status 2, when it is started and each time it is resumed, with the task named."
    ));
}

#[test]
fn the_manual_of_start_says_that_cpus_and_memory_that_are_not_above_0_are_refused() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "is refused in the same way, and so are CPUs or megabytes of memory that are not a number \
         above 0."
    ));
}
