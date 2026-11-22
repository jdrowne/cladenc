use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::containers::ContainerError;
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::journal::{self, Settings};
use tempfile::TempDir;

struct Evolved {
    directory: TempDir,
    settings: Settings,
    found: Result<Found, DirectedEvolutionError>,
    said: String,
}

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
        harness_run_command: test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 17.0,
        holdout_tasks: vec!["benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 17.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn evolved_with_t2_built_from(steps: &str) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\n{steps}", test_inputs::IMAGE),
    )
    .unwrap();
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("outputs");
    let found = directed_evolution::start(&settings, &outputs_directory, &stop, &mut written);
    Evolved {
        directory,
        settings,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn evolved() -> Evolved {
    evolved_with_t2_built_from("")
}

fn not_built() -> Evolved {
    evolved_with_t2_built_from("RUN exit 3\n")
}

fn stopped_while_t2_is_built() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\nRUN sleep 300\n", test_inputs::IMAGE),
    )
    .unwrap();
    let mut written = Vec::new();
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("outputs");
    let printed = outputs_directory.join("environments/benchmark/t2/stdout");
    let found = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| printed.exists());
            stop.store(true, Ordering::SeqCst);
        });
        directed_evolution::start(&settings, &outputs_directory, &stop, &mut written)
    });
    Evolved {
        directory,
        settings,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn stopped_and_resumed_with_its_stop_flag_at(
    stopping: bool,
    change: impl FnOnce(&Path),
) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_on(directory.path(), "t2");
    let _ = directed_evolution::start(&settings, &outputs_directory, &stop, &mut io::sink());
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t2");
    change(&outputs_directory);
    let mut written = Vec::new();
    let found =
        directed_evolution::resume(&outputs_directory, &AtomicBool::new(stopping), &mut written);
    Evolved {
        directory,
        settings,
        found,
        said: String::from_utf8(written).unwrap(),
    }
}

fn resumed() -> Evolved {
    stopped_and_resumed_with_its_stop_flag_at(false, |_| ())
}

fn resumed_with_an_environment_that_no_longer_builds() -> Evolved {
    stopped_and_resumed_with_its_stop_flag_at(false, |outputs_directory| {
        fs::write(
            outputs_directory.join("benchmarks/benchmark/t1/environment/Dockerfile"),
            format!("FROM {}\nRUN exit 3\n", test_inputs::IMAGE),
        )
        .unwrap();
    })
}

fn environments(settings: &Settings) -> Vec<(String, String)> {
    ["t1", "t2", "t3"]
        .iter()
        .enumerate()
        .map(|(index, task)| {
            (
                format!("benchmark/{task}"),
                format!("{}:{index}", settings.environment_image_repository),
            )
        })
        .collect()
}

