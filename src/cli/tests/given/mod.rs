use std::sync::atomic::AtomicBool;

use crate::cli::{self, Cli, Command, StartOptions};
use crate::containers::tests::double::Double;
use crate::file_system::tests::double::Memory;

pub(super) struct Ran {
    pub(super) status: u8,
    pub(super) output: String,
    pub(super) errors: String,
}

pub(super) fn options(directory: &Memory) -> StartOptions {
    StartOptions {
        primogenitor: test_inputs::primogenitor_in(directory, directory.path()),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: vec![test_inputs::benchmark_in(
            directory,
            directory.path(),
            &["t1", "t2", "t3"],
        )],
        procreation_docker_image: Some("image".to_owned()),
        procreation_dockerfile: None,
        procreation_docker_image_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.path().join("outputs"),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 30,
        max_grader_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 30.0,
        corroboration_tasks_percentage: 30.0,
    }
}

pub(super) fn executed(docker: Double, directory: &Memory, command: Command) -> Ran {
    let cli = Cli { command };
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let status = cli::execute(
        docker.on(directory).outside(),
        &cli,
        &AtomicBool::new(false),
        &mut output,
        &mut errors,
    );
    Ran {
        status,
        output: String::from_utf8(output).unwrap(),
        errors: String::from_utf8(errors).unwrap(),
    }
}
