use std::path::PathBuf;

use crate::dataset::{self, Dataset, DatasetError};

fn dataset(directory: &str) -> Dataset {
    Dataset {
        directory: PathBuf::from(directory),
        tasks: vec!["t1".to_owned()],
    }
}

#[test]
fn datasets_in_directories_of_different_names_are_told_apart() {
    assert_eq!(
        dataset::told_apart(&[
            dataset("/data/parsing"),
            dataset("/data/refactoring"),
            dataset("/more/typing")
        ]),
        Ok(())
    );
}

#[test]
fn one_dataset_is_told_apart_from_no_other() {
    assert_eq!(dataset::told_apart(&[dataset("/data/parsing")]), Ok(()));
}

#[test]
fn a_dataset_that_is_given_twice_is_not_told_apart_from_itself() {
    assert_eq!(
        dataset::told_apart(&[
            dataset("/data/parsing"),
            dataset("/data/refactoring"),
            dataset("/data/parsing")
        ]),
        Err(DatasetError::GivenTwice {
            directory: PathBuf::from("/data/parsing")
        })
    );
}

#[test]
fn two_datasets_in_directories_of_the_same_name_are_not_told_apart() {
    assert_eq!(
        dataset::told_apart(&[dataset("/data/parsing"), dataset("/more/parsing")]),
        Err(DatasetError::SameName {
            name: "parsing".to_owned()
        })
    );
}
