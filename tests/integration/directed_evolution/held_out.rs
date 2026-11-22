use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::directed_evolution;
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

const KEEP_THE_TASKS_FOUND: &str = "if [ -d /var/log/cladenc/attempts ]; then ls \
                                    /var/log/cladenc/attempts/benchmark > tasks-found; fi";

const KEEP_WHAT_IS_GIVEN_TO_READ: &str = "if [ -d /var/log/cladenc/attempts ]; then ls \
                                          /var/log/cladenc > given; fi";

const SOLVE_A_SELECTION_TASK: &str = "if [ -d /var/log/cladenc/attempts ]; then echo t2 > \
                                      solved; else touch \"$CLADENC_HARNESS/solved\"; cp \
                                      \"$CLADENC_HARNESS/solved\" solved; fi";

const SOLVE_A_CORROBORATION_TASK: &str = "if [ -d /var/log/cladenc/attempts ]; then echo t4 > \
                                          solved; else touch \"$CLADENC_HARNESS/solved\"; cp \
                                          \"$CLADENC_HARNESS/solved\" solved; fi";

fn settings(directory: &Path, run_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(
                directory,
                &["t1", "t2", "t3", "t4", "t5"],
            ))
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: run_command.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        selection_tasks_percentage: 20.0,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t4".to_owned()],
        corroboration_tasks_percentage: 20.0,
        corroboration_tasks: vec!["benchmark/t4".to_owned()],
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved() -> (TempDir, Vec<String>) {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), KEEP_THE_TASKS_FOUND);
    let outputs_directory = directory.path().join("outputs");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut written).unwrap();
    let said = String::from_utf8(written).unwrap();
    (directory, said.lines().map(str::to_owned).collect())
}

fn best_harness_of_a_directed_evolution_whose_child_is_made_by(run_command: &str) -> u64 {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), run_command);
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink())
        .unwrap()
        .best_harness
}

fn given_to_the_makers_of_two_children() -> [String; 2] {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path(), KEEP_WHAT_IS_GIVEN_TO_READ);
    settings.max_procreations = 2;
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink()).unwrap();
    ["1", "2"].map(|child| {
        fs::read_to_string(
            outputs_directory
                .join("harnesses")
                .join(child)
                .join("given"),
        )
        .unwrap()
    })
}

fn stopped_at_a_holdout_task_and_resumed() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let settings = settings(
        directory.path(),
        test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125,
    );
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_on(directory.path(), "t4");
    let _ = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t4");
    directed_evolution::resume(&outputs_directory, &stop, &mut io::sink()).unwrap();
    directory
}

fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn tasks_kept_under(directory: &TempDir, kept: &str) -> [Vec<String>; 2] {
    ["0", "1"].map(|harness| {
        names_in(
            &directory
                .path()
                .join("outputs")
                .join(kept)
                .join(harness)
                .join("benchmark"),
        )
    })
}

#[test]
fn a_progenitor_finds_its_attempts_at_the_tasks_that_are_not_held_out_and_no_other() {
    let (directory, _said) = evolved();
    assert_eq!(
        fs::read_to_string(directory.path().join("outputs/harnesses/1/tasks-found")).unwrap(),
        "t1\nt3\nt5\n"
    );
}

#[test]
fn every_harness_attempts_the_holdout_tasks_as_well_as_the_others() {
    let (directory, _said) = evolved();
    let journal = fs::read_to_string(journal::path(&directory.path().join("outputs"))).unwrap();
    assert_eq!(
        journal.matches(r#""record":"attempt_finished""#).count(),
        10
    );
}

#[test]
fn the_attempts_of_each_harness_at_the_holdout_tasks_are_kept_under_holdout() {
    let (directory, _said) = evolved();
    assert_eq!(
        tasks_kept_under(&directory, "holdout"),
        [vec!["t2", "t4"], vec!["t2", "t4"]]
    );
}

#[test]
fn the_attempts_of_each_harness_at_the_non_holdout_tasks_are_kept_under_attempts() {
    let (directory, _said) = evolved();
    assert_eq!(
        tasks_kept_under(&directory, "attempts"),
        [vec!["t1", "t3", "t5"], vec!["t1", "t3", "t5"]]
    );
}

#[test]
fn a_directed_evolution_says_how_many_tasks_of_a_benchmark_it_holds_out() {
    let (directory, said) = evolved();
    assert_eq!(
        said[2],
        format!(
            "cladenc: holding out 2 of the 5 tasks of the benchmark {} from every progenitor: 1 \
             selection task and 1 corroboration task",
            directory.path().join("benchmark").display()
        )
    );
}

#[test]
fn what_is_said_of_an_attempt_at_a_holdout_task_names_where_the_attempt_is_kept() {
    let (directory, said) = evolved();
    assert!(said.contains(&format!(
        "cladenc: harness 0 did not complete task benchmark/t2: what the attempt left is kept \
         in {}",
        directory
            .path()
            .join("outputs/holdout/0/benchmark/t2")
            .display()
    )));
}

#[test]
fn a_resumed_directed_evolution_keeps_the_attempt_it_carries_out_again_under_holdout() {
    let directory = stopped_at_a_holdout_task_and_resumed();
    assert_eq!(
        tasks_kept_under(&directory, "holdout"),
        [vec!["t2", "t4"], vec!["t2", "t4"]]
    );
}

#[test]
fn a_resumed_directed_evolution_keeps_no_holdout_task_where_a_progenitor_reads() {
    let directory = stopped_at_a_holdout_task_and_resumed();
    assert_eq!(
        tasks_kept_under(&directory, "attempts"),
        [vec!["t1", "t3", "t5"], vec!["t1", "t3", "t5"]]
    );
}

#[test]
fn a_child_that_completes_a_selection_task_its_progenitor_did_not_is_named_the_best_harness() {
    assert_eq!(
        best_harness_of_a_directed_evolution_whose_child_is_made_by(SOLVE_A_SELECTION_TASK),
        1
    );
}

#[test]
fn a_child_that_completes_only_a_corroboration_task_is_not_named_the_best_harness() {
    assert_eq!(
        best_harness_of_a_directed_evolution_whose_child_is_made_by(SOLVE_A_CORROBORATION_TASK),
        0
    );
}

#[test]
fn a_child_that_completes_only_one_more_task_that_is_not_held_out_is_not_named_the_best_harness() {
    assert_eq!(
        best_harness_of_a_directed_evolution_whose_child_is_made_by(test_inputs::SOLVE_ONE_MORE),
        0
    );
}

#[test]
fn the_primogenitor_that_procreates_for_the_first_time_is_given_its_attempts_and_nothing_else() {
    assert_eq!(given_to_the_makers_of_two_children()[0], "attempts\n");
}

#[test]
fn the_harness_that_procreates_next_is_given_the_child_it_made_or_the_line_it_descends_from() {
    assert!(
        ["attempts\nchildren\n", "attempts\nline\n"]
            .contains(&given_to_the_makers_of_two_children()[1].as_str())
    );
}
