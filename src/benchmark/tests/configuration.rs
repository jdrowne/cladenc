use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::time::Duration;

use tempfile::TempDir;

use crate::benchmark::configuration::{self, CONFIGURATION, ConfigurationError};
use crate::benchmark::{self, BenchmarkError, TaskConfiguration};

fn task_that_says(written: &str) -> TempDir {
    let directory = test_inputs::temporary_directory();
    fs::write(directory.path().join("task.toml"), written).unwrap();
    directory
}

fn set_where_cladenc_runs(variable: &str) -> Option<String> {
    match variable {
        "TOKEN" => Some("a-made-up-token".to_owned()),
        "REGION" => Some("north".to_owned()),
        "LINES" => Some("one\ntwo".to_owned()),
        _ => None,
    }
}

fn read(written: &str) -> Result<TaskConfiguration, ConfigurationError> {
    configuration::read_with(task_that_says(written).path(), &set_where_cladenc_runs)
}

fn says(written: &str) -> TaskConfiguration {
    read(written).unwrap()
}

fn variables(given: &[(&str, &str)]) -> Vec<(String, String)> {
    given
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

fn not_valid(line: usize, reason: &str) -> Result<TaskConfiguration, ConfigurationError> {
    Err(ConfigurationError::NotValid {
        line,
        reason: reason.to_owned(),
    })
}

fn refused_for_its_task_toml(written: &str) -> Result<Vec<String>, BenchmarkError> {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::write(benchmark.join("t2/task.toml"), written).unwrap();
    benchmark::tasks(&benchmark)
}

#[test]
fn the_configuration_of_a_task_is_in_its_file_named_task_toml() {
    assert_eq!(CONFIGURATION, "task.toml");
}

#[test]
fn a_task_with_no_task_toml_is_refused() {
    let directory = test_inputs::temporary_directory();
    assert_eq!(
        configuration::read_with(directory.path(), &set_where_cladenc_runs),
        Err(ConfigurationError::Missing)
    );
}

#[test]
fn a_task_whose_task_toml_is_a_symbolic_link_to_nothing_is_refused_as_one_with_none() {
    let directory = test_inputs::temporary_directory();
    symlink("nowhere", directory.path().join("task.toml")).unwrap();
    assert_eq!(
        configuration::read_with(directory.path(), &set_where_cladenc_runs),
        Err(ConfigurationError::Missing)
    );
}

#[test]
fn a_task_toml_that_holds_nothing_leaves_every_setting_out() {
    assert_eq!(says(""), TaskConfiguration::default());
}

#[test]
fn a_task_toml_that_holds_only_what_cladenc_does_not_read_leaves_every_setting_out() {
    assert_eq!(
        says(
            "schema_version = \"1.4\"\n[metadata]\ndifficulty = \"hard\"\n[agent]\nnetwork_mode \
             = \"public\"\n[environment]\ngpus = 0\n[solution]\nenv = { ANSWER = \"42\" }\n"
        ),
        TaskConfiguration::default()
    );
}

fn not_a_time_limit(key: &'static str) -> Result<TaskConfiguration, ConfigurationError> {
    Err(ConfigurationError::NotATimeLimit { key })
}

#[test]
fn a_task_that_gives_the_harness_no_time_limit_sets_none() {
    assert_eq!(
        says("[agent]\nuser = \"worker\"\n").harness_time_limit,
        None
    );
}

#[test]
fn the_time_limit_of_the_harness_is_read_from_timeout_sec_under_agent() {
    assert_eq!(
        says("[agent]\ntimeout_sec = 1800.0\n").harness_time_limit,
        Some(Duration::from_secs(1800))
    );
}

#[test]
fn a_time_limit_may_be_a_whole_number_of_seconds() {
    assert_eq!(
        says("[agent]\ntimeout_sec = 90\n").harness_time_limit,
        Some(Duration::from_secs(90))
    );
}

#[test]
fn a_time_limit_may_be_a_part_of_a_second() {
    assert_eq!(
        says("[agent]\ntimeout_sec = 0.5\n").harness_time_limit,
        Some(Duration::from_millis(500))
    );
}

#[test]
fn the_time_limit_of_the_grader_is_read_from_timeout_sec_under_verifier() {
    assert_eq!(
        says("[verifier]\ntimeout_sec = 120\n").grader_time_limit,
        Duration::from_secs(120)
    );
}

#[test]
fn a_task_that_gives_the_grader_no_time_limit_gives_it_600_seconds() {
    assert_eq!(
        says("[agent]\ntimeout_sec = 90\n").grader_time_limit,
        Duration::from_secs(600)
    );
}

#[test]
fn the_time_limit_of_the_build_of_the_environment_is_read_from_build_timeout_sec() {
    assert_eq!(
        says("[environment]\nbuild_timeout_sec = 1800.0\n").build_time_limit,
        Duration::from_secs(1800)
    );
}

#[test]
fn a_task_that_gives_the_build_of_its_environment_no_time_limit_gives_it_600_seconds() {
    assert_eq!(
        says("[environment]\nworkdir = \"/app\"\n").build_time_limit,
        Duration::from_secs(600)
    );
}

#[test]
fn the_settings_of_a_task_that_sets_none_give_the_grader_and_the_build_600_seconds_each() {
    let configuration = TaskConfiguration::default();
    assert_eq!(
        (
            configuration.grader_time_limit,
            configuration.build_time_limit
        ),
        (Duration::from_secs(600), Duration::from_secs(600))
    );
}

#[test]
fn a_time_limit_of_0_seconds_is_refused() {
    assert_eq!(
        read("[agent]\ntimeout_sec = 0\n"),
        not_a_time_limit("agent.timeout_sec")
    );
}

#[test]
fn a_time_limit_below_0_is_refused() {
    assert_eq!(
        read("[verifier]\ntimeout_sec = -5.0\n"),
        not_a_time_limit("verifier.timeout_sec")
    );
}

#[test]
fn a_time_limit_that_is_not_a_number_is_refused() {
    assert_eq!(
        read("[environment]\nbuild_timeout_sec = nan\n"),
        not_a_time_limit("environment.build_timeout_sec")
    );
}

#[test]
fn a_time_limit_without_end_is_refused() {
    assert_eq!(
        read("[agent]\ntimeout_sec = inf\n"),
        not_a_time_limit("agent.timeout_sec")
    );
}

#[test]
fn a_time_limit_written_as_text_is_refused() {
    assert_eq!(
        read("[agent]\ntimeout_sec = \"long\"\n"),
        not_valid(2, "invalid type: string \"long\", expected f64")
    );
}

#[test]
fn a_task_that_asks_for_no_cpus_and_no_memory_is_given_no_limit_of_either() {
    let configuration = says("[environment]\nworkdir = \"/app\"\n");
    assert_eq!(
        (configuration.cpus, configuration.memory_megabytes),
        (None, None)
    );
}

#[test]
fn the_cpus_of_a_task_are_read_from_cpus_under_environment() {
    assert_eq!(says("[environment]\ncpus = 2\n").cpus, Some(2.0));
}

#[test]
fn the_cpus_of_a_task_may_be_a_part_of_one() {
    assert_eq!(says("[environment]\ncpus = 0.5\n").cpus, Some(0.5));
}

#[test]
fn the_memory_of_a_task_is_read_in_megabytes_from_memory_mb_under_environment() {
    assert_eq!(
        says("[environment]\nmemory_mb = 4096\n").memory_megabytes,
        Some(4096)
    );
}

#[test]
fn the_memory_of_a_task_is_read_from_the_older_spelling_in_gigabytes() {
    assert_eq!(
        says("[environment]\nmemory = '4G'\n").memory_megabytes,
        Some(4096)
    );
}

#[test]
fn the_older_spelling_of_memory_may_be_in_megabytes() {
    assert_eq!(
        says("[environment]\nmemory = '512M'\n").memory_megabytes,
        Some(512)
    );
}

#[test]
fn the_older_spelling_of_memory_may_be_in_kilobytes() {
    assert_eq!(
        says("[environment]\nmemory = '2048K'\n").memory_megabytes,
        Some(2)
    );
}

#[test]
fn the_older_spelling_of_memory_may_be_in_small_letters_with_spaces_around_it() {
    assert_eq!(
        says("[environment]\nmemory = ' 1.5g '\n").memory_megabytes,
        Some(1536)
    );
}

#[test]
fn the_two_spellings_of_memory_may_both_be_given_where_they_agree() {
    assert_eq!(
        says("[environment]\nmemory = '4G'\nmemory_mb = 4096\n").memory_megabytes,
        Some(4096)
    );
}

#[test]
fn an_older_spelling_of_memory_that_is_not_text_is_left_out() {
    assert_eq!(says("[environment]\nmemory = 4\n").memory_megabytes, None);
}

#[test]
fn two_spellings_of_memory_that_differ_are_refused() {
    assert_eq!(
        read("[environment]\nmemory = '4G'\nmemory_mb = 2048\n"),
        Err(ConfigurationError::TwoSizes {
            key: "environment.memory"
        })
    );
}

#[test]
fn an_older_spelling_of_memory_with_no_unit_is_refused() {
    assert_eq!(
        read("[environment]\nmemory = '4096'\n"),
        Err(ConfigurationError::NotASize {
            key: "environment.memory"
        })
    );
}

#[test]
fn an_older_spelling_of_memory_that_is_no_number_is_refused() {
    assert_eq!(
        read("[environment]\nmemory = 'manyG'\n"),
        Err(ConfigurationError::NotASize {
            key: "environment.memory"
        })
    );
}

#[test]
fn an_older_spelling_of_memory_that_comes_to_no_megabyte_is_refused() {
    assert_eq!(
        read("[environment]\nmemory = '100K'\n"),
        Err(ConfigurationError::NotASize {
            key: "environment.memory"
        })
    );
}

#[test]
fn cpus_that_are_not_above_0_are_refused() {
    assert_eq!(
        read("[environment]\ncpus = 0\n"),
        Err(ConfigurationError::NotAbove0 {
            key: "environment.cpus"
        })
    );
}

#[test]
fn cpus_without_end_are_refused() {
    assert_eq!(
        read("[environment]\ncpus = inf\n"),
        Err(ConfigurationError::NotAbove0 {
            key: "environment.cpus"
        })
    );
}

#[test]
fn memory_that_is_not_above_0_megabytes_is_refused() {
    assert_eq!(
        read("[environment]\nmemory_mb = -1\n"),
        Err(ConfigurationError::NotAbove0 {
            key: "environment.memory_mb"
        })
    );
}

fn needs(what: &'static str) -> Result<TaskConfiguration, ConfigurationError> {
    Err(ConfigurationError::Needs { what })
}

#[test]
fn a_task_of_several_steps_is_refused() {
    assert_eq!(
        read("[[steps]]\nname = \"first\"\n\n[[steps]]\nname = \"second\"\n"),
        needs("several steps")
    );
}

#[test]
fn a_task_that_asks_for_a_gpu_is_refused() {
    assert_eq!(read("[environment]\ngpus = 1\n"), needs("a GPU"));
}

#[test]
fn a_task_that_asks_for_0_gpus_is_not_refused() {
    assert!(read("[environment]\ngpus = 0\n").is_ok());
}

#[test]
fn a_task_that_asks_for_a_tpu_is_refused() {
    assert_eq!(
        read("[environment.tpu]\ntype = \"v6e\"\ntopology = \"2x4\"\n"),
        needs("a TPU")
    );
}

#[test]
fn a_task_for_windows_is_refused() {
    assert_eq!(read("[environment]\nos = \"windows\"\n"), needs("Windows"));
}

#[test]
fn a_task_for_linux_is_not_refused() {
    assert!(read("[environment]\nos = \"linux\"\n").is_ok());
}

#[test]
fn a_task_with_mcp_servers_is_refused() {
    assert_eq!(
        read("[[environment.mcp_servers]]\nname = \"search\"\nurl = \"http://mcp:8000\"\n"),
        needs("MCP servers")
    );
}

#[test]
fn a_task_with_an_empty_list_of_mcp_servers_is_not_refused() {
    assert!(read("[environment]\nmcp_servers = []\n").is_ok());
}

#[test]
fn a_task_with_a_skills_directory_is_refused() {
    assert_eq!(
        read("[environment]\nskills_dir = \"/skills\"\n"),
        needs("a skills directory")
    );
}

#[test]
fn a_task_with_an_empty_list_of_steps_is_not_refused() {
    assert!(read("steps = []\n").is_ok());
}

#[test]
fn the_image_of_the_environment_is_read_from_docker_image_under_environment() {
    assert_eq!(
        says("[environment]\ndocker_image = \"registry.example/tasks/parser:3\"\n").docker_image,
        Some("registry.example/tasks/parser:3".to_owned())
    );
}

#[test]
fn the_working_directory_is_read_from_workdir_under_environment() {
    assert_eq!(
        says("[environment]\nworkdir = \"/srv/work\"\n").working_directory,
        Some("/srv/work".to_owned())
    );
}

#[test]
fn the_variables_of_the_environment_are_read_from_env_under_environment_in_order_of_name() {
    assert_eq!(
        says("[environment.env]\nMODE = \"strict\"\nDEPTH = \"3\"\n").variables,
        variables(&[("DEPTH", "3"), ("MODE", "strict")])
    );
}

#[test]
fn the_variables_of_the_environment_may_be_written_as_an_inline_table() {
    assert_eq!(
        says("[environment]\nenv = { MODE = \"strict\" }\n").variables,
        variables(&[("MODE", "strict")])
    );
}

#[test]
fn the_user_of_the_harness_is_read_from_user_under_agent() {
    assert_eq!(
        says("[agent]\nuser = \"worker\"\n").harness_user,
        Some("worker".to_owned())
    );
}

#[test]
fn a_user_given_as_a_number_is_read_as_that_number() {
    assert_eq!(
        says("[agent]\nuser = 1000\n").harness_user,
        Some("1000".to_owned())
    );
}

#[test]
fn the_user_of_the_grader_is_read_from_user_under_verifier() {
    assert_eq!(
        says("[verifier]\nuser = \"judge\"\n").grader_user,
        Some("judge".to_owned())
    );
}

#[test]
fn the_user_of_the_harness_is_not_the_user_of_the_grader() {
    assert_eq!(says("[agent]\nuser = \"worker\"\n").grader_user, None);
}

#[test]
fn the_variables_of_the_grader_are_read_from_env_under_verifier() {
    assert_eq!(
        says("[verifier.env]\nJUDGE = \"strict\"\n").grader_variables,
        variables(&[("JUDGE", "strict")])
    );
}

#[test]
fn the_variables_of_the_grader_are_not_those_of_the_environment() {
    assert_eq!(
        says("[verifier.env]\nJUDGE = \"strict\"\n").variables,
        variables(&[])
    );
}

#[test]
fn an_image_named_with_nothing_is_left_out() {
    assert_eq!(
        says("[environment]\ndocker_image = \"\"\n").docker_image,
        None
    );
}

#[test]
fn a_working_directory_named_with_nothing_is_left_out() {
    assert_eq!(
        says("[environment]\nworkdir = \"\"\n").working_directory,
        None
    );
}

#[test]
fn a_user_named_with_nothing_is_left_out() {
    assert_eq!(says("[verifier]\nuser = \"\"\n").grader_user, None);
}

#[test]
fn a_variable_written_with_braces_is_given_the_value_it_has_where_cladenc_runs() {
    assert_eq!(
        says("[environment.env]\nAPI_TOKEN = \"${TOKEN}\"\n").variables,
        variables(&[("API_TOKEN", "a-made-up-token")])
    );
}

#[test]
fn a_variable_of_the_grader_is_given_the_value_it_has_where_cladenc_runs() {
    assert_eq!(
        says("[verifier.env]\nAPI_TOKEN = \"${TOKEN}\"\n").grader_variables,
        variables(&[("API_TOKEN", "a-made-up-token")])
    );
}

#[test]
fn a_variable_that_is_set_is_not_given_its_default() {
    assert_eq!(
        says("[environment.env]\nPLACE = \"${REGION:-south}\"\n").variables,
        variables(&[("PLACE", "north")])
    );
}

#[test]
fn a_variable_that_is_not_set_is_given_its_default() {
    assert_eq!(
        says("[environment.env]\nPLACE = \"${ELSEWHERE:-south}\"\n").variables,
        variables(&[("PLACE", "south")])
    );
}

#[test]
fn a_default_may_be_nothing() {
    assert_eq!(
        says("[environment.env]\nPLACE = \"${ELSEWHERE:-}\"\n").variables,
        variables(&[("PLACE", "")])
    );
}

#[test]
fn a_value_is_replaced_inside_the_text_around_it() {
    assert_eq!(
        says("[environment.env]\nHEADER = \"Bearer ${TOKEN} of ${REGION}!\"\n").variables,
        variables(&[("HEADER", "Bearer a-made-up-token of north!")])
    );
}

#[test]
fn a_dollar_sign_without_braces_is_left_as_it_is_written() {
    assert_eq!(
        says("[environment.env]\nPRICE = \"$TOKEN and $5\"\n").variables,
        variables(&[("PRICE", "$TOKEN and $5")])
    );
}

#[test]
fn braces_that_are_never_closed_are_left_as_they_are_written() {
    assert_eq!(
        says("[environment.env]\nOPEN = \"${REGION} and ${TOKEN\"\n").variables,
        variables(&[("OPEN", "north and ${TOKEN")])
    );
}

#[test]
fn a_variable_that_is_not_set_and_has_no_default_is_refused() {
    assert_eq!(
        read("[environment.env]\nAPI_TOKEN = \"${ELSEWHERE}\"\n"),
        Err(ConfigurationError::NotSet {
            table: "environment",
            name: "API_TOKEN".to_owned(),
            variable: "ELSEWHERE".to_owned(),
        })
    );
}

#[test]
fn a_variable_of_the_grader_that_is_not_set_is_refused_under_verifier() {
    assert_eq!(
        read("[verifier.env]\nAPI_TOKEN = \"${ELSEWHERE}\"\n"),
        Err(ConfigurationError::NotSet {
            table: "verifier",
            name: "API_TOKEN".to_owned(),
            variable: "ELSEWHERE".to_owned(),
        })
    );
}

#[test]
fn a_variable_whose_name_holds_a_space_is_refused() {
    assert_eq!(
        read("[environment.env]\n\"API TOKEN\" = \"x\"\n"),
        Err(ConfigurationError::NotAName {
            table: "environment",
            name: "API TOKEN".to_owned(),
        })
    );
}

#[test]
fn a_variable_whose_name_holds_an_equals_sign_is_refused() {
    assert_eq!(
        read("[verifier.env]\n\"A=B\" = \"x\"\n"),
        Err(ConfigurationError::NotAName {
            table: "verifier",
            name: "A=B".to_owned(),
        })
    );
}

#[test]
fn a_variable_with_no_name_is_refused() {
    assert_eq!(
        read("[environment.env]\n\"\" = \"x\"\n"),
        Err(ConfigurationError::NotAName {
            table: "environment",
            name: String::new(),
        })
    );
}

#[test]
fn a_variable_written_over_two_lines_is_refused() {
    assert_eq!(
        read("[environment.env]\nNOTE = \"one\\ntwo\"\n"),
        Err(ConfigurationError::NotOneLine {
            table: "environment",
            name: "NOTE".to_owned(),
        })
    );
}

#[test]
fn a_variable_whose_value_where_cladenc_runs_is_of_two_lines_is_refused() {
    assert_eq!(
        read("[verifier.env]\nNOTE = \"${LINES}\"\n"),
        Err(ConfigurationError::NotOneLine {
            table: "verifier",
            name: "NOTE".to_owned(),
        })
    );
}

#[test]
fn a_variable_whose_value_holds_a_carriage_return_is_refused() {
    assert_eq!(
        read("[environment.env]\nNOTE = \"one\\rtwo\"\n"),
        Err(ConfigurationError::NotOneLine {
            table: "environment",
            name: "NOTE".to_owned(),
        })
    );
}

#[test]
fn a_variable_whose_value_holds_a_nul_is_refused() {
    assert_eq!(
        read("[environment.env]\nNOTE = \"one\\u0000two\"\n"),
        Err(ConfigurationError::NotOneLine {
            table: "environment",
            name: "NOTE".to_owned(),
        })
    );
}

#[test]
fn a_task_toml_that_is_not_toml_is_refused_with_the_line_it_fails_at() {
    assert_eq!(
        read("[agent]\nuser = \"worker\"\n[environment\n"),
        not_valid(3, "unclosed table, expected `]`")
    );
}

#[test]
fn a_working_directory_that_is_not_text_is_refused() {
    assert_eq!(
        read("[environment]\nworkdir = 3\n"),
        not_valid(2, "invalid type: integer `3`, expected a string")
    );
}

#[test]
fn an_image_that_is_not_text_is_refused() {
    assert_eq!(
        read("[environment]\ndocker_image = true\n"),
        not_valid(2, "invalid type: boolean `true`, expected a string")
    );
}

#[test]
fn a_variable_that_is_a_number_is_refused() {
    assert_eq!(
        read("[environment]\n\n[environment.env]\nDEPTH = 3\n"),
        not_valid(4, "invalid type: integer `3`, expected a string")
    );
}

#[test]
fn a_user_that_is_neither_a_name_nor_a_whole_number_is_refused() {
    assert_eq!(
        read("[agent]\nuser = 1.5\n"),
        not_valid(
            2,
            "invalid type: expected the name of a user, or the number of one, 0 or more"
        )
    );
}

#[test]
fn a_user_given_as_a_number_below_0_is_refused() {
    assert_eq!(
        read("[verifier]\nuser = -1\n"),
        not_valid(
            2,
            "invalid type: expected the name of a user, or the number of one, 0 or more"
        )
    );
}

#[test]
fn a_task_toml_that_is_a_directory_is_refused() {
    let directory = test_inputs::temporary_directory();
    fs::create_dir(directory.path().join("task.toml")).unwrap();
    assert!(matches!(
        configuration::read_with(directory.path(), &set_where_cladenc_runs),
        Err(ConfigurationError::Unreadable { .. })
    ));
}

#[test]
fn a_task_toml_that_is_not_text_is_refused() {
    let directory = test_inputs::temporary_directory();
    fs::write(directory.path().join("task.toml"), [0xff, 0xfe]).unwrap();
    assert!(matches!(
        configuration::read_with(directory.path(), &set_where_cladenc_runs),
        Err(ConfigurationError::Unreadable { .. })
    ));
}

#[test]
fn a_task_reads_the_variables_that_are_set_where_cladenc_runs() {
    let task = task_that_says("[environment.env]\nSEARCHED = \"${PATH}\"\n");
    assert_eq!(
        TaskConfiguration::read(task.path()).map(|configuration| configuration.variables),
        Ok(variables(&[("SEARCHED", &env::var("PATH").unwrap())]))
    );
}

#[test]
fn a_task_whose_task_toml_cannot_be_obeyed_is_refused_with_its_id_and_why() {
    assert_eq!(
        refused_for_its_task_toml("[environment]\nworkdir = 3\n"),
        Err(BenchmarkError::Configuration {
            task: "t2".to_owned(),
            error: ConfigurationError::NotValid {
                line: 2,
                reason: "invalid type: integer `3`, expected a string".to_owned(),
            },
        })
    );
}

#[test]
fn a_task_that_names_a_variable_that_is_not_set_where_cladenc_runs_is_refused() {
    assert_eq!(
        refused_for_its_task_toml(
            "[verifier.env]\nKEY = \"${CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET}\"\n"
        ),
        Err(BenchmarkError::Configuration {
            task: "t2".to_owned(),
            error: ConfigurationError::NotSet {
                table: "verifier",
                name: "KEY".to_owned(),
                variable: "CLADENC_TEST_A_VARIABLE_THAT_IS_NEVER_SET".to_owned(),
            },
        })
    );
}

#[test]
fn a_task_that_holds_no_task_toml_is_refused_with_its_id() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_file(benchmark.join("t2/task.toml")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::NoConfiguration {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_with_no_task_toml_and_no_environment_is_refused_for_its_task_toml() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_file(benchmark.join("t2/task.toml")).unwrap();
    fs::remove_dir_all(benchmark.join("t2/environment")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::NoConfiguration {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_with_no_grader_and_no_task_toml_is_refused_for_its_grader() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_file(benchmark.join("t2/task.toml")).unwrap();
    fs::remove_dir_all(benchmark.join("t2/tests")).unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::NoGrader {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_whose_task_toml_names_an_image_needs_no_directory_named_environment() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_dir_all(benchmark.join("t2/environment")).unwrap();
    fs::write(
        benchmark.join("t2/task.toml"),
        "[environment]\ndocker_image = \"debian:stable-slim\"\n",
    )
    .unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Ok(vec!["t1".to_owned(), "t2".to_owned()])
    );
}

#[test]
fn a_task_whose_task_toml_names_no_image_still_needs_a_dockerfile() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_dir_all(benchmark.join("t2/environment")).unwrap();
    fs::write(
        benchmark.join("t2/task.toml"),
        "[environment]\nworkdir = \"/app\"\n",
    )
    .unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::NoEnvironment {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_whose_task_toml_names_an_image_is_still_refused_for_a_compose_file() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::write(
        benchmark.join("t2/environment/compose.yaml"),
        "services: {}\n",
    )
    .unwrap();
    fs::write(
        benchmark.join("t2/task.toml"),
        "[environment]\ndocker_image = \"debian:stable-slim\"\n",
    )
    .unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::ManyContainers {
            task: "t2".to_owned()
        })
    );
}

#[test]
fn a_task_with_no_grader_and_a_task_toml_that_cannot_be_obeyed_is_refused_for_its_grader() {
    let directory = test_inputs::temporary_directory();
    let benchmark = test_inputs::benchmark(directory.path(), &["t1", "t2"]);
    fs::remove_dir_all(benchmark.join("t2/tests")).unwrap();
    fs::write(benchmark.join("t2/task.toml"), "[environment\n").unwrap();
    assert_eq!(
        benchmark::tasks(&benchmark),
        Err(BenchmarkError::NoGrader {
            task: "t2".to_owned()
        })
    );
}
