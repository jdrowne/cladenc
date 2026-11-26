fn readme() -> String {
    test_inputs::README.to_owned()
}

#[test]
fn the_readme_says_where_the_package_installs_the_program_and_how_a_person_calls_it() {
    assert!(readme().contains(
        "The `.deb` package of a release installs the program at `/opt/cladenc/bin/cladenc` and \
         nothing outside `/opt/cladenc`, so a person calls the program by that full path, or \
         adds `/opt/cladenc/bin` to `PATH` and calls it as `cladenc`."
    ));
}

#[test]
fn the_readme_names_no_directory_under_var_lib() {
    assert!(!readme().contains("/var/lib"));
}

#[test]
fn the_readme_says_where_the_source_code_of_the_harness_of_a_trial_is() {
    assert!(readme().contains("with its source code copied in at `/var/opt/cladenc/harness`."));
}

#[test]
fn the_readme_says_where_the_source_code_and_the_copy_are_in_the_sandbox_of_a_diversification() {
    assert!(readme().contains(
        "In the sandbox of a diversification, the source code of the progenitor is at \
         `/var/opt/cladenc/harness`, and the clone that the progenitor diversifies is at \
         `/var/opt/cladenc/work`, the working directory."
    ));
}

#[test]
fn the_readme_says_what_the_environment_of_every_task_has_to_hold() {
    assert!(readme().contains(
        "the environment of every task has to hold `sh`, `sleep`, `rm`, `mkdir`, `chown` and \
         `ln`."
    ));
}

#[test]
fn the_readme_says_that_tests_is_a_symbolic_link_to_the_copy_of_the_tests_of_the_task() {
    assert!(readme().contains(
        "Once the harness has ended, the task's `tests` directory is copied into the same \
         sandbox, where `/tests` is a symbolic link to the copy, at \
         `/var/opt/cladenc/harbor/tests`, and the verifier, `/tests/test.sh`, writes its verdict"
    ));
}

#[test]
fn the_readme_says_what_cladenc_makes_at_logs_before_the_harness_is_started() {
    assert!(readme().contains(
        "Before the harness is started, `cladenc` makes `/logs` a symbolic link to \
         `/var/log/cladenc/harbor` and makes `/logs/artifacts` an empty directory that belongs \
         to the user the harness runs as, so a harness that does not run as root saves a file \
         there as well."
    ));
}

#[test]
fn the_readme_says_what_cladenc_does_where_the_environment_holds_logs() {
    assert!(readme().contains(
        "Where the image of the environment already holds something at `/logs`, `cladenc` \
         makes no link, leaves what is there, and makes `/logs/artifacts` inside it."
    ));
}

#[test]
fn the_readme_says_that_cladenc_makes_no_directory_at_the_top_of_the_filesystem() {
    assert!(readme().contains(
        "`/logs` and `/tests` are the two names that Harbor's task layout fixes at the top of \
         the filesystem, and `cladenc` makes no directory there: each of the two is a symbolic \
         link, as the Filesystem Hierarchy Standard asks of an application."
    ));
}

#[test]
fn the_readme_says_where_the_package_installs_the_manual_pages_and_how_man_shows_one() {
    assert!(readme().contains(
        "`--help` after a sub-command prints its flags, and the manual page of the sub-command \
         says the rest. The package installs the manual pages under `/opt/cladenc/share/man`: \
         `man -M /opt/cladenc/share/man cladenc-start` shows the page of `cladenc start`, and \
         `man cladenc-start` shows it where `/opt/cladenc/bin` is on `PATH`."
    ));
}

#[test]
fn the_readme_says_how_cladenc_copies_out_of_a_sandbox_and_what_it_keeps() {
    assert!(readme().contains(
        "Whatever `cladenc` copies out of a sandbox, it has Docker print as an archive and \
         unpacks itself: a directory that cannot be written in is kept with all it holds, a file \
         that no one may read is kept as a file that its owner can read, and a file that is not \
         a regular file, a directory or a symbolic link, such as a named pipe, is not kept."
    ));
}

#[test]
fn the_readme_says_what_cladenc_does_where_a_copy_out_of_a_sandbox_fails() {
    assert!(readme().contains(
        "Where a copy out of a sandbox fails though what it would copy is in the sandbox, \
         `cladenc` does not keep part of what was left as though it were all of it: no result is \
         recorded for the trial, the directed evolution stops with exit status 1, `cladenc` \
         names the harness and the task, says which copy failed and why, and `cladenc resume` \
         carries out the trial again."
    ));
}

#[test]
fn the_readme_says_what_cladenc_does_where_docker_does_not_list_the_changes_of_a_sandbox() {
    assert!(readme().contains(
        "Where Docker does not list what was changed in the sandbox, before the harness is \
         started or once it has ended, no list is kept as though it were what the harness \
         changed: no result is recorded for the trial either, the directed evolution stops with \
         exit status 1, `cladenc` says what Docker said, and `cladenc resume` carries out the \
         trial again."
    ));
}

#[test]
fn the_readme_says_what_cladenc_does_where_it_cannot_write_what_it_records_of_a_trial() {
    assert!(readme().contains(
        "Where `cladenc` cannot write the ending of the harness or the list of what it changed \
         in the directory of the trial, as where the outputs directory has no room left, the \
         trial is not kept without it: no result is recorded for the trial either, the directed \
         evolution stops with exit status 1, `cladenc` names the file that was not written and \
         says why, and `cladenc resume` carries out the trial again."
    ));
}

#[test]
fn the_readme_says_that_the_permission_bits_a_harness_leaves_keep_no_child_from_being_made() {
    assert!(readme().contains(
        "Once the progenitor has ended, and again once a build has ended, `cladenc` makes every \
         directory of the source code one that its owner can list, write in and enter, and \
         every file one that its owner can read, so that the permission bits a harness or a \
         build leaves keep no child from being made or evaluated."
    ));
}
