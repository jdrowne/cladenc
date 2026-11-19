use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

use crate::benchmark::Benchmark;
use crate::cli::StartOptions;
use crate::cli::checks::Refused;
use crate::holdout;
use crate::journal::Settings;
use tempfile::TempDir;

fn options(directory: &Path) -> StartOptions {
    StartOptions {
        primogenitor: test_inputs::primogenitor(directory),
        model: "a-model".to_owned(),
        model_url: "http://models.example/v1".to_owned(),
        model_key_file: None,
        benchmark: vec![test_inputs::benchmark(directory, &["t3", "t2", "t1"])],
        docker_image: Some("image".to_owned()),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        outputs_dir: directory.join("outputs"),
        max_procreations: 800,
        genealogy_growth: 0.6,
        max_concurrent_containers: 8,
        procreation_timeout: 3600,
        attempt_timeout: 1800,
        holdout_percentage: 60.0,
        holdout_corroboration_percentage: 50.0,
    }
}

fn with_a_dockerfile(directory: &TempDir, options: &mut StartOptions) {
    options.docker_image = None;
    options.dockerfile = Some(test_inputs::dockerfile(
        directory.path(),
        "RUN touch /made-here\n",
    ));
    options.docker_build_timeout = Some(600);
}

fn is_named_by_cladenc(image: &str) -> bool {
    image.strip_prefix("cladenc-").is_some_and(|digits| {
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

fn option_refused(change: impl FnOnce(&TempDir, &mut StartOptions)) -> &'static str {
    checked(change).unwrap_err().option
}

fn with_no_grader_in_a_task() -> Refused {
    checked(|_, options| fs::remove_file(options.benchmark[0].join("t2/grader")).unwrap())
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
            genealogy_growth: 0.6,
            max_concurrent_containers: 8,
            procreation_timeout: 3600,
            attempt_timeout: 1800,
            holdout_percentage: 60.0,
            holdout_tasks: settings.holdout_tasks.clone(),
            holdout_corroboration_percentage: 50.0,
            corroboration_tasks: settings.corroboration_tasks.clone(),
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
        option_refused(|directory, options| options.primogenitor = directory.path().join("missing")),
        "--primogenitor"
    );
}

#[test]
fn a_primogenitor_that_is_a_file_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            options.primogenitor = test_inputs::model_key_file(directory.path(), "a-made-up-key\n");
        }),
        "--primogenitor"
    );
}

#[test]
fn a_primogenitor_that_holds_a_socket_is_refused() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path());
    let _socket = UnixListener::bind(options.primogenitor.join("socket")).unwrap();
    assert_eq!(options.settings().unwrap_err().option, "--primogenitor");
}

#[test]
fn the_image_and_the_run_command_are_settings_as_they_were_given() {
    let settings = checked(|_, options| {
        options.docker_image = Some("registry.example/harness:1".to_owned());
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
fn two_directed_evolutions_given_a_dockerfile_are_given_images_of_different_names() {
    assert_ne!(
        checked(with_a_dockerfile).unwrap().docker_image,
        checked(with_a_dockerfile).unwrap().docker_image
    );
}

#[test]
fn a_dockerfile_that_does_not_exist_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            with_a_dockerfile(directory, options);
            options.dockerfile = Some(directory.path().join("missing"));
        }),
        "--dockerfile"
    );
}

#[test]
fn a_dockerfile_that_is_a_directory_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            with_a_dockerfile(directory, options);
            options.dockerfile = Some(options.primogenitor.clone());
        }),
        "--dockerfile"
    );
}

#[test]
fn a_dockerfile_with_no_path_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            with_a_dockerfile(directory, options);
            options.dockerfile = Some(PathBuf::new());
        }),
        "--dockerfile"
    );
}

#[test]
fn a_dockerfile_that_is_refused_is_said_with_what_the_flag_takes() {
    let refused = checked(|directory, options| {
        with_a_dockerfile(directory, options);
        options.dockerfile = Some(directory.path().join("missing"));
    });
    assert_eq!(
        refused.unwrap_err().to_string(),
        "--dockerfile takes an existing file"
    );
}

