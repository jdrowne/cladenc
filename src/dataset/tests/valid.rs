use crate::dataset::configuration::{self, ConfigurationError};
use crate::dataset::{self, DatasetError, TaskConfiguration};
use crate::file_system::tests::double::Memory;

const NAMED: &str = "[task]\nname = \"a-team/fix-the-parser\"\n";
const CHECKED: &str = "[environment.healthcheck]\ncommand = \"test -e /ready\"\n";
const WITH_A_SERVER: &str = "[[environment.mcp_servers]]\nname = \"search\"\n";
const KEPT: &str = "[[artifacts]]\nsource = \"/app/out\"\n";
const NOT_ONE_OF_THE_NETWORK_MODES: &str =
    "invalid value: string \"private\", expected one of no-network, public, allowlist";

fn read(written: &str) -> Result<TaskConfiguration, ConfigurationError> {
    let directory = Memory::default();
    directory
        .write(directory.path().join("task.toml"), written)
        .unwrap();
    configuration::read_with(&directory, directory.path(), &|_| None)
}

fn not_valid(line: usize, reason: &str) -> Result<TaskConfiguration, ConfigurationError> {
    Err(ConfigurationError::NotValid {
        line,
        reason: reason.to_owned(),
    })
}

fn why_it_is_not_valid(written: &str) -> Option<String> {
    match read(written) {
        Err(ConfigurationError::NotValid { reason, .. }) => Some(reason),
        _ => None,
    }
}

fn lacks(key: &str) -> Option<String> {
    Some(format!("missing field `{key}`"))
}

fn left_out() -> Result<TaskConfiguration, ConfigurationError> {
    Ok(TaskConfiguration::default())
}

#[test]
fn a_task_toml_in_the_form_of_harbor_s_documentation_is_valid() {
    assert_eq!(
        read(
            "schema_version = \"1.4\"\nsource = \"a-collection\"\nmulti_step_reward_strategy = \
             \"final\"\n\n[task]\nname = \"a-team/fix-the-parser\"\nauthors = [{ name = \"A \
             Person\", email = \"a.person@team.example\" }]\ndescription = \"Fix the \
             parser.\"\nkeywords = [\"parsing\"]\n\n[metadata]\ncategory = \
             \"programming\"\n\n[solution]\nenv = { ANSWER = \"42\" }\n\n[environment]\nos = \
             \"linux\"\nnetwork_mode = \"public\"\nallow_internet = true\nstorage_mb = \
             10240\ngpus = 0\ngpu_types = [\"T4\"]\n"
        ),
        left_out()
    );
}

#[test]
fn a_setting_that_harbor_s_documentation_does_not_name_is_let_through() {
    assert_eq!(
        read("version = \"1.0\"\n\n[environment]\nstorage = '10G'\nflavour = 5\n"),
        left_out()
    );
}

#[test]
fn the_metadata_of_a_task_may_hold_anything() {
    assert_eq!(
        read("[metadata]\ntags = [\"parsing\", 3]\nlevel = { hard = true }\n"),
        left_out()
    );
}

#[test]
fn a_schema_version_that_is_not_text_is_refused() {
    assert_eq!(
        read("schema_version = 1.4\n"),
        not_valid(1, "invalid type: floating point `1.4`, expected a string")
    );
}

#[test]
fn a_source_that_is_not_text_is_refused() {
    assert_eq!(
        read("schema_version = \"1.4\"\nsource = 3\n"),
        not_valid(2, "invalid type: integer `3`, expected a string")
    );
}

#[test]
fn a_reward_strategy_that_is_neither_mean_nor_final_is_refused() {
    assert_eq!(
        read("multi_step_reward_strategy = \"best\"\n"),
        not_valid(
            1,
            "invalid value: string \"best\", expected one of mean, final"
        )
    );
}

#[test]
fn a_table_named_task_that_names_no_task_is_refused() {
    assert_eq!(
        why_it_is_not_valid("[task]\ndescription = \"Fix the parser.\"\n"),
        lacks("name")
    );
}

