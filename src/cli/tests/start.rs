use std::path::PathBuf;

use crate::cli::tests::given::{self, Ran};
use crate::cli::{Command, StartOptions};
use crate::containers::tests::double::Double;
use crate::file_system::tests::double::Memory;

fn ran(change: impl FnOnce(&Memory, &mut StartOptions)) -> Ran {
    let directory = Memory::default();
    let mut options = given::options(&directory);
    change(&directory, &mut options);
    let start = Command::Start(Box::new(options));
    given::executed(Double::default(), &directory, start)
}

fn outputs_directory_is_left_by(change: impl FnOnce(&Memory, &mut StartOptions)) -> bool {
    let directory = Memory::default();
    let mut options = given::options(&directory);
    change(&directory, &mut options);
    let outputs_directory = options.outputs_dir.clone();
    let start = Command::Start(Box::new(options));
    let _refused = given::executed(Double::default(), &directory, start);
    directory.exists(outputs_directory)
}

fn with_a_task_that_lacks_its_grader(directory: &Memory, options: &mut StartOptions) {
    directory
        .remove_dir_all(options.benchmark[0].join("t1/tests"))
        .unwrap();
}

fn with_percentages_that_together_hold_out_every_task(_: &Memory, options: &mut StartOptions) {
    options.selection_tasks_percentage = 50.0;
    options.corroboration_tasks_percentage = 25.0;
}

fn with_a_missing_benchmark(directory: &Memory, options: &mut StartOptions) {
    options.benchmark = vec![directory.path().join("missing")];
}

fn with_a_missing_file_of_the_key(directory: &Memory, options: &mut StartOptions) {
    options.model_key_file = Some(directory.path().join("missing"));
}

fn with_a_missing_dockerfile(directory: &Memory, options: &mut StartOptions) {
    options.procreation_docker_image = None;
    options.procreation_dockerfile = Some(directory.path().join("missing"));
    options.procreation_docker_image_build_timeout = Some(600);
}

fn with_a_percentage_that_leaves_no_selection_task(_: &Memory, options: &mut StartOptions) {
    options.selection_tasks_percentage = 10.0;
}

fn with_a_percentage_that_leaves_no_corroboration_task(_: &Memory, options: &mut StartOptions) {
    options.corroboration_tasks_percentage = 10.0;
}

fn in_an_outputs_directory_that_exists(directory: &Memory, options: &mut StartOptions) {
    options.outputs_dir = directory.path().to_owned();
}

fn in_an_outputs_directory_with_a_colon_in_its_path(
    directory: &Memory,
    options: &mut StartOptions,
) {
    options.outputs_dir = directory.path().join("first:second");
}

#[test]
fn a_directed_evolution_that_finishes_prints_where_its_best_harness_is_kept() {
    let finished = ran(|_, _| {});
    assert!(finished.output.ends_with("/outputs/harnesses/0\n"));
}

#[test]
fn a_directed_evolution_that_finishes_exits_with_zero() {
    assert_eq!(ran(|_, _| {}).status, 0);
}

#[test]
fn a_refused_value_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_benchmark).status, 2);
}

#[test]
fn a_refused_value_is_reported_on_standard_error() {
    assert!(
        ran(with_a_missing_benchmark)
            .errors
            .starts_with("cladenc: --benchmark takes ")
    );
}

#[test]
fn a_refused_benchmark_is_reported_with_what_is_wrong_with_it() {
    assert!(
        ran(with_a_missing_benchmark)
            .errors
            .ends_with(": what was given is not a directory that can be read\n")
    );
}

#[test]
fn a_refused_value_prints_nothing_on_standard_output() {
    assert_eq!(ran(with_a_missing_benchmark).output, "");
}

#[test]
fn a_file_of_the_key_that_holds_no_key_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_file_of_the_key).status, 2);
}

#[test]
fn a_file_of_the_key_that_holds_no_key_is_refused_for_what_the_flag_takes() {
    assert_eq!(
        ran(with_a_missing_file_of_the_key).errors,
        "cladenc: --model-key-file takes a file that holds a key on one line and nothing else\n"
    );
}

#[test]
fn a_dockerfile_that_does_not_exist_gives_exit_status_2() {
    assert_eq!(ran(with_a_missing_dockerfile).status, 2);
}

#[test]
fn a_dockerfile_that_does_not_exist_is_refused_for_what_the_flag_takes() {
    assert_eq!(
        ran(with_a_missing_dockerfile).errors,
        "cladenc: --procreation-dockerfile takes an existing file\n"
    );
}

