use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::cli::{self, Cli};
use cladenc::directed_evolution::{self, Found};
use cladenc::genealogy::CreationOutcome;
use cladenc::journal::{self, Record, Settings};
use clap::Parser;
use serde_json::Value;
use tempfile::TempDir;

const CHANGE_NOTHING: &str = "exit 0";

const A_PRIMOGENITOR_THAT_CHANGES_NOTHING_FOR_A_BASELINE: &str = r#"if [ -d /var/log/cladenc/attempts ]
then
  [ -e improved ] && [ ! -e "$CLADENC_HARNESS/improved" ] && exit 0
  echo improved >> improved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

struct Evolved {
    directory: TempDir,
    found: Found,
    said: Vec<String>,
}

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(directory, &["t1", "t2", "t3"])).unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        max_procreations: 2,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        attempt_timeout: 30,
        selection_tasks_percentage: 33.5,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 33.5,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 1,
    }
}

fn evolved_by(run_command: &str) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), run_command);
    let outputs_directory = directory.path().join("outputs");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let found =
        directed_evolution::start(&settings, &outputs_directory, &stop, &mut written).unwrap();
    let said = String::from_utf8(written).unwrap();
    Evolved {
        directory,
        found,
        said: said.lines().map(str::to_owned).collect(),
    }
}

fn outputs_of(evolved: &Evolved) -> String {
    evolved
        .directory
        .path()
        .join("outputs")
        .display()
        .to_string()
}

fn records_of(evolved: &Evolved) -> Vec<Record> {
    fs::read_to_string(journal::path(&evolved.directory.path().join("outputs")))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn starts(evolved: &Evolved) -> Vec<String> {
    records_of(evolved)
        .into_iter()
        .filter_map(|record| match record {
            Record::AttemptStarted { harness, .. } => Some(format!("attempt of {harness}")),
            Record::ProcreationStarted { child, progenitor } => {
                Some(format!("procreation of {child} by {progenitor}"))
            }
            Record::BaselineStarted {
                harness,
                child,
                progenitor,
            } => Some(format!(
                "baseline of {harness} for {child} from {progenitor}"
            )),
            _ => None,
        })
        .collect()
}

fn reported(evolved: &Evolved) -> Value {
    let outputs_dir = format!("--outputs-dir={}", outputs_of(evolved));
    let cli = Cli::try_parse_from(["cladenc", "report", "--json", outputs_dir.as_str()]).unwrap();
    let mut output = Vec::new();
    cli::execute(&cli, &AtomicBool::new(false), &mut output, &mut io::sink());
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn a_child_that_would_be_the_progenitor_s_copy_as_it_was_attempts_no_task() {
    let evolved = evolved_by(CHANGE_NOTHING);
    assert_eq!(
        starts(&evolved),
        [
            "attempt of 0",
            "attempt of 0",
            "attempt of 0",
            "procreation of 1 by 0",
            "procreation of 2 by 0",
        ]
    );
}

#[test]
fn a_procreation_that_left_the_progenitor_s_copy_as_it_was_is_written_in_the_journal_as_unchanged()
{
    let evolved = evolved_by(CHANGE_NOTHING);
    assert_eq!(
        records_of(&evolved)
            .iter()
            .filter(|record| {
                matches!(
                    record,
                    Record::ProcreationFinished {
                        outcome: CreationOutcome::Unchanged,
                        ..
                    }
                )
            })
            .count(),
        2
    );
}

#[test]
fn a_directed_evolution_says_that_a_procreation_left_the_copy_unchanged_and_created_no_harness() {
    let evolved = evolved_by(CHANGE_NOTHING);
    assert!(evolved.said.contains(&format!(
        "cladenc: the procreation of harness 1 by harness 0 left the copy of the source code of \
         harness 0 unchanged, and created no harness: what the harness printed is kept in \
         {}/procreations/1",
        outputs_of(&evolved)
    )));
}

#[test]
fn the_best_harness_of_a_directed_evolution_whose_procreations_changed_nothing_is_the_primogenitor()
{
    assert_eq!(evolved_by(CHANGE_NOTHING).found.best_harness, 0);
}

#[test]
fn what_a_procreation_that_changed_nothing_left_is_kept_under_harnesses() {
    let evolved = evolved_by(CHANGE_NOTHING);
    assert!(
        evolved
            .directory
            .path()
            .join("outputs/harnesses/1/harness.sh")
            .is_file()
    );
}

#[test]
fn the_report_gives_the_state_of_a_child_that_would_be_the_copy_as_it_was_as_unchanged() {
    let evolved = evolved_by(CHANGE_NOTHING);
    assert_eq!(
        reported(&evolved)["harnesses"][1]["state"],
        serde_json::json!("unchanged")
    );
}

#[test]
fn a_singleton_control_group_harness_that_would_be_the_copy_as_it_was_attempts_no_task() {
    let evolved = evolved_by(A_PRIMOGENITOR_THAT_CHANGES_NOTHING_FOR_A_BASELINE);
    assert_eq!(
        starts(&evolved),
        [
            "attempt of 0",
            "attempt of 0",
            "attempt of 0",
            "procreation of 1 by 0",
            "attempt of 1",
            "attempt of 1",
            "attempt of 1",
            "procreation of 2 by 1",
            "baseline of 3 for 2 from 1",
            "attempt of 2",
            "attempt of 2",
            "attempt of 2",
        ]
    );
}

#[test]
fn a_directed_evolution_says_that_a_baseline_left_the_copy_unchanged_and_created_no_harness() {
    let evolved = evolved_by(A_PRIMOGENITOR_THAT_CHANGES_NOTHING_FOR_A_BASELINE);
    assert!(evolved.said.contains(&format!(
        "cladenc: the creation of harness 3, the singleton control group harness, by the \
         primogenitor from the source code of harness 1 left the copy of the source code of \
         harness 1 unchanged, and created no harness: what the primogenitor printed is kept in \
         {}/baselines/3",
        outputs_of(&evolved)
    )));
}

#[test]
fn the_report_counts_no_singleton_control_group_harness_created_where_the_baseline_changed_nothing()
{
    let evolved = evolved_by(A_PRIMOGENITOR_THAT_CHANGES_NOTHING_FOR_A_BASELINE);
    let experiments = &reported(&evolved)["experiments"];
    assert_eq!(
        [
            &experiments["conducted"],
            &experiments["progenitors_child_created"],
            &experiments["singleton_control_group_harness_created"],
            &experiments["both_created"],
        ],
        [
            &serde_json::json!(1),
            &serde_json::json!(1),
            &serde_json::json!(0),
            &serde_json::json!(0),
        ]
    );
}
