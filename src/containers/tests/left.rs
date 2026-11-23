use std::io;

use crate::containers;
use crate::containers::tests::double::{Answer, Double};
use crate::containers::tests::given;

const LISTED: Answer = Answer::Printed("a\nb\n");
const REFUSED: Answer = Answer::Refused("docker is not there");

struct Removed {
    commands: Vec<String>,
    result: io::Result<u64>,
}

fn removed_by(docker: Double) -> Removed {
    let (_directory, outputs_directory) = given::made();
    let result = containers::remove_left_behind(given::host(&docker, &outputs_directory));
    Removed {
        commands: docker.commands(&outputs_directory),
        result,
    }
}

#[test]
fn the_containers_left_behind_are_looked_for_by_the_label_of_the_directed_evolution() {
    let removed = removed_by(Double::default());
    assert_eq!(
        removed.commands,
        ["docker ps --all --quiet --filter label=cladenc.directed-evolution=OUTPUTS"]
    );
}

#[test]
fn where_no_container_is_left_behind_none_is_removed() {
    let removed = removed_by(Double::default());
    assert_eq!(removed.result.ok(), Some(0));
}

#[test]
fn the_containers_left_behind_are_all_removed_in_one_command() {
    let removed = removed_by(Double::default().answering(&["ps"], &[LISTED]));
    assert_eq!(removed.commands[1..], ["docker rm --force a b"]);
}

#[test]
fn the_containers_left_behind_that_were_removed_are_counted() {
    let removed = removed_by(Double::default().answering(&["ps"], &[LISTED]));
    assert_eq!(removed.result.ok(), Some(2));
}

#[test]
fn containers_that_docker_cannot_list_are_not_said_to_be_removed() {
    let removed = removed_by(Double::default().answering(&["ps"], &[REFUSED]));
    assert_eq!(
        removed.result.map_err(|error| error.to_string()),
        Err("docker is not there".to_owned())
    );
}

#[test]
fn containers_that_docker_does_not_remove_are_not_said_to_be_removed() {
    let docker = Double::default()
        .answering(&["ps"], &[LISTED])
        .answering(&["rm"], &[REFUSED]);
    assert!(removed_by(docker).result.is_err());
}
