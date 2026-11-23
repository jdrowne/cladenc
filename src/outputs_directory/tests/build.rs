use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;

fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    (directory, outputs_directory)
}

#[test]
fn the_directory_of_a_build_is_named_after_its_harness_under_builds() {
    let (directory, outputs_directory) = made();
    assert_eq!(
        outputs_directory.build(3),
        directory.path().join("outputs/builds/3")
    );
}

#[test]
fn a_new_outputs_directory_holds_no_builds() {
    let (directory, _outputs_directory) = made();
    assert!(!directory.exists(&directory.path().join("outputs/builds")));
}

#[test]
fn starting_a_build_returns_its_directory() {
    let (_directory, outputs_directory) = made();
    assert_eq!(
        outputs_directory.start_build(0).unwrap(),
        outputs_directory.build(0)
    );
}

#[test]
fn a_started_build_has_an_empty_directory() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_build(0).unwrap();
    assert_eq!(directory.names_in(outputs_directory.build(0)).len(), 0);
}

#[test]
fn starting_a_build_again_removes_what_the_earlier_build_left() {
    let (directory, outputs_directory) = made();
    outputs_directory.start_build(0).unwrap();
    directory
        .write(outputs_directory.build(0).join("stdout"), "earlier\n")
        .unwrap();
    outputs_directory.start_build(0).unwrap();
    assert_eq!(directory.names_in(outputs_directory.build(0)).len(), 0);
}

#[test]
fn a_build_cannot_start_where_a_file_is_in_the_way_of_its_directory() {
    let (directory, outputs_directory) = made();
    directory
        .write(directory.path().join("outputs/builds"), "")
        .unwrap();
    assert!(outputs_directory.start_build(0).is_err());
}
