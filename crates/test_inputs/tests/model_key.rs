use std::fs;

use test_inputs::{model_key_file, temporary_directory};

#[test]
fn the_file_of_a_key_is_named_model_key_in_the_directory() {
    let directory = temporary_directory();
    assert_eq!(
        model_key_file(directory.path(), "a-made-up-key\n"),
        directory.path().join("model-key")
    );
}

#[test]
fn the_file_of_a_key_holds_the_text() {
    let directory = temporary_directory();
    let file = model_key_file(directory.path(), "a-made-up-key\n");
    assert_eq!(fs::read_to_string(file).unwrap(), "a-made-up-key\n");
}

#[test]
fn a_text_that_has_no_line_ending_is_given_none() {
    let directory = temporary_directory();
    let file = model_key_file(directory.path(), "a key ");
    assert_eq!(fs::read_to_string(file).unwrap(), "a key ");
}
