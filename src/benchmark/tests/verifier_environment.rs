use std::path::Path;
use std::time::Duration;

use crate::benchmark::configuration::{self, ConfigurationError};
use crate::benchmark::{self, BenchmarkError, EnvironmentConfiguration, TaskConfiguration};
use crate::file_system::tests::double::Memory;

const SEPARATE: &str = "[verifier]\nenvironment_mode = \"separate\"\n";
const OWN: &str = "[verifier.environment]\n";
const CHECKED: &str = "[verifier.environment.healthcheck]\ncommand = \"test -e /ready\"\n";
const NAMED: &str =
    "[verifier.environment]\ndocker_image = \"registry.example/verifiers/parser:3\"\n";
const EVERYTHING: &str = "the whole container of the harness in the verifier container";

fn read(written: &str) -> Result<TaskConfiguration, ConfigurationError> {
    let directory = Memory::default();
    directory
        .write(directory.path().join("task.toml"), written)
        .unwrap();
    configuration::read_with(&directory, directory.path(), &|variable| {
        (variable == "TOKEN").then(|| "a-made-up-token".to_owned())
    })
}

fn of_the_verifier(written: &str) -> Option<EnvironmentConfiguration> {
    read(written).unwrap().verifier_environment
}

fn own(written: &str) -> EnvironmentConfiguration {
    of_the_verifier(&format!("{OWN}{written}")).unwrap()
}

fn refused(written: &str) -> Option<ConfigurationError> {
    read(written).err()
}

fn tasks_with_t2_changed(
    change: impl FnOnce(&Memory, &Path),
) -> Result<Vec<String>, BenchmarkError> {
    let directory = Memory::default();
    let benchmark = test_inputs::benchmark_in(&directory, directory.path(), &["t1", "t2"]);
    change(&directory, &benchmark.join("t2"));
    benchmark::tasks(&directory, &benchmark)
}

fn both_tasks() -> Result<Vec<String>, BenchmarkError> {
    Ok(vec!["t1".to_owned(), "t2".to_owned()])
}

#[test]
fn a_task_that_says_nothing_of_where_its_verifier_runs_gives_the_verifier_no_environment() {
    assert_eq!(of_the_verifier(""), None);
}

#[test]
fn a_task_that_says_its_verifier_shares_the_environment_gives_the_verifier_none() {
    assert_eq!(
        of_the_verifier("[verifier]\nenvironment_mode = \"shared\"\n"),
        None
    );
}

#[test]
fn a_task_that_says_its_verifier_runs_separately_gives_the_verifier_an_environment() {
    assert_eq!(
        of_the_verifier(SEPARATE),
        Some(EnvironmentConfiguration::default())
    );
}

#[test]
fn a_task_with_an_environment_under_verifier_gives_the_verifier_an_environment_by_having_it() {
    assert_eq!(
        of_the_verifier(OWN),
        Some(EnvironmentConfiguration::default())
    );
}

#[test]
fn a_task_may_say_that_its_verifier_runs_separately_and_give_it_an_environment() {
    assert_eq!(
        of_the_verifier(&format!("{SEPARATE}\n{OWN}cpus = 2\n")),
        Some(EnvironmentConfiguration {
            cpus: Some(2.0),
            ..EnvironmentConfiguration::default()
        })
    );
}

#[test]
fn the_verifier_image_is_read_from_docker_image_under_verifier_environment() {
    assert_eq!(
        of_the_verifier(NAMED).unwrap().docker_image.as_deref(),
        Some("registry.example/verifiers/parser:3")
    );
}

#[test]
fn a_verifier_image_named_with_nothing_is_left_out() {
    assert_eq!(own("docker_image = \"\"\n").docker_image, None);
}

#[test]
fn the_working_directory_of_the_verifier_is_read_from_workdir_under_verifier_environment() {
    assert_eq!(
        own("workdir = \"/verifying\"\n")
            .working_directory
            .as_deref(),
        Some("/verifying")
    );
}

#[test]
fn the_variables_of_the_verifier_container_are_read_from_env_under_verifier_environment() {
    assert_eq!(
        own("env = { MODE = \"strict\" }\n").variables,
        [("MODE".to_owned(), "strict".to_owned())]
    );
}

