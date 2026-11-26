use super::given::{RUN, error_of, error_of_text};
use crate::file_system::tests::double::Memory;
use crate::genealogy::GenealogyError;
use crate::journal::{self, JournalError};

fn run_that_builds() -> String {
    RUN.replace(
        r#""harness_build_command":null"#,
        r#""harness_build_command":"make harness""#,
    )
}

#[test]
fn a_journal_whose_file_is_missing_cannot_be_read() {
    let directory = Memory::default();
    assert!(matches!(
        journal::read(&directory, directory.path()),
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
        error_of(&[r#"{"record":"directed_evolution_finished","best_harness":0}"#]),
        JournalError::FirstRecordNotRun
    ));
}

#[test]
fn the_end_of_a_trial_of_this_format_that_holds_no_seconds_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_started","harness":0,"task":"benchmark/t1"}"#,
            r#"{"record":"trial_finished","harness":0,"task":"benchmark/t1","result":"completed"}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn the_end_of_a_diversification_of_this_format_that_holds_no_seconds_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_started","child":1,"progenitor":0}"#,
            r#"{"record":"diversification_finished","child":1,"outcome":"created"}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_lacks_a_setting_is_not_a_record() {
    assert!(matches!(
        error_of(&[r#"{"record":"run","format":32,"primogenitor":"/abs/harness"}"#]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_holdout_tasks_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""holdout_tasks":["benchmark/t2","benchmark/t3"],"#, "")]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_repository_of_environments_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(
            r#""environment_image_repository":"cladenc-environments-0123456789abcdef","#,
            ""
        )]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_corroboration_tasks_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""corroboration_tasks":["benchmark/t3"],"#, "")]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_corroboration_tasks_percentage_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""corroboration_tasks_percentage":25.0,"#, "")]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_selection_tasks_percentage_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""selection_tasks_percentage":25.0,"#, "")]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_a_budget_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""max_diversifications":800"#, r#""budget":800"#)]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_the_progenitor_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""primogenitor":"#, r#""progenitor":"#)]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn the_start_of_a_diversification_of_this_format_that_names_the_parent_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_started","child":1,"parent":0}"#
        ]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_record_of_this_format_named_progenitor_built_is_not_a_record() {
    assert!(matches!(
        error_of(&[&run_that_builds(), r#"{"record":"progenitor_built"}"#]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_time_limit_of_a_diversification_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(
            r#""diversification_timeout":"#,
            r#""diversification_time_limit":"#
        )]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_no_most_retries_of_a_verifier_s_health_check_is_not_a_record()
 {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""max_verifier_health_check_retries":3,"#, "")]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_record_of_this_format_named_assessment_started_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"assessment_started","harness":0,"task":"t1"}"#
        ]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_record_of_this_format_named_attempt_started_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
        ]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_record_of_this_format_named_procreation_started_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":1,"progenitor":0}"#
        ]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn the_end_of_a_trial_of_this_format_that_names_a_score_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#,
            r#"{"record":"trial_finished","harness":0,"task":"t1","score":"completed","seconds":1.5}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn the_end_of_a_trial_of_this_format_whose_result_is_success_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#,
            r#"{"record":"trial_finished","harness":0,"task":"t1","result":"success","seconds":1.5}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_run_record_of_this_format_whose_limit_of_containers_has_another_key_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""max_concurrent_containers":8"#, r#""workers":8"#)]),
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
        error_of(&[RUN, r#"{"record":"diversification_started","child":1}"#]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_journal_with_an_unknown_outcome_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_started","child":1,"progenitor":0}"#,
            r#"{"record":"diversification_finished","child":1,"outcome":"lost","seconds":1.5}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_journal_with_an_unknown_result_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#,
            r#"{"record":"trial_finished","harness":0,"task":"t1","result":"passed","seconds":1.5}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_journal_that_calls_the_result_of_a_trial_an_outcome_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#,
            r#"{"record":"trial_finished","harness":0,"task":"t1","outcome":"completed","seconds":1.5}"#
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
fn a_journal_with_a_diversification_of_a_harness_that_was_not_created_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_started","child":1,"progenitor":5}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ProgenitorNotCreated { progenitor: 5 }
        }
    ));
}

#[test]
fn a_journal_with_the_end_of_a_diversification_that_never_started_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_finished","child":1,"outcome":"created","seconds":1.5}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ChildNotBeingCreated { child: 1 }
        }
    ));
}

