use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_resume_says_that_a_cladenc_killed_outright_ends_none_of_its_containers() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "A cladenc that is killed outright, as by kill -9, ends none of its containers, and each \
         goes on until its time limit has passed."
    ));
}

#[test]
fn the_manual_of_resume_says_that_it_first_removes_the_containers_that_were_left_running() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "Before it does anything else, cladenc resume removes the containers that the directed \
         evolution left running, so that it never works beside one of them."
    ));
}

#[test]
fn the_manual_of_resume_shows_the_label_that_every_container_of_a_directed_evolution_carries() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "cladenc.directed-evolution=<DIR> where <DIR> is the path of the outputs directory, with \
         every symbolic link in it resolved."
    ));
}

#[test]
fn the_manual_of_resume_says_that_the_containers_are_removed_once_no_other_cladenc_is_there() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "Once it has the outputs directory to itself, cladenc resume asks Docker for the \
         containers that carry the label of its directed evolution, removes them, and says how \
         many it removed."
    ));
}

#[test]
fn the_manual_of_resume_says_what_it_does_where_docker_cannot_be_asked() {
    assert!(
        manual_on_one_line("cladenc-resume").contains(
            "Where Docker cannot be asked, it exits with status 2 and continues nothing."
        )
    );
}

#[test]
fn the_manual_of_resume_shows_how_the_containers_of_a_directed_evolution_are_listed_by_hand() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "Where the directed evolution is not resumed, its containers are removed by hand. docker \
         ps --filter label=cladenc.directed-evolution=<DIR> lists them"
    ));
}

#[test]
fn the_manual_of_resume_says_how_a_container_is_ended_by_hand() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "their names begin cladenc-, and docker kill with a name ends that container. Docker \
         removes a container of cladenc once it has ended."
    ));
}

#[test]
fn the_manual_of_start_says_that_resume_removes_the_containers_that_were_left_running() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Until then a container that was left goes on. cladenc resume removes the containers that \
         the directed evolution left running before it does anything else, and cladenc-resume(1) \
         says how they are found, and how they are removed by hand where the directed evolution is \
         not resumed."
    ));
}

#[test]
fn the_manual_of_start_shows_how_a_harness_is_started() {
    assert!(manual_on_one_line("cladenc-start").contains("sh -c <TIMER> <COMMAND> <SECONDS>"));
}

#[test]
fn the_manual_of_start_shows_the_timer_that_a_command_is_started_under() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<TIMER> sh -c \"$0\" & run=$!; (sleep \"$1\"; kill -9 $run) > /dev/null 2>&1 & wait $run"
    ));
}

#[test]
fn the_manual_of_start_says_which_command_the_timer_of_a_harness_starts() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("<COMMAND> the value of --harness-run-command")
    );
}

#[test]
fn the_manual_of_start_says_how_long_the_timer_of_a_container_waits() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<SECONDS> what is left of the time limit of the command when it is started, in whole \
         seconds, and five seconds more"
    ));
}

#[test]
fn the_manual_of_start_says_what_the_timer_does() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The timer is a line of shell that starts the command, ends it once the seconds have \
         passed, and hands on its exit status."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_container_ends_itself_when_cladenc_was_killed_outright() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "that a container ends itself once its time limit has passed, also when cladenc was killed \
         outright, as by kill -9, and is not there to remove it."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_container_of_a_trial_is_started_under_sleep() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Every container of a directed evolution is started under it, or, for a trial, under \
         sleep,"
    ));
}

#[test]
fn the_manual_of_start_says_what_the_image_has_to_hold_for_the_timer() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("it. The image of the directed evolution has to hold sh and sleep, and the")
    );
}

#[test]
fn the_manual_of_start_says_what_the_environment_of_every_task_has_to_hold() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("environment of every task sh, sleep, rm, mkdir, chown and ln.")
    );
}

#[test]
fn the_manual_of_start_says_in_which_image_the_harness_of_a_trial_is_started() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A harness is started in a container, of the image of the directed evolution for a \
         procreation and of the environment of its task for a trial, as"
    ));
}

#[test]
fn the_manual_of_start_says_as_which_user_the_command_of_a_harness_runs() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "In a procreation the command runs as the user cladenc runs as, and in a trial as the \
         user of the environment of the task, or as the user that the task.toml of the task names."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_image_of_the_directed_evolution_is_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Every procreation, and every build of a harness, is carried out in a container of one \
         image, the image of the directed evolution."
    ));
}

#[test]
fn the_manual_of_start_shows_how_a_harness_is_built() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "in a container of the same image, as sh -c <TIMER> <COMMAND> <SECONDS> where <COMMAND> is \
         the value of that flag and the timer is the same one,"
    ));
}

#[test]
fn the_manual_of_start_names_what_a_harness_is_told() {
    let help = manual_on_one_line("cladenc-start");
    let told = [
        "/var/opt/cladenc/harness holds the source code of the harness",
        "/var/opt/cladenc/work in a procreation, the directory the harness",
        "CLADENC_HARNESS /var/opt/cladenc/harness",
        "CLADENC_INSTRUCTION",
    ];
    assert!(told.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_names_the_variables_a_harness_is_told_the_model_in() {
    let help = manual_on_one_line("cladenc-start");
    let told = [
        "CLADENC_MODEL the name of the model, the value of --model",
        "CLADENC_MODEL_URL the address the model is served at",
        "CLADENC_MODEL_KEY the key the model is called with",
    ];
    assert!(told.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_how_a_harness_reaches_the_machine_cladenc_runs_on() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("host.docker.internal names the machine cladenc runs on")
    );
}

#[test]
fn the_manual_of_start_says_that_a_build_is_not_told_the_model() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A build calls no model, and is given none of the three variables of the model."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_verifier_is_not_told_the_model() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A verifier calls no model, and is given none of the three variables of the model."
    ));
}

