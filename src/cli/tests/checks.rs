use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use crate::benchmark::Benchmark;
use crate::cli::StartOptions;
use crate::cli::checks::Refused;
use crate::holdout::{self, Percentages};
use crate::journal::Settings;
use tempfile::TempDir;

const PERCENTAGES: Percentages = Percentages {
    selection: 30.0,
    corroboration: 30.0,
};

fn options(directory: &Path) -> StartOptions {
    StartOptions {
        primogenitor: test_inputs::primogenitor(directory),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: vec![test_inputs::benchmark(directory, &["t3", "t2", "t1"])],
        procreation_docker_image: Some("image".to_owned()),
        procreation_dockerfile: None,
        procreation_docker_image_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.join("outputs"),
        max_procreations: 800,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        selection_tasks_percentage: 30.0,
        corroboration_tasks_percentage: 30.0,
    }
}

fn with_a_dockerfile(directory: &TempDir, options: &mut StartOptions) {
    options.procreation_docker_image = None;
    options.procreation_dockerfile = Some(test_inputs::dockerfile(
        directory.path(),
        "RUN touch /made-here\n",
    ));
    options.procreation_docker_image_build_timeout = Some(600);
}

fn is_named_by_cladenc(image: &str) -> bool {
    is_named_after("cladenc-", image)
}

fn is_named_after(prefix: &str, image: &str) -> bool {
    image.strip_prefix(prefix).is_some_and(|digits| {
        digits.len() == 16
            && digits
                .chars()
                .all(|digit| matches!(digit, '0'..='9' | 'a'..='f'))
    })
}

fn checked(change: impl FnOnce(&TempDir, &mut StartOptions)) -> Result<Settings, Refused> {
    let directory = test_inputs::temporary_directory();
    let mut options = options(directory.path());
    change(&directory, &mut options);
    options.settings()
}

fn flags_refused(change: impl FnOnce(&TempDir, &mut StartOptions)) -> &'static [&'static str] {
    checked(change).unwrap_err().flags
}

fn with_no_grader_in_a_task() -> Refused {
    checked(|_, options| fs::remove_dir_all(options.benchmark[0].join("t2/tests")).unwrap())
        .unwrap_err()
}

#[test]
fn values_that_fit_their_rows_give_the_settings_of_the_run() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    let root = fs::canonicalize(directory.path()).unwrap();
    let settings = options.settings().unwrap();
    assert_eq!(
        settings,
        Settings {
            primogenitor: root.join("primogenitor"),
            model: "a-model".to_owned(),
            model_url: "http://models.example/v1".to_owned(),
            model_key_file: None,
            benchmarks: vec![Benchmark {
                directory: root.join("benchmark"),
                tasks: vec!["t1".to_owned(), "t2".to_owned(), "t3".to_owned()],
            }],
            docker_image: "image".to_owned(),
            dockerfile: None,
            docker_build_timeout: None,
            harness_build_command: None,
            harness_run_command: "run a harness".to_owned(),
            max_procreations: 800,
            max_concurrent_containers: 8,
            procreation_timeout: 3600,
            selection_tasks_percentage: 30.0,
            holdout_tasks: settings.holdout_tasks.clone(),
            corroboration_tasks_percentage: 30.0,
            corroboration_tasks: settings.corroboration_tasks.clone(),
            environment_image_repository: settings.environment_image_repository.clone(),
            seed: settings.seed,
        }
    );
}

#[test]
fn two_directed_evolutions_started_from_one_command_line_are_given_different_seeds() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    assert_ne!(
        options.settings().unwrap().seed,
        options.settings().unwrap().seed
    );
}

#[test]
fn a_path_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|_, options| options.primogenitor = PathBuf::from("src")).unwrap();
    assert!(settings.primogenitor.is_absolute());
}

#[test]
fn a_primogenitor_that_does_not_exist_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| options.primogenitor = directory.path().join("missing")),
        ["--primogenitor"]
    );
}

#[test]
fn a_primogenitor_that_is_a_file_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| {
            options.primogenitor = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
        }),
        ["--primogenitor"]
    );
}

#[test]
fn a_primogenitor_that_holds_a_socket_is_refused() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    let _socket = UnixListener::bind(options.primogenitor.join("socket")).unwrap();
    assert_eq!(options.settings().unwrap_err().flags, ["--primogenitor"]);
}

#[test]
fn the_image_and_the_run_command_are_settings_as_they_were_given() {
    let settings = checked(|_, options| {
        options.procreation_docker_image = Some("registry.example/harness:1".to_owned());
        options.harness_run_command = "sh \"$CLADENC_HARNESS/harness.sh\"".to_owned();
    })
    .unwrap();
    assert_eq!(
        (settings.docker_image, settings.harness_run_command),
        (
            "registry.example/harness:1".to_owned(),
            "sh \"$CLADENC_HARNESS/harness.sh\"".to_owned()
        )
    );
}

