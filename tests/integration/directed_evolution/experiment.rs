use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::cli::{self, Cli};
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Record, Settings, Started};
use clap::Parser;
use serde_json::Value;
use tempfile::TempDir;

const A_PROGENITOR_THAT_PROCREATES_BETTER_THAN_THE_PRIMOGENITOR: &str = r#"if [ -d /var/log/cladenc/attempts ]
then
  if [ -e improved ] && [ ! -e "$CLADENC_HARNESS/improved" ]
  then
    [ -e "$CLADENC_HARNESS/end-with-125-at" ] && exit 125
  else
    [ -e "$CLADENC_HARNESS/improved" ] && echo t3 >> solved
  fi
  echo improved >> improved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

const THE_BASELINE: &str = "the creation of harness 3, the singleton control group harness, by \
                            the primogenitor from the source code of harness 1";

fn settings(directory: &Path) -> Settings {
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
        harness_run_command: A_PROGENITOR_THAT_PROCREATES_BETTER_THAN_THE_PRIMOGENITOR.to_owned(),
        max_procreations: 2,
        max_concurrent_containers: 1,
        procreation_timeout: 120,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 33.5,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 33.5,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 1,
    }
}

fn evolved() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut written).unwrap();
    let said = String::from_utf8(written).unwrap();
    (directory, said.lines().map(str::to_owned).collect())
}

