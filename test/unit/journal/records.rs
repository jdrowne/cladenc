use cladenc::genealogy::{AttemptResult, ProcreationOutcome};
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

fn attempt_finished(result: AttemptResult) -> Record {
    Record::AttemptFinished {
        harness: 1,
        task: "t2".to_owned(),
        result,
    }
}

#[test]
fn the_start_of_a_procreation_is_written_with_its_child_and_progenitor() {
    assert_eq!(
        line(&Record::ProcreationStarted {
            child: 1,
            progenitor: 0
        }),
        r#"{"record":"procreation_started","child":1,"progenitor":0}"#
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
fn the_build_of_the_primogenitor_is_written_with_its_name_alone() {
    assert_eq!(
        line(&Record::PrimogenitorBuilt),
        r#"{"record":"primogenitor_built"}"#
    );
}

#[test]
fn the_build_of_the_primogenitor_survives_being_written_and_read_back() {
    assert_eq!(
        read_back(&Record::PrimogenitorBuilt),
        Record::PrimogenitorBuilt
    );
}

#[test]
fn the_start_of_an_attempt_is_written_with_its_harness_and_task() {
    assert_eq!(
        line(&Record::AttemptStarted {
            harness: 1,
            task: "t2".to_owned()
        }),
        r#"{"record":"attempt_started","harness":1,"task":"t2"}"#
    );
}

#[test]
fn the_end_of_an_attempt_is_written_with_a_result_of_completed() {
    assert_eq!(
        line(&attempt_finished(AttemptResult::Completed)),
        r#"{"record":"attempt_finished","harness":1,"task":"t2","result":"completed"}"#
    );
}

#[test]
fn the_end_of_an_attempt_is_written_with_a_result_of_not_completed() {
    assert_eq!(
        line(&attempt_finished(AttemptResult::NotCompleted)),
        r#"{"record":"attempt_finished","harness":1,"task":"t2","result":"not_completed"}"#
    );
}

#[test]
fn the_end_of_an_attempt_is_written_with_a_result_of_timed_out() {
    assert_eq!(
        line(&attempt_finished(AttemptResult::TimedOut)),
        r#"{"record":"attempt_finished","harness":1,"task":"t2","result":"timed_out"}"#
    );
}

#[test]
fn the_end_of_a_directed_evolution_is_written_with_its_best_harness() {
    assert_eq!(
        line(&Record::DirectedEvolutionFinished { best_harness: 1 }),
        r#"{"record":"directed_evolution_finished","best_harness":1}"#
    );
}

#[test]
fn the_start_of_a_procreation_survives_being_written_and_read_back() {
    let record = Record::ProcreationStarted {
        child: 1,
        progenitor: 0,
    };
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_a_procreation_survives_being_written_and_read_back() {
    let record = procreation_finished(ProcreationOutcome::TimedOut);
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_start_of_an_attempt_survives_being_written_and_read_back() {
    let record = Record::AttemptStarted {
        harness: 1,
        task: "t2".to_owned(),
    };
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_an_attempt_survives_being_written_and_read_back() {
    let record = attempt_finished(AttemptResult::TimedOut);
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_a_directed_evolution_survives_being_written_and_read_back() {
    let record = Record::DirectedEvolutionFinished { best_harness: 1 };
    assert_eq!(read_back(&record), record);
}
