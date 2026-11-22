use crate::cli::Cli;
use clap::CommandFactory;

fn long_help_of(sub_command: &str) -> String {
    let mut cladenc = Cli::command();
    let sub_command = cladenc.find_subcommand_mut(sub_command).unwrap();
    sub_command.render_long_help().to_string()
}

#[test]
fn the_long_help_of_start_says_that_a_task_may_hold_the_task_configuration_of_harbor() {
    assert!(long_help_of("start").contains(
        "A task may hold a file named task.toml, the task configuration of Harbor.\ncladenc \
         obeys what it says of the environment of the task:\n"
    ));
}

#[test]
fn the_long_help_of_start_names_each_setting_of_a_task_toml_that_cladenc_obeys() {
    let help = long_help_of("start");
    let settings = [
        "  docker_image under environment  the image that is the environment, in\n",
        "  workdir under environment       the working directory of the harness\n",
        "  env under environment           variables that the container is given,\n",
        "  user under agent                the user the harness runs as, a name or\n",
        "  user under verifier             the user the grader runs as, in the\n",
        "  env under verifier              variables that the grader alone is\n",
    ];
    assert!(settings.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_that_cladenc_reads_nothing_else_of_a_task_toml() {
    assert!(long_help_of("start").contains("cladenc reads nothing else of a task.toml."));
}

#[test]
fn the_long_help_of_start_says_that_a_setting_that_is_left_out_changes_nothing() {
    assert!(long_help_of("start").contains(
        "A setting that is left out, or\nwritten with nothing between its quotation marks, \
         changes nothing, and\nneither does a task that holds no task.toml."
    ));
}

#[test]
fn the_long_help_of_start_says_to_whom_what_cladenc_copies_into_the_container_belongs() {
    assert!(long_help_of("start").contains(
        "The source code of the\nharness belongs to the user the harness runs as, and /tests \
         and\n/logs/verifier belong to the user the grader runs as."
    ));
}

#[test]
fn the_long_help_of_start_says_what_a_name_in_braces_stands_for_in_the_value_of_a_variable() {
    assert!(long_help_of("start").contains(
        "In the value of a variable, ${NAME} stands for the value that the variable\nNAME has \
         where cladenc runs, and ${NAME:-TEXT} for that value, or for TEXT\nwhere NAME is not \
         set there."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_variable_cladenc_sets_keeps_its_value() {
    assert!(long_help_of("start").contains(
        "A variable that cladenc sets for a harness,\nsuch as CLADENC_MODEL, keeps the value \
         cladenc gives it."
    ));
}

#[test]
fn the_long_help_of_start_says_when_the_values_are_read_and_that_the_journal_holds_none() {
    assert!(long_help_of("start").contains(
        "cladenc reads the\nvalues when the directed evolution is started, and again each time \
         it is\nresumed, and writes none of them in the journal."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_values_are_on_no_command_line() {
    assert!(long_help_of("start").contains(
        "It keeps them off every\ncommand line: Docker reads them from its standard input, one \
         line for each\nvariable, so the name of a variable is made of letters, digits, \
         dots,\nunderscores and hyphens, and its value is one line of text."
    ));
}

#[test]
fn the_long_help_of_start_says_which_task_toml_has_its_task_refused() {
    assert!(long_help_of("start").contains(
        "A task is refused when the directed evolution is started where its\ntask.toml is not \
         TOML, gives one of these settings a value of another\nkind, names a variable \
         otherwise, or takes a value from a variable that is\nnot set where cladenc runs and \
         gives no text in its place."
    ));
}

#[test]
fn the_long_help_of_start_says_that_the_grader_runs_as_the_user_the_task_toml_names_for_it() {
    assert!(long_help_of("start").contains(
        "as the same user,\nunless the task.toml of the task names a user for the grader, as\n"
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_task_that_names_an_image_has_it_as_its_environment() {
    assert!(long_help_of("start").contains(
        "A task whose task.toml names an image, as docker_image under environment,\nhas that \
         image as its environment: cladenc builds nothing for the task,\nand the task needs no \
         environment directory."
    ));
}

#[test]
fn the_long_help_of_start_shows_how_docker_is_asked_for_the_image_a_task_names() {
    assert!(
        long_help_of("start").contains("\n    docker image inspect --format {{.Id}} <IMAGE>\n")
    );
}

#[test]
fn the_long_help_of_start_says_that_an_image_that_is_not_on_the_machine_stops_with_exit_status_2() {
    assert!(long_help_of("start").contains(
        "and a task whose image is not on the machine stops the directed evolution\nwith exit \
         status 2. `cladenc start` then leaves it no journal."
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_task_that_names_an_image_needs_no_dockerfile() {
    assert!(long_help_of("start").contains(
        "A task whose task.toml names an image needs\nneither the Dockerfile nor the \
         environment directory."
    ));
}

#[test]
fn the_long_help_of_resume_says_that_it_reads_the_task_toml_of_each_task_again() {
    assert!(long_help_of("resume").contains(
        "`cladenc resume` reads the task.toml of the copy of each task again, and\nthe \
         variables it names as they are set where `cladenc resume` runs."
    ));
}

#[test]
fn the_long_help_of_resume_says_what_of_a_task_toml_stops_it_with_exit_status_2() {
    assert!(long_help_of("resume").contains(
        "A task whose image is not there, or which takes a\nvalue from a variable that is not \
         set, stops `cladenc resume` with exit\nstatus 2."
    ));
}

#[test]
fn the_long_help_of_start_names_the_time_limits_of_a_task_toml_that_cladenc_obeys() {
    let help = long_help_of("start");
    let settings = [
        "  timeout_sec under agent         the time limit of the harness, in\n",
        "  timeout_sec under verifier      the time limit of the grader, in\n",
        "  build_timeout_sec under         the time limit of the build of the\n",
    ];
    assert!(settings.iter().all(|line| help.contains(line)));
}

#[test]
fn the_long_help_of_start_says_what_time_limit_the_harness_of_an_attempt_has() {
    assert!(long_help_of("start").contains(
        "The harness has the time limit that the task.toml of the task gives, as\ntimeout_sec \
         under agent, and no time limit where it gives none, as in\nHarbor."
    ));
}

#[test]
fn the_long_help_of_start_says_that_what_a_stopped_harness_left_is_judged_all_the_same() {
    assert!(long_help_of("start").contains(
        "A harness that runs past its time limit is stopped, and every\nprocess it left \
         running is ended, as by `kill -9 -1`. What it left is\njudged all the same."
    ));
}

#[test]
fn the_long_help_of_start_says_what_time_limit_the_grader_has_and_from_when_it_is_counted() {
    assert!(long_help_of("start").contains(
        "The grader has a time limit of its own, timeout_sec\nunder verifier, or 600 seconds \
         where the task gives none, counted from the\nmoment the harness ended or was stopped."
    ));
}

#[test]
fn the_long_help_of_start_says_that_no_flag_gives_a_time_limit_of_an_attempt() {
    assert!(long_help_of("start").contains("No flag gives a time limit of an\nattempt."));
}

#[test]
fn the_long_help_of_start_says_that_a_container_with_no_time_limit_does_not_end_itself() {
    assert!(long_help_of("start").contains(
        "Where a task gives its harness no time limit, the container of an\nattempt does not \
         end itself: a cladenc that was killed outright leaves it\nrunning until `cladenc \
         resume` removes it, or a person does."
    ));
}

#[test]
fn the_long_help_of_start_says_how_long_the_container_of_an_attempt_sleeps() {
    assert!(long_help_of("start").contains(
        "  <SECONDS>  the time limit of the harness and the time limit of the\n             \
         grader together, and ten minutes more, for what cladenc\n             copies in and \
         out\n"
    ));
}

#[test]
fn the_long_help_of_start_says_that_a_time_limit_that_is_not_above_0_is_refused() {
    assert!(long_help_of("start").contains(
        "A time limit\nthat is not a number of seconds above 0 is refused in the same way."
    ));
}

#[test]
fn the_long_help_of_start_says_that_an_attempt_keeps_whether_its_harness_was_stopped() {
    assert!(long_help_of("start").contains(
        "  ending         one word: ended, or stopped where the harness ran past\n                 \
         its time limit\n"
    ));
}
