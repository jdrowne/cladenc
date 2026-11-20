use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use crate::artificial_selection::ArtificialSelection;
use crate::benchmark::Benchmark;
use crate::containers::Launch;
use crate::genealogy::Genealogy;
use crate::holdout::TaskSet;
use crate::journal::{self, Journal, JournalError, Record, Settings};
use crate::model::Model;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/abs/harness"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned(), "t3".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 800,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        attempt_timeout: 3600,
        holdout_percentage: 50.0,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        holdout_corroboration_percentage: 50.0,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        seed: 7,
    }
}

fn built_with(command: &str) -> Settings {
    Settings {
        harness_build_command: Some(command.to_owned()),
        ..settings()
    }
}

fn given_a_dockerfile() -> Settings {
    Settings {
        docker_image: "cladenc-0123456789abcdef".to_owned(),
        dockerfile: Some(PathBuf::from("/abs/images/Dockerfile")),
        docker_build_timeout: Some(600),
        ..settings()
    }
}

fn keyed() -> Settings {
    Settings {
        model_key_file: Some(PathBuf::from("/abs/keys/model")),
        ..settings()
    }
}

fn run(settings: Settings) -> Record {
    Record::Run {
        format: journal::FORMAT,
        settings: Box::new(settings),
    }
}

fn read_back(record: &Record) -> Record {
    serde_json::from_str(&record.line().unwrap()).unwrap()
}

fn with_a_path_that_is_not_utf8() -> Record {
    run(Settings {
        primogenitor: PathBuf::from(OsStr::from_bytes(&[0xff])),
        ..settings()
    })
}

#[test]
fn the_format_of_the_journal_is_24() {
    assert_eq!(journal::FORMAT, 24);
}

#[test]
fn the_run_record_is_written_key_for_key() {
    assert_eq!(
        run(settings()).line().unwrap(),
        concat!(
            r#"{"record":"run","format":24,"primogenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2","t3"]}],"#,
            r#""holdout_percentage":50.0,"holdout_tasks":["benchmark/t2","benchmark/t3"],"#,
            r#""holdout_corroboration_percentage":50.0,"#,
            r#""corroboration_tasks":["benchmark/t3"],"max_procreations":800,"#,
            r#""max_concurrent_containers":8,"#,
            r#""procreation_timeout":3600,"attempt_timeout":3600,"seed":7}"#
        )
    );
}

#[test]
fn the_run_record_holds_the_limit_of_concurrent_containers_between_two_other_settings() {
    assert!(run(settings()).line().unwrap().contains(
        r#""max_procreations":800,"max_concurrent_containers":8,"procreation_timeout":3600,"#
    ));
}

#[test]
fn the_run_record_holds_the_holdout_tasks_after_the_percentage_they_were_drawn_by() {
    assert!(run(settings()).line().unwrap().contains(concat!(
        r#""tasks":["t1","t2","t3"]}],"holdout_percentage":50.0,"#,
        r#""holdout_tasks":["benchmark/t2","benchmark/t3"],"#,
        r#""holdout_corroboration_percentage":50.0,"#
    )));
}

#[test]
fn the_run_record_holds_the_corroboration_tasks_after_the_percentage_they_were_drawn_by() {
    assert!(run(settings()).line().unwrap().contains(concat!(
        r#""holdout_corroboration_percentage":50.0,"#,
        r#""corroboration_tasks":["benchmark/t3"],"max_procreations":800,"#
    )));
}

#[test]
fn a_holdout_corroboration_percentage_with_a_decimal_point_survives_being_written_and_read_back() {
    let corroborated = run(Settings {
        holdout_corroboration_percentage: 12.5,
        ..settings()
    });
    assert_eq!(read_back(&corroborated), corroborated);
}

#[test]
fn a_holdout_percentage_with_a_decimal_point_survives_being_written_and_read_back() {
    let held_out = run(Settings {
        holdout_percentage: 12.5,
        ..settings()
    });
    assert_eq!(read_back(&held_out), held_out);
}

#[test]
fn the_settings_give_how_many_tasks_of_a_benchmark_are_in_each_set() {
    let settings = settings();
    assert_eq!(
        [
            TaskSet::Selection,
            TaskSet::Corroboration,
            TaskSet::NonHoldout
        ]
        .map(|set| settings.count_of(&settings.benchmarks[0], set)),
        [1, 1, 1]
    );
}

