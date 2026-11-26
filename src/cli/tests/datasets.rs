use std::path::{Path, PathBuf};

use crate::cli::StartOptions;
use crate::cli::checks::Refused;
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;

const TAKES: &str = "the directory of a dataset, in which every directory is a task that holds \
                     a file of text named instruction.md, a file named task.toml that is valid as \
                     the task configuration of Harbor, a directory named tests with an executable \
                     file named test.sh in it, or a verifier in an image of its own, and a \
                     directory named environment with a file named Dockerfile and no Compose file \
                     in it, or, in place of the Dockerfile, an image named in its task.toml";

fn options(directory: &Memory) -> StartOptions {
    StartOptions {
        primogenitor: test_inputs::primogenitor_in(directory, directory.path()),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        dataset: vec![test_inputs::dataset_in(
            directory,
            directory.path(),
            &["t3", "t2", "t1"],
        )],
        diversification_docker_image: Some("image".to_owned()),
        diversification_dockerfile: None,
        diversification_docker_image_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.path().join("outputs"),
        max_diversifications: 800,
        max_concurrent_sandboxes: 8,
        diversification_timeout: 3600,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 30.0,
        corroboration_tasks_percentage: 30.0,
    }
}

fn checked(change: impl FnOnce(&Memory, &mut Vec<PathBuf>)) -> (Memory, Result<Settings, Refused>) {
    let directory = Memory::default();
    let mut options = options(&directory);
    change(&directory, &mut options.dataset);
    let settings = options.settings(&directory);
    (directory, settings)
}

fn settings_of(change: impl FnOnce(&Memory, &mut Vec<PathBuf>)) -> Settings {
    checked(change).1.unwrap()
}

fn reason_of(change: impl FnOnce(&Memory, &mut Vec<PathBuf>)) -> (Memory, String) {
    let (directory, settings) = checked(change);
    (directory, settings.unwrap_err().reason.unwrap_or_default())
}

fn with_a_second(directory: &Memory, datasets: &mut Vec<PathBuf>) {
    datasets.push(test_inputs::dataset_named_in(
        directory,
        directory.path(),
        "refactoring",
        &["t3", "t1", "t4"],
    ));
}

fn with_one_of_the_same_name(directory: &Memory, datasets: &mut Vec<PathBuf>) {
    datasets.push(test_inputs::dataset_in(
        directory,
        &directory.path().join("elsewhere"),
        &["t1"],
    ));
}

fn given_twice(_: &Memory, datasets: &mut Vec<PathBuf>) {
    datasets.push(datasets[0].clone());
}

fn missing(directory: &Memory, datasets: &mut Vec<PathBuf>) {
    *datasets = vec![directory.path().join("missing")];
}

fn one_task(_: &Memory, datasets: &mut Vec<PathBuf>) {
    *datasets = vec![datasets[0].join("t1")];
}

fn with_a_space_in_its_name(directory: &Memory, datasets: &mut Vec<PathBuf>) {
    *datasets = vec![test_inputs::dataset_named_in(
        directory,
        directory.path(),
        "my dataset",
        &["t1"],
    )];
}

fn from_the_working_directory(directory: &Memory, datasets: &mut Vec<PathBuf>) {
    test_inputs::dataset_in(
        directory,
        Path::new("/tests/samples"),
        &["add-a-flag", "fix-the-parser", "rename-a-module"],
    );
    *datasets = vec![PathBuf::from("tests/samples/dataset")];
}

fn through_a_symbolic_link(directory: &Memory, datasets: &mut Vec<PathBuf>) {
    let link = directory.path().join("link");
    directory.symlink(&datasets[0], &link).unwrap();
    *datasets = vec![link];
}

#[test]
fn a_refused_dataset_is_refused_by_the_flag_that_names_it() {
    let (_directory, settings) = checked(missing);
    assert_eq!(settings.unwrap_err().flags, ["--dataset"]);
}

#[test]
fn a_refused_dataset_is_refused_with_what_the_flag_takes() {
    let (_directory, settings) = checked(missing);
    assert_eq!(settings.unwrap_err().accepted, TAKES);
}

#[test]
fn a_dataset_that_does_not_exist_is_said_as_it_was_given_not_to_be_a_directory() {
    let (directory, reason) = reason_of(missing);
    assert_eq!(
        reason,
        format!(
            "{}: what was given is not a directory that can be read",
            directory.path().join("missing").display()
        )
    );
}

#[test]
fn a_dataset_that_is_one_task_is_said_to_be_one_task() {
    let (directory, reason) = reason_of(one_task);
    assert_eq!(
        reason,
        format!(
            "{}: what was given is the directory of one task, and a dataset is the directory \
             that holds the tasks",
            directory.path().join("dataset/t1").display()
        )
    );
}