#[test]
fn a_variable_of_the_verifier_container_is_given_the_value_it_has_where_cladenc_runs() {
    assert_eq!(
        own("env = { KEY = \"${TOKEN}\" }\n").variables,
        [("KEY".to_owned(), "a-made-up-token".to_owned())]
    );
}

#[test]
fn the_time_limit_of_the_build_of_the_verifier_environment_is_read() {
    assert_eq!(
        own("build_timeout_sec = 90\n").build_time_limit,
        Duration::from_secs(90)
    );
}

#[test]
fn the_build_of_the_verifier_environment_is_given_600_seconds_where_the_task_gives_none() {
    assert_eq!(own("").build_time_limit, Duration::from_secs(600));
}

#[test]
fn the_cpus_of_the_verifier_container_are_read_from_cpus_under_verifier_environment() {
    assert_eq!(own("cpus = 0.5\n").cpus, Some(0.5));
}

#[test]
fn the_memory_of_the_verifier_container_is_read_in_megabytes() {
    assert_eq!(own("memory_mb = 512\n").memory_megabytes, Some(512));
}

#[test]
fn the_memory_of_the_verifier_container_is_read_from_the_older_spelling() {
    assert_eq!(own("memory = \"2G\"\n").memory_megabytes, Some(2048));
}

#[test]
fn the_health_check_of_the_verifier_environment_is_read() {
    assert_eq!(
        of_the_verifier(&format!("{CHECKED}retries = 7\n"))
            .unwrap()
            .health_check
            .map(|health_check| (health_check.command, health_check.retries)),
        Some(("test -e /ready".to_owned(), 7))
    );
}

#[test]
fn a_setting_the_verifier_environment_leaves_out_is_not_taken_from_that_of_the_task() {
    assert_eq!(
        of_the_verifier(
            "[environment]\ncpus = 4\nworkdir = \"/app\"\n\n[verifier.environment]\nmemory_mb = \
             512\n"
        ),
        Some(EnvironmentConfiguration {
            memory_megabytes: Some(512),
            ..EnvironmentConfiguration::default()
        })
    );
}

#[test]
fn a_verifier_that_runs_separately_with_no_environment_of_its_own_follows_that_of_the_task() {
    assert_eq!(
        of_the_verifier(&format!(
            "[environment]\ncpus = 4\nworkdir = \"/app\"\nenv = {{ MODE = \"strict\" }}\n\n{SEPARATE}"
        )),
        Some(EnvironmentConfiguration {
            cpus: Some(4.0),
            working_directory: Some("/app".to_owned()),
            variables: vec![("MODE".to_owned(), "strict".to_owned())],
            ..EnvironmentConfiguration::default()
        })
    );
}

#[test]
fn a_verifier_that_follows_the_environment_of_the_task_has_its_health_check_as_well() {
    assert_eq!(
        of_the_verifier(&format!(
            "[environment.healthcheck]\ncommand = \"test -e /ready\"\n\n{SEPARATE}"
        ))
        .unwrap()
        .health_check
        .map(|health_check| health_check.command),
        Some("test -e /ready".to_owned())
    );
}

#[test]
fn the_image_that_the_environment_of_the_task_names_is_not_a_verifier_image_s_own() {
    assert_eq!(
        of_the_verifier(&format!(
            "[environment]\ndocker_image = \"registry.example/tasks/parser:3\"\n\n{SEPARATE}"
        ))
        .unwrap()
        .docker_image,
        None
    );
}

#[test]
fn the_verifier_environment_changes_nothing_of_the_environment_of_the_task() {
    assert_eq!(
        read(&format!("{OWN}cpus = 2\nworkdir = \"/verifying\"\n"))
            .map(|configuration| { (configuration.cpus, configuration.working_directory) }),
        Ok((None, None))
    );
}

#[test]
fn cpus_of_the_verifier_that_are_not_above_0_are_refused_under_verifier_environment() {
    assert_eq!(
        refused(&format!("{OWN}cpus = 0\n")),
        Some(ConfigurationError::NotAbove0 {
            key: "verifier.environment.cpus"
        })
    );
}

#[test]
fn a_time_limit_of_0_for_the_build_of_the_verifier_environment_is_refused() {
    assert_eq!(
        refused(&format!("{OWN}build_timeout_sec = 0\n")),
        Some(ConfigurationError::NotATimeLimit {
            key: "verifier.environment.build_timeout_sec"
        })
    );
}