#[test]
fn the_settings_count_no_holdout_task_of_another_benchmark_as_one_of_a_benchmark() {
    let settings = settings();
    let another = Benchmark {
        directory: PathBuf::from("/abs/refactoring"),
        tasks: vec!["t1".to_owned(), "t2".to_owned(), "t3".to_owned()],
    };
    assert_eq!(
        [
            TaskSet::Selection,
            TaskSet::Corroboration,
            TaskSet::NonHoldout
        ]
        .map(|set| settings.count_of(&another, set)),
        [0, 0, 3]
    );
}

#[test]
fn the_selection_tasks_are_the_holdout_tasks_that_are_not_corroboration_tasks() {
    assert_eq!(settings().tasks_of(TaskSet::Selection), ["benchmark/t2"]);
}

#[test]
fn the_corroboration_tasks_are_the_ones_the_run_record_names() {
    assert_eq!(
        settings().tasks_of(TaskSet::Corroboration),
        ["benchmark/t3"]
    );
}

#[test]
fn the_non_holdout_tasks_are_the_tasks_that_are_not_held_out() {
    assert_eq!(settings().tasks_of(TaskSet::NonHoldout), ["benchmark/t1"]);
}

#[test]
fn a_task_named_a_corroboration_task_that_is_not_held_out_is_a_non_holdout_task() {
    let settings = Settings {
        corroboration_tasks: vec!["benchmark/t1".to_owned()],
        ..settings()
    };
    assert_eq!(settings.tasks_of(TaskSet::NonHoldout), ["benchmark/t1"]);
}

#[test]
fn the_run_record_holds_no_confidence() {
    assert!(!run(settings()).line().unwrap().contains("confidence"));
}

#[test]
fn the_run_record_holds_no_initial_procreations() {
    assert!(
        !run(settings())
            .line()
            .unwrap()
            .contains("initial_procreations")
    );
}