#[test]
fn a_journal_with_the_build_of_a_primogenitor_that_awaited_none_cannot_be_read() {
    assert!(matches!(
        error_of(&[RUN, r#"{"record":"primogenitor_built"}"#]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::PrimogenitorNotAwaitingBuild
        }
    ));
}

#[test]
fn a_journal_that_builds_its_primogenitor_twice_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            &run_that_builds(),
            r#"{"record":"primogenitor_built"}"#,
            r#"{"record":"primogenitor_built"}"#
        ]),
        JournalError::Refused {
            line: 3,
            error: GenealogyError::PrimogenitorNotAwaitingBuild
        }
    ));
}

#[test]
fn a_journal_that_starts_a_diversification_before_its_primogenitor_was_built_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            &run_that_builds(),
            r#"{"record":"diversification_started","child":1,"progenitor":0}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ProgenitorNotCreated { progenitor: 0 }
        }
    ));
}

#[test]
fn a_journal_that_starts_a_trial_before_its_primogenitor_was_built_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            &run_that_builds(),
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::HarnessNotCreated { harness: 0 }
        }
    ));
}

#[test]
fn a_journal_that_gives_a_harness_one_task_twice_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#,
            r#"{"record":"trial_started","harness":0,"task":"t1"}"#
        ]),
        JournalError::Refused {
            line: 3,
            error: GenealogyError::TaskAlreadyGiven { harness: 0, .. }
        }
    ));
}

#[test]
fn a_journal_with_the_end_of_a_trial_that_never_started_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"trial_finished","harness":0,"task":"t1","result":"completed","seconds":1.5}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::TrialNotRunning { harness: 0, .. }
        }
    ));
}

#[test]
fn a_journal_whose_diversification_names_a_child_other_than_the_next_id_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_started","child":7,"progenitor":0}"#
        ]),
        JournalError::WrongChild { line: 2, child: 7 }
    ));
}

#[test]
fn a_journal_with_a_record_after_the_end_of_its_directed_evolution_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"directed_evolution_finished","best_harness":0}"#,
            r#"{"record":"diversification_started","child":1,"progenitor":0}"#
        ]),
        JournalError::AfterDirectedEvolutionFinished { line: 3 }
    ));
}

const A_CHILD_OF_A_CHILD: [&str; 5] = [
    RUN,
    r#"{"record":"diversification_started","child":1,"progenitor":0}"#,
    r#"{"record":"diversification_finished","child":1,"outcome":"created","seconds":1.5}"#,
    r#"{"record":"diversification_started","child":2,"progenitor":1}"#,
    r#"{"record":"baseline_started","harness":3,"child":2,"progenitor":1}"#,
];

fn error_after_a_baseline_started(line: &str) -> JournalError {
    let lines: Vec<&str> = A_CHILD_OF_A_CHILD.iter().copied().chain([line]).collect();
    error_of(&lines)
}

#[test]
fn a_journal_with_a_baseline_where_the_primogenitor_was_cloned_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"diversification_started","child":1,"progenitor":0}"#,
            r#"{"record":"baseline_started","harness":2,"child":1,"progenitor":0}"#
        ]),
        JournalError::Refused {
            line: 3,
            error: GenealogyError::ExperimentNotAwaitingBaseline {
                child: 1,
                progenitor: 0
            }
        }
    ));
}

#[test]
fn a_journal_with_a_second_baseline_of_one_experiment_cannot_be_read() {
    assert!(matches!(
        error_after_a_baseline_started(
            r#"{"record":"baseline_started","harness":4,"child":2,"progenitor":1}"#
        ),
        JournalError::Refused {
            line: 6,
            error: GenealogyError::ExperimentNotAwaitingBaseline {
                child: 2,
                progenitor: 1
            }
        }
    ));
}

#[test]
fn a_journal_whose_baseline_names_a_harness_other_than_the_next_id_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"baseline_started","harness":7,"child":2,"progenitor":1}"#
        ]),
        JournalError::WrongChild { line: 2, child: 7 }
    ));
}

#[test]
fn a_journal_with_the_end_of_a_baseline_that_never_started_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"baseline_finished","harness":1,"outcome":"created","seconds":1.5}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::SingletonControlGroupHarnessNotBeingCreated { harness: 1 }
        }
    ));
}

#[test]
fn a_journal_that_ends_a_baseline_as_though_it_were_a_diversification_cannot_be_read() {
    assert!(matches!(
        error_after_a_baseline_started(
            r#"{"record":"diversification_finished","child":3,"outcome":"created","seconds":1.5}"#
        ),
        JournalError::Refused {
            line: 6,
            error: GenealogyError::ChildNotBeingCreated { child: 3 }
        }
    ));
}

#[test]
fn the_end_of_a_baseline_of_this_format_that_holds_no_seconds_is_not_a_record() {
    assert!(matches!(
        error_after_a_baseline_started(
            r#"{"record":"baseline_finished","harness":3,"outcome":"created"}"#
        ),
        JournalError::NotARecord { line: 6 }
    ));
}
