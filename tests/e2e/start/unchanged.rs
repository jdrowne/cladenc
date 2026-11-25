use crate::shared::UNCHANGED;

#[test]
fn a_run_whose_procreation_changed_nothing_exits_with_0() {
    let (_directory, output) = &*UNCHANGED;
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_run_whose_procreation_changed_nothing_prints_the_directory_of_the_primogenitor() {
    let (directory, output) = &*UNCHANGED;
    assert_eq!(
        str::from_utf8(&output.stdout).unwrap(),
        format!(
            "{}\n",
            directory.path().join("outputs/harnesses/0").display()
        )
    );
}

#[test]
fn a_run_says_that_a_procreation_left_the_copy_unchanged_and_created_no_harness() {
    let (directory, output) = &*UNCHANGED;
    assert!(str::from_utf8(&output.stderr).unwrap().contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 left the copy of the source code of \
         harness 0 unchanged, and created no harness: what the harness printed is kept in {}\n",
        directory.path().join("outputs/procreations/1").display()
    )));
}

#[test]
fn the_report_of_a_run_shows_a_child_that_would_be_the_copy_as_it_was_as_unchanged() {
    let (_directory, output) = &*UNCHANGED;
    assert!(
        str::from_utf8(&output.stderr)
            .unwrap()
            .lines()
            .last()
            .is_some_and(|row| row.starts_with("1   0           -           unchanged  "))
    );
}

#[test]
fn a_run_benchmarks_no_child_that_would_be_the_copy_as_it_was() {
    let (directory, _output) = &*UNCHANGED;
    assert!(!directory.path().join("outputs/attempts/1").exists());
}
