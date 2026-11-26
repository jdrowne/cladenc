use test_inputs::manual_on_one_line;

#[test]
fn the_manual_of_start_says_which_directed_evolution_report_and_status_are_each_for() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "cladenc report is for a directed evolution that has ended, and cladenc status is for one \
         that has not ended."
    ));
}

#[test]
fn the_manual_of_start_says_that_report_and_status_each_refuse_the_other_kind() {
    assert!(
        manual_on_one_line("cladenc-start")
            .contains("Each refuses the other kind, and says which of the two to call.")
    );
}

#[test]
fn the_manual_of_start_says_what_start_prints_when_a_directed_evolution_ends() {
    assert!(manual_on_one_line("cladenc-start").contains(
        "When a directed evolution ends, cladenc start prints on standard error what cladenc \
         report prints, after the line that says that the directed evolution finished, and prints \
         on standard output the directory of the best harness and nothing else."
    ));
}

#[test]
fn the_manual_of_status_says_that_a_directed_evolution_that_has_not_ended_has_no_best_harness() {
    assert!(
        manual_on_one_line("cladenc-status")
            .contains("A directed evolution that has not ended has no best harness.")
    );
}

#[test]
fn the_manual_of_status_says_which_harness_status_names_in_place_of_the_best_harness() {
    assert!(manual_on_one_line("cladenc-status").contains(
        "In its place cladenc status names the harness that is ahead so far: the harness that \
         would be named the best harness if the directed evolution ended now."
    ));
}

#[test]
fn the_manual_of_status_says_that_what_status_prints_is_the_state_so_far_which_can_change() {
    assert!(manual_on_one_line("cladenc-status").contains(
        "What cladenc status prints is the state so far, which can change until the directed \
         evolution ends: another harness can be ahead later, and the lead of the harness that is \
         ahead so far can grow or shrink."
    ));
}

#[test]
fn the_manual_of_status_says_once_that_what_status_prints_can_change() {
    assert_eq!(
        manual_on_one_line("cladenc-status")
            .matches("is the state so far, which can change")
            .count(),
        1
    );
}

#[test]
fn the_manual_of_status_says_which_three_things_status_says_of_the_harness_ahead_so_far() {
    assert!(manual_on_one_line("cladenc-status").contains(
        "cladenc status says the same three things of the harness that is ahead so far and the \
         primogenitor."
    ));
}

#[test]
fn the_manual_of_status_says_that_one_of_many_likelihoods_is_low_by_luck_more_often() {
    assert!(manual_on_one_line("cladenc-status").contains(
        "A person who calls cladenc status many times reads many likelihoods, and one of many \
         likelihoods is low by luck more often than one that is read once."
    ));
}

#[test]
fn the_manual_of_status_says_that_a_likelihood_status_prints_is_not_the_result() {
    assert!(manual_on_one_line("cladenc-status").contains(
        "Each of them is the state so far, and not the result of the directed evolution."
    ));
}

#[test]
fn the_manual_of_resume_says_what_resume_prints_when_the_directed_evolution_ends() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "When the directed evolution that it continued ends, cladenc resume prints on standard \
         error what cladenc report prints, after the line that says that the directed evolution \
         finished, and prints on standard output the directory of the best harness and nothing \
         else."
    ));
}

#[test]
fn the_manual_of_resume_says_what_resume_prints_of_a_directed_evolution_that_had_ended() {
    assert!(manual_on_one_line("cladenc-resume").contains(
        "Of a directed evolution that had ended already, it prints the same after a line that says \
         that the directed evolution had finished."
    ));
}

#[test]
fn the_manual_of_report_says_what_the_clade_completion_rate_is() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "the clade completion rate, which is the part of the finished trials of the clade on the \
         selection tasks in which the task was completed,"
    ));
}

#[test]
fn the_manual_of_report_says_which_three_counts_it_gives_of_each_set_of_tasks() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "how many of the selection tasks it completed, how many it did not complete and how \
         many of its trials on them have no verdict, the same three counts of the corroboration \
         tasks and of the non-holdout tasks, the three counts of the selection tasks for its \
         clade, which is the harness and all its descendants together,"
    ));
}

#[test]
fn the_manual_of_report_says_what_a_trial_with_no_verdict_is() {
    assert!(manual_on_one_line("cladenc-report").contains(
        "A trial has no verdict where its verifier left none that can be read or ran past its \
         time limit: the harness is counted neither as having completed the task nor as not \
         having completed it. Such a trial on a selection task is one of the finished trials \
         of the clade completion rate all the same."
    ));
}
