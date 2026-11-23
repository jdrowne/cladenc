use crate::file_system::tests::double::Memory;
use crate::model;

fn key_in(text: &[u8]) -> Option<String> {
    let directory = Memory::default();
    let file = directory.path().join("model-key");
    directory.write(&file, text).unwrap();
    model::read_key(&directory, &file)
}

#[test]
fn the_key_is_the_line_its_file_holds_without_the_line_ending() {
    assert_eq!(key_in(b"a-made-up-key\n"), Some("a-made-up-key".to_owned()));
}

#[test]
fn a_key_with_no_line_ending_after_it_is_read_whole() {
    assert_eq!(key_in(b"a-made-up-key"), Some("a-made-up-key".to_owned()));
}

#[test]
fn a_line_ending_of_a_carriage_return_and_a_line_feed_is_left_out_of_the_key() {
    assert_eq!(
        key_in(b"a-made-up-key\r\n"),
        Some("a-made-up-key".to_owned())
    );
}

#[test]
fn a_space_in_the_line_is_part_of_the_key() {
    assert_eq!(key_in(b"a key \n"), Some("a key ".to_owned()));
}

#[test]
fn a_file_that_does_not_exist_holds_no_key() {
    let directory = Memory::default();
    assert_eq!(
        model::read_key(&directory, &directory.path().join("missing")),
        None
    );
}

#[test]
fn a_directory_holds_no_key() {
    let directory = Memory::default();
    assert_eq!(model::read_key(&directory, directory.path()), None);
}

#[test]
fn an_empty_file_holds_no_key() {
    assert_eq!(key_in(b""), None);
}

#[test]
fn a_file_that_holds_a_line_ending_alone_holds_no_key() {
    assert_eq!(key_in(b"\n"), None);
}

#[test]
fn a_file_of_two_lines_holds_no_key() {
    assert_eq!(key_in(b"a-made-up-key\nanother\n"), None);
}

#[test]
fn a_file_with_an_empty_line_after_its_key_holds_no_key() {
    assert_eq!(key_in(b"a-made-up-key\n\n"), None);
}

#[test]
fn a_file_that_holds_a_nul_byte_holds_no_key() {
    assert_eq!(key_in(b"a-made\0up-key\n"), None);
}

#[test]
fn a_file_that_is_not_text_holds_no_key() {
    assert_eq!(key_in(&[0xff, b'\n']), None);
}
