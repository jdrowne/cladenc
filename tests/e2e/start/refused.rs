use std::fs;
use std::path::Path;
use std::process::Command;

fn options(directory: &Path, outputs_directory: &Path) -> Vec<String> {
    [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        (
            "--benchmark",
            test_inputs::benchmark(directory, &["t1", "t2", "t3"]),
        ),
        ("--procreation-docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--outputs-dir", outputs_directory.to_owned()),
        ("--max-procreations", "1".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--selection-tasks-percentage", "25".into()),
        ("--corroboration-tasks-percentage", "25".into()),
    ]
    .iter()
    .map(|(option, value)| format!("{option}={}", value.display()))
    .collect()
}

fn status(options: &[String]) -> Option<i32> {
    Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap()
        .status
        .code()
}

fn said(options: &[String]) -> String {
    let refused = Command::new(env!("CARGO_BIN_EXE_cladenc"))
        .arg("start")
        .args(options)
        .output()
        .unwrap();
    String::from_utf8(refused.stderr).unwrap()
}

fn with_a_dockerfile(directory: &Path, dockerfile: &Path, image_too: bool) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .filter(|option| image_too || !option.starts_with("--procreation-docker-image="))
        .chain([
            format!("--procreation-dockerfile={}", dockerfile.display()),
            "--procreation-docker-image-build-timeout=60".to_owned(),
        ])
        .collect()
}