#[test]
fn a_table_named_task_that_names_its_task_is_valid() {
    assert_eq!(read(NAMED), left_out());
}

#[test]
fn a_task_that_is_not_a_table_is_refused() {
    assert_eq!(
        read("task = \"a-team/fix-the-parser\"\n"),
        not_valid(
            1,
            "invalid type: string \"a-team/fix-the-parser\", expected a table"
        )
    );
}

#[test]
fn an_author_without_a_name_is_refused() {
    assert_eq!(
        why_it_is_not_valid(&format!(
            "{NAMED}authors = [{{ email = \"a.person@team.example\" }}]\n"
        )),
        lacks("name")
    );
}

#[test]
fn authors_that_are_not_a_list_are_refused() {
    assert_eq!(
        read(&format!("{NAMED}authors = \"A Person\"\n")),
        not_valid(3, "invalid type: string \"A Person\", expected a list")
    );
}

#[test]
fn a_keyword_that_is_not_text_is_refused() {
    assert_eq!(
        read(&format!("{NAMED}keywords = [\"parsing\", 7]\n")),
        not_valid(3, "invalid type: integer `7`, expected a string")
    );
}

#[test]
fn metadata_that_is_not_a_table_is_refused() {
    assert_eq!(
        read("metadata = true\n"),
        not_valid(1, "invalid type: boolean `true`, expected a table")
    );
}

#[test]
fn a_network_mode_of_the_harness_that_harbor_does_not_name_is_refused() {
    assert_eq!(
        read("[agent]\nnetwork_mode = \"private\"\n"),
        not_valid(2, NOT_ONE_OF_THE_NETWORK_MODES)
    );
}

#[test]
fn a_network_mode_of_the_verifier_that_harbor_does_not_name_is_refused() {
    assert_eq!(
        read("[verifier]\nnetwork_mode = \"private\"\n"),
        not_valid(2, NOT_ONE_OF_THE_NETWORK_MODES)
    );
}

#[test]
fn a_network_mode_of_the_environment_that_harbor_does_not_name_is_refused() {
    assert_eq!(
        read("[environment]\ncpus = 1\nnetwork_mode = \"private\"\n"),
        not_valid(3, NOT_ONE_OF_THE_NETWORK_MODES)
    );
}

#[test]
fn a_network_mode_that_is_not_text_is_refused() {
    assert_eq!(
        read("[agent]\nnetwork_mode = 5\n"),
        not_valid(2, "invalid type: integer `5`, expected a string")
    );
}

#[test]
fn each_network_mode_that_harbor_names_is_valid() {
    assert_eq!(
        read(
            "[agent]\nnetwork_mode = \"no-network\"\n\n[verifier]\nnetwork_mode = \
             \"allowlist\"\nallowed_hosts = [\"pypi.org\"]\n\n[environment]\nnetwork_mode = \
             \"public\"\n"
        ),
        left_out()
    );
}

#[test]
fn allowed_hosts_that_are_not_a_list_are_refused() {
    assert_eq!(
        read("[agent]\nallowed_hosts = \"pypi.org\"\n"),
        not_valid(2, "invalid type: string \"pypi.org\", expected a list")
    );
}

#[test]
fn an_allowed_host_that_is_not_text_is_refused() {
    assert_eq!(
        read("[environment]\nallowed_hosts = [\"pypi.org\", 443]\n"),
        not_valid(2, "invalid type: integer `443`, expected a string")
    );
}

#[test]
fn a_variable_of_the_solution_that_is_not_text_is_refused() {
    assert_eq!(
        read("[solution]\nenv = { ANSWER = 42 }\n"),
        not_valid(2, "invalid type: integer `42`, expected a string")
    );
}

#[test]
fn an_os_that_is_neither_linux_nor_windows_is_refused() {
    assert_eq!(
        read("[environment]\nos = \"Windows\"\n"),
        not_valid(
            2,
            "invalid value: string \"Windows\", expected one of linux, windows"
        )
    );
}

