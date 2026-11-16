use std::fs;
use std::path::Path;
use std::process::Command;

fn options(directory: &Path, run_directory: &Path) -> Vec<String> {
    [
        ("--primogenitor", test_inputs::primogenitor(directory)),
        ("--model", test_inputs::MODEL.into()),
        ("--model-url", test_inputs::MODEL_URL.into()),
        ("--benchmark", test_inputs::benchmark(directory, &["t1"])),
        ("--docker-image", test_inputs::IMAGE.into()),
        ("--harness-run-command", "exit 0".into()),
        ("--run-dir", run_directory.to_owned()),
        ("--budget", "2".into()),
        ("--genealogy-growth", "0.6".into()),
        ("--confidence", "99".into()),
        ("--max-concurrent-containers", "1".into()),
        ("--procreation-timeout", "30".into()),
        ("--assessment-timeout", "30".into()),
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
    options(directory, &directory.join("run"))
        .into_iter()
        .filter(|option| image_too || !option.starts_with("--docker-image="))
        .chain([
            format!("--dockerfile={}", dockerfile.display()),
            "--docker-build-timeout=60".to_owned(),
        ])
        .collect()
}

fn with_a_task_that_lacks_its_check(directory: &Path) -> Vec<String> {
    let options = options(directory, &directory.join("run"));
    fs::remove_file(directory.join("benchmark/t1/check")).unwrap();
    options
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_check_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_check(directory.path());
    assert_eq!(status(&options), Some(2));
}

#[test]
fn a_benchmark_with_a_task_that_lacks_its_check_is_refused_by_its_directory_and_the_id_of_the_task()
{
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_check(directory.path());
    assert!(said(&options).ends_with("/benchmark: task t1 holds no executable file named check\n"));
}

#[test]
fn a_benchmark_that_is_refused_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_task_that_lacks_its_check(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

fn with_its_benchmark_given_twice(directory: &Path) -> Vec<String> {
    let options = options(directory, &directory.join("run"));
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
    options(directory, &directory.join("run"))
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
fn quiet_which_is_no_flag_of_start_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_quiet(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

fn with_a_seed(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("run"))
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
fn a_seed_which_is_no_flag_of_start_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_seed(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

fn with_initial_procreations(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("run"))
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
fn initial_procreations_which_are_no_flag_of_start_leave_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_initial_procreations(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

fn with_the_limit_of_concurrent_containers_under_a_name_it_does_not_have(
    directory: &Path,
) -> Vec<String> {
    options(directory, &directory.join("run"))
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
fn the_limit_of_concurrent_containers_under_a_name_it_does_not_have_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options =
        with_the_limit_of_concurrent_containers_under_a_name_it_does_not_have(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

fn with_the_primogenitor_under_a_name_it_does_not_have(directory: &Path) -> Vec<String> {
    options(directory, &directory.join("run"))
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
fn the_primogenitor_under_a_name_it_does_not_have_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_the_primogenitor_under_a_name_it_does_not_have(directory.path());
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}

#[test]
fn a_missing_option_exits_with_2() {
    let directory = test_inputs::temporary_directory();
    let options = options(directory.path(), &directory.path().join("run"));
    assert_eq!(status(&options[..options.len() - 1]), Some(2));
}

#[test]
fn a_run_directory_at_which_something_exists_exits_with_2() {
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
        "error: the argument '--docker-image <IMAGE>' cannot be used with:\n  \
         --dockerfile <FILE>\n"
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
        "cladenc: --dockerfile takes an existing file\n"
    );
}

#[test]
fn a_dockerfile_that_is_refused_leaves_no_run_directory() {
    let directory = test_inputs::temporary_directory();
    let options = with_a_dockerfile(directory.path(), &directory.path().join("missing"), false);
    let _ = status(&options);
    assert!(!directory.path().join("run").exists());
}
