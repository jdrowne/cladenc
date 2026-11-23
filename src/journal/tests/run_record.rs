use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use crate::artificial_selection::ArtificialSelection;
use crate::benchmark::Benchmark;
use crate::containers::Launch;
use crate::file_system::tests::double::Memory;
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
        max_grader_health_check_retries: 3,
        max_task_health_check_retries: 2,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
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
fn the_format_of_the_journal_is_29() {
    assert_eq!(journal::FORMAT, 29);
}

#[test]
fn the_run_record_is_written_key_for_key() {
    assert_eq!(
        run(settings()).line().unwrap(),
        concat!(
            r#"{"record":"run","format":29,"primogenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""environment_image_repository":"cladenc-environments-0123456789abcdef","#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2","t3"]}],"#,
            r#""selection_tasks_percentage":25.0,"holdout_tasks":["benchmark/t2","benchmark/t3"],"#,
            r#""corroboration_tasks_percentage":25.0,"#,
            r#""corroboration_tasks":["benchmark/t3"],"max_procreations":800,"#,
            r#""max_concurrent_containers":8,"#,
            r#""procreation_timeout":3600,"max_grader_health_check_retries":3,"#,
            r#""max_task_health_check_retries":2,"seed":7}"#
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
        r#""tasks":["t1","t2","t3"]}],"selection_tasks_percentage":25.0,"#,
        r#""holdout_tasks":["benchmark/t2","benchmark/t3"],"#,
        r#""corroboration_tasks_percentage":25.0,"#
    )));
}

#[test]
fn the_run_record_holds_the_corroboration_tasks_after_the_percentage_they_were_drawn_by() {
    assert!(run(settings()).line().unwrap().contains(concat!(
        r#""corroboration_tasks_percentage":25.0,"#,
        r#""corroboration_tasks":["benchmark/t3"],"max_procreations":800,"#
    )));
}

#[test]
fn a_corroboration_tasks_percentage_with_a_decimal_point_survives_being_written_and_read_back() {
    let corroborated = run(Settings {
        corroboration_tasks_percentage: 12.5,
        ..settings()
    });
    assert_eq!(read_back(&corroborated), corroborated);
}

#[test]
fn a_selection_tasks_percentage_with_a_decimal_point_survives_being_written_and_read_back() {
    let held_out = run(Settings {
        selection_tasks_percentage: 12.5,
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
            .contains(r#""harness_build_command":"make harness","harness_run_command":"#)
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
    assert_eq!(settings().model_key(&Memory::default()), Ok(None));
}

#[test]
fn settings_that_name_a_file_of_a_key_have_the_key_the_file_holds() {
    let directory = Memory::default();
    let settings = Settings {
        model_key_file: Some(test_inputs::model_key_file_in(
            &directory,
            directory.path(),
            "a-made-up-key\n",
        )),
        ..settings()
    };
    assert_eq!(
        settings.model_key(&directory),
        Ok(Some("a-made-up-key".to_owned()))
    );
}

#[test]
fn settings_whose_file_of_a_key_holds_no_key_give_the_file() {
    assert_eq!(
        keyed().model_key(&Memory::default()),
        Err(PathBuf::from("/abs/keys/model"))
    );
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
            max_grader_health_check_retries: 3,
            max_task_health_check_retries: 2,
        }
    );
}

#[test]
fn the_settings_give_how_long_a_procreation_may_take() {
    let settings = Settings {
        procreation_timeout: 1800,
        ..settings()
    };
    assert_eq!(settings.launch(None).procreation_timeout, 1800);
}

#[test]
fn the_settings_give_the_most_retries_of_the_health_check_of_a_grader() {
    let settings = Settings {
        max_grader_health_check_retries: 5,
        ..settings()
    };
    assert_eq!(settings.launch(None).max_grader_health_check_retries, 5);
}

#[test]
fn the_most_retries_of_the_health_check_of_a_grader_being_0_survives_being_written_and_read_back() {
    let never_retried = run(Settings {
        max_grader_health_check_retries: 0,
        ..settings()
    });
    assert_eq!(read_back(&never_retried), never_retried);
}

#[test]
fn the_settings_give_the_most_retries_of_the_health_check_of_a_task() {
    let settings = Settings {
        max_task_health_check_retries: 4,
        ..settings()
    };
    assert_eq!(settings.launch(None).max_task_health_check_retries, 4);
}

#[test]
fn the_most_retries_of_the_health_check_of_a_task_being_0_survives_being_written_and_read_back() {
    let never_retried = run(Settings {
        max_task_health_check_retries: 0,
        ..settings()
    });
    assert_eq!(read_back(&never_retried), never_retried);
}

#[test]
fn the_run_record_holds_the_most_retries_of_a_task_after_those_of_a_grader_and_before_the_seed() {
    assert!(run(settings()).line().unwrap().contains(
        r#""max_grader_health_check_retries":3,"max_task_health_check_retries":2,"seed":7}"#
    ));
}

#[test]
fn a_run_record_that_names_no_most_retries_of_the_health_check_of_a_task_is_not_a_record() {
    let line = run(settings())
        .line()
        .unwrap()
        .replace(r#""max_task_health_check_retries":2,"#, "");
    assert!(serde_json::from_str::<Record>(&line).is_err());
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
fn the_settings_give_each_task_the_image_of_its_environment_tagged_with_its_place_among_the_tasks()
{
    assert_eq!(
        settings().environments(),
        [
            (
                "benchmark/t1".to_owned(),
                "cladenc-environments-0123456789abcdef:0".to_owned()
            ),
            (
                "benchmark/t2".to_owned(),
                "cladenc-environments-0123456789abcdef:1".to_owned()
            ),
            (
                "benchmark/t3".to_owned(),
                "cladenc-environments-0123456789abcdef:2".to_owned()
            )
        ]
    );
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
    let directory = Memory::default();
    let mut journal = Journal::open(&directory, directory.path()).unwrap();
    assert!(matches!(
        journal.append(&with_a_path_that_is_not_utf8()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_record_that_cannot_be_written_appends_nothing() {
    let directory = Memory::default();
    let mut journal = Journal::open(&directory, directory.path()).unwrap();
    let _ = journal.append(&with_a_path_that_is_not_utf8());
    assert_eq!(
        directory
            .read_to_string(journal::path(directory.path()))
            .unwrap(),
        ""
    );
}
