use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use crate::dataset::Dataset;
use crate::directed_evolution::{self, DirectedEvolutionError, Found};
use crate::file_system::tests::double::Memory;
use crate::journal::{self, Record, Settings};
use crate::outputs_directory::OutputsDirectory;
use crate::sandboxes::tests::double::Double;

pub(super) struct Evolved {
    directory: Memory,
    docker: Double,
    pub(super) result: Result<Found, DirectedEvolutionError>,
    pub(super) said: String,
}

impl Evolved {
    pub(super) fn files(&self) -> &Memory {
        &self.directory
    }

    pub(super) fn outputs(&self) -> PathBuf {
        self.directory.path().join("outputs")
    }

    pub(super) fn outputs_directory(&self) -> OutputsDirectory {
        OutputsDirectory::at(self.directory.shared(), &self.outputs())
    }

    pub(super) fn commands(&self) -> Vec<String> {
        self.docker.commands(&self.outputs_directory())
    }

    pub(super) fn journal(&self) -> String {
        self.directory
            .read_to_string(journal::path(&self.outputs()))
            .unwrap()
    }

    pub(super) fn records(&self) -> Vec<Record> {
        self.journal()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    pub(super) fn starts(&self) -> Vec<String> {
        self.records()
            .into_iter()
            .filter_map(|record| match record {
                Record::TrialStarted { harness, .. } => Some(format!("trial of {harness}")),
                Record::DiversificationStarted { child, progenitor } => {
                    Some(format!("diversification of {child} by {progenitor}"))
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

    pub(super) fn lines(&self) -> Vec<String> {
        self.said.lines().map(str::to_owned).collect()
    }

    pub(super) fn best_harness(&self) -> Option<u64> {
        self.result.as_ref().ok().map(|found| found.best_harness)
    }

    pub(super) fn inputs(&self) -> Vec<String> {
        self.docker
            .asked()
            .into_iter()
            .map(|asked| asked.input)
            .collect()
    }

    pub(super) fn count_of(&self, part: &str) -> usize {
        self.commands()
            .iter()
            .filter(|command| command.contains(part))
            .count()
    }
}

pub(super) fn settings(directory: &Memory) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor_in(directory, directory.path()),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        datasets: vec![
            Dataset::read(
                directory,
                &test_inputs::dataset_in(directory, directory.path(), &["t1", "t2"]),
            )
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_diversifications: 1,
        max_concurrent_sandboxes: 1,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

pub(super) fn configured(directory: &Memory, task: &str, configuration: &str) {
    let written = format!("schema_version = \"1.4\"\n{configuration}");
    let file = directory
        .path()
        .join("dataset")
        .join(task)
        .join("task.toml");
    directory.write(file, written).unwrap();
}

pub(super) fn unchanged(_: &Memory, _: &mut Settings) {}

pub(super) fn over_one_task(_: &Memory, settings: &mut Settings) {
    settings.datasets[0].tasks = vec!["t1".to_owned()];
}

pub(super) fn over_five_tasks(directory: &Memory, settings: &mut Settings) {
    let tasks = ["t1", "t2", "t3", "t4", "t5"];
    let dataset = test_inputs::dataset_in(directory, directory.path(), &tasks);
    settings.datasets = vec![Dataset::read(directory, &dataset).unwrap()];
}

pub(super) fn started(docker: Double, change: impl FnOnce(&Memory, &mut Settings)) -> Evolved {
    started_until(docker, change, &AtomicBool::new(false))
}

pub(super) fn started_until(
    docker: Double,
    change: impl FnOnce(&Memory, &mut Settings),
    stop: &AtomicBool,
) -> Evolved {
    let directory = Memory::default();
    let docker = docker.on(&directory);
    let mut settings = settings(&directory);
    change(&directory, &mut settings);
    let mut said = Vec::new();
    let result = directed_evolution::start(
        docker.outside(),
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
    resumed_until(evolved, docker, &AtomicBool::new(false))
}

pub(super) fn resumed_until(evolved: Evolved, docker: Double, stop: &AtomicBool) -> Evolved {
    let docker = docker.on(&evolved.directory);
    let mut said = Vec::new();
    let result = directed_evolution::resume(
        docker.outside(),
        &evolved.directory.path().join("outputs"),
        stop,
        &mut said,
    );
    Evolved {
        directory: evolved.directory,
        docker,
        result,
        said: String::from_utf8(said).unwrap(),
    }
}