#[test]
fn a_directed_evolution_given_an_image_has_no_dockerfile_and_no_time_limit_of_a_build() {
    let settings = checked(|_, _| {}).unwrap();
    assert_eq!(
        (settings.dockerfile, settings.docker_build_timeout),
        (None, None)
    );
}

#[test]
fn a_dockerfile_is_a_setting_with_the_time_limit_of_its_build() {
    let directory = test_inputs::temporary_directory();
    let mut options = options(directory.path());
    with_a_dockerfile(&directory, &mut options);
    let settings = options.settings().unwrap();
    assert_eq!(
        (settings.dockerfile, settings.docker_build_timeout),
        (
            Some(
                fs::canonicalize(directory.path())
                    .unwrap()
                    .join("Dockerfile")
            ),
            Some(600)
        )
    );
}

#[test]
fn the_image_built_from_a_dockerfile_is_named_cladenc_and_sixteen_hexadecimal_digits() {
    let settings = checked(with_a_dockerfile).unwrap();
    assert!(is_named_by_cladenc(&settings.docker_image));
}

#[test]
fn the_repository_of_the_environments_is_named_cladenc_environments_and_sixteen_hexadecimal_digits()
{
    let settings = checked(|_, _| ()).unwrap();
    assert!(is_named_after(
        "cladenc-environments-",
        &settings.environment_image_repository
    ));
}

#[test]
fn two_directed_evolutions_started_from_one_command_line_are_given_different_repositories() {
    assert_ne!(
        checked(|_, _| ()).unwrap().environment_image_repository,
        checked(|_, _| ()).unwrap().environment_image_repository
    );
}

#[test]
fn two_directed_evolutions_given_a_dockerfile_are_given_images_of_different_names() {
    assert_ne!(
        checked(with_a_dockerfile).unwrap().docker_image,
        checked(with_a_dockerfile).unwrap().docker_image
    );
}

#[test]
fn a_dockerfile_that_does_not_exist_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| {
            with_a_dockerfile(directory, options);
            options.procreation_dockerfile = Some(directory.path().join("missing"));
        }),
        ["--procreation-dockerfile"]
    );
}

#[test]
fn a_dockerfile_that_is_a_directory_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| {
            with_a_dockerfile(directory, options);
            options.procreation_dockerfile = Some(options.primogenitor.clone());
        }),
        ["--procreation-dockerfile"]
    );
}

#[test]
fn a_dockerfile_with_no_path_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| {
            with_a_dockerfile(directory, options);
            options.procreation_dockerfile = Some(PathBuf::new());
        }),
        ["--procreation-dockerfile"]
    );
}

#[test]
fn a_dockerfile_that_is_refused_is_said_with_what_the_flag_takes() {
    let refused = checked(|directory, options| {
        with_a_dockerfile(directory, options);
        options.procreation_dockerfile = Some(directory.path().join("missing"));
    });
    assert_eq!(
        refused.unwrap_err().to_string(),
        "--procreation-dockerfile takes an existing file"
    );
}

#[test]
fn a_dockerfile_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|directory, options| {
        with_a_dockerfile(directory, options);
        options.procreation_dockerfile = Some(PathBuf::from("tests/samples/Dockerfile"));
    })
    .unwrap();
    assert!(settings.dockerfile.unwrap().is_absolute());
}

#[test]
fn a_dockerfile_given_through_a_symbolic_link_is_the_file_it_leads_to() {
    let settings = checked(|directory, options| {
        with_a_dockerfile(directory, options);
        let link = directory.path().join("link");
        symlink(directory.path().join("Dockerfile"), &link).unwrap();
        options.procreation_dockerfile = Some(link);
    })
    .unwrap();
    assert!(settings.dockerfile.unwrap().ends_with("Dockerfile"));
}

#[test]
fn the_build_command_is_a_setting_as_it_was_given() {
    let settings = checked(|_, options| {
        options.harness_build_command = Some("make harness".to_owned());
    })
    .unwrap();
    assert_eq!(
        settings.harness_build_command,
        Some("make harness".to_owned())
    );
}

#[test]
fn a_refused_value_is_said_with_its_flag_and_what_the_flag_takes() {
    let refused = Refused {
        flags: &["--procreation-dockerfile"],
        accepted: "an existing file",
        reason: None,
    };
    assert_eq!(
        refused.to_string(),
        "--procreation-dockerfile takes an existing file"
    );
}

