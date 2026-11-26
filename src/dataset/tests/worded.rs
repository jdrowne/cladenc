use std::path::PathBuf;

use crate::dataset::DatasetError;
use crate::dataset::configuration::ConfigurationError;

fn task() -> String {
    "fix-the-parser".to_owned()
}

#[test]
fn what_is_not_a_directory_is_said_to_be_none() {
    assert_eq!(
        DatasetError::NotADirectory.to_string(),
        "what was given is not a directory that can be read"
    );
}

#[test]
fn a_dataset_with_no_directory_is_said_to_hold_no_task() {
    assert_eq!(
        DatasetError::NoTask.to_string(),
        "the directory given holds no directory, and so no task"
    );
}

#[test]
fn the_directory_of_one_task_is_said_to_be_one_task() {
    assert_eq!(
        DatasetError::OneTask.to_string(),
        "what was given is the directory of one task, and a dataset is the directory that \
         holds the tasks"
    );
}

#[test]
fn a_directory_that_is_not_named_as_a_task_is_said_with_its_name() {
    assert_eq!(
        DatasetError::NotAnId {
            directory: "task one".to_owned()
        }
        .to_string(),
        "the directory `task one` in it is not named with letters, digits, dots, underscores and \
         hyphens alone, as a task is"
    );
}

#[test]
fn a_task_with_no_instruction_is_said_with_its_id() {
    assert_eq!(
        DatasetError::NoInstruction { task: task() }.to_string(),
        "task fix-the-parser holds no file of text named instruction.md"
    );
}

#[test]
fn a_task_with_no_verifier_is_said_with_its_id() {
    assert_eq!(
        DatasetError::NoVerifier { task: task() }.to_string(),
        "task fix-the-parser holds no directory named tests with an executable file named \
         test.sh in it"
    );
}

#[test]
fn a_task_with_no_environment_is_said_with_its_id() {
    assert_eq!(
        DatasetError::NoEnvironment { task: task() }.to_string(),
        "task fix-the-parser holds no directory named environment with a file named \
         Dockerfile in it, and its task.toml names no image in its place"
    );
}

#[test]
fn a_task_with_no_task_toml_is_said_with_its_id() {
    assert_eq!(
        DatasetError::NoConfiguration { task: task() }.to_string(),
        "task fix-the-parser holds no file named task.toml"
    );
}

#[test]
fn a_task_toml_that_is_not_there_is_said_not_to_be_there() {
    assert_eq!(ConfigurationError::Missing.to_string(), "is not there");
}

#[test]
fn a_verifier_said_to_share_an_environment_and_to_have_its_own_is_said_as_both() {
    assert_eq!(
        ConfigurationError::SharedAndSeparate.to_string(),
        "says under verifier that the verifier shares the environment of the harness, as \
         environment_mode, and gives the verifier an environment of its own, as environment"
    );
}

#[test]
fn a_task_whose_task_toml_cannot_be_obeyed_is_said_with_its_id_and_why() {
    assert_eq!(
        DatasetError::Configuration {
            task: task(),
            error: ConfigurationError::Unreadable {
                reason: "Is a directory (os error 21)".to_owned()
            },
        }
        .to_string(),
        "task fix-the-parser holds a task.toml that cannot be read: Is a directory (os error 21)"
    );
}

#[test]
fn a_task_toml_that_is_not_valid_is_said_with_its_line_and_why() {
    assert_eq!(
        ConfigurationError::NotValid {
            line: 7,
            reason: "invalid type: integer `3`, expected a string".to_owned()
        }
        .to_string(),
        "is not valid at line 7: invalid type: integer `3`, expected a string"
    );
}

#[test]
fn a_variable_that_is_not_named_as_one_is_said_with_its_table_and_its_name() {
    assert_eq!(
        ConfigurationError::NotAName {
            table: "environment",
            name: "API TOKEN".to_owned()
        }
        .to_string(),
        "names a variable `API TOKEN` under environment.env, and the name of a variable is made \
         of letters, digits, dots, underscores and hyphens alone"
    );
}

#[test]
fn a_variable_that_is_not_set_is_said_with_the_variable_it_takes_its_value_from() {
    assert_eq!(
        ConfigurationError::NotSet {
            table: "verifier",
            name: "API_TOKEN".to_owned(),
            variable: "TOKEN".to_owned()
        }
        .to_string(),
        "gives API_TOKEN under verifier.env the value of TOKEN, which is not set where cladenc \
         runs"
    );
}