#[test]
fn storage_that_is_not_a_whole_number_of_megabytes_is_refused() {
    assert_eq!(
        read("[environment]\nstorage_mb = \"10G\"\n"),
        not_valid(2, "invalid type: string \"10G\", expected i64")
    );
}

#[test]
fn gpu_types_that_are_not_a_list_are_refused() {
    assert_eq!(
        read("[environment]\ngpu_types = \"T4\"\n"),
        not_valid(2, "invalid type: string \"T4\", expected a list")
    );
}

#[test]
fn a_tpu_without_a_topology_is_refused() {
    assert_eq!(
        why_it_is_not_valid("[environment.tpu]\ntype = \"v6e\"\n"),
        lacks("topology")
    );
}

#[test]
fn a_tpu_without_a_type_is_refused() {
    assert_eq!(
        why_it_is_not_valid("[environment.tpu]\ntopology = \"2x4\"\n"),
        lacks("type")
    );
}

#[test]
fn whether_the_internet_is_allowed_is_refused_where_it_is_neither_true_nor_false() {
    assert_eq!(
        read("[environment]\nallow_internet = \"no\"\n"),
        not_valid(2, "invalid type: string \"no\", expected a boolean")
    );
}

#[test]
fn a_health_check_without_a_command_is_refused() {
    assert_eq!(
        why_it_is_not_valid("[environment.healthcheck]\nretries = 3\n"),
        lacks("command")
    );
}

#[test]
fn a_health_check_with_every_setting_harbor_names_is_valid() {
    assert!(
        read(&format!(
            "{CHECKED}interval_sec = 5\ntimeout_sec = 30.0\nstart_period_sec = \
             0\nstart_interval_sec = 0.5\nretries = 3\n"
        ))
        .is_ok()
    );
}

#[test]
fn the_retries_of_a_health_check_that_are_not_a_whole_number_are_refused() {
    assert_eq!(
        read(&format!("{CHECKED}retries = 1.5\n")),
        not_valid(3, "invalid type: floating point `1.5`, expected i64")
    );
}

#[test]
fn the_interval_of_a_health_check_that_is_not_a_number_is_refused() {
    assert_eq!(
        read(&format!("{CHECKED}interval_sec = \"5s\"\n")),
        not_valid(3, "invalid type: string \"5s\", expected f64")
    );
}

#[test]
fn a_health_check_that_is_not_a_table_is_refused() {
    assert_eq!(
        read("[environment]\nhealthcheck = \"test -e /ready\"\n"),
        not_valid(
            2,
            "invalid type: string \"test -e /ready\", expected a table"
        )
    );
}

#[test]
fn an_mcp_server_without_a_name_is_refused() {
    assert_eq!(
        why_it_is_not_valid("[[environment.mcp_servers]]\nurl = \"http://mcp:8000\"\n"),
        lacks("name")
    );
}

#[test]
fn an_mcp_server_with_a_transport_that_harbor_does_not_name_is_refused() {
    assert_eq!(
        read(&format!("{WITH_A_SERVER}transport = \"pipe\"\n")),
        not_valid(
            3,
            "invalid value: string \"pipe\", expected one of stdio, sse, streamable-http, http"
        )
    );
}

#[test]
fn the_headers_of_an_mcp_server_that_are_not_text_by_name_are_refused() {
    assert_eq!(
        read(&format!("{WITH_A_SERVER}headers = [\"Accept\"]\n")),
        not_valid(3, "invalid type: sequence, expected a map")
    );
}

#[test]
fn mcp_servers_that_are_not_a_list_are_refused() {
    assert_eq!(
        read("[environment]\nmcp_servers = 5\n"),
        not_valid(2, "invalid type: integer `5`, expected a list")
    );
}

#[test]
fn an_artifact_may_be_named_by_its_path_alone() {
    assert!(read("artifacts = [\"/app/out\"]\n").is_ok());
}

#[test]
fn an_artifact_may_be_a_table_with_every_setting_harbor_names() {
    assert!(
        read(&format!(
            "{KEPT}destination = \"out\"\nexclude = [\"*.tmp\"]\nservice = \"main\"\n"
        ))
        .is_ok()
    );
}

