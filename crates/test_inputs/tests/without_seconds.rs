use test_inputs::without_seconds;

const STARTED: &str = r#"{"record":"attempt_started","harness":1,"task":"t2"}"#;
const FINISHED: &str =
    r#"{"record":"attempt_finished","harness":1,"task":"t2","result":"completed","seconds":3.25}"#;
const FINISHED_LATER: &str =
    r#"{"record":"attempt_finished","harness":1,"task":"t2","result":"completed","seconds":41.5}"#;
const FINISHED_WITHOUT_SECONDS: &str =
    r#"{"record":"attempt_finished","harness":1,"task":"t2","result":"completed"}"#;

#[test]
fn the_seconds_at_the_end_of_a_record_are_left_out() {
    assert_eq!(
        without_seconds(&format!("{FINISHED}\n")),
        format!("{FINISHED_WITHOUT_SECONDS}\n")
    );
}

#[test]
fn two_records_that_differ_in_their_seconds_alone_come_out_the_same() {
    assert_eq!(
        without_seconds(&format!("{FINISHED}\n")),
        without_seconds(&format!("{FINISHED_LATER}\n"))
    );
}

#[test]
fn a_record_that_has_no_seconds_is_left_as_it_is() {
    assert_eq!(
        without_seconds(&format!("{STARTED}\n")),
        format!("{STARTED}\n")
    );
}

#[test]
fn each_record_of_a_journal_is_taken_by_itself() {
    assert_eq!(
        without_seconds(&format!("{STARTED}\n{FINISHED}\n")),
        format!("{STARTED}\n{FINISHED_WITHOUT_SECONDS}\n")
    );
}

#[test]
fn a_journal_of_no_record_stays_empty() {
    assert_eq!(without_seconds(""), "");
}
