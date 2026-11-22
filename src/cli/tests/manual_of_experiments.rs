use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_what_experiment_cladenc_conducts_and_at_which_procreations() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "At each procreation of a harness other than the primogenitor, cladenc conducts an \
         experiment to test whether a progenitor is better at procreating better harnesses than \
         the primogenitor."
    ));
}

#[test]
fn the_manual_of_start_says_how_cladenc_establishes_the_baseline_of_an_experiment() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "To conduct the experiment, cladenc establishes a baseline: it has the primogenitor create \
         a harness from the same source code that the progenitor uses to procreate the \
         progenitor's child."
    ));
}

#[test]
fn the_manual_of_start_names_the_singleton_control_group_harness() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The harness the primogenitor creates from the source code of the progenitor is the \
         singleton control group harness."
    ));
}

#[test]
fn the_manual_of_start_says_which_source_code_runs_and_which_is_changed_for_a_baseline() {
    let help = manual_on_one_line("cladenc-start");
    let given = [
        "/var/opt/cladenc/harness holds a copy of the source code of the",
        "primogenitor, which is what runs",
        "/var/opt/cladenc/work holds a second copy of the source code of the",
        "progenitor, which is what is changed",
    ];
    assert!(given.iter().all(|line| help.contains(line)));
}

#[test]
fn the_manual_of_start_says_that_the_primogenitor_is_given_what_the_progenitor_is_given() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The primogenitor is given the instruction the progenitor is given, and what the \
         progenitor is given to read:"
    ));
}

#[test]
fn the_manual_of_start_says_which_children_of_the_progenitor_the_primogenitor_is_given() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "under /var/log/cladenc/children the children the progenitor procreated before the \
         progenitor's child of the experiment."
    ));
}

#[test]
fn the_manual_of_start_says_that_the_singleton_control_group_harness_is_benchmarked_in_full() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "The singleton control group harness is built and benchmarked in full, as the progenitor's \
         child is, and it takes no other part in the directed evolution:"
    ));
}

#[test]
fn the_manual_of_start_says_what_the_singleton_control_group_harness_takes_no_part_in() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "it is never chosen to procreate, it is counted in no clade, it is not named the best \
         harness, and the container it is created in does not count toward --max-procreations."
    ));
}

#[test]
fn the_manual_of_start_says_that_no_experiment_is_conducted_where_the_primogenitor_procreates() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the primogenitor procreates, no experiment is conducted: the progenitor's child and \
         the singleton control group harness would be created in the same way."
    ));
}

#[test]
fn the_manual_of_start_says_what_the_outputs_directory_keeps_of_a_baseline() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "baselines in the outputs directory keeps, in a directory named after the singleton \
         control group harness, the standard output and standard error of the primogenitor, the id \
         of the progenitor in the file named progenitor, the id of the progenitor's child in the \
         file named child, and how the creation ended in the file named outcome."
    ));
}

#[test]
fn the_manual_of_report_says_what_report_and_status_say_of_the_experiments() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "cladenc report and cladenc status say of the experiments how many were conducted, in how \
         many the progenitor's child was created, in how many the singleton control group harness \
         was created, and in how many both were."
    ));
}

#[test]
fn the_manual_of_report_says_how_the_harnesses_of_the_experiments_are_told_apart() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "Over the experiments where both were created, each counts the corroboration tasks that \
         only the progenitor's child completed and those that only the singleton control group \
         harness completed, and says how likely luck alone gives a lead at least as large where \
         the two are created equally well"
    ));
}

#[test]
fn the_manual_of_start_says_that_a_baseline_that_changes_nothing_creates_no_harness() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "Where the primogenitor leaves /var/opt/cladenc/work as it was, no singleton control group \
         harness is created, and the creation is recorded as unchanged."
    ));
}

#[test]
fn the_manual_of_report_says_what_report_and_status_say_where_no_experiment_created_both() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "Where no experiment created both the progenitor's child and the singleton control group \
         harness, each says so in the place of those counts and of the likelihood, and --json \
         gives no likelihood for the experiments."
    ));
}
