use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

fn settings(directory: &Path) -> Settings {
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
        harness_build_command: None,
        harness_run_command: "hostname; sleep 300".to_owned(),
        max_procreations: 1,
        max_concurrent_containers: 1,
        procreation_timeout: 60,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 25.0,
        holdout_tasks: Vec::new(),
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: Vec::new(),
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 7,
    }
}

fn stopped_by_its_flag() -> TempDir {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let id_file = outputs_directory.join("running/attempt-0-benchmark+t1/stdout");
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
fn a_directed_evolution_stopped_by_its_flag_leaves_no_container_behind() {
    let directory = stopped_by_its_flag();
    assert!(test_inputs::container_is_gone(
        &directory
            .path()
            .join("outputs/running/attempt-0-benchmark+t1/stdout")
    ));
}