#[test]
fn memory_of_the_verifier_that_is_not_above_0_megabytes_is_refused() {
    assert_eq!(
        refused(&format!("{OWN}memory_mb = 0\n")),
        Some(ConfigurationError::NotAbove0 {
            key: "verifier.environment.memory_mb"
        })
    );
}

#[test]
fn an_older_spelling_of_the_memory_of_the_verifier_that_is_no_size_is_refused() {
    assert_eq!(
        refused(&format!("{OWN}memory = \"lots\"\n")),
        Some(ConfigurationError::NotASize {
            key: "verifier.environment.memory"
        })
    );
}

#[test]
fn two_spellings_of_the_memory_of_the_verifier_that_differ_are_refused() {
    assert_eq!(
        refused(&format!("{OWN}memory = \"1G\"\nmemory_mb = 512\n")),
        Some(ConfigurationError::TwoSizes {
            key: "verifier.environment.memory"
        })
    );
}

#[test]
fn a_time_limit_of_0_for_a_run_of_the_health_check_of_the_verifier_is_refused() {
    assert_eq!(
        refused(&format!("{CHECKED}timeout_sec = 0\n")),
        Some(ConfigurationError::NotATimeLimit {
            key: "verifier.environment.healthcheck.timeout_sec"
        })
    );
}

#[test]
fn an_interval_below_0_for_the_health_check_of_the_verifier_is_refused() {
    assert_eq!(
        refused(&format!("{CHECKED}interval_sec = -1\n")),
        Some(ConfigurationError::NotAWait {
            key: "verifier.environment.healthcheck.interval_sec"
        })
    );
}

#[test]
fn a_start_period_below_0_for_the_health_check_of_the_verifier_is_refused() {
    assert_eq!(
        refused(&format!("{CHECKED}start_period_sec = -1\n")),
        Some(ConfigurationError::NotAWait {
            key: "verifier.environment.healthcheck.start_period_sec"
        })
    );
}

#[test]
fn a_start_interval_below_0_for_the_health_check_of_the_verifier_is_refused() {
    assert_eq!(
        refused(&format!("{CHECKED}start_interval_sec = -1\n")),
        Some(ConfigurationError::NotAWait {
            key: "verifier.environment.healthcheck.start_interval_sec"
        })
    );
}

#[test]
fn retries_of_0_for_the_health_check_of_the_verifier_are_refused() {
    assert_eq!(
        refused(&format!("{CHECKED}retries = 0\n")),
        Some(ConfigurationError::NotAbove0 {
            key: "verifier.environment.healthcheck.retries"
        })
    );
}

#[test]
fn a_variable_of_the_verifier_container_named_otherwise_is_refused_under_its_table() {
    assert_eq!(
        refused(&format!("{OWN}env = {{ \"A B\" = \"strict\" }}\n")),
        Some(ConfigurationError::NotAName {
            table: "verifier.environment",
            name: "A B".to_owned()
        })
    );
}

#[test]
fn a_variable_of_the_verifier_container_that_is_not_set_is_refused_under_its_table() {
    assert_eq!(
        refused(&format!("{OWN}env = {{ KEY = \"${{MISSING}}\" }}\n")),
        Some(ConfigurationError::NotSet {
            table: "verifier.environment",
            name: "KEY".to_owned(),
            variable: "MISSING".to_owned()
        })
    );
}

#[test]
fn a_verifier_whose_environment_asks_for_a_gpu_is_refused() {
    assert_eq!(
        refused(&format!("{OWN}gpus = 1\n")),
        Some(ConfigurationError::Needs { what: "a GPU" })
    );
}

#[test]
fn a_verifier_whose_environment_is_for_windows_is_refused() {
    assert_eq!(
        refused(&format!("{OWN}os = \"windows\"\n")),
        Some(ConfigurationError::Needs { what: "Windows" })
    );
}

#[test]
fn a_verifier_whose_environment_asks_for_0_gpus_is_not_refused() {
    assert_eq!(refused(&format!("{OWN}gpus = 0\n")), None);
}

#[test]
fn a_task_that_lists_the_whole_container_for_a_verifier_that_runs_separately_is_refused() {
    assert_eq!(
        refused(&format!("artifacts = [\"/\"]\n\n{SEPARATE}")),
        Some(ConfigurationError::Needs { what: EVERYTHING })
    );
}

