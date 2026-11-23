use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use crate::benchmark::{Benchmark, HealthCheck, TaskConfiguration};
use crate::containers::tests::double::Double;
use crate::containers::{
    self, Attempt, Attempted, ContainerError, Environment, Host, Launch, Retry,
};
use crate::file_system::tests::double::Memory;
use crate::model::Model;
use crate::outputs_directory::OutputsDirectory;

pub(super) const TASK: &str = "benchmark/t1";

pub(super) const TIMER: &str =
    r#"sh -c "$0" & run=$!; (sleep "$1"; kill -9 $run) > /dev/null 2>&1 & wait $run"#;

pub(super) const LOGS_LINKED: &str =
    r#"[ -e "$1" ] || [ -L "$1" ] || { rm -rf "$2" && mkdir -p "$2" && ln -s "$2" "$1"; }"#;

pub(super) const EMPTIED: &str = r#"rm -rf "$1" && mkdir -p "$1""#;

pub(super) const ARTIFACTS_MADE: &str = r#"mkdir -p "$3" && chown "$4" "$3""#;

pub(super) const VERIFIER_EMPTIED: &str = r#"rm -rf "$3" && mkdir -p "$3" && chown "$4" "$3""#;

pub(super) const TESTS_LINKED: &str = r#"rm -rf "$1" && ln -s "$2" "$1""#;

pub(super) const HEALTH_CHECK: &str = "test -e /ready";

pub(super) const LAUNCH: Launch<'static> = Launch {
    image: test_inputs::IMAGE,
    run_command: "run a harness",
    build_command: None,
    model: Model {
        name: test_inputs::MODEL,
        url: test_inputs::MODEL_URL,
        key: None,
    },
    procreation_timeout: 30,
    max_grader_health_check_retries: 0,
    max_task_health_check_retries: 0,
};

pub(super) struct Tried {
    pub(super) directory: Memory,
    pub(super) outputs_directory: OutputsDirectory,
    pub(super) docker: Double,
    pub(super) result: Result<Attempted, ContainerError>,
    pub(super) retries: Vec<Retry>,
}

impl Tried {
    pub(super) fn commands(&self) -> Vec<String> {
        self.docker.commands(&self.outputs_directory)
    }

    pub(super) fn running(&self) -> PathBuf {
        self.outputs_directory.running_attempt(0, TASK)
    }
}

pub(super) fn made() -> (Memory, OutputsDirectory) {
    let directory = Memory::default();
    let primogenitor = test_inputs::primogenitor_in(&directory, directory.path());
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1"]);
    let benchmark = Benchmark::read(&directory, &benchmark).unwrap();
    let outputs_directory = OutputsDirectory::create(
        directory.shared(),
        &directory.path().join("outputs"),
        &primogenitor,
    )
    .unwrap();
    outputs_directory.copy_benchmark(&benchmark).unwrap();
    (directory, outputs_directory)
}

pub(super) fn host<'a>(docker: &'a Double, outputs_directory: &'a OutputsDirectory) -> Host<'a> {
    Host {
        docker,
        outputs_directory,
    }
}

pub(super) fn following(flag: &str, command: &str) -> Vec<String> {
    let words: Vec<&str> = command.split(' ').collect();
    words
        .windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].to_owned())
        .collect()
}

pub(super) fn leaving_a_socket(
    docker: Double,
    needles: &[&str],
    mounted_at: &'static str,
) -> Double {
    docker.doing(needles, move |files, asked| {
        let mounted = asked
            .arguments
            .iter()
            .find_map(|argument| argument.strip_suffix(mounted_at));
        if let Some(mounted) = mounted {
            files.socket(Path::new(mounted).join("socket")).unwrap();
        }
    })
}

pub(super) fn checked(retries: u64) -> HealthCheck {
    HealthCheck {
        command: HEALTH_CHECK.to_owned(),
        interval: Duration::ZERO,
        time_limit: Duration::from_secs(30),
        start_period: Duration::ZERO,
        start_interval: Duration::ZERO,
        retries,
    }
}

pub(super) fn environment(configuration: TaskConfiguration) -> Environment {
    Environment {
        image: test_inputs::IMAGE.to_owned(),
        configuration,
        grader: None,
    }
}

pub(super) fn attempted(docker: Double, launch: &Launch<'_>, environment: &Environment) -> Tried {
    attempted_until(docker, launch, environment, &AtomicBool::new(false))
}

pub(super) fn attempted_until(
    docker: Double,
    launch: &Launch<'_>,
    environment: &Environment,
    stop: &AtomicBool,
) -> Tried {
    let (directory, outputs_directory) = made();
    let docker = docker.on(&directory);
    let retries = RefCell::new(Vec::new());
    let attempt = Attempt {
        harness: 0,
        task: TASK,
        environment,
    };
    let result = containers::run_attempt(
        host(&docker, &outputs_directory),
        launch,
        attempt,
        stop,
        &|retry| retries.borrow_mut().push(retry),
    );
    Tried {
        directory,
        outputs_directory,
        docker,
        result,
        retries: retries.into_inner(),
    }
}
