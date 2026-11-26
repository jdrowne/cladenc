use crate::containers::ContainerError;
use crate::containers::tests::double::{Answer, Double};
use crate::directed_evolution::DirectedEvolutionError;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::{self, Settings};

const IMAGE_BUILT: &str = "timeout -s KILL 35 docker build --tag debian:stable-slim --file \
                           OUTPUTS/image/Dockerfile OUTPUTS/image/context";

fn with_a_dockerfile(directory: &Memory, settings: &mut Settings) {
    settings.dockerfile = Some(test_inputs::dockerfile_in(directory, directory.path(), ""));
    settings.docker_build_timeout = Some(30);
}

fn with_a_dockerfile_of_an_image_of_its_own(directory: &Memory, settings: &mut Settings) {
    with_a_dockerfile(directory, settings);
    settings.docker_image = "an-image:built".to_owned();
}

fn imaged_by(docker: Double) -> Evolved {
    given::started(docker, with_a_dockerfile)
}

fn not_built() -> Evolved {
    imaged_by(Double::default().answering(&["--file"], &[Answer::Exited(2)]))
}

#[test]
fn a_directed_evolution_given_a_dockerfile_first_builds_the_image_of_its_harnesses() {
    let commands = imaged_by(Double::default()).commands();
    assert_eq!(commands.first().map(String::as_str), Some(IMAGE_BUILT));
}

#[test]
fn an_image_that_is_built_is_told() {
    let said = imaged_by(Double::default()).said;
    assert!(said.contains("the image debian:stable-slim is built"));
}

#[test]
fn a_directed_evolution_given_no_dockerfile_builds_no_image_of_its_harnesses() {
    let evolved = given::started(Double::default(), given::unchanged);
    assert_eq!(evolved.count_of("--file"), 0);
}

#[test]
fn an_image_whose_build_fails_ends_the_directed_evolution_with_the_cause() {
    assert!(matches!(
        not_built().result,
        Err(DirectedEvolutionError::Image(ContainerError::Exit(2)))
    ));
}

#[test]
fn an_image_whose_build_fails_is_told_with_the_cause() {
    let said = not_built().said;
    assert!(said.contains("the build of the image exited with status 2"));
}

#[test]
fn no_environment_is_built_after_an_image_whose_build_fails() {
    assert_eq!(not_built().count_of("cladenc.task="), 0);
}

#[test]
fn a_directed_evolution_that_is_stopped_while_its_image_is_built_has_not_started() {
    let stopped = imaged_by(Double::default().answering(&["--file"], &[Answer::Stopped]));
    assert!(matches!(
        stopped.result,
        Err(DirectedEvolutionError::NotStarted)
    ));
}

#[test]
fn the_container_of_a_diversification_is_started_from_the_image_that_was_built() {
    let evolved = given::started(Double::default(), with_a_dockerfile_of_an_image_of_its_own);
    let built = evolved.count_of(" docker build --tag an-image:built --file ");
    let started_from = evolved.count_of(" an-image:built sh -c ");
    assert_eq!((built, started_from), (1, 1));
}

#[test]
fn the_image_is_built_once_and_the_environment_of_each_task_once() {
    assert_eq!(imaged_by(Double::default()).count_of("docker build "), 3);
}

#[test]
fn every_container_is_told_never_to_fetch_the_image_that_was_built() {
    let commands = imaged_by(Double::default()).commands();
    let containers = commands
        .iter()
        .filter(|command| command.starts_with("docker run "));
    let told_never = commands
        .iter()
        .filter(|command| command.starts_with("docker run --rm --pull never "));
    assert_eq!((containers.count(), told_never.count()), (3, 3));
}

fn stopped_at_a_trial() -> Evolved {
    let needles = ["run a harness", "trial-0-benchmark+t2"];
    imaged_by(Double::default().answering(&needles, &[Answer::Exited(125)]))
}

fn resumed_with_its_dockerfile_gone() -> Evolved {
    let stopped = stopped_at_a_trial();
    stopped
        .files()
        .remove_file(stopped.files().path().join("Dockerfile"))
        .unwrap();
    given::resumed(stopped, Double::default())
}

#[test]
fn the_image_is_built_after_the_tasks_are_copied_and_before_the_first_action() {
    let lines = imaged_by(Double::default()).lines();
    let places = [
        "cladenc: copying the ",
        "cladenc: building the image ",
        "cladenc: started the ",
    ]
    .map(|start| lines.iter().position(|line| line.starts_with(start)));
    assert_eq!(places, [Some(1), Some(3), Some(8)]);
}

#[test]
fn the_run_record_names_the_image_that_was_built_its_dockerfile_and_the_time_limit() {
    let evolved = imaged_by(Double::default());
    let named = format!(
        r#""docker_image":"debian:stable-slim","dockerfile":"{}","docker_build_timeout":30,"#,
        evolved.files().path().join("Dockerfile").display()
    );
    assert!(
        evolved
            .journal()
            .lines()
            .next()
            .is_some_and(|run| run.contains(&named))
    );
}

#[test]
fn an_image_whose_build_fails_leaves_the_directed_evolution_no_journal() {
    let evolved = not_built();
    assert!(!evolved.files().exists(journal::path(&evolved.outputs())));
}

#[test]
fn an_image_whose_build_fails_is_said_last_with_where_what_docker_printed_is_kept() {
    let evolved = not_built();
    assert!(evolved.said.ends_with(&format!(
        ": what Docker printed is kept in {}\n",
        evolved.outputs().join("image").display()
    )));
}

#[test]
fn a_directed_evolution_that_is_resumed_builds_no_image() {
    let resumed = given::resumed(stopped_at_a_trial(), Double::default());
    assert_eq!(resumed.count_of("--file"), 0);
}

#[test]
fn a_directed_evolution_that_is_resumed_does_not_say_that_it_builds_an_image() {
    let resumed = given::resumed(stopped_at_a_trial(), Double::default());
    assert!(!resumed.said.contains("building the image"));
}

#[test]
fn a_directed_evolution_that_is_resumed_ends_though_the_dockerfile_it_was_given_is_gone() {
    assert!(resumed_with_its_dockerfile_gone().result.is_ok());
}
