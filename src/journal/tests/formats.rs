use super::given::{RUN, error_of};
use crate::journal::JournalError;

const RUN_OF_FORMAT_20: &str = concat!(
    r#"{"record":"run","format":20,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""holdout_percentage":50.0,"holdout_tasks":["benchmark/t2"],"#,
    r#""max_procreations":800,"#,
    r#""genealogy_growth":0.6,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"seed":7}"#
);

const RUN_OF_FORMAT_18: &str = concat!(
    r#"{"record":"run","format":18,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""max_procreations":800,"#,
    r#""genealogy_growth":0.6,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"seed":7}"#
);

const RUN_OF_FORMAT_17: &str = concat!(
    r#"{"record":"run","format":17,"primogenitor":"/abs/harness","model":"a-model","#,
    r#""model_url":"http://models.example/v1","model_key_file":null,"docker_image":"image","#,
    r#""dockerfile":null,"docker_build_timeout":null,"#,
    r#""harness_build_command":null,"harness_run_command":"run a harness","#,
    r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
    r#""max_procreations":800,"#,
    r#""genealogy_growth":0.6,"confidence":99.0,"#,
    r#""max_concurrent_containers":8,"#,
    r#""procreation_timeout":3600,"seed":7}"#
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
    r#""procreation_timeout":3600,"seed":7}"#
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

#[test]
fn a_journal_whose_format_is_not_31_cannot_be_read() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":31"#, r#""format":32"#)]),
        JournalError::UnknownFormat { format: 32 }
    ));
}

#[test]
fn a_journal_of_format_30_is_refused_for_its_format_since_it_calls_a_trial_an_attempt() {
    assert!(matches!(
        error_of(&[
            &RUN.replace(r#""format":31"#, r#""format":30"#),
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
        ]),
        JournalError::UnknownFormat { format: 30 }
    ));
}

#[test]
fn a_journal_of_format_29_is_refused_for_its_format_since_it_calls_a_verifier_a_grader() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":31"#, r#""format":29"#).replace(
            r#""max_verifier_health_check_retries":3,"#,
            r#""max_grader_health_check_retries":3,"#
        )]),
        JournalError::UnknownFormat { format: 29 }
    ));
}

#[test]
fn a_journal_of_format_28_is_refused_for_its_format_since_it_names_no_most_retries_of_a_task() {
    assert!(matches!(
        error_of(&[&RUN
            .replace(r#""format":31"#, r#""format":28"#)
            .replace(r#""max_task_health_check_retries":2,"#, "")]),
        JournalError::UnknownFormat { format: 28 }
    ));
}

#[test]
fn a_journal_of_format_27_is_refused_for_its_format_since_it_names_no_most_retries_of_a_verifier() {
    assert!(matches!(
        error_of(&[&RUN
            .replace(r#""format":31"#, r#""format":27"#)
            .replace(r#""max_verifier_health_check_retries":3,"#, "")
            .replace(r#""max_task_health_check_retries":2,"#, "")]),
        JournalError::UnknownFormat { format: 27 }
    ));
}

#[test]
fn a_journal_of_format_26_is_refused_for_its_format_since_its_run_record_names_an_attempt_timeout()
{
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":31"#, r#""format":26"#).replace(
            r#""procreation_timeout":3600,"#,
            r#""procreation_timeout":3600,"attempt_timeout":3600,"#
        )]),
        JournalError::UnknownFormat { format: 26 }
    ));
}

#[test]
fn a_journal_of_format_25_is_refused_for_its_format_since_its_run_record_names_a_holdout_percentage()
 {
    assert!(matches!(
        error_of(&[&RUN
            .replace(r#""format":31"#, r#""format":25"#)
            .replace(
                r#""selection_tasks_percentage":25.0"#,
                r#""holdout_percentage":50.0"#
            )
            .replace(
                r#""corroboration_tasks_percentage":25.0"#,
                r#""holdout_corroboration_percentage":50.0"#
            )]),
        JournalError::UnknownFormat { format: 25 }
    ));
}

#[test]
fn a_journal_of_format_24_is_refused_for_its_format_since_it_names_no_repository_of_environments() {
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":31"#, r#""format":24"#).replace(
            r#""environment_image_repository":"cladenc-environments-0123456789abcdef","#,
            ""
        )]),
        JournalError::UnknownFormat { format: 24 }
    ));
}

#[test]
fn a_journal_of_format_23_is_refused_for_its_format_since_its_run_record_holds_a_genealogy_growth()
{
    assert!(matches!(
        error_of(&[&RUN.replace(r#""format":31"#, r#""format":23"#).replace(
            r#""max_procreations":800,"#,
            r#""max_procreations":800,"genealogy_growth":0.6,"#
        )]),
        JournalError::UnknownFormat { format: 23 }
    ));
}

#[test]
fn a_journal_of_format_22_is_refused_for_its_format_since_it_counts_an_unchanged_child_as_created()
{
    assert!(matches!(
        error_of(&[
            &RUN.replace(r#""format":31"#, r#""format":22"#),
            r#"{"record":"procreation_started","child":1,"progenitor":0}"#,
            r#"{"record":"procreation_finished","child":1,"outcome":"created","seconds":1.5}"#
        ]),
        JournalError::UnknownFormat { format: 22 }
    ));
}

#[test]
fn a_journal_of_format_21_is_refused_for_its_format_since_it_records_no_baseline() {
    assert!(matches!(
        error_of(&[
            &RUN.replace(r#""format":31"#, r#""format":21"#),
            r#"{"record":"procreation_started","child":1,"progenitor":0}"#
        ]),
        JournalError::UnknownFormat { format: 21 }
    ));
}

#[test]
fn a_journal_of_format_20_is_refused_for_its_format_since_it_names_no_corroboration_tasks() {
    assert!(matches!(
        error_of(&[
            RUN_OF_FORMAT_20,
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
        ]),
        JournalError::UnknownFormat { format: 20 }
    ));
}

#[test]
fn a_journal_of_format_19_is_refused_for_its_format_since_its_records_hold_no_seconds() {
    assert!(matches!(
        error_of(&[
            &RUN.replace(r#""format":31"#, r#""format":19"#),
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#,
            r#"{"record":"attempt_finished","harness":0,"task":"benchmark/t1","result":"completed"}"#
        ]),
        JournalError::UnknownFormat { format: 19 }
    ));
}

#[test]
fn a_journal_of_format_18_is_refused_for_its_format_since_it_names_no_holdout_tasks() {
    assert!(matches!(
        error_of(&[
            RUN_OF_FORMAT_18,
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
        ]),
        JournalError::UnknownFormat { format: 18 }
    ));
}

#[test]
fn a_journal_of_format_17_is_refused_for_its_format_since_it_names_a_confidence() {
    assert!(matches!(
        error_of(&[
            RUN_OF_FORMAT_17,
            r#"{"record":"attempt_started","harness":0,"task":"benchmark/t1"}"#
        ]),
        JournalError::UnknownFormat { format: 17 }
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
        error_of(&[&RUN.replace(r#""format":31"#, r#""format":2"#)]),
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