#[test]
fn a_value_refused_for_a_reason_is_said_with_the_reason_after_what_the_option_takes() {
    assert!(with_no_grader_in_a_task().to_string().starts_with(
        "--benchmark takes the directory of a benchmark, in which every directory is a task that \
         holds a file of text named instruction.md, a file named task.toml that is valid as the \
         task configuration of Harbor, a directory named tests with an executable file named \
         test.sh in it and a directory named environment with a file named Dockerfile and no \
         Compose file in it, or, in place of the Dockerfile, an image named in its task.toml: /"
    ));
}

#[test]
fn a_value_refused_for_what_its_option_takes_alone_has_no_reason() {
    let refused = checked(|directory, options| {
        options.primogenitor = directory.path().join("missing");
    })
    .unwrap_err();
    assert_eq!(refused.reason, None);
}

#[test]
fn the_model_and_its_address_are_settings_as_they_were_given() {
    let settings = checked(|_, options| {
        options.model = "llama3.3:70b".to_owned();
        options.model_url = "http://host.docker.internal:11434".to_owned();
    })
    .unwrap();
    assert_eq!(
        (settings.model, settings.model_url),
        (
            "llama3.3:70b".to_owned(),
            "http://host.docker.internal:11434".to_owned()
        )
    );
}

#[test]
fn the_file_of_the_key_is_a_setting_and_the_key_is_not() {
    let directory = test_inputs::temporary_directory();
    let mut options = options(directory.path());
    let file = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
    options.model_key_file = Some(file.clone());
    assert_eq!(options.settings().unwrap().model_key_file, Some(file));
}

#[test]
fn a_file_of_the_key_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|_, options| {
        options.model_key_file = Some(PathBuf::from("tests/samples/model-key"));
    })
    .unwrap();
    assert!(settings.model_key_file.unwrap().is_absolute());
}

#[test]
fn a_file_of_the_key_that_does_not_exist_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| {
            options.model_key_file = Some(directory.path().join("missing"));
        }),
        ["--model-key-file"]
    );
}

#[test]
fn a_file_of_the_key_that_holds_no_key_is_refused() {
    assert_eq!(
        flags_refused(|directory, options| {
            options.model_key_file = Some(test_inputs::model_key_file(directory.path(), "\n"));
        }),
        ["--model-key-file"]
    );
}

#[test]
fn a_file_of_the_key_with_no_path_is_refused() {
    assert_eq!(
        flags_refused(|_, options| options.model_key_file = Some(PathBuf::new())),
        ["--model-key-file"]
    );
}

#[test]
fn the_selection_tasks_percentage_is_a_setting_as_it_was_given() {
    let settings = checked(|_, options| options.selection_tasks_percentage = 32.5).unwrap();
    assert_eq!(settings.selection_tasks_percentage, 32.5);
}

#[test]
fn thirty_in_a_hundred_as_selection_and_as_corroboration_holds_out_two_of_three_tasks() {
    let settings = checked(|_, _| {}).unwrap();
    assert!(
        [
            ["benchmark/t1", "benchmark/t2"],
            ["benchmark/t1", "benchmark/t3"],
            ["benchmark/t2", "benchmark/t3"]
        ]
        .iter()
        .any(|held_out| settings.holdout_tasks == held_out)
    );
}

#[test]
fn the_holdout_tasks_are_the_ones_that_the_seed_of_the_directed_evolution_draws() {
    let settings = checked(|_, _| {}).unwrap();
    assert_eq!(
        settings.holdout_tasks,
        holdout::draw(&settings.benchmarks, PERCENTAGES, settings.seed)
            .unwrap()
            .holdout_tasks
    );
}

#[test]
fn the_percentages_are_taken_of_the_tasks_of_each_benchmark_given() {
    let settings = checked(|directory, options| {
        options.benchmark.push(test_inputs::benchmark_named(
            directory.path(),
            "refactoring",
            &["t1", "t2", "t3", "t4", "t5"],
        ));
    })
    .unwrap();
    assert_eq!(
        ["benchmark/", "refactoring/"].map(|name| {
            settings
                .holdout_tasks
                .iter()
                .filter(|task| task.starts_with(name))
                .count()
        }),
        [2, 4]
    );
}

#[test]
fn a_selection_tasks_percentage_that_makes_no_task_of_a_benchmark_a_selection_task_is_refused() {
    assert_eq!(
        flags_refused(|_, options| options.selection_tasks_percentage = 10.0),
        ["--selection-tasks-percentage"]
    );
}

#[test]
fn a_refused_selection_tasks_percentage_is_said_with_what_the_flag_takes_and_the_benchmark() {
    let refused = checked(|_, options| options.selection_tasks_percentage = 10.0).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--selection-tasks-percentage takes a number above 0 and below 100 that makes at least \
         one task of every benchmark a selection task: the benchmark `benchmark` holds 3 tasks, \
         and none of them would be a selection task"
    );
}