fn environments_left_by(evolved: &Evolved) -> Vec<String> {
    let listed = Command::new("docker")
        .args(["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"])
        .arg(&evolved.settings.environment_image_repository)
        .output()
        .unwrap();
    String::from_utf8(listed.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn outputs_of(evolved: &Evolved) -> String {
    evolved
        .directory
        .path()
        .join("outputs")
        .display()
        .to_string()
}

fn building(evolved: &Evolved) -> Vec<String> {
    environments(&evolved.settings)
        .into_iter()
        .map(|(task, image)| {
            format!("cladenc: building the environment of task {task} as the image {image}")
        })
        .collect()
}

#[test]
fn a_directed_evolution_that_ended_leaves_no_environment_it_built_on_the_machine() {
    assert_eq!(environments_left_by(&evolved()), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_whose_environment_is_not_built_leaves_none_it_built_on_the_machine() {
    assert_eq!(environments_left_by(&not_built()), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_stopped_while_an_environment_is_built_leaves_none_on_the_machine() {
    assert_eq!(
        environments_left_by(&stopped_while_t2_is_built()),
        Vec::<String>::new()
    );
}

#[test]
fn a_directed_evolution_says_that_it_builds_the_environment_of_each_task_and_as_what_image() {
    let evolved = evolved();
    assert_eq!(
        evolved
            .said
            .lines()
            .filter(|line| line.starts_with("cladenc: building the environment of task "))
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        building(&evolved)
    );
}

#[test]
fn a_directed_evolution_says_that_the_environments_are_built_and_where_docker_s_output_is_kept() {
    let evolved = evolved();
    assert!(evolved.said.contains(&format!(
        "cladenc: built the environments of the 3 tasks: what Docker printed is kept in \
         {}/environments\n",
        outputs_of(&evolved)
    )));
}

#[test]
fn what_docker_printed_while_it_built_an_environment_is_kept_in_the_outputs_directory() {
    let evolved = evolved();
    assert!(
        Path::new(&outputs_of(&evolved))
            .join("environments/benchmark/t2/stdout")
            .is_file()
    );
}

#[test]
fn the_environments_are_built_after_the_tasks_are_held_out_and_before_the_first_action() {
    let evolved = evolved();
    let places = [
        "cladenc: holding out ",
        "cladenc: building the environment of task benchmark/t1 ",
        "cladenc: built the environments ",
        "cladenc: started the ",
    ]
    .map(|start| {
        evolved
            .said
            .lines()
            .position(|line| line.starts_with(start))
    });
    assert_eq!(places, [Some(2), Some(3), Some(6), Some(7)]);
}

#[test]
fn a_task_whose_environment_is_not_built_stops_the_directed_evolution_naming_the_task() {
    assert!(matches!(
        not_built().found,
        Err(DirectedEvolutionError::Environment {
            task,
            error: ContainerError::Exit(_),
        }) if task == "benchmark/t2"
    ));
}

#[test]
fn a_task_whose_environment_is_not_built_is_said_with_how_its_build_ended() {
    assert!(
        not_built()
            .said
            .lines()
            .last()
            .unwrap()
            .starts_with("cladenc: the build of the environment of task benchmark/t2 exited with ")
    );
}

#[test]
fn a_task_whose_environment_is_not_built_is_said_with_where_what_docker_printed_is_kept() {
    let evolved = not_built();
    let kept = format!(
        ": what Docker printed is kept in {}/environments/benchmark/t2\n",
        outputs_of(&evolved)
    );
    assert!(evolved.said.ends_with(&kept));
}

#[test]
fn what_docker_printed_for_an_environment_that_was_not_built_says_which_step_failed() {
    let evolved = not_built();
    let kept = Path::new(&outputs_of(&evolved)).join("environments/benchmark/t2");
    let printed = ["stdout", "stderr"]
        .map(|file| fs::read_to_string(kept.join(file)).unwrap())
        .concat();
    assert!(printed.contains("RUN exit 3"));
}

#[test]
fn a_task_whose_environment_is_not_built_leaves_the_directed_evolution_no_journal() {
    let evolved = not_built();
    assert!(!journal::path(Path::new(&outputs_of(&evolved))).exists());
}

#[test]
fn a_task_whose_environment_is_not_built_leaves_the_environments_after_it_unbuilt() {
    assert!(
        !not_built()
            .said
            .contains("environment of task benchmark/t3")
    );
}

#[test]
fn a_resumed_directed_evolution_that_ended_leaves_no_environment_it_built_on_the_machine() {
    assert_eq!(environments_left_by(&resumed()), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_that_stopped_at_an_attempt_leaves_no_environment_on_the_machine() {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    test_inputs::end_with_125_on(directory.path(), "t2");
    let found = directed_evolution::start(
        &settings,
        &directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    let stopped = Evolved {
        directory,
        settings,
        found,
        said: String::new(),
    };
    assert_eq!(environments_left_by(&stopped), Vec::<String>::new());
}

#[test]
fn a_resumed_directed_evolution_says_that_it_builds_each_environment_after_it_says_how_far_it_got()
{
    let evolved = resumed();
    assert_eq!(
        evolved.said.lines().skip(1).take(3).collect::<Vec<_>>(),
        building(&evolved)
    );
}

#[test]
fn a_resumed_directed_evolution_ends_once_its_environments_are_built_again() {
    assert!(resumed().found.is_ok());
}

#[test]
fn a_resumed_directed_evolution_builds_the_environments_from_the_tasks_in_its_outputs_directory() {
    assert!(matches!(
        resumed_with_an_environment_that_no_longer_builds().found,
        Err(DirectedEvolutionError::Environment {
            task,
            error: ContainerError::Exit(_),
        }) if task == "benchmark/t1"
    ));
}

#[test]
fn a_directed_evolution_asked_to_stop_while_an_environment_is_built_was_never_started() {
    assert!(matches!(
        stopped_while_t2_is_built().found,
        Err(DirectedEvolutionError::NotStarted)
    ));
}

#[test]
fn a_directed_evolution_asked_to_stop_while_an_environment_is_built_does_not_say_it_was_not_built()
{
    assert!(
        !stopped_while_t2_is_built()
            .said
            .contains("cladenc: the build of the environment")
    );
}

#[test]
fn a_resumed_directed_evolution_asked_to_stop_before_its_environments_are_built_stops_as_asked() {
    assert!(matches!(
        stopped_and_resumed_with_its_stop_flag_at(true, |_| ()).found,
        Err(DirectedEvolutionError::Asked)
    ));
}
