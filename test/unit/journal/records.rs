use cladenc::genealogy::{ProcreationOutcome, Score};
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

fn assessment_finished(score: Score) -> Record {
    Record::AssessmentFinished {
        harness: 1,
        task: "t2".to_owned(),
        score,
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
fn the_end_of_an_assessment_is_written_with_a_score_of_success() {
    assert_eq!(
        line(&assessment_finished(Score::Success)),
        r#"{"record":"assessment_finished","harness":1,"task":"t2","score":"success"}"#
    );
}

#[test]
fn the_end_of_an_assessment_is_written_with_a_score_of_failure() {
    assert_eq!(
        line(&assessment_finished(Score::Failure)),
        r#"{"record":"assessment_finished","harness":1,"task":"t2","score":"failure"}"#
    );
}

#[test]
fn the_end_of_an_assessment_is_written_with_a_score_of_timed_out() {
    assert_eq!(
        line(&assessment_finished(Score::TimedOut)),
        r#"{"record":"assessment_finished","harness":1,"task":"t2","score":"timed_out"}"#
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
    let record = assessment_finished(Score::TimedOut);
    assert_eq!(read_back(&record), record);
}

#[test]
fn the_end_of_a_directed_evolution_survives_being_written_and_read_back() {
    let record = Record::DirectedEvolutionFinished { best_harness: 1 };
    assert_eq!(read_back(&record), record);
}
