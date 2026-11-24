use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;
use test_inputs::Image;

const SOLVES_T1: &str = "RUN echo t1 > /solved-by-the-image\n";

struct Evolved {
    directory: TempDir,
    _image: Image,
}

fn settings(directory: &Path, image: &Image, steps: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&Disk, &test_inputs::benchmark(directory, &["t1"])).unwrap(),
        ],
        docker_image: image.name.clone(),
        dockerfile: Some(test_inputs::dockerfile(directory, steps)),
        docker_build_timeout: Some(60),
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t1".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved(steps: &str) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let settings = settings(directory.path(), &image, steps);
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    let _ = crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink());
    Evolved {
        directory,
        _image: image,
    }
}

fn stopped_and_resumed_with_its_dockerfile_gone() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let settings = Settings {
        harness_run_command: format!(
            "[ -e end-with-125 ] && exit 125\n{}",
            test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES
        ),
        ..settings(directory.path(), &image, SOLVES_T1)
    };
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_on(directory.path(), "t1");
    let _ = crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t1");
    fs::remove_file(directory.path().join("Dockerfile")).unwrap();
    let _ = crate::real::resume(&outputs_directory, &stop, &mut io::sink());
    Evolved {
        directory,
        _image: image,
    }
}

fn solved_by_harness_1(evolved: &Evolved) -> String {
    fs::read_to_string(evolved.directory.path().join("outputs/harnesses/1/solved")).unwrap()
}

#[test]
fn a_directed_evolution_given_a_dockerfile_runs_its_harnesses_in_the_image_built_from_it() {
    assert_eq!(solved_by_harness_1(&evolved(SOLVES_T1)), "t1\n");
}

#[test]
fn a_directed_evolution_that_is_resumed_runs_its_harnesses_in_the_image_that_was_built() {
    assert_eq!(
        solved_by_harness_1(&stopped_and_resumed_with_its_dockerfile_gone()),
        "t1\n"
    );
}
