use std::fs;

use cladenc::genealogy::GenealogyError;
use cladenc::journal::{self, JournalError};

const RUN: &str = concat!(
    r#"{"record":"run","format":2,"progenitor":"/abs/harness","docker_image":"image","#,
    r#""harness_run_command":"run a harness","tasks":["t1","t2"],"#,
    r#""evaluator":"/abs/evaluate","budget":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
    r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
);

fn error_of_text(text: &str) -> JournalError {
    let run_directory = tempfile::tempdir().unwrap();
    fs::write(journal::path(run_directory.path()), text).unwrap();
    journal::read(run_directory.path()).unwrap_err()
}

fn error_of(lines: &[&str]) -> JournalError {
    error_of_text(
        &lines
            .iter()
            .map(|line| format!("{line}\n"))
            .collect::<String>(),
    )
}

#[test]
fn a_journal_whose_file_is_missing_cannot_be_read() {
    let run_directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        journal::read(run_directory.path()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_journal_that_is_empty_cannot_be_read() {
    assert!(matches!(error_of(&[]), JournalError::FirstRecordNotRun));
}

#[test]
fn a_journal_that_is_only_a_cut_short_line_cannot_be_read() {
    assert!(matches!(
        error_of_text(RUN),
        JournalError::FirstRecordNotRun
    ));
}

#[test]
fn a_journal_whose_first_record_is_not_run_cannot_be_read() {
    assert!(matches!(
        error_of(&[r#"{"record":"search_finished","best_harness":0}"#]),
        JournalError::FirstRecordNotRun
    ));
}

#[test]
fn a_journal_whose_format_is_not_2_cannot_be_read() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":2"#, r#""format":3"#)]),
        JournalError::UnknownFormat { format: 3 }
    ));
}

#[test]
fn a_journal_of_format_1_is_refused_for_its_format_though_its_settings_are_others() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":1,"progenitor":"/abs/harness","tasks":["t1","t2"],"#,
            r#""procreator":"/abs/procreate","evaluator":"/abs/evaluate","budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_lacks_a_setting_is_not_a_record() {
    assert!(matches!(
        error_of(&[r#"{"record":"run","format":2,"progenitor":"/abs/harness"}"#]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_journal_whose_first_line_is_not_a_record_cannot_be_read() {
    assert!(matches!(
        error_of(&["not a record"]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_journal_with_a_line_that_is_not_json_cannot_be_read() {
    assert!(matches!(
        error_of(&[RUN, "not a record"]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_journal_with_a_record_of_an_unknown_name_cannot_be_read() {
    assert!(matches!(
        error_of(&[RUN, r#"{"record":"paused"}"#]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_journal_with_a_record_that_lacks_a_key_cannot_be_read() {
    assert!(matches!(
        error_of(&[RUN, r#"{"record":"procreation_started","child":1}"#]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_journal_with_an_unknown_outcome_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":1,"parent":0}"#,
            r#"{"record":"procreation_finished","child":1,"outcome":"lost"}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_journal_with_a_blank_line_cannot_be_read() {
    assert!(matches!(
        error_of(&[RUN, ""]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_journal_with_a_second_run_record_cannot_be_read() {
    assert!(matches!(
        error_of(&[RUN, RUN]),
        JournalError::RunRecordRepeated { line: 2 }
    ));
}

#[test]
fn a_journal_with_a_procreation_of_a_harness_that_was_not_created_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":1,"parent":5}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ParentNotCreated { parent: 5 }
        }
    ));
}

#[test]
fn a_journal_with_the_end_of_a_procreation_that_never_started_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_finished","child":1,"outcome":"created"}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ChildNotBeingCreated { child: 1 }
        }
    ));
}

#[test]
fn a_journal_that_gives_a_harness_one_task_twice_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"evaluation_started","harness":0,"task":"t1"}"#,
            r#"{"record":"evaluation_started","harness":0,"task":"t1"}"#
        ]),
        JournalError::Refused {
            line: 3,
            error: GenealogyError::TaskAlreadyGiven { harness: 0, .. }
        }
    ));
}

#[test]
fn a_journal_with_the_end_of_an_evaluation_that_never_started_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"evaluation_finished","harness":0,"task":"t1","outcome":"success"}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::EvaluationNotRunning { harness: 0, .. }
        }
    ));
}

#[test]
fn a_journal_whose_procreation_names_a_child_other_than_the_next_id_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":7,"parent":0}"#
        ]),
        JournalError::WrongChild { line: 2, child: 7 }
    ));
}

#[test]
fn a_journal_with_a_record_after_the_end_of_its_search_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"search_finished","best_harness":0}"#,
            r#"{"record":"procreation_started","child":1,"parent":0}"#
        ]),
        JournalError::AfterSearchFinished { line: 3 }
    ));
}
