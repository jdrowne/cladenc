use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;
use test_inputs::Image;

const SOLVES_T1: &str = "RUN echo t1 > /solved-by-the-image\n";

struct Evolved {
    directory: TempDir,
    image: Image,
    found: Result<Found, DirectedEvolutionError>,
    said: String,
}

fn settings(directory: &Path, image: &Image, steps: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
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
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let found = directed_evolution::start(&settings, &outputs_directory, &stop, &mut written);
    Evolved {
        directory,
        image,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn stopped_and_resumed_with_its_dockerfile_gone() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let settings = Settings {
        harness_run_command: format!(
            "[ -e end-with-125-at ] && exit 125\n{}",
            test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES
        ),
        ..settings(directory.path(), &image, SOLVES_T1)
    };
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_at(&settings.primogenitor, 0);
    let _ = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_no_more(&outputs_directory);
    fs::remove_file(directory.path().join("Dockerfile")).unwrap();
    let mut written = Vec::new();
    let found = directed_evolution::resume(&outputs_directory, &stop, &mut written);
    Evolved {
        directory,
        image,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn run_of(evolved: &Evolved) -> String {
    evolved
        .directory
        .path()
        .join("outputs")
        .display()
        .to_string()
}

fn solved_by_harness_1(evolved: &Evolved) -> String {
    fs::read_to_string(evolved.directory.path().join("outputs/harnesses/1/solved")).unwrap()
}

#[test]
fn a_directed_evolution_given_a_dockerfile_runs_its_harnesses_in_the_image_built_from_it() {
    assert_eq!(solved_by_harness_1(&evolved(SOLVES_T1)), "t1\n");
}

#[test]
fn a_directed_evolution_given_a_dockerfile_finds_the_harness_made_in_its_image_to_be_the_best() {
    assert_eq!(evolved(SOLVES_T1).found.unwrap().best_harness, 1);
}

#[test]
fn a_directed_evolution_says_that_it_builds_its_image_and_that_the_image_is_built() {
    let evolved = evolved(SOLVES_T1);
    let said = format!(
        "cladenc: building the image {0} from the Dockerfile {1}\n\
         cladenc: the image {0} is built: what Docker printed is kept in {2}/image\n",
        evolved.image.name,
        evolved.directory.path().join("Dockerfile").display(),
        run_of(&evolved)
    );
    assert!(evolved.said.contains(&said));
}

#[test]
fn the_image_is_built_after_the_tasks_are_copied_and_before_the_first_action() {
    let evolved = evolved(SOLVES_T1);
    let places = [
        "cladenc: copying the 1 task ",
        "cladenc: building the image ",
        "cladenc: started the ",
    ]
    .map(|start| {
        evolved
            .said
            .lines()
            .position(|line| line.starts_with(start))
    });
    assert_eq!(places, [Some(1), Some(3), Some(7)]);
}

#[test]
fn the_run_record_names_the_image_that_was_built_its_dockerfile_and_the_time_limit() {
    let evolved = evolved(SOLVES_T1);
    let journal = fs::read_to_string(journal::path(Path::new(&run_of(&evolved)))).unwrap();
    let named = format!(
        r#""docker_image":"{}","dockerfile":"{}","docker_build_timeout":60,"#,
        evolved.image.name,
        evolved.directory.path().join("Dockerfile").display()
    );
    assert!(journal.lines().next().unwrap().contains(&named));
}

#[test]
fn a_dockerfile_that_is_not_built_stops_the_directed_evolution_before_its_first_action() {
    assert!(matches!(
        evolved("RUN exit 3\n").found,
        Err(DirectedEvolutionError::Image(ContainerError::Exit(_)))
    ));
}

#[test]
fn a_dockerfile_that_is_not_built_leaves_the_directed_evolution_no_journal() {
    let evolved = evolved("RUN exit 3\n");
    assert!(!journal::path(Path::new(&run_of(&evolved))).exists());
}

#[test]
fn an_image_that_was_not_built_is_said_with_how_its_build_ended() {
    let evolved = evolved("RUN exit 3\n");
    assert!(
        evolved
            .said
            .lines()
            .last()
            .unwrap()
            .starts_with("cladenc: the build of the image exited with status ")
    );
}

#[test]
fn an_image_that_was_not_built_is_said_with_where_what_docker_printed_is_kept() {
    let evolved = evolved("RUN exit 3\n");
    let kept = format!(
        ": what Docker printed is kept in {}/image\n",
        run_of(&evolved)
    );
    assert!(evolved.said.ends_with(&kept));
}

#[test]
fn a_directed_evolution_that_is_resumed_builds_no_image() {
    assert!(
        !stopped_and_resumed_with_its_dockerfile_gone()
            .said
            .contains("building the image")
    );
}

#[test]
fn a_directed_evolution_that_is_resumed_runs_its_harnesses_in_the_image_that_was_built() {
    assert_eq!(
        solved_by_harness_1(&stopped_and_resumed_with_its_dockerfile_gone()),
        "t1\n"
    );
}

#[test]
fn a_directed_evolution_that_is_resumed_ends_though_the_dockerfile_it_was_given_is_gone() {
    assert!(stopped_and_resumed_with_its_dockerfile_gone().found.is_ok());
}

#[test]
fn a_directed_evolution_asked_to_stop_while_its_image_is_built_was_never_started() {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let settings = settings(directory.path(), &image, "RUN sleep 300\n");
    let outputs_directory = directory.path().join("outputs");
    let printed = outputs_directory.join("image/stdout");
    let stop = AtomicBool::new(false);
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| printed.exists());
            stop.store(true, Ordering::SeqCst);
        });
        directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink())
    });
    assert!(matches!(found, Err(DirectedEvolutionError::NotStarted)));
}
