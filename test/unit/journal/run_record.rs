use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use cladenc::genealogy::Genealogy;
use cladenc::job::Launch;
use cladenc::journal::{self, Journal, JournalError, Record, Settings};
use cladenc::policy::Policy;

fn settings() -> Settings {
    Settings {
        progenitor: PathBuf::from("/abs/harness"),
        tasks: vec!["t1".to_owned(), "t2".to_owned()],
        docker_image: "image".to_owned(),
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        evaluator: PathBuf::from("/abs/evaluate"),
        budget: 800,
        genealogy_growth: 0.6,
        confidence: 99.0,
        initial_procreations: 5,
        workers: 8,
        procreation_timeout: 3600,
        evaluation_timeout: 3600,
        seed: 7,
    }
}

fn built_with(command: &str) -> Settings {
    Settings {
        harness_build_command: Some(command.to_owned()),
        ..settings()
    }
}

fn run(settings: Settings) -> Record {
    Record::Run {
        format: journal::FORMAT,
        settings,
    }
}

fn read_back(record: &Record) -> Record {
    serde_json::from_str(&record.line().unwrap()).unwrap()
}

fn with_a_path_that_is_not_utf8() -> Record {
    run(Settings {
        progenitor: PathBuf::from(OsStr::from_bytes(&[0xff])),
        ..settings()
    })
}

#[test]
fn the_format_of_the_journal_is_3() {
    assert_eq!(journal::FORMAT, 3);
}

#[test]
fn the_run_record_is_written_key_for_key() {
    assert_eq!(
        run(settings()).line().unwrap(),
        concat!(
            r#"{"record":"run","format":3,"progenitor":"/abs/harness","docker_image":"image","#,
            r#""harness_build_command":null,"harness_run_command":"run a harness","#,
            r#""tasks":["t1","t2"],"#,
            r#""evaluator":"/abs/evaluate","budget":800,"#,
            r#""genealogy_growth":0.6,"confidence":99.0,"initial_procreations":5,"workers":8,"#,
            r#""procreation_timeout":3600,"evaluation_timeout":3600,"seed":7}"#
        )
    );
}

#[test]
fn the_run_record_of_a_search_that_builds_its_harnesses_holds_the_build_command() {
    assert!(
        run(built_with("make harness"))
            .line()
            .unwrap()
            .contains(r#""docker_image":"image","harness_build_command":"make harness","#)
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
        built_with("make harness").launch(),
        Launch {
            image: "image",
            run_command: "run a harness",
            build_command: Some("make harness"),
        }
    );
}

#[test]
fn settings_with_no_build_command_give_a_launch_that_builds_nothing() {
    assert_eq!(settings().launch().build_command, None);
}

#[test]
fn settings_with_a_build_command_give_a_genealogy_whose_progenitor_awaits_its_build() {
    assert_eq!(built_with("make harness").genealogy(), Genealogy::unbuilt());
}

#[test]
fn settings_with_no_build_command_give_a_genealogy_whose_progenitor_is_created() {
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
fn the_settings_give_the_policy_made_from_five_of_them() {
    assert_eq!(
        settings().policy(),
        Policy {
            budget: 800,
            growth: 0.6,
            confidence: 99.0,
            tasks: vec!["t1".to_owned(), "t2".to_owned()],
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