#[test]
fn the_service_of_an_artifact_is_refused_where_it_is_not_text() {
    assert_eq!(
        read(&format!("{KEPT}service = 5\n")),
        not_valid(3, "invalid type: integer `5`, expected a string")
    );
}

#[test]
fn an_artifact_without_a_source_is_refused() {
    assert_eq!(
        why_it_is_not_valid("[[artifacts]]\ndestination = \"out\"\n"),
        lacks("source")
    );
}

#[test]
fn an_artifact_that_is_neither_a_path_nor_a_table_is_refused() {
    assert_eq!(
        read("artifacts = [\"/app/out\", 5]\n"),
        not_valid(
            1,
            "invalid type: integer `5`, expected a path, or a table that names one as source"
        )
    );
}

#[test]
fn artifacts_that_are_not_a_list_are_refused() {
    assert_eq!(
        read("artifacts = \"/app/out\"\n"),
        not_valid(1, "invalid type: string \"/app/out\", expected a list")
    );
}

#[test]
fn what_an_artifact_leaves_out_is_refused_where_it_is_not_a_list() {
    assert_eq!(
        read(&format!("{KEPT}exclude = \"*.tmp\"\n")),
        not_valid(3, "invalid type: string \"*.tmp\", expected a list")
    );
}

#[test]
fn an_environment_mode_of_the_verifier_that_harbor_does_not_name_is_refused() {
    assert_eq!(
        read("[verifier]\nenvironment_mode = \"apart\"\n"),
        not_valid(
            2,
            "invalid value: string \"apart\", expected one of shared, separate"
        )
    );
}

#[test]
fn a_setting_of_the_verifier_environment_is_checked_as_one_of_the_environment_is() {
    assert_eq!(
        read("[verifier.environment]\ncpus = \"two\"\n"),
        not_valid(2, "invalid type: string \"two\", expected f64")
    );
}

#[test]
fn a_verifier_environment_that_is_not_a_table_is_refused() {
    assert_eq!(
        read("[verifier]\nenvironment = \"debian:stable-slim\"\n"),
        not_valid(
            2,
            "invalid type: string \"debian:stable-slim\", expected a table"
        )
    );
}

#[test]
fn a_verifier_said_to_share_the_environment_and_given_its_own_is_refused() {
    assert_eq!(
        read(
            "[verifier]\nenvironment_mode = \"shared\"\n\n[verifier.environment]\ndocker_image \
             = \"debian:stable-slim\"\n"
        ),
        Err(ConfigurationError::SharedAndSeparate)
    );
}

#[test]
fn a_verifier_said_to_have_a_separate_environment_and_given_one_is_valid() {
    assert!(
        read(
            "[verifier]\nenvironment_mode = \"separate\"\n\n[verifier.environment]\ndocker_image \
             = \"debian:stable-slim\"\n"
        )
        .is_ok()
    );
}

#[test]
fn a_verifier_said_to_share_the_environment_and_given_none_is_valid() {
    assert_eq!(
        read("[verifier]\nenvironment_mode = \"shared\"\n"),
        left_out()
    );
}

#[test]
fn a_verifier_given_an_environment_and_no_environment_mode_is_valid() {
    assert!(read("[verifier.environment]\ndocker_image = \"debian:stable-slim\"\n").is_ok());
}

#[test]
fn a_task_whose_task_toml_is_not_valid_in_what_cladenc_does_not_obey_is_refused_with_its_id() {
    let directory = Memory::default();
    let dataset = test_inputs::dataset_in(&directory, directory.path(), &["t1", "t2"]);
    directory
        .write(
            dataset.join("t2/task.toml"),
            "[agent]\nnetwork_mode = \"private\"\n",
        )
        .unwrap();
    assert_eq!(
        dataset::tasks(&directory, &dataset),
        Err(DatasetError::Configuration {
            task: "t2".to_owned(),
            error: ConfigurationError::NotValid {
                line: 2,
                reason: NOT_ONE_OF_THE_NETWORK_MODES.to_owned(),
            },
        })
    );
}
