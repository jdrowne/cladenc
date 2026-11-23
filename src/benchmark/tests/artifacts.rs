use crate::benchmark::Artifact;
use crate::benchmark::configuration::{self, ConfigurationError};
use crate::file_system::tests::double::Memory;

const A_DIRECTORY: &str = "[[artifacts]]\nsource = \"/app/out\"\n";

fn read(written: &str) -> Result<Vec<Artifact>, ConfigurationError> {
    let directory = Memory::default();
    directory
        .write(directory.path().join("task.toml"), written)
        .unwrap();
    configuration::read_with(&directory, directory.path(), &|_| None)
        .map(|configuration| configuration.artifacts)
}

fn sources(written: &str) -> Vec<String> {
    read(written)
        .unwrap()
        .into_iter()
        .map(|artifact| artifact.source)
        .collect()
}

fn places(written: &str) -> Vec<String> {
    read(written)
        .unwrap()
        .into_iter()
        .map(|artifact| artifact.destination)
        .collect()
}

fn left_out_of(written: &str) -> Vec<Vec<String>> {
    read(written)
        .unwrap()
        .into_iter()
        .map(|artifact| artifact.exclude)
        .collect()
}

fn refused(written: &str) -> Option<ConfigurationError> {
    read(written).err()
}

fn not_a_place(destination: &str) -> Option<ConfigurationError> {
    Some(ConfigurationError::NotAPlace {
        destination: destination.to_owned(),
    })
}

#[test]
fn a_task_that_lists_no_artifacts_has_none() {
    assert_eq!(read(""), Ok(Vec::new()));
}

#[test]
fn the_path_of_an_artifact_named_by_its_path_alone_is_read() {
    assert_eq!(sources("artifacts = [\"/app/out\"]\n"), ["/app/out"]);
}

#[test]
fn an_artifact_named_by_its_path_alone_is_kept_at_its_path_without_the_leading_slash() {
    assert_eq!(places("artifacts = [\"/app/out\"]\n"), ["app/out"]);
}

#[test]
fn an_artifact_named_by_its_path_alone_has_nothing_left_out() {
    assert_eq!(
        left_out_of("artifacts = [\"/app/out\"]\n"),
        [Vec::<String>::new()]
    );
}

#[test]
fn the_path_of_an_artifact_written_as_a_table_is_read_from_its_source() {
    assert_eq!(sources(A_DIRECTORY), ["/app/out"]);
}

#[test]
fn an_artifact_that_gives_no_destination_is_kept_at_its_path_without_the_leading_slash() {
    assert_eq!(places(A_DIRECTORY), ["app/out"]);
}

#[test]
fn an_artifact_is_kept_at_the_destination_it_gives() {
    assert_eq!(
        places(&format!("{A_DIRECTORY}destination = \"kept/results\"\n")),
        ["kept/results"]
    );
}

#[test]
fn a_destination_written_with_nothing_in_it_is_none() {
    assert_eq!(
        places(&format!("{A_DIRECTORY}destination = \"\"\n")),
        ["app/out"]
    );
}

#[test]
fn a_destination_is_read_without_the_parts_that_name_nothing() {
    assert_eq!(
        places(&format!(
            "{A_DIRECTORY}destination = \"./kept//results/\"\n"
        )),
        ["kept/results"]
    );
}

#[test]
fn the_place_of_a_path_is_read_without_the_parts_that_name_nothing() {
    assert_eq!(
        places("artifacts = [\"/app//out/./logs/\"]\n"),
        ["app/out/logs"]
    );
}

#[test]
fn the_top_of_the_filesystem_is_kept_at_no_place_below_the_artifacts() {
    assert_eq!(places("artifacts = [\"/\"]\n"), [""]);
}

#[test]
fn what_an_artifact_leaves_out_is_read() {
    assert_eq!(
        left_out_of(&format!("{A_DIRECTORY}exclude = [\"*.tmp\", \"cache\"]\n")),
        [["*.tmp", "cache"]]
    );
}

