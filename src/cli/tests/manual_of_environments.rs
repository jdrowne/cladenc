use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_when_the_environment_of_every_task_is_built() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc builds the environment of every task of every benchmark when the directed \
         evolution is started, and again each time it is resumed, before it brings up any \
         container, as"
    ));
}

#[test]
fn the_manual_of_start_shows_how_docker_is_asked_to_build_an_environment() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "docker build --tag <NAME> --label cladenc.task=<TASK> \\ --label \
         cladenc.directed-evolution=<PATH> <DIR>"
    ));
}

#[test]
fn the_manual_of_start_says_what_path_the_label_of_an_environment_holds() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<PATH> the path of the outputs directory, with every symbolic link in it resolved"
    ));
}

#[test]
fn the_manual_of_start_says_what_the_labels_of_an_environment_are() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The two labels are Docker labels, each a name and a value that Docker keeps with an \
         image. The second is the label that every container of the directed evolution carries."
    ));
}

#[test]
fn the_manual_of_start_says_that_two_directed_evolutions_build_two_images_of_one_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "It makes the environments of two directed evolutions two images, even where both are \
         built from one task of one benchmark, so that removing those of one leaves those of the \
         other on the machine."
    ));
}

#[test]
fn the_manual_of_start_says_how_cladenc_names_the_environment_of_a_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<NAME> the name cladenc gives the environment: cladenc-environments- and sixteen random \
         hexadecimal digits, the same for every task of the directed evolution, then a colon and \
         the place of the task among the tasks of the directed evolution, counted from 0"
    ));
}

#[test]
fn the_manual_of_start_says_that_an_environment_is_built_from_the_copy_of_its_task() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<DIR> the environment directory of the copy of the task in the outputs directory, the \
         build context of its Dockerfile"
    ));
}

#[test]
fn the_manual_of_start_says_where_what_docker_printed_for_an_environment_is_kept() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "environments/<BENCHMARK>/<TASK> in the outputs directory keeps what Docker printed."
    ));
}

#[test]
fn the_manual_of_start_says_what_time_limit_the_build_of_an_environment_has() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The build has the time limit that the task.toml of the task gives, as build_timeout_sec \
         under environment, or 600 seconds where it gives none."
    ));
}

#[test]
fn the_manual_of_start_says_that_an_environment_that_is_not_built_stops_with_exit_status_2() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A task whose environment is not built stops the directed evolution with exit status 2, \
         and cladenc start leaves it no journal."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_base_image_of_an_environment_has_to_be_on_the_machine() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The image that the Dockerfile of a task starts from has to be on the machine: cladenc \
         does not ask Docker to fetch it and cannot forbid it, so Docker fetches a base image that \
         is not on the machine, from the registry of its name."
    ));
}

#[test]
fn the_manual_of_resume_says_that_it_builds_the_environments_again_under_the_same_names() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "Once the containers are removed, cladenc resume builds the environment of every task \
         again, from the copy of the task in the outputs directory and under the name it was given \
         when the directed evolution was started, before it brings up any container."
    ));
}

#[test]
fn the_manual_of_resume_says_that_the_base_image_of_an_environment_has_to_be_on_the_machine() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "The image that the Dockerfile of each task starts from has to be on the machine: Docker \
         fetches one that is not, from the registry of its name."
    ));
}

#[test]
fn the_manual_of_resume_says_that_an_environment_that_is_not_built_stops_with_exit_status_2() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "A task whose environment is not built stops cladenc resume with exit status 2, and \
         environments/<BENCHMARK>/<TASK> in the outputs directory keeps what Docker printed."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_environments_cladenc_built_are_removed_when_it_ends() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc removes the environments it built once cladenc start or cladenc resume ends, \
         whether the directed evolution finished, stopped or could not go on, as"
    ));
}

#[test]
fn the_manual_of_start_shows_how_docker_is_asked_to_remove_an_environment() {
    assert!(manual_on_one_line("cladenc-start").contains("docker image rm --force <NAME>..."));
}

#[test]
fn the_manual_of_start_says_that_an_image_a_task_names_is_left_alone() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "An image that the task.toml of a task names is not one cladenc built, and is left alone."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_cladenc_killed_outright_removes_no_environment() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A cladenc that was killed outright removes nothing: cladenc resume builds the \
         environments again, and removes them when it ends."
    ));
}

#[test]
fn the_manual_of_start_says_what_a_stop_before_the_journal_does() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc start then exits with status 2 and says that the directed evolution was stopped \
         before it was started: cladenc resume has nothing to continue, and the outputs directory \
         is removed by hand before it is named to cladenc start again."
    ));
}