#[test]
fn the_manual_of_start_shows_how_an_image_is_built_from_a_dockerfile() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("timeout -s KILL <SECONDS> docker build --tag <NAME> --file <FILE> <DIR>")
    );
}

#[test]
fn the_manual_of_start_says_what_docker_is_given_to_build_an_image_from() {
    let help = manual_on_one_line("cladenc-start");
    let given = [
        "<NAME> the name cladenc gives the image:",
        "<FILE> a copy of the Dockerfile, kept in image in the outputs",
        "directory",
        "<DIR> an empty directory, so the Dockerfile has no file to copy into",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_how_long_the_timeout_of_the_build_of_the_image_waits() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "<SECONDS> the value of --procreation-docker-image-build-timeout, and five seconds more"
    ));
}

#[test]
fn the_manual_of_start_says_that_the_build_of_the_image_ends_itself_at_its_time_limit() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc ends a build that runs past its time limit, and the build ends itself five \
         seconds later where cladenc was killed outright, as by kill -9, and is not there to end \
         it."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_machine_has_to_hold_timeout() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The machine cladenc runs on has to hold timeout, which the package coreutils brings."
    ));
}

#[test]
fn the_manual_of_start_says_that_cladenc_fetches_no_image_and_has_no_flag_for_a_registry() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc fetches no image, and has no flag for a registry, the server that Docker fetches \
         an image from."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_name_of_an_image_says_which_registry_it_is_of() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The name of an image says which registry it is of, as \
         registry.example:5000/tools/harness:1 does, and a name that says none is of Docker Hub."
    ));
}

#[test]
fn the_manual_of_start_says_how_an_image_is_brought_to_the_machine() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The image that --procreation-docker-image names has to be on the machine before the \
         directed evolution: fetched there with docker pull, carried there as a file with docker \
         save and docker load, or built there."
    ));
}

#[test]
fn the_manual_of_start_says_that_docker_is_told_never_to_fetch_the_image_of_a_container() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc tells Docker never to fetch the image of a container, so a directed evolution \
         whose image is not on the machine stops at the first container of that image, and cladenc \
         resume continues it once the image is there."
    ));
}

#[test]
fn the_manual_of_start_says_that_a_dockerfile_can_be_built_with_no_connection() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "A Dockerfile whose base image is on the machine, and whose steps fetch nothing, is built \
         with no connection to the internet."
    ));
}

#[test]
fn the_manual_of_start_says_that_docker_fetches_a_base_image_the_machine_does_not_hold() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc does not ask Docker to fetch a base image and cannot forbid it: Docker fetches a \
         base image that is not on the machine, from the registry of its name."
    ));
}

#[test]
fn the_manual_of_start_says_that_resume_builds_no_image() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc resume carries out its procreations and builds in it, and does not build it again."
    ));
}

#[test]
fn the_manual_of_start_says_what_each_way_a_harness_ends_means() {
    let help = manual_on_one_line("cladenc-start");
    let endings = [
        "exit status 0 the procreation created a harness, or is recorded as unchanged where the \
         harness changed nothing",
        "exit status 125 Docker did not start the container: the directed evolution stops",
        "any other status the procreation failed",
        "the time limit the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_manual_of_start_says_where_a_harness_is_built() {
    let help = manual_on_one_line("cladenc-start");
    let said = [
        "Where --harness-build-command is given, a harness is built before it is",
        "with /var/opt/cladenc/harness as the working directory",
        "builds in the outputs directory",
    ];
    assert!(said.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_what_each_way_a_build_ends_means() {
    let help = manual_on_one_line("cladenc-start");
    let endings = [
        "exit status 0 the harness is built",
        "exit status 125 Docker did not start the container: the directed evolution stops",
        "any other status a child is recorded as build_failed",
        "the time limit the container is removed",
    ];
    assert!(endings.iter().all(|ending| help.contains(ending)));
}

#[test]
fn the_manual_of_start_names_no_budget() {
    assert!(!manual_on_one_line("cladenc-start").contains("budget"));
}

#[test]
fn the_manual_of_start_says_where_the_copy_that_a_procreation_improves_is() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "/var/opt/cladenc/work holds a second copy of its source code, and the instruction asks \
         the harness to improve that copy: what the harness leaves there is its child."
    ));
}

#[test]
fn the_manual_of_start_names_no_directory_under_var_lib() {
    assert!(!manual_on_one_line("cladenc-start").contains("/var/lib"));
}

#[test]
fn the_manual_of_start_names_no_directory_at_the_top_of_the_filesystem_of_a_container() {
    assert_eq!(
        manual_on_one_line("cladenc-start")
            .split_whitespace()
            .filter(|word| {
                ["/harness", "/work", "/task", "/trials"]
                    .iter()
                    .any(|top| word.starts_with(top))
            })
            .collect::<Vec<&str>>(),
        Vec::<&str>::new()
    );
}

#[test]
fn the_manual_of_start_says_which_containers_are_started_from_the_image() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The creation of a singleton control group harness is carried out in a container of that \
         image as well, and no trial is: a trial is carried out in the environment of its \
         task."
    ));
}

#[test]
fn the_manual_of_start_names_the_flags_of_the_image_by_what_the_image_is_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The image of the directed evolution is the image that --procreation-docker-image names, \
         or the image that cladenc builds when the directed evolution is started, from the \
         Dockerfile that --procreation-dockerfile names, as"
    ));
}
