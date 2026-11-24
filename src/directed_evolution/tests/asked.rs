use std::collections::BTreeSet;

use crate::containers::tests::double::Double;
use crate::containers::tests::given::TIMER;
use crate::directed_evolution::tests::given::{self, Evolved};
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;

const TOKEN: &str = "a-made-up-token";
const OBEYED: &str = "[agent]\nuser = 1000\ntimeout_sec = 30\n\n[verifier]\nuser = \
                      2000\ntimeout_sec = 20.0\n\n[verifier.env]\nJUDGE = \
                      \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET:-a-made-up-token}\"\n\n\
                      [environment]\nworkdir = \"/srv/elsewhere\"\n\n[environment.env]\n\
                      API_TOKEN = \
                      \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET:-a-made-up-token}\"\n";
const CONTAINER: &str = "docker run ";
const COMMAND: &str = "docker exec ";
const AS_ROOT: &str = "docker exec --user 0 ";
const STAYS: &str = " --detach ";
const HARNESS: &str = " run a harness ";
const GRADER: &str = " /tests/test.sh ";

fn with_a_key(directory: &Memory, settings: &mut Settings) {
    let key = format!("{}\n", test_inputs::MODEL_KEY);
    let file = test_inputs::model_key_file_in(directory, directory.path(), &key);
    settings.model_key_file = Some(file);
}

fn with_a_task_that_sets_all_it_can(directory: &Memory, _: &mut Settings) {
    given::configured(directory, "t2", OBEYED);
}

fn evolved() -> Evolved {
    given::started(Double::default(), given::unchanged)
}

fn keyed() -> Evolved {
    given::started(Double::default(), with_a_key)
}

fn obeyed() -> Evolved {
    given::started(Double::default(), with_a_task_that_sets_all_it_can)
}

fn containers(commands: &[String]) -> Vec<&String> {
    commands
        .iter()
        .filter(|command| command.starts_with(CONTAINER))
        .collect()
}

fn containers_of_the_attempts(commands: &[String]) -> Vec<&String> {
    commands
        .iter()
        .filter(|command| command.contains(STAYS))
        .collect()
}

fn run_in_a_container<'a>(commands: &'a [String], run: &str) -> Vec<&'a String> {
    commands
        .iter()
        .filter(|command| command.starts_with(COMMAND) && command.contains(run))
        .collect()
}

fn under_a_timer(commands: &[String]) -> Vec<&String> {
    commands
        .iter()
        .filter(|command| {
            let alone = command.starts_with(CONTAINER) && !command.contains(STAYS);
            let beside = command.starts_with(COMMAND) && !command.starts_with(AS_ROOT);
            alone || beside
        })
        .collect()
}

fn each_holds(commands: &[&String], part: &str) -> bool {
    !commands.is_empty() && commands.iter().all(|command| command.contains(part))
}

fn following(flag: &str, commands: &[&String]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|command| command.split(' ').skip_while(|word| *word != flag).nth(1))
        .map(str::to_owned)
        .collect()
}

fn last_words(commands: &[&String]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|command| command.rsplit(' ').next())
        .map(str::to_owned)
        .collect()
}

#[test]
fn docker_is_given_the_name_of_the_variable_that_holds_the_key() {
    let commands = keyed().commands();
    assert!(
        commands
            .iter()
            .any(|command| command.contains(" --env CLADENC_MODEL_KEY "))
    );
}

#[test]
fn the_key_is_on_no_command_line_of_docker() {
    let commands = keyed().commands();
    assert!(!commands.concat().contains(test_inputs::MODEL_KEY));
}

#[test]
fn every_container_is_told_never_to_fetch_its_image() {
    let commands = evolved().commands();
    assert!(each_holds(
        &containers(&commands),
        "docker run --rm --pull never "
    ));
}

#[test]
fn every_container_is_started_so_that_it_can_name_the_machine_cladenc_runs_on() {
    let commands = evolved().commands();
    assert!(each_holds(
        &containers(&commands),
        " --add-host host.docker.internal:host-gateway "
    ));
}