#[test]
fn the_run_record_of_a_directed_evolution_that_builds_its_harnesses_holds_the_build_command() {
    assert!(
        run(built_with("make harness"))
            .line()
            .unwrap()
            .contains(r#""docker_build_timeout":null,"harness_build_command":"make harness","#)
    );
}

#[test]
fn the_run_record_holds_a_dockerfile_that_was_given_with_its_image_and_its_time_limit() {
    assert!(run(given_a_dockerfile()).line().unwrap().contains(concat!(
        r#""docker_image":"cladenc-0123456789abcdef","#,
        r#""dockerfile":"/abs/images/Dockerfile","docker_build_timeout":600,"#
    )));
}

#[test]
fn a_dockerfile_survives_being_written_and_read_back() {
    assert_eq!(
        read_back(&run(given_a_dockerfile())),
        run(given_a_dockerfile())
    );
}

#[test]
fn the_image_a_harness_is_started_from_is_the_one_built_from_the_dockerfile() {
    assert_eq!(
        given_a_dockerfile().launch(None).image,
        "cladenc-0123456789abcdef"
    );
}

#[test]
fn the_run_record_of_a_directed_evolution_with_a_key_holds_the_file_of_the_key() {
    assert!(
        run(keyed())
            .line()
            .unwrap()
            .contains(r#""model_key_file":"/abs/keys/model","docker_image":"image","#)
    );
}

#[test]
fn the_file_of_a_key_survives_being_written_and_read_back() {
    assert_eq!(read_back(&run(keyed())), run(keyed()));
}

#[test]
fn the_settings_and_a_key_give_the_model_a_harness_is_told() {
    assert_eq!(
        settings().model(Some("a-made-up-key")),
        Model {
            name: "a-model",
            url: "http://models.example/v1",
            key: Some("a-made-up-key"),
        }
    );
}

#[test]
fn settings_that_name_no_file_of_a_key_have_no_key() {
    assert_eq!(settings().model_key(), Ok(None));
}

#[test]
fn settings_that_name_a_file_of_a_key_have_the_key_the_file_holds() {
    let directory = test_inputs::temporary_directory();
    let settings = Settings {
        model_key_file: Some(test_inputs::model_key_file(
            directory.path(),
            "a-made-up-key\n",
        )),
        ..settings()
    };
    assert_eq!(settings.model_key(), Ok(Some("a-made-up-key".to_owned())));
}

#[test]
fn settings_whose_file_of_a_key_holds_no_key_give_the_file() {
    assert_eq!(keyed().model_key(), Err(PathBuf::from("/abs/keys/model")));
}

#[test]
fn a_build_command_survives_being_written_and_read_back() {
    assert_eq!(
        read_back(&run(built_with("make harness"))),
        run(built_with("make harness"))
    );
}

#[test]
fn the_settings_give_how_a_harness_is_built_and_run() {
    assert_eq!(
        built_with("make harness").launch(None),
        Launch {
            image: "image",
            run_command: "run a harness",
            build_command: Some("make harness"),
            model: Model {
                name: "a-model",
                url: "http://models.example/v1",
                key: None,
            },
            procreation_timeout: 3600,
            attempt_timeout: 3600,
        }
    );
}

#[test]
fn the_settings_give_how_long_a_procreation_may_take() {
    let settings = Settings {
        procreation_timeout: 1800,
        attempt_timeout: 900,
        ..settings()
    };
    assert_eq!(settings.launch(None).procreation_timeout, 1800);
}

#[test]
fn the_settings_give_how_long_an_attempt_may_take() {
    let settings = Settings {
        procreation_timeout: 1800,
        attempt_timeout: 900,
        ..settings()
    };
    assert_eq!(settings.launch(None).attempt_timeout, 900);
}

#[test]
fn settings_with_no_build_command_give_a_launch_that_builds_nothing() {
    assert_eq!(settings().launch(None).build_command, None);
}

#[test]
fn settings_with_a_build_command_give_a_genealogy_whose_primogenitor_awaits_its_build() {
    assert_eq!(
        built_with("make harness").genealogy(),
        Genealogy::unbuilt()
            .selecting_by(&["benchmark/t2".to_owned()])
            .corroborating_with(&["benchmark/t3".to_owned()])
    );
}

#[test]
fn settings_with_no_build_command_give_a_genealogy_whose_primogenitor_is_created() {
    assert_eq!(
        settings().genealogy(),
        Genealogy::new()
            .selecting_by(&["benchmark/t2".to_owned()])
            .corroborating_with(&["benchmark/t3".to_owned()])
    );
}

#[test]
fn the_settings_give_a_genealogy_that_is_told_of_the_holdout_tasks_of_the_directed_evolution() {
    assert_ne!(settings().genealogy(), Genealogy::new());
}

#[test]
fn the_settings_give_a_genealogy_that_selects_by_no_corroboration_task() {
    assert_ne!(
        settings().genealogy(),
        Genealogy::new().selecting_by(&["benchmark/t2".to_owned(), "benchmark/t3".to_owned()])
    );
}

#[test]
fn the_run_record_survives_being_written_and_read_back() {
    assert_eq!(read_back(&run(settings())), run(settings()));
}

#[test]
fn the_largest_seed_survives_being_written_and_read_back() {
    let largest = run(Settings {
        seed: u64::MAX,
        ..settings()
    });
    assert_eq!(read_back(&largest), largest);
}

#[test]
fn the_settings_give_the_artificial_selection_made_from_them() {
    assert_eq!(
        settings().artificial_selection(),
        ArtificialSelection {
            max_procreations: 800,
            tasks: vec![
                "benchmark/t1".to_owned(),
                "benchmark/t2".to_owned(),
                "benchmark/t3".to_owned()
            ],
            seed: 7,
        }
    );
}

#[test]
fn a_run_record_holding_a_path_that_is_not_utf8_has_no_line() {
    assert!(matches!(
        with_a_path_that_is_not_utf8().line(),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_record_that_cannot_be_written_gives_an_error() {
    let outputs_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(outputs_directory.path()).unwrap();
    assert!(matches!(
        journal.append(&with_a_path_that_is_not_utf8()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_record_that_cannot_be_written_appends_nothing() {
    let outputs_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(outputs_directory.path()).unwrap();
    let _ = journal.append(&with_a_path_that_is_not_utf8());
    assert_eq!(
        fs::read_to_string(journal::path(outputs_directory.path())).unwrap(),
        ""
    );
}