#[test]
fn a_benchmark_that_is_refused_is_refused_ahead_of_the_selection_tasks_percentage() {
    assert_eq!(
        flags_refused(|directory, options| {
            options.benchmark = vec![directory.path().join("missing")];
            options.selection_tasks_percentage = 10.0;
        }),
        ["--benchmark"]
    );
}

#[test]
fn a_benchmark_of_two_tasks_is_refused_by_the_flag_of_the_benchmarks() {
    assert_eq!(
        flags_refused(|directory, options| {
            options.benchmark = vec![test_inputs::benchmark_named(
                directory.path(),
                "pair",
                &["t1", "t2"],
            )];
        }),
        ["--benchmark"]
    );
}

#[test]
fn a_benchmark_of_two_tasks_is_said_to_hold_fewer_than_three() {
    let refused = checked(|directory, options| {
        options.benchmark = vec![test_inputs::benchmark_named(
            directory.path(),
            "pair",
            &["t1", "t2"],
        )];
    })
    .unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--benchmark takes the directory of a benchmark of three tasks at the least: the \
         benchmark `pair` holds fewer than three tasks, which cannot be divided into a selection \
         task, a corroboration task and a non-holdout task"
    );
}

#[test]
fn the_corroboration_tasks_percentage_is_a_setting_as_it_was_given() {
    let settings = checked(|_, options| options.corroboration_tasks_percentage = 37.5).unwrap();
    assert_eq!(settings.corroboration_tasks_percentage, 37.5);
}

#[test]
fn thirty_in_a_hundred_of_a_benchmark_of_three_tasks_is_one_corroboration_task() {
    let settings = checked(|_, _| {}).unwrap();
    assert_eq!(settings.corroboration_tasks.len(), 1);
}

#[test]
fn every_corroboration_task_is_one_of_the_holdout_tasks() {
    let settings = checked(|_, _| {}).unwrap();
    assert!(
        settings
            .corroboration_tasks
            .iter()
            .all(|task| settings.holdout_tasks.contains(task))
    );
}

#[test]
fn the_corroboration_tasks_are_the_ones_that_the_seed_of_the_directed_evolution_draws() {
    let settings = checked(|_, _| {}).unwrap();
    assert_eq!(
        settings.corroboration_tasks,
        holdout::draw(&settings.benchmarks, PERCENTAGES, settings.seed)
            .unwrap()
            .corroboration_tasks
    );
}

#[test]
fn the_corroboration_tasks_percentage_is_taken_of_the_tasks_of_each_benchmark_given() {
    let settings = checked(|directory, options| {
        options.benchmark.push(test_inputs::benchmark_named(
            directory.path(),
            "refactoring",
            &["t1", "t2", "t3", "t4", "t5"],
        ));
    })
    .unwrap();
    assert_eq!(
        ["benchmark/", "refactoring/"].map(|name| {
            settings
                .corroboration_tasks
                .iter()
                .filter(|task| task.starts_with(name))
                .count()
        }),
        [1, 2]
    );
}

#[test]
fn a_corroboration_tasks_percentage_that_makes_no_task_a_corroboration_task_is_refused() {
    assert_eq!(
        flags_refused(|_, options| options.corroboration_tasks_percentage = 10.0),
        ["--corroboration-tasks-percentage"]
    );
}

#[test]
fn a_refused_corroboration_tasks_percentage_is_said_with_what_the_flag_takes_and_the_benchmark() {
    let refused = checked(|_, options| options.corroboration_tasks_percentage = 10.0).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--corroboration-tasks-percentage takes a number above 0 and below 100 that makes at \
         least one task of every benchmark a corroboration task: the benchmark `benchmark` holds \
         3 tasks, and none of them would be a corroboration task"
    );
}

#[test]
fn a_selection_tasks_percentage_that_is_refused_is_refused_ahead_of_the_corroboration_one() {
    assert_eq!(
        flags_refused(|_, options| {
            options.selection_tasks_percentage = 10.0;
            options.corroboration_tasks_percentage = 10.0;
        }),
        ["--selection-tasks-percentage"]
    );
}

#[test]
fn percentages_that_hold_out_every_task_of_a_benchmark_are_refused_by_both_flags() {
    assert_eq!(
        flags_refused(|_, options| options.selection_tasks_percentage = 50.0),
        [
            "--selection-tasks-percentage",
            "--corroboration-tasks-percentage"
        ]
    );
}

#[test]
fn percentages_that_hold_out_every_task_are_said_with_what_the_two_flags_take_and_the_benchmark() {
    let refused = checked(|_, options| options.selection_tasks_percentage = 50.0).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--selection-tasks-percentage and --corroboration-tasks-percentage take numbers that \
         together leave at least one task of every benchmark a non-holdout task: the benchmark \
         `benchmark` holds 3 tasks, and every one of them would be held out"
    );
}