#[test]
fn a_dockerfile_given_from_the_working_directory_is_made_absolute() {
    let settings = checked(|directory, options| {
        with_a_dockerfile(directory, options);
        options.dockerfile = Some(PathBuf::from("tests/samples/Dockerfile"));
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
        options.dockerfile = Some(link);
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
fn a_refused_value_is_said_with_its_option_and_what_the_option_takes() {
    let refused = Refused {
        option: "--dockerfile",
        accepted: "an existing file",
        reason: None,
    };
    assert_eq!(refused.to_string(), "--dockerfile takes an existing file");
}

#[test]
fn a_value_refused_for_a_reason_is_said_with_the_reason_after_what_the_option_takes() {
    assert!(with_no_grader_in_a_task().to_string().starts_with(
        "--benchmark takes the directory of a benchmark, in which every directory is a task that \
         holds a file of text named specification, a directory named workspace and an executable \
         file named grader: /"
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
        option_refused(|directory, options| {
            options.model_key_file = Some(directory.path().join("missing"));
        }),
        "--model-key-file"
    );
}

#[test]
fn a_file_of_the_key_that_holds_no_key_is_refused() {
    assert_eq!(
        option_refused(|directory, options| {
            options.model_key_file = Some(test_inputs::model_key_file(directory.path(), "\n"));
        }),
        "--model-key-file"
    );
}

#[test]
fn a_file_of_the_key_with_no_path_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.model_key_file = Some(PathBuf::new())),
        "--model-key-file"
    );
}

#[test]
fn the_holdout_percentage_is_a_setting_as_it_was_given() {
    let settings = checked(|_, options| options.holdout_percentage = 37.5).unwrap();
    assert_eq!(settings.holdout_percentage, 37.5);
}

#[test]
fn sixty_in_a_hundred_of_a_benchmark_of_three_tasks_holds_out_two_of_the_three() {
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
        holdout::draw(&settings.benchmarks, 60.0, settings.seed).unwrap()
    );
}

#[test]
fn the_percentage_is_taken_of_the_tasks_of_each_benchmark_given() {
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
        [2, 3]
    );
}

#[test]
fn a_holdout_percentage_that_holds_out_no_task_of_a_benchmark_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.holdout_percentage = 10.0),
        "--holdout-percentage"
    );
}

#[test]
fn a_holdout_percentage_that_holds_out_every_task_of_a_benchmark_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.holdout_percentage = 90.0),
        "--holdout-percentage"
    );
}

#[test]
fn a_benchmark_of_one_task_is_refused_by_the_flag_of_the_holdout_percentage() {
    assert_eq!(
        option_refused(|directory, options| {
            options.benchmark = vec![test_inputs::benchmark_named(
                directory.path(),
                "single",
                &["t1"],
            )];
        }),
        "--holdout-percentage"
    );
}

#[test]
fn a_refused_holdout_percentage_is_said_with_what_the_flag_takes_and_the_benchmark_at_fault() {
    let refused = checked(|_, options| options.holdout_percentage = 10.0).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--holdout-percentage takes a number above 0 and below 100 that holds out at least one \
         task of every benchmark and leaves at least one: the benchmark `benchmark` holds 3 \
         tasks, and none of them would be held out"
    );
}

#[test]
fn a_benchmark_that_is_refused_is_refused_ahead_of_the_holdout_percentage() {
    assert_eq!(
        option_refused(|directory, options| {
            options.benchmark = vec![directory.path().join("missing")];
            options.holdout_percentage = 10.0;
        }),
        "--benchmark"
    );
}

#[test]
fn the_holdout_corroboration_percentage_is_a_setting_as_it_was_given() {
    let settings = checked(|_, options| options.holdout_corroboration_percentage = 37.5).unwrap();
    assert_eq!(settings.holdout_corroboration_percentage, 37.5);
}

#[test]
fn a_half_of_two_holdout_tasks_is_one_corroboration_task() {
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
        holdout::draw_corroboration(
            &settings.benchmarks,
            &settings.holdout_tasks,
            50.0,
            settings.seed
        )
        .unwrap()
    );
}

#[test]
fn the_corroboration_percentage_is_taken_of_the_holdout_tasks_of_each_benchmark_given() {
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
fn a_holdout_corroboration_percentage_that_leaves_a_benchmark_no_corroboration_task_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.holdout_corroboration_percentage = 10.0),
        "--holdout-corroboration-percentage"
    );
}

#[test]
fn a_holdout_corroboration_percentage_that_leaves_a_benchmark_no_selection_task_is_refused() {
    assert_eq!(
        option_refused(|_, options| options.holdout_corroboration_percentage = 90.0),
        "--holdout-corroboration-percentage"
    );
}

#[test]
fn a_benchmark_with_one_holdout_task_is_refused_by_the_flag_that_divides_the_holdout_tasks() {
    assert_eq!(
        option_refused(|_, options| options.holdout_percentage = 40.0),
        "--holdout-corroboration-percentage"
    );
}

#[test]
fn a_refused_holdout_corroboration_percentage_is_said_with_what_the_flag_takes_and_the_benchmark() {
    let refused =
        checked(|_, options| options.holdout_corroboration_percentage = 10.0).unwrap_err();
    assert_eq!(
        refused.to_string(),
        "--holdout-corroboration-percentage takes a number above 0 and below 100 that makes at \
         least one holdout task of every benchmark a corroboration task and leaves at least one \
         a selection task: the benchmark `benchmark` has 2 holdout tasks, and none of them would \
         be a corroboration task"
    );
}

#[test]
fn a_holdout_percentage_that_is_refused_is_refused_ahead_of_the_one_that_divides_the_holdout_tasks()
{
    assert_eq!(
        option_refused(|_, options| {
            options.holdout_percentage = 10.0;
            options.holdout_corroboration_percentage = 10.0;
        }),
        "--holdout-percentage"
    );
}