#[test]
fn a_path_that_steps_back_is_refused() {
    assert_eq!(
        refused("artifacts = [\"/app/../etc/passwd\"]\n"),
        Some(ConfigurationError::StepsBack {
            path: "/app/../etc/passwd".to_owned()
        })
    );
}

#[test]
fn a_path_with_two_dots_inside_a_name_is_taken() {
    assert_eq!(
        places("artifacts = [\"/app/..hidden/a..b\"]\n"),
        ["app/..hidden/a..b"]
    );
}

#[test]
fn a_destination_that_starts_at_the_top_of_the_filesystem_is_refused() {
    assert_eq!(
        refused(&format!("{A_DIRECTORY}destination = \"/srv/kept\"\n")),
        not_a_place("/srv/kept")
    );
}

#[test]
fn a_destination_that_steps_back_is_refused() {
    assert_eq!(
        refused(&format!("{A_DIRECTORY}destination = \"kept/../../out\"\n")),
        not_a_place("kept/../../out")
    );
}

#[test]
fn a_destination_that_names_nothing_is_refused() {
    assert_eq!(
        refused(&format!("{A_DIRECTORY}destination = \"./\"\n")),
        not_a_place("./")
    );
}

#[test]
fn an_artifact_of_a_container_other_than_that_of_the_harness_is_refused() {
    assert_eq!(
        refused(&format!("{A_DIRECTORY}service = \"api\"\n")),
        Some(ConfigurationError::Needs {
            what: "a path of a container other than that of the harness"
        })
    );
}

#[test]
fn an_artifact_of_the_container_of_the_harness_named_as_harbor_names_it_is_taken() {
    assert_eq!(
        sources(&format!("{A_DIRECTORY}service = \"main\"\n")),
        ["/app/out"]
    );
}

#[test]
fn a_service_written_with_nothing_in_it_is_none() {
    assert_eq!(
        sources(&format!("{A_DIRECTORY}service = \"\"\n")),
        ["/app/out"]
    );
}

#[test]
fn artifacts_kept_at_places_apart_are_read_in_the_order_they_are_listed_in() {
    assert_eq!(
        sources("artifacts = [\"/srv/b\", \"/app/a\", \"/srv/c\"]\n"),
        ["/srv/b", "/app/a", "/srv/c"]
    );
}

#[test]
fn of_two_artifacts_kept_at_the_same_place_the_first_is_read() {
    assert_eq!(
        sources(
            "artifacts = [\n  { source = \"/app/out\", destination = \"kept\" },\n  { source = \
             \"/srv/out\", destination = \"kept\" },\n]\n"
        ),
        ["/app/out"]
    );
}

#[test]
fn an_artifact_kept_inside_an_earlier_one_is_not_read() {
    assert_eq!(
        sources("artifacts = [\"/app\", \"/app/out/report.json\"]\n"),
        ["/app"]
    );
}

#[test]
fn an_artifact_that_an_earlier_one_would_be_kept_inside_is_not_read() {
    assert_eq!(
        sources("artifacts = [\"/app/out/report.json\", \"/app\"]\n"),
        ["/app/out/report.json"]
    );
}

#[test]
fn an_artifact_whose_place_only_begins_as_the_name_of_an_earlier_one_does_is_read() {
    assert_eq!(
        sources("artifacts = [\"/app/out\", \"/app/output\"]\n"),
        ["/app/out", "/app/output"]
    );
}

#[test]
fn an_artifact_listed_after_one_that_is_not_read_is_read() {
    assert_eq!(
        sources("artifacts = [\"/app\", \"/app/out\", \"/srv/logs\"]\n"),
        ["/app", "/srv/logs"]
    );
}

#[test]
fn nothing_else_is_kept_once_the_top_of_the_filesystem_is() {
    assert_eq!(sources("artifacts = [\"/\", \"/app/out\"]\n"), ["/"]);
}