#[test]
fn a_dataset_with_a_task_that_has_no_environment_names_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(|directory, datasets| {
        directory
            .remove_dir_all(datasets[0].join("t2/environment"))
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no directory named environment with a file named Dockerfile in \
             it, and its task.toml names no image in its place",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_with_a_task_that_has_no_task_toml_names_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(|directory, datasets| {
        directory
            .remove_file(datasets[0].join("t2/task.toml"))
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no file named task.toml",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_with_a_task_whose_task_toml_is_not_valid_names_the_task_the_line_and_why() {
    let (directory, reason) = reason_of(|directory, datasets| {
        directory
            .write(
                datasets[0].join("t2/task.toml"),
                "[environment]\ncpus = 1\n\n[environment.healthcheck]\nretries = 1.5\n",
            )
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds a task.toml that is not valid at line 5: invalid type: floating \
             point `1.5`, expected i64",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_with_a_task_whose_task_toml_cannot_be_obeyed_names_the_task_and_why() {
    let (directory, reason) = reason_of(|directory, datasets| {
        directory
            .write(
                datasets[0].join("t2/task.toml"),
                "[agent]\nuser = \"worker\"\n\n[environment]\nworkdir = 3\n",
            )
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds a task.toml that is not valid at line 5: invalid type: integer \
             `3`, expected a string",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_with_a_task_that_asks_for_a_gpu_names_the_task_and_what_it_asks_for() {
    let (directory, reason) = reason_of(|directory, datasets| {
        directory
            .write(
                datasets[0].join("t2/task.toml"),
                "[environment]\ngpus = 1\n",
            )
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds a task.toml that asks for a GPU, which cladenc has no way to give",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_with_a_task_that_names_an_image_in_its_task_toml_needs_no_dockerfile_for_it() {
    let settings = settings_of(|directory, datasets| {
        directory
            .remove_dir_all(datasets[0].join("t2/environment"))
            .unwrap();
        directory
            .write(
                datasets[0].join("t2/task.toml"),
                "[environment]\ndocker_image = \"debian:stable-slim\"\n",
            )
            .unwrap();
    });
    assert_eq!(settings.tasks(), ["dataset/t1", "dataset/t2", "dataset/t3"]);
}

#[test]
fn a_dataset_that_is_refused_for_a_task_names_the_dataset_the_task_and_what_it_lacks() {
    let (directory, reason) = reason_of(|directory, datasets| {
        directory
            .remove_dir_all(datasets[0].join("t2/tests"))
            .unwrap();
    });
    assert_eq!(
        reason,
        format!(
            "{}: task t2 holds no directory named tests with an executable file named test.sh \
             in it",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_in_a_directory_with_a_space_in_its_name_is_refused_for_its_name() {
    let (directory, reason) = reason_of(with_a_space_in_its_name);
    assert_eq!(
        reason,
        format!(
            "{}: its directory is named `my dataset`, and the name of a dataset is made of \
             letters, digits, dots, underscores and hyphens alone",
            directory.path().join("my dataset").display()
        )
    );
}

#[test]
fn a_dataset_given_from_the_working_directory_is_made_absolute() {
    assert!(
        settings_of(from_the_working_directory).datasets[0]
            .directory
            .is_absolute()
    );
}

#[test]
fn a_task_of_a_directed_evolution_is_known_by_the_name_of_its_dataset_and_its_id() {
    assert_eq!(
        settings_of(from_the_working_directory).tasks(),
        [
            "dataset/add-a-flag",
            "dataset/fix-the-parser",
            "dataset/rename-a-module"
        ]
    );
}

#[test]
fn a_dataset_given_through_a_symbolic_link_is_the_directory_it_leads_to() {
    assert!(
        settings_of(through_a_symbolic_link).datasets[0]
            .directory
            .ends_with("dataset")
    );
}

#[test]
fn a_dataset_given_through_a_symbolic_link_is_named_after_the_directory_it_leads_to() {
    assert_eq!(
        settings_of(through_a_symbolic_link).datasets[0].name(),
        "dataset"
    );
}

#[test]
fn a_directed_evolution_given_two_datasets_has_the_tasks_of_both() {
    assert_eq!(
        settings_of(with_a_second).tasks(),
        [
            "dataset/t1",
            "dataset/t2",
            "dataset/t3",
            "refactoring/t1",
            "refactoring/t3",
            "refactoring/t4"
        ]
    );
}

#[test]
fn two_datasets_that_each_hold_a_task_of_the_same_id_give_the_directed_evolution_two_tasks() {
    let tasks = settings_of(with_a_second).tasks();
    assert_eq!(tasks.iter().filter(|task| task.ends_with("/t1")).count(), 2);
}

#[test]
fn the_datasets_of_a_directed_evolution_are_in_the_order_of_their_names_and_not_as_given() {
    let settings = settings_of(|directory, datasets| {
        datasets.insert(
            0,
            test_inputs::dataset_named_in(
                directory,
                directory.path(),
                "refactoring",
                &["t3", "t4", "t5"],
            ),
        );
    });
    assert_eq!(
        settings
            .datasets
            .iter()
            .map(|dataset| dataset.name())
            .collect::<Vec<_>>(),
        ["dataset", "refactoring"]
    );
}

#[test]
fn a_dataset_given_twice_is_refused() {
    let (directory, reason) = reason_of(given_twice);
    assert_eq!(
        reason,
        format!(
            "the dataset {} is given twice",
            directory.path().join("dataset").display()
        )
    );
}

#[test]
fn a_dataset_given_as_itself_and_through_a_symbolic_link_is_given_twice() {
    let (_directory, reason) = reason_of(|directory, datasets| {
        let link = directory.path().join("link");
        directory.symlink(&datasets[0], &link).unwrap();
        datasets.push(link);
    });
    assert!(reason.ends_with(" is given twice"));
}

#[test]
fn two_datasets_in_directories_of_the_same_name_are_refused() {
    let (_directory, reason) = reason_of(with_one_of_the_same_name);
    assert_eq!(
        reason,
        "two of the datasets given are in directories named `dataset`, and a task is known by \
         the name of its dataset and its id together"
    );
}

#[test]
fn a_second_dataset_that_cannot_be_used_is_refused_as_the_first_would_be() {
    let (directory, reason) = reason_of(|directory, datasets| {
        datasets.push(directory.path().join("missing"));
    });
    assert_eq!(
        reason,
        format!(
            "{}: what was given is not a directory that can be read",
            directory.path().join("missing").display()
        )
    );
}
