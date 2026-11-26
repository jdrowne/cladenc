use crate::dataset::Artifact;

fn listed(paths: &[(&str, &str)]) -> Vec<Artifact> {
    paths
        .iter()
        .map(|(source, destination)| Artifact {
            source: (*source).to_owned(),
            destination: (*destination).to_owned(),
            exclude: Vec::new(),
        })
        .collect()
}

fn published() -> Option<Artifact> {
    Some(Artifact {
        source: "/logs/artifacts".to_owned(),
        destination: "logs/artifacts".to_owned(),
        exclude: Vec::new(),
    })
}

#[test]
fn the_directory_every_harness_is_given_is_kept_where_the_task_lists_no_path() {
    assert_eq!(Artifact::published(&[]), published());
}

#[test]
fn the_directory_every_harness_is_given_is_kept_beside_the_paths_the_task_lists() {
    assert_eq!(
        Artifact::published(&listed(&[
            ("/app/out", "app/out"),
            ("/logs/verifier", "kept")
        ])),
        published()
    );
}

#[test]
fn a_task_that_lists_the_directory_itself_has_its_own_entry_obeyed() {
    assert_eq!(
        Artifact::published(&listed(&[("/logs/artifacts", "published")])),
        None
    );
}

#[test]
fn a_task_that_writes_the_directory_another_way_has_its_own_entry_obeyed() {
    assert_eq!(
        Artifact::published(&listed(&[("logs//artifacts/", "published")])),
        None
    );
}

#[test]
fn a_task_that_keeps_another_path_at_the_place_of_the_directory_has_that_entry_obeyed() {
    assert_eq!(
        Artifact::published(&listed(&[("/app/out", "logs/artifacts")])),
        None
    );
}

#[test]
fn a_task_that_keeps_a_path_above_the_place_of_the_directory_has_that_entry_obeyed() {
    assert_eq!(Artifact::published(&listed(&[("/logs", "logs")])), None);
}
