use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::journal::{self, Settings};
use cladenc::search::{self, Found, SearchError};
use tempfile::TempDir;
use test_inputs::Image;

const SOLVES_T1: &str = "RUN echo t1 > /solved-by-the-image\n";

struct Searched {
    directory: TempDir,
    image: Image,
    found: Result<Found, SearchError>,
    said: String,
}

fn settings(directory: &Path, image: &Image, steps: &str) -> Settings {
    Settings {
        progenitor: test_inputs::progenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![Benchmark::read(&test_inputs::benchmark(directory, &["t1"])).unwrap()],
        docker_image: image.name.clone(),
        dockerfile: Some(test_inputs::dockerfile(directory, steps)),
        docker_build_timeout: Some(60),
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES.to_owned(),
        budget: 2,
        genealogy_growth: 0.0,
        confidence: 99.0,
        initial_procreations: 1,
        workers: 1,
        procreation_timeout: 30,
        evaluation_timeout: 30,
        seed: 7,
    }
}

fn searched(steps: &str) -> Searched {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let settings = settings(directory.path(), &image, steps);
    let run_directory = directory.path().join("run");
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let found = search::search(&settings, &run_directory, &stop, &mut written);
    Searched {
        directory,
        image,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn stopped_and_resumed_with_its_dockerfile_gone() -> Searched {
    let directory = test_inputs::temporary_directory();
    let image = test_inputs::image();
    let settings = Settings {
        harness_run_command: format!(
            "[ -e end-with-125-at ] && exit 125\n{}",
            test_inputs::SOLVE_WHAT_THE_IMAGE_SOLVES
        ),
        ..settings(directory.path(), &image, SOLVES_T1)
    };
    let run_directory = directory.path().join("run");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_at(&settings.progenitor, 0);
    let _ = search::search(&settings, &run_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_no_more(&run_directory);
    fs::remove_file(directory.path().join("Dockerfile")).unwrap();
    let mut written = Vec::new();
    let found = search::resume(&run_directory, &stop, &mut written);
    Searched {
        directory,
        image,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn run_of(searched: &Searched) -> String {
    searched.directory.path().join("run").display().to_string()
}

fn solved_by_harness_1(searched: &Searched) -> String {
    fs::read_to_string(searched.directory.path().join("run/harnesses/1/solved")).unwrap()
}

#[test]
fn a_search_given_a_dockerfile_runs_its_harnesses_in_the_image_built_from_it() {
    assert_eq!(solved_by_harness_1(&searched(SOLVES_T1)), "t1\n");
}

#[test]
fn a_search_given_a_dockerfile_finds_the_harness_made_in_its_image_to_be_the_best() {
    assert_eq!(searched(SOLVES_T1).found.unwrap().best_harness, 1);
}

#[test]
fn a_search_says_that_it_builds_its_image_and_that_the_image_is_built() {
    let searched = searched(SOLVES_T1);
    let said = format!(
        "cladenc: building the image {0} from the Dockerfile {1}\n\
         cladenc: the image {0} is built: what Docker printed is kept in {2}/image\n",
        searched.image.name,
        searched.directory.path().join("Dockerfile").display(),
        run_of(&searched)
    );
    assert!(searched.said.contains(&said));
}

#[test]
fn the_image_is_built_after_the_tasks_are_copied_and_before_the_first_action() {
    let searched = searched(SOLVES_T1);
    let places = [
        "cladenc: copying the 1 task ",
        "cladenc: building the image ",
        "cladenc: started the ",
    ]
    .map(|start| {
        searched
            .said
            .lines()
            .position(|line| line.starts_with(start))
    });
    assert_eq!(places, [Some(1), Some(2), Some(4)]);
}

#[test]
fn the_run_record_names_the_image_that_was_built_its_dockerfile_and_the_time_limit() {
    let searched = searched(SOLVES_T1);
    let journal = fs::read_to_string(journal::path(Path::new(&run_of(&searched)))).unwrap();
    let named = format!(
        r#""docker_image":"{}","dockerfile":"{}","docker_build_timeout":60,"#,
        searched.image.name,
        searched.directory.path().join("Dockerfile").display()
    );
    assert!(journal.lines().next().unwrap().contains(&named));
}

#[test]
fn a_dockerfile_that_is_not_built_stops_the_search_before_its_first_action() {
    assert!(matches!(
        searched("RUN exit 3\n").found,
        Err(SearchError::Image(ContainerError::Exit(_)))
    ));
}

#[test]
fn a_dockerfile_that_is_not_built_leaves_the_search_no_journal() {
    let searched = searched("RUN exit 3\n");
    assert!(!journal::path(Path::new(&run_of(&searched))).exists());
}

#[test]
fn an_image_that_was_not_built_is_said_with_how_its_build_ended() {
    let searched = searched("RUN exit 3\n");
    assert!(
        searched
            .said
            .lines()
            .last()
            .unwrap()
            .starts_with("cladenc: the build of the image exited with status ")
    );
}

#[test]
fn an_image_that_was_not_built_is_said_with_where_what_docker_printed_is_kept() {
    let searched = searched("RUN exit 3\n");
    let kept = format!(
        ": what Docker printed is kept in {}/image\n",
        run_of(&searched)
    );
    assert!(searched.said.ends_with(&kept));
}

#[test]
fn a_search_that_is_resumed_builds_no_image() {
    assert!(
        !stopped_and_resumed_with_its_dockerfile_gone()
            .said
            .contains("building the image")
    );
}

#[test]
fn a_search_that_is_resumed_runs_its_harnesses_in_the_image_that_was_built() {
    assert_eq!(
        solved_by_harness_1(&stopped_and_resumed_with_its_dockerfile_gone()),
        "t1\n"
    );
}

#[test]
fn a_search_that_is_resumed_ends_though_the_dockerfile_it_was_given_is_gone() {
    assert!(stopped_and_resumed_with_its_dockerfile_gone().found.is_ok());
}