fn with_a_task_that_lacks_its_grader(directory: &Path) -> Vec<String> {
    let options = options(directory, &directory.join("outputs"));
    fs::remove_dir_all(directory.join("benchmark/t1/tests")).unwrap();
    options
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_grader_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_grader(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_grader_is_refused_by_its_directory_and_the_id_of_the_task()
 {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_grader(directory.path());
    assert!(said(&options).ends_with(
        "/benchmark: task t1 holds no directory named tests with an executable file named \
         test.sh in it\n"
    ));
}

#[test]
fn a_benchmark_that_is_refused_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_grader(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_its_benchmark_given_twice(directory: &Path) -> Vec<String> {
    let options = options(directory, &directory.join("outputs"));
    let again = options
        .iter()
        .filter(|option| option.starts_with("--benchmark="))
        .cloned()
        .collect::<Vec<_>>();
    options.into_iter().chain(again).collect()
}

#[test]
fn a_benchmark_given_twice_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_its_benchmark_given_twice(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_benchmark_given_twice_is_said_to_be_given_twice() {
    let directory = test_inputs::temporary_directory();
    let options = with_its_benchmark_given_twice(directory.path());
    assert!(said(&options).ends_with("/benchmark is given twice\n"));
}

fn with_quiet(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .chain(["--quiet".to_owned()])
        .collect()
}

#[test]
fn quiet_which_is_no_flag_of_start_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_quiet(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn quiet_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    let directory = test_inputs::temporary_directory();
    let options = with_quiet(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--quiet' found\n"));
}

#[test]
fn quiet_which_is_no_flag_of_start_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_quiet(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_a_seed(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .chain(["--seed=7".to_owned()])
        .collect()
}

#[test]
fn a_seed_which_is_no_flag_of_start_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_seed(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_seed_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_seed(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--seed' found\n"));
}

#[test]
fn a_seed_which_is_no_flag_of_start_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_seed(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_a_genealogy_growth(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .chain(["--genealogy-growth=0.5".to_owned()])
        .collect()
}

#[test]
fn a_genealogy_growth_which_is_no_flag_of_start_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_genealogy_growth(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_genealogy_growth_which_is_no_flag_of_start_is_refused_as_an_argument_not_expected() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_genealogy_growth(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--genealogy-growth' found\n"));
}

#[test]
fn a_genealogy_growth_which_is_no_flag_of_start_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_genealogy_growth(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_initial_procreations(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .chain(["--initial-procreations=5".to_owned()])
        .collect()
}

#[test]
fn initial_procreations_which_are_no_flag_of_start_exit_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_initial_procreations(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn initial_procreations_which_are_no_flag_of_start_are_refused_as_an_argument_not_expected() {
    let directory = test_inputs::temporary_directory();
    let options = with_initial_procreations(directory.path());
    assert!(
        said(&options).starts_with("error: unexpected argument '--initial-procreations' found\n")
    );
}

#[test]
fn initial_procreations_which_are_no_flag_of_start_leave_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_initial_procreations(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_the_limit_of_concurrent_containers_under_a_name_it_does_not_have(
    directory: &Path,
) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .iter()
        .map(|option| option.replace("--max-concurrent-containers=", "--workers="))
        .collect()
}

#[test]
fn the_limit_of_concurrent_containers_under_a_name_it_does_not_have_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options =
        with_the_limit_of_concurrent_containers_under_a_name_it_does_not_have(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn the_limit_of_concurrent_containers_under_a_name_it_does_not_have_is_refused_for_the_name() {
    let directory = test_inputs::temporary_directory();
    let options =
        with_the_limit_of_concurrent_containers_under_a_name_it_does_not_have(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--workers' found\n"));
}

#[test]
fn the_limit_of_concurrent_containers_under_a_name_it_does_not_have_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options =
        with_the_limit_of_concurrent_containers_under_a_name_it_does_not_have(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_the_primogenitor_under_a_name_it_does_not_have(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .iter()
        .map(|option| option.replace("--primogenitor=", "--progenitor="))
        .collect()
}

#[test]
fn the_primogenitor_under_a_name_it_does_not_have_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_primogenitor_under_a_name_it_does_not_have(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn the_primogenitor_under_a_name_it_does_not_have_is_refused_for_the_name() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_primogenitor_under_a_name_it_does_not_have(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--progenitor' found\n"));
}

#[test]
fn the_primogenitor_under_a_name_it_does_not_have_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_primogenitor_under_a_name_it_does_not_have(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_a_budget_in_the_place_of_the_most_procreations(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .iter()
        .map(|option| option.replace("--max-procreations=", "--budget="))
        .collect()
}

#[test]
fn a_budget_which_is_no_flag_of_start_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_budget_in_the_place_of_the_most_procreations(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_budget_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_budget_in_the_place_of_the_most_procreations(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--budget' found\n"));
}

#[test]
fn a_budget_which_is_no_flag_of_start_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_budget_in_the_place_of_the_most_procreations(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_a_confidence(directory: &Path) -> Vec<String> {
    let mut options = options(directory, &directory.join("outputs"));
    options.push("--confidence=99".to_owned());
    options
}

#[test]
fn a_confidence_which_is_no_flag_of_start_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_confidence(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_confidence_which_is_no_flag_of_start_is_refused_as_an_argument_that_is_not_expected() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_confidence(directory.path());
    assert!(said(&options).starts_with("error: unexpected argument '--confidence' found\n"));
}

#[test]
fn a_confidence_which_is_no_flag_of_start_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_confidence(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn without_the_most_procreations(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .filter(|option| !option.starts_with("--max-procreations="))
        .collect()
}

#[test]
fn a_command_line_without_the_most_procreations_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = without_the_most_procreations(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_command_line_without_the_most_procreations_is_refused_for_the_flag_it_lacks() {
    let directory = test_inputs::temporary_directory();
    let options = without_the_most_procreations(directory.path());
    assert!(said(&options).contains("  --max-procreations <COUNT>\n"));
}

fn with_no_procreation_at_most(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .iter()
        .map(|option| option.replace("--max-procreations=1", "--max-procreations=0"))
        .collect()
}

#[test]
fn the_most_procreations_being_0_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_no_procreation_at_most(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn the_most_procreations_being_0_is_refused_for_the_value() {
    let directory = test_inputs::temporary_directory();
    let options = with_no_procreation_at_most(directory.path());
    assert!(
        said(&options).starts_with("error: invalid value '0' for '--max-procreations <COUNT>': ")
    );
}

#[test]
fn the_most_procreations_being_0_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_no_procreation_at_most(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

#[test]
fn a_missing_option_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path(), &directory.path().join("outputs"));
    assert_eq!(status(&options[..options.len() - 1]), Some(2));
}

#[test]
fn an_outputs_directory_at_which_something_exists_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path(), directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn an_image_and_a_dockerfile_together_exit_with_2() {
    let directory = test_inputs::temporary_directory();
    let dockerfile = test_inputs::dockerfile(directory.path(), "RUN touch /made-here\n");
    let options = with_a_dockerfile(directory.path(), &dockerfile, true);
    assert_eq!(status(&options), Some(2));
}

#[test]
fn an_image_and_a_dockerfile_together_are_refused_by_the_flags_of_both() {
    let directory = test_inputs::temporary_directory();
    let dockerfile = test_inputs::dockerfile(directory.path(), "RUN touch /made-here\n");
    let options = with_a_dockerfile(directory.path(), &dockerfile, true);
    assert!(said(&options).starts_with(
        "error: the argument '--procreation-docker-image <IMAGE>' cannot be used with:\n  \
         --procreation-dockerfile <FILE>\n"
    ));
}

#[test]
fn a_dockerfile_that_does_not_exist_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_dockerfile(directory.path(), &directory.path().join("missing"), false);
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_dockerfile_that_does_not_exist_is_refused_for_what_its_flag_takes() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_dockerfile(directory.path(), &directory.path().join("missing"), false);
    assert_eq!(
        said(&options),
        "cladenc: --procreation-dockerfile takes an existing file\n"
    );
}

#[test]
fn a_dockerfile_that_is_refused_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_dockerfile(directory.path(), &directory.path().join("missing"), false);
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_the_percentage(directory: &Path, flag: &str, percentage: &str) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .iter()
        .map(|option| option.replace(&format!("{flag}=25"), &format!("{flag}={percentage}")))
        .collect()
}

fn with_the_selection_tasks_percentage(directory: &Path, percentage: &str) -> Vec<String> {
    with_the_percentage(directory, "--selection-tasks-percentage", percentage)
}

fn with_the_corroboration_tasks_percentage(directory: &Path, percentage: &str) -> Vec<String> {
    with_the_percentage(directory, "--corroboration-tasks-percentage", percentage)
}

#[test]
fn a_selection_tasks_percentage_that_makes_no_task_of_a_benchmark_a_selection_task_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "10");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_selection_tasks_percentage_that_makes_no_selection_task_is_refused_with_the_benchmark() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "10");
    assert_eq!(
        said(&options),
        "cladenc: --selection-tasks-percentage takes a number above 0 and below 100 that makes \
         at least one task of every benchmark a selection task: the benchmark `benchmark` holds \
         3 tasks, and none of them would be a selection task\n"
    );
}

#[test]
fn a_selection_tasks_percentage_that_is_refused_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "10");
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

#[test]
fn a_selection_tasks_percentage_of_100_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "100");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_selection_tasks_percentage_of_100_is_refused_for_the_value() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "100");
    assert!(said(&options).starts_with(
        "error: invalid value '100' for '--selection-tasks-percentage <NUMBER>': a number above \
         0 and below 100 is expected\n"
    ));
}

#[test]
fn a_corroboration_tasks_percentage_that_makes_no_corroboration_task_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_corroboration_tasks_percentage(directory.path(), "10");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_corroboration_tasks_percentage_that_makes_no_corroboration_task_is_refused_with_the_benchmark()
{
    let directory = test_inputs::temporary_directory();
    let options = with_the_corroboration_tasks_percentage(directory.path(), "10");
    assert_eq!(
        said(&options),
        "cladenc: --corroboration-tasks-percentage takes a number above 0 and below 100 that \
         makes at least one task of every benchmark a corroboration task: the benchmark \
         `benchmark` holds 3 tasks, and none of them would be a corroboration task\n"
    );
}

#[test]
fn a_corroboration_tasks_percentage_that_is_refused_leaves_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_corroboration_tasks_percentage(directory.path(), "10");
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

#[test]
fn a_corroboration_tasks_percentage_of_100_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_corroboration_tasks_percentage(directory.path(), "100");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_corroboration_tasks_percentage_of_100_is_refused_for_the_value() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_corroboration_tasks_percentage(directory.path(), "100");
    assert!(said(&options).starts_with(
        "error: invalid value '100' for '--corroboration-tasks-percentage <NUMBER>': a number \
         above 0 and below 100 is expected\n"
    ));
}

#[test]
fn percentages_that_together_hold_out_every_task_of_a_benchmark_exit_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "50");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn percentages_that_together_hold_out_every_task_of_a_benchmark_are_refused_by_both_flags() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "50");
    assert_eq!(
        said(&options),
        "cladenc: --selection-tasks-percentage and --corroboration-tasks-percentage take numbers \
         that together leave at least one task of every benchmark a non-holdout task: the \
         benchmark `benchmark` holds 3 tasks, and every one of them would be held out\n"
    );
}

#[test]
fn percentages_that_together_hold_out_every_task_leave_no_outputs_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_selection_tasks_percentage(directory.path(), "50");
    let _ = status(&options);
    assert!(!directory.path().join("outputs").exists());
}

fn with_a_benchmark_of_two_tasks(directory: &Path) -> Vec<String> {
    let pair = test_inputs::benchmark_named(directory, "pair", &["t1", "t2"]);
    options(directory, &directory.join("outputs"))
        .into_iter()
        .filter(|option| !option.starts_with("--benchmark="))
        .chain([format!("--benchmark={}", pair.display())])
        .collect()
}

#[test]
fn a_benchmark_of_two_tasks_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_benchmark_of_two_tasks(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_benchmark_of_two_tasks_is_refused_as_one_that_cannot_be_divided() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_benchmark_of_two_tasks(directory.path());
    assert_eq!(
        said(&options),
        "cladenc: --benchmark takes the directory of a benchmark of three tasks at the least: \
         the benchmark `pair` holds fewer than three tasks, which cannot be divided into a \
         selection task, a corroboration task and a non-holdout task\n"
    );
}

fn without(directory: &Path, flag: &str) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .filter(|option| !option.starts_with(&format!("{flag}=")))
        .collect()
}

#[test]
fn a_command_line_without_the_selection_tasks_percentage_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = without(directory.path(), "--selection-tasks-percentage");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_command_line_without_the_selection_tasks_percentage_is_refused_for_the_flag_it_lacks() {
    let directory = test_inputs::temporary_directory();
    let options = without(directory.path(), "--selection-tasks-percentage");
    assert!(said(&options).contains("  --selection-tasks-percentage <NUMBER>\n"));
}

#[test]
fn a_command_line_without_the_corroboration_tasks_percentage_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = without(directory.path(), "--corroboration-tasks-percentage");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_command_line_without_the_corroboration_tasks_percentage_is_refused_for_the_flag_it_lacks() {
    let directory = test_inputs::temporary_directory();
    let options = without(directory.path(), "--corroboration-tasks-percentage");
    assert!(said(&options).contains("  --corroboration-tasks-percentage <NUMBER>\n"));
}

fn with_a_flag_that_is_gone(directory: &Path, flag: &str) -> Vec<String> {
    options(directory, &directory.join("outputs"))
        .into_iter()
        .chain([format!("{flag}=50")])
        .collect()
}

#[test]
fn a_command_line_that_gives_the_holdout_percentage_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_flag_that_is_gone(directory.path(), "--holdout-percentage");
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_command_line_that_gives_the_holdout_corroboration_percentage_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_flag_that_is_gone(directory.path(), "--holdout-corroboration-percentage");
    assert_eq!(status(&options), Some(2));
}
