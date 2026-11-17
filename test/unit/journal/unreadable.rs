use std::fs;

use cladenc::genealogy::GenealogyError;
use cladenc::journal::{self, JournalError};

const RUN: &str = concat!(
    r#"{"record":"run","format":17,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""max_procreations":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"attempt_timeout":3600,"seed":7}"#
);

const RUN_OF_FORMAT_16: &str = concat!(
    r#"{"record":"run","format":16,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""budget":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"attempt_timeout":3600,"seed":7}"#
);

const RUN_OF_FORMAT_15: &str = concat!(
    r#"{"record":"run","format":15,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""budget":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"assessment_timeout":3600,"seed":7}"#
);

const RUN_OF_FORMAT_13: &str = concat!(
    r#"{"record":"run","format":13,"progenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""budget":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"assessment_timeout":3600,"seed":7}"#
);

const RUN_OF_FORMAT_11: &str = concat!(
    r#"{"record":"run","format":11,"progenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""budget":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"assessment_timeout":3600,"seed":7}"#
);

fn run_that_builds() -> String {
    RUN.replace(
        r#""harness_build_command":null"#,
        r#""harness_build_command":"make harness""#,
    )
}

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
        error_of(&[r#"{"record":"directed_evolution_finished","best_harness":0}"#]),
        JournalError::FirstRecordNotRun
    ));
}

#[test]
fn a_journal_whose_format_is_not_17_cannot_be_read() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":17"#, r#""format":18"#)]),
        JournalError::UnknownFormat { format: 18 }
    ));
}

#[test]
fn a_journal_of_format_16_is_refused_for_its_format_since_it_names_a_budget() {
    assert!(matches!(
        error_of(&[
            RUN_OF_FORMAT_16,
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
        ]),
        JournalError::UnknownFormat { format: 16 }
    ));
}

#[test]
fn a_journal_of_format_15_is_refused_for_its_format_since_it_names_assessments_and_scores() {
    assert!(matches!(
        error_of(&[
            RUN_OF_FORMAT_15,
            r#"{"record":"assessment_started","harness":0,"task":"t1"}"#,
            r#"{"record":"assessment_finished","harness":0,"task":"t1","score":"success"}"#
        ]),
        JournalError::UnknownFormat { format: 15 }
    ));
}

#[test]
fn a_journal_of_format_14_is_refused_for_its_format_since_it_names_the_parent() {
    assert!(matches!(
        error_of(&[
            &RUN_OF_FORMAT_15.replace(r#""format":15"#, r#""format":14"#),
            r#"{"record":"procreation_started","child":1,"parent":0}"#
        ]),
        JournalError::UnknownFormat { format: 14 }
    ));
}

#[test]
fn a_journal_of_format_13_is_refused_for_its_format_since_it_names_the_progenitor() {
    assert!(matches!(
        error_of(&[RUN_OF_FORMAT_13, r#"{"record":"progenitor_built"}"#]),
        JournalError::UnknownFormat { format: 13 }
    ));
}

#[test]
fn a_journal_of_format_12_is_refused_for_its_format_since_it_calls_a_score_an_outcome() {
    assert!(matches!(
        error_of(&[
            &RUN_OF_FORMAT_13.replace(r#""format":13"#, r#""format":12"#),
            r#"{"record":"assessment_started","harness":0,"task":"t1"}"#,
            r#"{"record":"assessment_finished","harness":0,"task":"t1","outcome":"success"}"#
        ]),
        JournalError::UnknownFormat { format: 12 }
    ));
}

#[test]
fn a_journal_of_format_11_is_refused_for_its_format_since_it_names_initial_procreations() {
    assert!(matches!(
        error_of(&[RUN_OF_FORMAT_11]),
        JournalError::UnknownFormat { format: 11 }
    ));
}

#[test]
fn a_journal_of_format_10_is_refused_for_its_format_since_its_end_is_search_finished() {
    assert!(matches!(
        error_of(&[
            &RUN_OF_FORMAT_11.replace(r#""format":11"#, r#""format":10"#),
            r#"{"record":"search_finished","best_harness":0}"#
        ]),
        JournalError::UnknownFormat { format: 10 }
    ));
}

#[test]
fn a_journal_of_format_9_is_refused_for_its_format_since_it_names_evaluations() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":9,"progenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
            r#""budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"#,
            r#""max_concurrent_containers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 9 }
    ));
}

#[test]
fn a_journal_of_format_8_is_refused_for_its_format_since_it_names_the_limit_otherwise() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":8,"progenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
            r#""budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 8 }
    ));
}

#[test]
fn a_journal_of_format_7_is_refused_for_its_format_since_it_names_one_benchmark() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":7,"progenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmark":"/abs/benchmark","tasks":["t1","t2"],"budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 7 }
    ));
}

