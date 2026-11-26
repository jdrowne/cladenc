use std::path::PathBuf;

use crate::dataset::{Dataset, DatasetError};
use crate::file_system::tests::double::Memory;

fn parsing() -> Dataset {
    Dataset {
        directory: PathBuf::from("/data/parsing"),
        tasks: vec!["fix-the-parser".to_owned()],
    }
}

fn read_in_a_directory_named(name: &str) -> Result<String, DatasetError> {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_named_in(&directory, directory.path(), name, &["t1"]);
    Dataset::read(&directory, &dataset).map(|dataset| dataset.name())
}

#[test]
fn reading_a_dataset_gives_its_directory_and_the_ids_of_its_tasks_in_order() {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t2", "t1"]);
    assert_eq!(
        Dataset::read(&directory, &dataset),
        Ok(Dataset {
            directory: dataset,
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        })
    );
}

#[test]
fn the_name_of_a_dataset_is_the_name_of_its_directory() {
    assert_eq!(parsing().name(), "parsing");
}

#[test]
fn a_task_is_known_by_the_name_of_its_dataset_and_its_id_together() {
    assert_eq!(parsing().task("fix-the-parser"), "parsing/fix-the-parser");
}

#[test]
fn a_dataset_is_named_by_letters_digits_dots_underscores_and_hyphens() {
    assert_eq!(
        read_in_a_directory_named("Bench-1.a_b"),
        Ok("Bench-1.a_b".to_owned())
    );
}

#[test]
fn a_dataset_in_a_directory_with_a_space_in_its_name_is_refused() {
    assert_eq!(
        read_in_a_directory_named("my dataset"),
        Err(DatasetError::NotNamed {
            directory: "my dataset".to_owned()
        })
    );
}

#[test]
fn a_dataset_given_by_a_path_that_ends_in_no_name_is_refused() {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1"]);
    assert_eq!(
        Dataset::read(&directory, &dataset.join("t1/..")),
        Err(DatasetError::NotNamed {
            directory: String::new()
        })
    );
}

#[test]
fn what_is_not_a_dataset_is_refused_for_that_and_not_for_its_name() {
    let directory = Memory::default();
    assert_eq!(
        Dataset::read(&directory, &directory.path().join("my dataset")),
        Err(DatasetError::NotADirectory)
    );
}
