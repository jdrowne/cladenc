use std::path::Path;

use crate::benchmark::Benchmark;
use crate::directed_evolution;
use crate::journal::Settings;

fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(
                directory,
                &["t1", "t2", "t3", "t4"],
            ))
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 50.0,
        holdout_tasks: vec![
            "benchmark/t1".to_owned(),
            "benchmark/t3".to_owned(),
            "benchmark/t4".to_owned(),
        ],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: vec!["benchmark/t4".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn said(settings: &Settings) -> String {
    let mut written: Vec<u8> = Vec::new();
    directed_evolution::say_the_holdout(settings, &mut written);
    String::from_utf8(written).unwrap()
}

#[test]
fn the_holdout_of_a_benchmark_is_said_with_its_selection_and_its_corroboration_tasks_counted() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    assert_eq!(
        said(&settings),
        format!(
            "cladenc: holding out 3 of the 4 tasks of the benchmark {} from every progenitor: 2 \
             selection tasks and 1 corroboration task\n",
            settings.benchmarks[0].directory.display()
        )
    );
}

#[test]
fn the_holdout_is_said_for_each_benchmark_in_turn_one_with_no_holdout_task_among_them() {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    let other = test_inputs::benchmark_named(directory.path(), "other", &["t1"]);
    settings.benchmarks.push(Benchmark::read(&other).unwrap());
    let written = said(&settings);
    let of_the_other = format!(
        "cladenc: holding out 0 of the 1 task of the benchmark {} from every progenitor: 0 \
         selection tasks and 0 corroboration tasks",
        settings.benchmarks[1].directory.display()
    );
    assert_eq!(written.lines().last(), Some(of_the_other.as_str()));
}