#[test]
fn a_path_that_names_nothing_below_the_top_is_the_whole_container() {
    assert_eq!(
        refused(&format!("artifacts = [\"/./\"]\n\n{OWN}")),
        Some(ConfigurationError::Needs { what: EVERYTHING })
    );
}

#[test]
fn a_task_that_lists_the_whole_container_for_a_verifier_that_shares_it_is_not_refused() {
    assert_eq!(refused("artifacts = [\"/\"]\n"), None);
}

#[test]
fn a_task_that_lists_a_path_for_a_verifier_that_runs_separately_is_not_refused() {
    assert_eq!(
        refused(&format!("artifacts = [\"/app/out\"]\n\n{SEPARATE}")),
        None
    );
}

#[test]
fn the_path_of_an_entry_under_artifacts_begins_at_the_top_and_holds_no_empty_part() {
    assert_eq!(
        read("artifacts = [\"app//out/./report.json\"]\n")
            .unwrap()
            .artifacts[0]
            .path(),
        "/app/out/report.json"
    );
}

#[test]
fn a_task_whose_verifier_is_in_an_image_it_names_needs_no_directory_named_tests() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory.remove_dir_all(task.join("tests")).unwrap();
            directory.write(task.join("task.toml"), NAMED).unwrap();
        }),
        both_tasks()
    );
}

#[test]
fn a_task_whose_verifier_is_in_an_image_built_from_its_tests_needs_no_test_sh() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory.remove_file(task.join("tests/test.sh")).unwrap();
            directory
                .write(task.join("tests/Dockerfile"), "FROM scratch\n")
                .unwrap();
            directory.write(task.join("task.toml"), SEPARATE).unwrap();
        }),
        both_tasks()
    );
}

#[test]
fn a_task_whose_verifier_runs_in_a_copy_of_its_environment_still_needs_its_test_sh() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory.remove_file(task.join("tests/test.sh")).unwrap();
            directory.write(task.join("task.toml"), SEPARATE).unwrap();
        }),
        Err(BenchmarkError::NoVerifier {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_dockerfile_in_the_tests_of_a_task_whose_verifier_shares_the_environment_is_no_verifier() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory.remove_file(task.join("tests/test.sh")).unwrap();
            directory
                .write(task.join("tests/Dockerfile"), "FROM scratch\n")
                .unwrap();
        }),
        Err(BenchmarkError::NoVerifier {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_whose_verifier_runs_separately_is_refused_for_a_compose_file_in_its_tests() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory
                .write(task.join("tests/docker-compose.yaml"), "services: {}\n")
                .unwrap();
            directory.write(task.join("task.toml"), SEPARATE).unwrap();
        }),
        Err(BenchmarkError::ManyVerifierContainers {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_that_names_its_verifier_image_is_still_refused_for_a_compose_file_in_its_tests() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory
                .write(task.join("tests/compose.yml"), "services: {}\n")
                .unwrap();
            directory.write(task.join("task.toml"), NAMED).unwrap();
        }),
        Err(BenchmarkError::ManyVerifierContainers {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_compose_file_in_the_tests_of_a_task_whose_verifier_shares_the_environment_is_let_through() {
    assert_eq!(
        tasks_with_t2_changed(|directory, task| {
            directory
                .write(task.join("tests/docker-compose.yaml"), "services: {}\n")
                .unwrap();
        }),
        both_tasks()
    );
}

#[test]
fn a_task_whose_verifier_needs_more_than_one_container_is_said_with_its_id() {
    assert_eq!(
        BenchmarkError::ManyVerifierContainers {
            task: "fix-the-parser".to_owned()
        }
        .to_string(),
        "task fix-the-parser has its verifier run in a container of its own and holds a Compose \
         file in its directory named tests, and an environment of more than one container cannot \
         be built"
    );
}

#[test]
fn a_task_that_lists_the_whole_container_for_its_verifier_is_said_with_what_it_asks_for() {
    assert_eq!(
        ConfigurationError::Needs { what: EVERYTHING }.to_string(),
        "asks for the whole container of the harness in the verifier container, which \
         cladenc has no way to give"
    );
}