#[test]
fn a_time_limit_that_is_not_one_is_said_with_its_key() {
    assert_eq!(
        ConfigurationError::NotATimeLimit {
            key: "agent.timeout_sec"
        }
        .to_string(),
        "gives agent.timeout_sec a value that is not a number of seconds above 0"
    );
}

#[test]
fn a_wait_that_is_not_one_is_said_with_its_key() {
    assert_eq!(
        ConfigurationError::NotAWait {
            key: "environment.healthcheck.interval_sec"
        }
        .to_string(),
        "gives environment.healthcheck.interval_sec a value that is not a number of seconds, 0 \
         or more"
    );
}

#[test]
fn a_number_that_is_not_above_0_is_said_with_its_key() {
    assert_eq!(
        ConfigurationError::NotAbove0 {
            key: "environment.cpus"
        }
        .to_string(),
        "gives environment.cpus a value that is not a number above 0"
    );
}

#[test]
fn a_size_that_is_not_one_is_said_with_its_key_and_how_a_size_is_written() {
    assert_eq!(
        ConfigurationError::NotASize {
            key: "environment.memory"
        }
        .to_string(),
        "gives environment.memory a value that is not a size of a megabyte or more, written \
         as a number and one of G, M and K, such as 4G"
    );
}

#[test]
fn two_spellings_of_a_size_that_differ_are_said_with_the_older_key() {
    assert_eq!(
        ConfigurationError::TwoSizes {
            key: "environment.memory"
        }
        .to_string(),
        "gives environment.memory and environment.memory_mb sizes that differ"
    );
}

#[test]
fn what_a_task_needs_and_cladenc_cannot_give_is_said() {
    assert_eq!(
        ConfigurationError::Needs { what: "a GPU" }.to_string(),
        "asks for a GPU, which cladenc has no way to give"
    );
}

#[test]
fn a_path_under_artifacts_that_steps_back_is_said_with_the_path() {
    assert_eq!(
        ConfigurationError::StepsBack {
            path: "/app/../etc".to_owned()
        }
        .to_string(),
        "lists the path `/app/../etc` under artifacts, and a path that is kept holds no `..`"
    );
}

#[test]
fn a_destination_under_artifacts_that_is_no_place_is_said_with_what_a_destination_is() {
    assert_eq!(
        ConfigurationError::NotAPlace {
            destination: "/srv/kept".to_owned()
        }
        .to_string(),
        "gives a path under artifacts the destination `/srv/kept`, and a destination names a \
         file or a directory by a path that does not begin with `/` and holds no `..`"
    );
}

#[test]
fn a_variable_whose_value_is_not_one_line_is_said_with_its_table_and_its_name() {
    assert_eq!(
        ConfigurationError::NotOneLine {
            table: "environment",
            name: "NOTE".to_owned()
        }
        .to_string(),
        "gives NOTE under environment.env a value that is not one line of text, which a \
         sandbox cannot be given"
    );
}

#[test]
fn a_task_whose_environment_needs_more_than_one_container_is_said_with_its_id() {
    assert_eq!(
        DatasetError::ManyContainers { task: task() }.to_string(),
        "task fix-the-parser holds a Compose file in its directory named environment, and an \
         environment of more than one container cannot be built"
    );
}

#[test]
fn a_task_that_cannot_be_copied_is_said_with_its_id() {
    assert_eq!(
        DatasetError::OtherFiles { task: task() }.to_string(),
        "task fix-the-parser holds something that is not a regular file, a directory or a \
         symbolic link"
    );
}

#[test]
fn a_dataset_that_is_not_named_as_one_is_said_with_the_name_of_its_directory() {
    assert_eq!(
        DatasetError::NotNamed {
            directory: "my dataset".to_owned()
        }
        .to_string(),
        "its directory is named `my dataset`, and the name of a dataset is made of letters, \
         digits, dots, underscores and hyphens alone"
    );
}

#[test]
fn a_dataset_that_is_given_twice_is_said_with_its_directory() {
    assert_eq!(
        DatasetError::GivenTwice {
            directory: PathBuf::from("/data/parsing")
        }
        .to_string(),
        "the dataset /data/parsing is given twice"
    );
}

#[test]
fn two_datasets_of_the_same_name_are_said_with_the_name_and_why_it_matters() {
    assert_eq!(
        DatasetError::SameName {
            name: "parsing".to_owned()
        }
        .to_string(),
        "two of the datasets given are in directories named `parsing`, and a task is known by \
         the name of its dataset and its id together"
    );
}
