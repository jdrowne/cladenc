use std::time::Duration;

use crate::benchmark::{EnvironmentConfiguration, HealthCheck, TaskConfiguration};
use crate::sandboxes::Environment;
use crate::sandboxes::sandbox::{self, Bringing};

const IMAGE_OF_THE_VERIFIER: &str = "registry.example/verifiers/parser:3";

fn checked() -> HealthCheck {
    HealthCheck {
        command: "test -e /ready".to_owned(),
        interval: Duration::from_secs(5),
        time_limit: Duration::from_secs(30),
        start_period: Duration::from_secs(10),
        start_interval: Duration::from_secs(5),
        retries: 2,
    }
}

fn with_a_verifier_of_its_own() -> TaskConfiguration {
    TaskConfiguration {
        working_directory: Some("/app".to_owned()),
        variables: vec![("MODE".to_owned(), "of the task".to_owned())],
        cpus: Some(4.0),
        memory_megabytes: Some(4096),
        health_check: Some(checked()),
        harness_time_limit: Some(Duration::from_secs(1800)),
        verifier_time_limit: Duration::from_secs(120),
        verifier_environment: Some(EnvironmentConfiguration {
            working_directory: Some("/verifying".to_owned()),
            variables: vec![("MODE".to_owned(), "of the verifier".to_owned())],
            cpus: Some(0.5),
            memory_megabytes: Some(512),
            ..EnvironmentConfiguration::default()
        }),
        ..TaskConfiguration::default()
    }
}

#[test]
fn the_sandbox_of_a_task_stays_for_its_harness_its_verifier_its_health_check_and_ten_minutes() {
    let environment = Environment {
        configuration: with_a_verifier_of_its_own(),
        ..Environment::default()
    };
    assert_eq!(
        Bringing::of_the_task(&environment).stays,
        Duration::from_secs(1800 + 120 + 110 + 600)
    );
}

#[test]
fn a_verifier_sandbox_stays_for_the_time_limit_of_the_verifier_and_ten_minutes_more() {
    let configuration = with_a_verifier_of_its_own();
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).stays,
        Duration::from_secs(120 + 600)
    );
}

#[test]
fn a_verifier_sandbox_stays_for_the_longest_its_own_health_check_can_take_as_well() {
    let configuration = TaskConfiguration {
        verifier_time_limit: Duration::from_secs(120),
        verifier_environment: Some(EnvironmentConfiguration {
            health_check: Some(checked()),
            ..EnvironmentConfiguration::default()
        }),
        ..TaskConfiguration::default()
    };
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).stays,
        Duration::from_secs(120 + 110 + 600)
    );
}

#[test]
fn a_verifier_sandbox_is_of_the_verifier_image() {
    let configuration = with_a_verifier_of_its_own();
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).image,
        IMAGE_OF_THE_VERIFIER
    );
}

#[test]
fn a_verifier_sandbox_has_the_working_directory_of_the_verifier_environment() {
    let configuration = with_a_verifier_of_its_own();
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).working_directory,
        Some("/verifying")
    );
}

#[test]
fn a_verifier_sandbox_has_the_cpus_of_the_verifier_environment() {
    let configuration = with_a_verifier_of_its_own();
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).cpus,
        Some(0.5)
    );
}

#[test]
fn a_verifier_sandbox_has_the_memory_of_the_verifier_environment() {
    let configuration = with_a_verifier_of_its_own();
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).memory_megabytes,
        Some(512)
    );
}

#[test]
fn a_verifier_sandbox_is_given_the_variables_of_the_verifier_environment() {
    let configuration = with_a_verifier_of_its_own();
    assert_eq!(
        Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration).variables,
        [("MODE".to_owned(), "of the verifier".to_owned())]
    );
}

#[test]
fn a_verifier_sandbox_whose_environment_sets_nothing_is_given_nothing_of_the_task() {
    let configuration = TaskConfiguration {
        verifier_environment: Some(EnvironmentConfiguration::default()),
        ..with_a_verifier_of_its_own()
    };
    let bringing = Bringing::of_the_verifier(IMAGE_OF_THE_VERIFIER, &configuration);
    assert_eq!(
        (
            bringing.working_directory,
            bringing.cpus,
            bringing.memory_megabytes,
            bringing.variables.len()
        ),
        (None, None, None, 0)
    );
}

#[test]
fn the_sandbox_of_a_task_is_given_the_settings_of_the_task_and_not_those_of_its_verifier() {
    let environment = Environment {
        image: test_inputs::IMAGE.to_owned(),
        configuration: with_a_verifier_of_its_own(),
        ..Environment::default()
    };
    let bringing = Bringing::of_the_task(&environment);
    assert_eq!(
        (
            bringing.image,
            bringing.working_directory,
            bringing.cpus,
            bringing.memory_megabytes,
            bringing.variables
        ),
        (
            test_inputs::IMAGE,
            Some("/app"),
            Some(4.0),
            Some(4096),
            &[("MODE".to_owned(), "of the task".to_owned())][..]
        )
    );
}

#[test]
fn the_variables_of_a_sandbox_are_written_for_docker_one_to_a_line_as_a_name_and_a_value() {
    assert_eq!(
        sandbox::file_of(&[
            ("MODE".to_owned(), "strict".to_owned()),
            ("NOTE".to_owned(), "a = b".to_owned())
        ]),
        "MODE=strict\nNOTE=a = b\n"
    );
}

#[test]
fn a_sandbox_with_no_variable_is_given_none() {
    assert_eq!(sandbox::file_of(&[]), "");
}
