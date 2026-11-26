use std::fs;
use std::io;
use std::path::Path;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;

use cladenc::benchmark::Benchmark;
use cladenc::file_system::Disk;
use cladenc::journal::Settings;
use tempfile::TempDir;

const A_PROGENITOR_THAT_DIVERSIFIES_BETTER_THAN_THE_PRIMOGENITOR: &str = r#"if [ -d /var/log/cladenc/trials ]
then
  [ -e "$CLADENC_HARNESS/improved" ] && echo t3 >> solved
  echo improved >> improved
else
  touch "$CLADENC_HARNESS/solved"
  cp "$CLADENC_HARNESS/solved" solved
fi"#;

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
        harness_run_command: A_PROGENITOR_THAT_DIVERSIFIES_BETTER_THAN_THE_PRIMOGENITOR.to_owned(),
        max_diversifications: 2,
        max_concurrent_containers: 1,
        diversification_timeout: 120,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 33.5,
        holdout_tasks: vec!["benchmark/t2".to_owned(), "benchmark/t3".to_owned()],
        corroboration_tasks_percentage: 33.5,
        corroboration_tasks: vec!["benchmark/t3".to_owned()],
        environment_image_repository: test_inputs::environment_image_repository(),
        seed: 1,
    }
}

static EVOLVED: LazyLock<TempDir> = LazyLock::new(|| {
    let directory = test_inputs::temporary_directory();
    let settings = settings(directory.path());
    let outputs_directory = directory.path().join("outputs");
    let stop = AtomicBool::new(false);
    crate::real::start(&settings, &outputs_directory, &stop, &mut io::sink()).unwrap();
    directory
});

#[test]
fn the_singleton_control_group_harness_is_created_from_the_source_code_of_the_progenitor() {
    let directory = &*EVOLVED;
    assert_eq!(
        fs::read_to_string(directory.path().join("outputs/harnesses/3/improved")).unwrap(),
        "improved\nimproved\n"
    );
}

#[test]
fn the_primogenitor_and_not_the_progenitor_creates_the_singleton_control_group_harness() {
    let directory = &*EVOLVED;
    assert_eq!(
        ["2", "3"].map(|harness| {
            directory
                .path()
                .join("outputs/harnesses")
                .join(harness)
                .join("solved")
                .exists()
        }),
        [true, false]
    );
}
