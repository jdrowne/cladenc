use std::error::Error;
use std::io;

use cladenc::genealogy::GenealogyError;
use cladenc::journal::{JournalError, Started};

fn worded(error: impl Error) -> String {
    error.to_string()
}

#[test]
fn a_reason_of_the_system_is_worded_as_the_system_words_it() {
    assert_eq!(
        worded(JournalError::Io(io::Error::from_raw_os_error(2))),
        "No such file or directory (os error 2)"
    );
}

#[test]
fn a_journal_in_use_is_worded() {
    assert_eq!(worded(JournalError::Locked), "another cladenc is using it");
}

#[test]
fn a_line_that_is_not_a_record_is_worded() {
    assert_eq!(
        worded(JournalError::NotARecord { line: 3 }),
        "line 3 of its journal is not a record"
    );
}

#[test]
fn a_journal_that_does_not_start_with_a_run_record_is_worded() {
    assert_eq!(
        worded(JournalError::FirstRecordNotRun),
        "its journal does not start with a run record"
    );
}

#[test]
fn a_format_that_is_not_read_is_worded() {
    assert_eq!(
        worded(JournalError::UnknownFormat { format: 1 }),
        "its journal has format 1, which this cladenc does not read"
    );
}

#[test]
fn a_second_run_record_is_worded() {
    assert_eq!(
        worded(JournalError::RunRecordRepeated { line: 3 }),
        "line 3 of its journal is a second run record"
    );
}

#[test]
fn a_record_the_genealogy_refuses_is_worded_with_the_genealogy_s_reason() {
    assert_eq!(
        worded(JournalError::Refused {
            line: 3,
            error: GenealogyError::ChildNotBeingCreated { child: 5 }
        }),
        "line 3 of its journal records what cannot have happened: harness 5 is not being created"
    );
}

#[test]
fn a_child_out_of_order_is_worded() {
    assert_eq!(
        worded(JournalError::WrongChild { line: 3, child: 5 }),
        "line 3 of its journal starts harness 5 out of order"
    );
}

#[test]
fn a_record_after_the_search_finished_is_worded() {
    assert_eq!(
        worded(JournalError::AfterSearchFinished { line: 3 }),
        "line 3 of its journal comes after the search finished"
    );
}

#[test]
fn a_procreation_is_named_by_its_parent_and_its_child() {
    assert_eq!(
        Started::Procreation {
            child: 3,
            parent: 0
        }
        .to_string(),
        "the procreation of harness 3 by harness 0"
    );
}

#[test]
fn an_assessment_is_named_by_its_harness_and_its_task() {
    assert_eq!(
        Started::Assessment {
            harness: 0,
            task: "t1".to_owned()
        }
        .to_string(),
        "the assessment of harness 0 on task t1"
    );
}