#[test]
fn a_journal_of_format_6_is_refused_for_its_format_since_it_names_an_evaluator() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":6,"progenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmark":"/abs/benchmark","tasks":["t1","t2"],"#,
            r#""evaluator":"/abs/evaluate","budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 6 }
    ));
}

#[test]
fn a_journal_of_format_5_is_refused_for_its_format_since_it_says_nothing_of_a_dockerfile() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":5,"progenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmark":"/abs/benchmark","tasks":["t1","t2"],"#,
            r#""evaluator":"/abs/evaluate","budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 5 }
    ));
}

#[test]
fn a_journal_of_format_4_is_refused_for_its_format_since_it_names_no_benchmark() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":4,"progenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""tasks":["t1","t2"],"evaluator":"/abs/evaluate","budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 4 }
    ));
}

#[test]
fn a_journal_of_format_3_is_refused_for_its_format_since_it_names_no_model() {
    assert!(matches!(
        error_of(&[concat!(
            r#"{"record":"run","format":3,"progenitor":"/abs/harness","docker_image":"image","#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""tasks":["t1","t2"],"evaluator":"/abs/evaluate","budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )]),
        JournalError::UnknownFormat { format: 3 }
    ));
}

#[test]
fn a_journal_of_format_2_is_refused_for_its_format_though_this_cladenc_could_read_its_records() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":17"#, r#""format":2"#)]),
        JournalError::UnknownFormat { format: 2 }
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
        error_of(&[r#"{"record":"run","format":17,"primogenitor":"/abs/harness"}"#]),
        JournalError::NotARecord { line: 1 }
    ));
}

#[test]
fn a_run_record_of_this_format_that_names_a_budget_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""max_procreations":800"#, r#""budget":800"#)]),
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
fn the_start_of_a_procreation_of_this_format_that_names_the_parent_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":1,"parent":0}"#
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
fn a_run_record_of_this_format_that_names_the_time_limit_of_an_assessment_is_not_a_record() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""attempt_timeout":"#, r#""assessment_timeout":"#)]),
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
fn the_end_of_an_attempt_of_this_format_that_names_a_score_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#,
            r#"{"record":"attempt_finished","harness":0,"task":"t1","score":"completed"}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn the_end_of_an_attempt_of_this_format_whose_result_is_success_is_not_a_record() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#,
            r#"{"record":"attempt_finished","harness":0,"task":"t1","result":"success"}"#
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
        error_of(&[RUN, r#"{"record":"procreation_started","child":1}"#]),
        JournalError::NotARecord { line: 2 }
    ));
}

#[test]
fn a_journal_with_an_unknown_outcome_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":1,"progenitor":0}"#,
            r#"{"record":"procreation_finished","child":1,"outcome":"lost"}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_journal_with_an_unknown_result_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#,
            r#"{"record":"attempt_finished","harness":0,"task":"t1","result":"passed"}"#
        ]),
        JournalError::NotARecord { line: 3 }
    ));
}

#[test]
fn a_journal_that_calls_the_result_of_an_attempt_an_outcome_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#,
            r#"{"record":"attempt_finished","harness":0,"task":"t1","outcome":"completed"}"#
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
            r#"{"record":"procreation_started","child":1,"progenitor":5}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ProgenitorNotCreated { progenitor: 5 }
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
fn a_journal_that_starts_a_procreation_before_its_primogenitor_was_built_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            &run_that_builds(),
            r#"{"record":"procreation_started","child":1,"progenitor":0}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::ProgenitorNotCreated { progenitor: 0 }
        }
    ));
}

#[test]
fn a_journal_that_starts_an_attempt_before_its_primogenitor_was_built_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            &run_that_builds(),
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#
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
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#,
            r#"{"record":"attempt_started","harness":0,"task":"t1"}"#
        ]),
        JournalError::Refused {
            line: 3,
            error: GenealogyError::TaskAlreadyGiven { harness: 0, .. }
        }
    ));
}

#[test]
fn a_journal_with_the_end_of_an_attempt_that_never_started_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"attempt_finished","harness":0,"task":"t1","result":"completed"}"#
        ]),
        JournalError::Refused {
            line: 2,
            error: GenealogyError::AttemptNotRunning { harness: 0, .. }
        }
    ));
}

#[test]
fn a_journal_whose_procreation_names_a_child_other_than_the_next_id_cannot_be_read() {
    assert!(matches!(
        error_of(&[
            RUN,
            r#"{"record":"procreation_started","child":7,"progenitor":0}"#
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
            r#"{"record":"procreation_started","child":1,"progenitor":0}"#
        ]),
        JournalError::AfterDirectedEvolutionFinished { line: 3 }
    ));
}
