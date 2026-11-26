use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

struct Evolved {
    directory: TempDir,
    settings: Settings,
}

fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(
                &Disk,
                &test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
            )
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: test_inputs::SOLVE_ONE_MORE_OR_END_WITH_125.to_owned(),
        max_diversifications: 1,
        max_concurrent_sandboxes: 1,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
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
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("outputs");
    let _ = crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink());
    Evolved {
        directory,
        settings,
    }
}

fn evolved() -> Evolved {
    evolved_with_t2_built_from("")
}

static NOT_BUILT: LazyLock<Evolved> = LazyLock::new(|| evolved_with_t2_built_from("RUN exit 3\n"));

fn stopped_while_t2_is_built() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    fs::write(
        directory.path().join("benchmark/t2/environment/Dockerfile"),
        format!("FROM {}\nRUN sleep 300\n", test_inputs::IMAGE),
    )
    .unwrap();
    let stop = AtomicBool::new(false);
    let outputs_directory = directory.path().join("outputs");
    let printed = outputs_directory.join("environments/benchmark/t2/stdout");
    let _ = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| printed.exists());
            stop.store(true, Ordering::SeqCst);
        });
        crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink())
    });
    Evolved {
        directory,
        settings,
    }
}

fn stopped_at_a_trial() -> Evolved {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    test_inputs::end_with_125_on(directory.path(), "t2");
    let _ = crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink());
    Evolved {
        directory,
        settings,
    }
}

fn resumed() -> Evolved {
    let stopped = stopped_at_a_trial();
    let outputs_directory = stopped.directory.path().join("outputs");
    test_inputs::end_with_125_on_no_more(&outputs_directory, "t2");
    let stop = AtomicBool::new(false);
    let _ = crate::real::resume(&outputs_directory, &stop, &mut io::sink());
    stopped
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

#[test]
fn a_directed_evolution_that_ended_leaves_no_environment_it_built_on_the_machine() {
    assert_eq!(environments_left_by(&evolved()), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_whose_environment_is_not_built_leaves_none_it_built_on_the_machine() {
    assert_eq!(environments_left_by(&NOT_BUILT), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_stopped_while_an_environment_is_built_leaves_none_on_the_machine() {
    assert_eq!(
        environments_left_by(&stopped_while_t2_is_built()),
        Vec::<String>::new()
    );
}

#[test]
fn what_docker_printed_for_an_environment_that_was_not_built_says_which_step_failed() {
    let kept = NOT_BUILT
        .directory
        .path()
        .join("outputs/environments/benchmark/t2");
    let printed = ["stdout", "stderr"]
        .map(|file| fs::read_to_string(kept.join(file)).unwrap())
        .concat();
    assert!(printed.contains("RUN exit 3"));
}

#[test]
fn a_resumed_directed_evolution_that_ended_leaves_no_environment_it_built_on_the_machine() {
    assert_eq!(environments_left_by(&resumed()), Vec::<String>::new());
}

#[test]
fn a_directed_evolution_that_stopped_at_a_trial_leaves_no_environment_on_the_machine() {
    assert_eq!(
        environments_left_by(&stopped_at_a_trial()),
        Vec::<String>::new()
    );
}
