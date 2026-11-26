use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

fn settings(directory: &Path, build_command: &str) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        benchmarks: vec![
            Benchmark::read(&Disk, &test_inputs::benchmark(directory, &["t1"])).unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: Some(build_command.to_owned()),
        harness_run_command: test_inputs::SOLVE_ONE_MORE.to_owned(),
        max_diversifications: 1,
        max_concurrent_containers: 1,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: vec!["benchmark/t1".to_owned()],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn stopped_while_its_primogenitor_is_built() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path(), "hostname > id; sleep 300");
    let outputs_directory = directory.path().join("outputs");
    let id_file = outputs_directory.join("harnesses/0/id");
    let stop = AtomicBool::new(false);
    let _ = thread::scope(|scope| {
        scope.spawn(|| {
            test_inputs::within_ten_seconds(|| test_inputs::line_is_written(&id_file));
            stop.store(true, Ordering::SeqCst);
        });
        crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink())
    });
    directory
}

#[test]
fn a_directed_evolution_asked_to_stop_while_its_primogenitor_is_built_leaves_no_container_behind() {
    let directory = stopped_while_its_primogenitor_is_built();
    assert!(test_inputs::container_is_gone(
        &directory.path().join("outputs/harnesses/0/id")
    ));
}
