use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use cladenc::artificial_selection::ArtificialSelection;
use cladenc::benchmark::Benchmark;
use cladenc::containers::Launch;
use cladenc::genealogy::Genealogy;
use cladenc::journal::{self, Journal, JournalError, Record, Settings};
use cladenc::model::Model;

fn settings() -> Settings {
    Settings {
        primogenitor: PathBuf::from("/abs/harness"),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark {
            directory: PathBuf::from("/abs/benchmark"),
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
        }],
        docker_image: "image".to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        assessment_timeout: 3600,
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
fn the_format_of_the_journal_is_14() {
    assert_eq!(journal::FORMAT, 14);
}

#[test]
fn the_run_record_is_written_key_for_key() {
    assert_eq!(
        run(settings()).line().unwrap(),
        concat!(
            r#"{"record":"run","format":14,"primogenitor":"/abs/harness","model":"a-model","#,
            r#""model_url":"http://models.example/v1","model_key_file":null,"#,
            r#""docker_image":"image","dockerfile":null,"docker_build_timeout":null,"#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""benchmarks":[{"directory":"/abs/benchmark","tasks":["t1","t2"]}],"#,
            r#""budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"#,
            r#""max_concurrent_containers":8,"#,
            r#""procreation_timeout":3600,"assessment_timeout":3600,"seed":7}"#
        )
    );
}

#[test]
fn the_run_record_holds_the_limit_of_concurrent_containers_between_two_other_settings() {
    assert!(run(settings()).line().unwrap().contains(
        r#""confidence":99.0,"max_concurrent_containers":8,"procreation_timeout":3600,"#
    ));
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
        }
    );
}

#[test]
fn settings_with_no_build_command_give_a_launch_that_builds_nothing() {
    assert_eq!(settings().launch(None).build_command, None);
}

#[test]
fn settings_with_a_build_command_give_a_genealogy_whose_primogenitor_awaits_its_build() {
    assert_eq!(built_with("make harness").genealogy(), Genealogy::unbuilt());
}

#[test]
fn settings_with_no_build_command_give_a_genealogy_whose_primogenitor_is_created() {
    assert_eq!(settings().genealogy(), Genealogy::new());
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
fn the_settings_give_the_artificial_selection_made_from_five_of_them() {
    assert_eq!(
        settings().artificial_selection(),
        ArtificialSelection {
            budget: 800,
            growth: 0.6,
            confidence: 99.0,
            tasks: vec!["benchmark/t1".to_owned(), "benchmark/t2".to_owned()],
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
    let run_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(run_directory.path()).unwrap();
    assert!(matches!(
        journal.append(&with_a_path_that_is_not_utf8()),
        Err(JournalError::Io(_))
    ));
}

#[test]
fn a_record_that_cannot_be_written_appends_nothing() {
    let run_directory = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(run_directory.path()).unwrap();
    let _ = journal.append(&with_a_path_that_is_not_utf8());
    assert_eq!(
        fs::read_to_string(journal::path(run_directory.path())).unwrap(),
        ""
    );
}
