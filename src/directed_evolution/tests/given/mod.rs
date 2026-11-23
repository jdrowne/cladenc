use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use tempfile::TempDir;

use crate::benchmark::Benchmark;
use crate::containers::tests::double::Double;
use crate::directed_evolution::{self, DirectedEvolutionError, Found};
use crate::journal::Settings;
use crate::outputs_directory::OutputsDirectory;

pub(super) struct Evolved {
    directory: TempDir,
    docker: Double,
    pub(super) result: Result<Found, DirectedEvolutionError>,
    pub(super) said: String,
}

impl Evolved {
    pub(super) fn directory(&self) -> &Path {
        self.directory.path()
    }

    pub(super) fn commands(&self) -> Vec<String> {
        let outputs_directory = OutputsDirectory::at(&self.directory.path().join("outputs"));
        self.docker.commands(&outputs_directory)
    }

    pub(super) fn count_of(&self, part: &str) -> usize {
        self.commands()
            .iter()
            .filter(|command| command.contains(part))
            .count()
    }
}

pub(super) fn settings(directory: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&test_inputs::benchmark(directory, &["t1", "t2"])).unwrap(),
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
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

pub(super) fn configured(directory: &Path, task: &str, configuration: &str) {
    let written = format!("schema_version = \"1.4\"\n{configuration}");
    fs::write(
        directory.join("benchmark").join(task).join("task.toml"),
        written,
    )
    .unwrap();
}

pub(super) fn unchanged(_: &Path, _: &mut Settings) {}

pub(super) fn started(docker: Double, change: impl FnOnce(&Path, &mut Settings)) -> Evolved {
    started_until(docker, change, &AtomicBool::new(false))
}

pub(super) fn started_until(
    docker: Double,
    change: impl FnOnce(&Path, &mut Settings),
    stop: &AtomicBool,
) -> Evolved {
    let directory = test_inputs::temporary_directory();
    let mut settings = settings(directory.path());
    change(directory.path(), &mut settings);
    let mut said = Vec::new();
    let result = directed_evolution::start(
        &docker,
        &settings,
        &directory.path().join("outputs"),
        stop,
        &mut said,
    );
    Evolved {
        directory,
        docker,
        result,
        said: String::from_utf8(said).unwrap(),
    }
}

pub(super) fn resumed(evolved: Evolved, docker: Double) -> Evolved {
    let mut said = Vec::new();
    let result = directed_evolution::resume(
        &docker,
        &evolved.directory.path().join("outputs"),
        &AtomicBool::new(false),
        &mut said,
    );
    Evolved {
        directory: evolved.directory,
        docker,
        result,
        said: String::from_utf8(said).unwrap(),
    }
}