#[test]
fn every_container_carries_the_label_of_its_directed_evolution() {
    let commands = evolved().commands();
    assert!(each_holds(
        &containers(&commands),
        " --name NAME --label cladenc.directed-evolution=OUTPUTS "
    ));
}

#[test]
fn every_command_is_started_under_a_timer_that_ends_it() {
    let commands = evolved().commands();
    assert!(each_holds(
        &under_a_timer(&commands),
        &format!(" sh -c {TIMER} ")
    ));
}

#[test]
fn the_container_of_an_attempt_whose_task_gives_no_time_limit_sleeps_without_end() {
    let commands = evolved().commands();
    assert_eq!(
        last_words(&containers_of_the_attempts(&commands)),
        ["4294968495", "4294968495"]
    );
}

#[test]
fn the_containers_of_the_attempts_are_started_from_the_images_built_from_the_environments() {
    let commands = evolved().commands();
    let all: Vec<&String> = commands.iter().collect();
    let started: BTreeSet<String> = following("--detach", &all).into_iter().collect();
    let built: BTreeSet<String> = following("--tag", &all).into_iter().collect();
    assert_eq!((started.len(), started), (2, built));
}

#[test]
fn the_timer_of_a_procreation_is_given_the_run_command_and_five_seconds_more_than_its_time_limit() {
    let commands = evolved().commands();
    let procreations: Vec<&String> = containers(&commands)
        .into_iter()
        .filter(|command| !command.contains(STAYS))
        .collect();
    assert!(each_holds(
        &procreations,
        &format!(" sh -c {TIMER} run a harness 35")
    ));
}

#[test]
fn the_timer_of_a_grader_is_given_the_grader_of_the_task() {
    let commands = evolved().commands();
    assert!(each_holds(
        &run_in_a_container(&commands, GRADER),
        &format!(" sh -c {TIMER} /tests/test.sh ")
    ));
}

#[test]
fn the_value_of_a_variable_of_a_task_is_given_to_docker_on_its_standard_input_alone() {
    let obeyed = obeyed();
    let on_a_command_line = obeyed.commands().concat().contains(TOKEN);
    let on_standard_input = obeyed.inputs().concat().contains(TOKEN);
    assert_eq!((on_a_command_line, on_standard_input), (false, true));
}

#[test]
fn the_container_of_an_attempt_is_started_in_the_working_directory_its_task_names() {
    let commands = obeyed().commands();
    let named: Vec<bool> = containers_of_the_attempts(&commands)
        .iter()
        .map(|command| command.contains(" --workdir /srv/elsewhere "))
        .collect();
    assert_eq!(named, [false, true]);
}

#[test]
fn docker_reads_the_variables_of_the_container_of_an_attempt_from_its_standard_input() {
    let commands = obeyed().commands();
    assert!(each_holds(
        &containers_of_the_attempts(&commands),
        " --env-file /dev/stdin "
    ));
}

#[test]
fn the_harness_and_the_grader_of_a_task_are_started_as_the_users_the_task_names() {
    let commands = obeyed().commands();
    let started: Vec<&String> = commands
        .iter()
        .filter(|command| {
            command.starts_with(COMMAND) && (command.contains(HARNESS) || command.contains(GRADER))
        })
        .collect();
    assert_eq!(following("--user", &started), ["0", "0", "1000", "2000"]);
}

#[test]
fn the_container_of_an_attempt_sleeps_for_both_time_limits_of_its_task_and_ten_minutes_more() {
    let commands = obeyed().commands();
    assert_eq!(
        last_words(&containers_of_the_attempts(&commands)),
        ["4294968495", "650"]
    );
}

#[test]
fn the_timer_of_a_harness_is_given_the_time_limit_its_task_gives_it_and_five_seconds_more() {
    let commands = obeyed().commands();
    assert_eq!(
        last_words(&run_in_a_container(&commands, HARNESS)),
        ["4294967300", "35"]
    );
}

#[test]
fn the_timer_of_a_grader_is_given_the_time_limit_its_task_gives_it_and_five_seconds_more() {
    let commands = obeyed().commands();
    assert_eq!(
        last_words(&run_in_a_container(&commands, GRADER)),
        ["605", "25"]
    );
}