#[test]
fn an_outputs_directory_at_which_something_exists_gives_exit_status_2() {
    assert_eq!(ran(in_an_outputs_directory_that_exists).status, 2);
}

#[test]
fn an_outputs_directory_at_which_something_exists_is_reported_last_on_standard_error() {
    assert!(
        ran(in_an_outputs_directory_that_exists)
            .errors
            .lines()
            .last()
            .is_some_and(|line| line.starts_with("cladenc: the outputs directory cannot be made: "))
    );
}

#[test]
fn an_outputs_directory_that_is_being_made_is_said_first_on_standard_error() {
    assert!(
        ran(in_an_outputs_directory_that_exists)
            .errors
            .starts_with("cladenc: making the outputs directory ")
    );
}

#[test]
fn an_outputs_directory_with_a_colon_in_its_path_gives_exit_status_2() {
    assert_eq!(
        ran(in_an_outputs_directory_with_a_colon_in_its_path).status,
        2
    );
}

#[test]
fn an_outputs_directory_with_a_colon_in_its_path_is_refused_for_what_docker_cannot_mount() {
    assert_eq!(
        ran(in_an_outputs_directory_with_a_colon_in_its_path).errors,
        "cladenc: --outputs-dir takes a path that holds no colon, because Docker cannot mount a \
         directory whose path holds one\n"
    );
}

#[test]
fn an_outputs_directory_with_no_path_gives_exit_status_2() {
    assert_eq!(
        ran(|_, options| options.outputs_dir = PathBuf::new()).status,
        2
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_selection_task_gives_exit_status_2() {
    assert_eq!(
        ran(with_a_percentage_that_leaves_no_selection_task).status,
        2
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_selection_task_is_refused_with_the_benchmark() {
    assert!(
        ran(with_a_percentage_that_leaves_no_selection_task)
            .errors
            .ends_with(
                ": the benchmark `benchmark` holds 3 tasks, and none of them would be a selection \
                 task\n"
            )
    );
}

#[test]
fn a_percentage_that_leaves_no_selection_task_is_refused_before_the_outputs_directory_is_made() {
    assert!(
        ran(with_a_percentage_that_leaves_no_selection_task)
            .errors
            .starts_with("cladenc: --selection-tasks-percentage takes ")
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_corroboration_task_gives_exit_status_2() {
    assert_eq!(
        ran(with_a_percentage_that_leaves_no_corroboration_task).status,
        2
    );
}

#[test]
fn a_percentage_that_leaves_a_benchmark_no_corroboration_task_is_refused_with_the_benchmark() {
    assert!(
        ran(with_a_percentage_that_leaves_no_corroboration_task)
            .errors
            .ends_with(
                ": the benchmark `benchmark` holds 3 tasks, and none of them would be a \
                 corroboration task\n"
            )
    );
}

#[test]
fn a_percentage_that_leaves_no_corroboration_task_is_refused_before_the_outputs_directory_is_made()
{
    assert!(
        ran(with_a_percentage_that_leaves_no_corroboration_task)
            .errors
            .starts_with("cladenc: --corroboration-tasks-percentage takes ")
    );
}

#[test]
fn a_benchmark_that_is_refused_leaves_no_outputs_directory() {
    assert!(!outputs_directory_is_left_by(
        with_a_task_that_lacks_its_grader
    ));
}

#[test]
fn a_dockerfile_that_is_refused_leaves_no_outputs_directory() {
    assert!(!outputs_directory_is_left_by(with_a_missing_dockerfile));
}

#[test]
fn a_selection_tasks_percentage_that_is_refused_leaves_no_outputs_directory() {
    assert!(!outputs_directory_is_left_by(
        with_a_percentage_that_leaves_no_selection_task
    ));
}

#[test]
fn a_corroboration_tasks_percentage_that_is_refused_leaves_no_outputs_directory() {
    assert!(!outputs_directory_is_left_by(
        with_a_percentage_that_leaves_no_corroboration_task
    ));
}

#[test]
fn percentages_that_together_hold_out_every_task_leave_no_outputs_directory() {
    assert!(!outputs_directory_is_left_by(
        with_percentages_that_together_hold_out_every_task
    ));
}

#[test]
fn a_directed_evolution_that_finishes_leaves_its_outputs_directory() {
    assert!(outputs_directory_is_left_by(|_, _| {}));
}

#[test]
fn every_line_of_standard_error_up_to_the_report_is_a_sentence_of_cladenc() {
    assert!(
        ran(|_, _| {})
            .errors
            .lines()
            .take_while(|line| !line.starts_with("model "))
            .all(|line| line.starts_with("cladenc: "))
    );
}
