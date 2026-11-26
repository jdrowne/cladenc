use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

struct Keyed {
    directory: TempDir,
    outputs_directory: PathBuf,
}

fn settings(directory: &Path, key_file: &Path) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor(directory),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: Some(key_file.to_owned()),
        benchmarks: vec![
            Benchmark::read(&Disk, &test_inputs::benchmark(directory, &["t1"])).unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "[ -e end-with-125 ] && exit 125; printf %s \"$CLADENC_MODEL_KEY\" > \
                              key-given"
            .to_owned(),
        max_diversifications: 1,
        max_concurrent_containers: 1,
        diversification_timeout: 30,
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

fn evolved_with(key: &str, stopped: bool) -> Keyed {
    let directory = test_inputs::temporary_directory();
    let key_file = test_inputs::model_key_file(directory.path(), key);
    let outputs_directory = directory.path().join("outputs");
    let settings = settings(directory.path(), &key_file);
    if stopped {
        test_inputs::end_with_125_on(directory.path(), "t1");
    }
    let _ = crate::real::start(
        &settings,
        &outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
    if stopped {
        test_inputs::end_with_125_on_no_more(&outputs_directory, "t1");
    }
    Keyed {
        directory,
        outputs_directory,
    }
}

fn resume(keyed: &Keyed) {
    let _ = crate::real::resume(
        &keyed.outputs_directory,
        &AtomicBool::new(false),
        &mut io::sink(),
    );
}

fn key_given_to_the_first_child(keyed: &Keyed) -> String {
    fs::read_to_string(keyed.outputs_directory.join("harnesses/1/key-given")).unwrap()
}

#[test]
fn a_directed_evolution_hands_the_key_its_file_holds_to_its_harnesses() {
    let keyed = evolved_with("a-made-up-key\n", false);
    assert_eq!(key_given_to_the_first_child(&keyed), "a-made-up-key");
}

#[test]
fn a_resumed_directed_evolution_reads_the_key_afresh_from_its_file() {
    let keyed = evolved_with("a-made-up-key\n", true);
    test_inputs::model_key_file(keyed.directory.path(), "a-key-that-replaced-it\n");
    resume(&keyed);
    assert_eq!(
        key_given_to_the_first_child(&keyed),
        "a-key-that-replaced-it"
    );
}
