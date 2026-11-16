use cladenc::genealogy::{AssessmentOutcome, ProcreationOutcome};
use cladenc::journal::Record;

fn line(record: &Record) -> String {
    record.line().unwrap()
}

fn read_back(record: &Record) -> Record {
    serde_json::from_str(&line(record)).unwrap()
}

fn procreation_finished(outcome: ProcreationOutcome) -> Record {
    Record::ProcreationFinished { child: 1, outcome }
}

fn assessment_finished(outcome: AssessmentOutcome) -> Record {
    Record::AssessmentFinished {
        harness: 1,
        task: "t2".to_owned(),
        outcome,
    }
}

#[test]
fn the_start_of_a_procreation_is_written_with_its_child_and_parent() {
    assert_eq!(
        line(&Record::ProcreationStarted {
            child: 1,
            parent: 0
        }),
        r#"{"record":"procreation_started","child":1,"parent":0}"#
    );
}

#[test]
fn the_end_of_a_procreation_that_created_its_child_is_written_as_created() {
    assert_eq!(
        line(&procreation_finished(ProcreationOutcome::Created)),
        r#"{"record":"procreation_finished","child":1,"outcome":"created"}"#
    );
}

#[test]
fn the_end_of_a_procreation_that_failed_is_written_as_failed() {
    assert_eq!(
        line(&procreation_finished(ProcreationOutcome::Failed)),
        r#"{"record":"procreation_finished","child":1,"outcome":"failed"}"#
    );
}

#[test]
fn the_end_of_a_procreation_that_timed_out_is_written_as_timed_out() {
    assert_eq!(
        line(&procreation_finished(ProcreationOutcome::TimedOut)),
        r#"{"record":"procreation_finished","child":1,"outcome":"timed_out"}"#
    );
}

#[test]
fn the_end_of_a_procreation_whose_child_did_not_build_is_written_as_build_failed() {
    assert_eq!(
        line(&procreation_finished(ProcreationOutcome::BuildFailed)),
        r#"{"record":"procreation_finished","child":1,"outcome":"build_failed"}"#
    );
}

#[test]
fn the_build_of_the_progenitor_is_written_with_its_name_alone() {
    assert_eq!(
        line(&Record::ProgenitorBuilt),
        r#"{"record":"progenitor_built"}"#
    );
}

#[test]
fn the_build_of_the_progenitor_survives_being_written_and_read_back() {
    assert_eq!(read_back(&Record::ProgenitorBuilt), Record::ProgenitorBuilt);
}

#[test]
fn the_start_of_an_assessment_is_written_with_its_harness_and_task() {
    assert_eq!(
        line(&Record::AssessmentStarted {
            harness: 1,
            task: "t2".to_owned()
        }),
        r#"{"record":"assessment_started","harness":1,"task":"t2"}"#
    );
}

#[test]
fn the_end_of_an_assessment_that_succeeded_is_written_as_success() {
    assert_eq!(
        line(&assessment_finished(AssessmentOutcome::Success)),
        r#"{"record":"assessment_finished","harness":1,"task":"t2","outcome":"success"}"#
    );
}

#[test]
fn the_end_of_an_assessment_that_failed_is_written_as_failure() {
    assert_eq!(
        line(&assessment_finished(AssessmentOutcome::Failure)),
        r#"{"record":"assessment_finished","harness":1,"task":"t2","outcome":"failure"}"#
    );
}

#[test]
fn the_end_of_an_assessment_that_timed_out_is_written_as_timed_out() {
    assert_eq!(
        line(&assessment_finished(AssessmentOutcome::TimedOut)),
        r#"{"record":"assessment_finished","harness":1,"task":"t2","outcome":"timed_out"}"#
    );
}

#[test]
fn the_end_of_a_search_is_written_with_its_best_harness() {
    assert_eq!(
        line(&Record::SearchFinished { best_harness: 1 }),
        r#"{"record":"search_finished","best_harness":1}"#
    );
}

#[test]
fn the_start_of_a_procreation_survives_being_written_and_read_back() {
    let record = Record::ProcreationStarted {
        child: 1,
        parent: 0,
    };
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_a_procreation_survives_being_written_and_read_back() {
    let record = procreation_finished(ProcreationOutcome::TimedOut);
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_start_of_an_assessment_survives_being_written_and_read_back() {
    let record = Record::AssessmentStarted {
        harness: 1,
        task: "t2".to_owned(),
    };
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_an_assessment_survives_being_written_and_read_back() {
    let record = assessment_finished(AssessmentOutcome::TimedOut);
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_a_search_survives_being_written_and_read_back() {
    let record = Record::SearchFinished { best_harness: 1 };
    assert_eq!(read_back(&record), record);
}