fn records_of(outputs_directory: &Path) -> Vec<Record> {
    fs::read_to_string(journal::path(outputs_directory))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn starts(directory: &TempDir) -> Vec<String> {
    records_of(&directory.path().join("outputs"))
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

fn reported(directory: &TempDir) -> Value {
    let outputs_dir = format!(
        "--outputs-dir={}",
        directory.path().join("outputs").display()
    );
    let cli = Cli::try_parse_from(["cladenc", "report", "--json", outputs_dir.as_str()]).unwrap();
    let mut output = Vec::new();
    cli::execute(&cli, &AtomicBool::new(false), &mut output, &mut io::sink());
    serde_json::from_slice(&output).unwrap()
}

struct DirectedEvolutions {
    directory: TempDir,
    stopped: Result<Found, DirectedEvolutionError>,
    said_when_resumed: Vec<String>,
}

fn stopped_at_its_baseline_and_resumed() -> DirectedEvolutions {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let stop = AtomicBool::new(false);
    fs::create_dir(directory.path().join("unbroken")).unwrap();
    fs::create_dir(directory.path().join("stopped")).unwrap();
    let unbroken = directory.path().join("unbroken/outputs");
    let outputs_directory = directory.path().join("stopped/outputs");
    directed_evolution::start(&settings, &unbroken, &stop, &mut io::sink()).unwrap();
    test_inputs::end_with_125_at(&settings.primogenitor, 0);
    let stopped = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_no_more(&outputs_directory);
    let mut written = Vec::new();
    directed_evolution::resume(&outputs_directory, &stop, &mut written).unwrap();
    let said = String::from_utf8(written).unwrap();
    DirectedEvolutions {
        directory,
        stopped,
        said_when_resumed: said.lines().map(str::to_owned).collect(),
    }
}

fn journal_of(directed_evolutions: &DirectedEvolutions, directed_evolution: &str) -> String {
    let outputs_directory = directed_evolutions
        .directory
        .path()
        .join(directed_evolution)
        .join("outputs");
    test_inputs::without_seconds(&fs::read_to_string(journal::path(&outputs_directory)).unwrap())
}

#[test]
fn a_baseline_is_established_at_the_procreation_of_a_harness_other_than_the_primogenitor_alone() {
    let (directory, _said) = evolved();
    assert_eq!(
        starts(&directory),
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
            "attempt of 3",
            "attempt of 3",
            "attempt of 3",
        ]
    );
}

#[test]
fn the_singleton_control_group_harness_is_created_from_the_source_code_of_the_progenitor() {
    let (directory, _said) = evolved();
    assert!(
        directory
            .path()
            .join("outputs/harnesses/3/improved")
            .exists()
    );
}

#[test]
fn the_primogenitor_and_not_the_progenitor_creates_the_singleton_control_group_harness() {
    let (directory, _said) = evolved();
    assert_eq!(
        ["2", "3"].map(|harness| {
            directory
                .path()
                .join("outputs/harnesses")
                .join(harness)
                .join("solved")
                .exists()
        }),
        [true, false]
    );
}

#[test]
fn what_the_primogenitor_printed_for_a_baseline_is_kept_under_baselines() {
    let (directory, _said) = evolved();
    assert_eq!(
        ["progenitor", "child", "outcome"].map(|file| {
            fs::read_to_string(directory.path().join("outputs/baselines/3").join(file)).unwrap()
        }),
        ["1\n", "2\n", "created\n"]
    );
}

#[test]
fn a_directed_evolution_says_that_it_starts_the_creation_of_the_singleton_control_group_harness() {
    let (_directory, said) = evolved();
    assert!(said.contains(&format!("cladenc: started {THE_BASELINE}")));
}

#[test]
fn a_directed_evolution_says_how_a_baseline_ended_and_where_its_harness_is_kept() {
    let (directory, said) = evolved();
    assert!(said.contains(&format!(
        "cladenc: {THE_BASELINE} created harness 3, kept in {}",
        directory.path().join("outputs/harnesses/3").display()
    )));
}

#[test]
fn a_directed_evolution_numbers_a_procreation_by_the_procreations_started_before_it() {
    let (_directory, said) = evolved();
    assert!(said.iter().any(|line| {
        line == "cladenc: started the procreation of harness 2 by harness 1, procreation 2 of 2"
    }));
}

#[test]
fn a_directed_evolution_says_last_how_many_harnesses_it_holds_with_the_singleton_control_group_harness()
 {
    let (directory, said) = evolved();
    assert_eq!(
        said.last().unwrap(),
        &format!(
            "cladenc: the directed evolution finished with 12 attempts started and 4 harnesses: \
             its best harness is harness 0, kept in {}",
            directory.path().join("outputs/harnesses/0").display()
        )
    );
}

#[test]
fn the_report_of_a_directed_evolution_counts_what_its_experiments_created_and_what_told_them_apart()
{
    let (directory, _said) = evolved();
    assert_eq!(
        reported(&directory)["experiments"],
        serde_json::json!({
            "conducted": 1,
            "progenitors_child_created": 1,
            "singleton_control_group_harness_created": 1,
            "both_created": 1,
            "only_progenitors_child_completed": 1,
            "only_singleton_control_group_harness_completed": 0,
            "likelihood_by_luck": 0.5
        })
    );
}

#[test]
fn the_singleton_control_group_harness_is_counted_in_no_clade() {
    let (directory, _said) = evolved();
    let report = reported(&directory);
    assert_eq!(
        [
            &report["harnesses"][1]["clade_not_completed"],
            &report["harnesses"][3]["clade_not_completed"]
        ],
        [&serde_json::json!(2), &serde_json::json!(0)]
    );
}

#[test]
fn the_container_of_a_baseline_does_not_count_toward_the_most_procreations() {
    let (directory, _said) = evolved();
    assert_eq!(reported(&directory)["procreations_started"], 2);
}

#[test]
fn a_container_that_ends_with_status_125_at_a_baseline_stops_the_directed_evolution() {
    assert!(matches!(
        stopped_at_its_baseline_and_resumed().stopped,
        Err(DirectedEvolutionError::Action {
            action: Started::Baseline {
                harness: 3,
                child: 2,
                progenitor: 1
            },
            error: ContainerError::Docker(_),
        })
    ));
}

#[test]
fn a_directed_evolution_a_baseline_stopped_continues_to_the_journal_an_unbroken_one_writes() {
    let directed_evolutions = stopped_at_its_baseline_and_resumed();
    assert_eq!(
        journal_of(&directed_evolutions, "stopped"),
        journal_of(&directed_evolutions, "unbroken")
    );
}

#[test]
fn a_resumed_directed_evolution_says_that_it_starts_again_the_baseline_that_was_under_way() {
    let directed_evolutions = stopped_at_its_baseline_and_resumed();
    assert_eq!(
        directed_evolutions.said_when_resumed[5],
        format!("cladenc: started again {THE_BASELINE}")
    );
}

#[test]
fn a_resumed_directed_evolution_counts_the_singleton_control_group_harness_among_its_harnesses() {
    let directed_evolutions = stopped_at_its_baseline_and_resumed();
    assert_eq!(
        directed_evolutions.said_when_resumed[0],
        format!(
            "cladenc: continuing the directed evolution in {}, which has started 2 of 2 \
             procreations and holds 4 harnesses",
            directed_evolutions
                .directory
                .path()
                .join("stopped/outputs")
                .display()
        )
    );
}
