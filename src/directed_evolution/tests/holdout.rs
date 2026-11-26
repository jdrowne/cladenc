use crate::dataset::Dataset;
use crate::directed_evolution;
use crate::file_system::tests::double::Memory;
use crate::journal::Settings;

fn settings(directory: &Memory) -> Settings {
    Settings {
        primogenitor: test_inputs::primogenitor_in(directory, directory.path()),
        model: test_inputs::MODEL.to_owned(),
        model_url: test_inputs::MODEL_URL.to_owned(),
        model_key_file: None,
        datasets: vec![
            Dataset::read(
                directory,
                &test_inputs::dataset_in(directory, directory.path(), &["t1", "t2", "t3", "t4"]),
            )
            .unwrap(),
        ],
        docker_image: test_inputs::IMAGE.to_owned(),
        dockerfile: None,
        docker_build_timeout: None,
        harness_build_command: None,
        harness_run_command: "run a harness".to_owned(),
        max_diversifications: 1,
        max_concurrent_sandboxes: 1,
        diversification_timeout: 30,
        max_verifier_health_check_retries: 0,
        max_task_health_check_retries: 0,
        selection_tasks_percentage: 50.0,
        holdout_tasks: vec![
            "dataset/t1".to_owned(),
            "dataset/t3".to_owned(),
            "dataset/t4".to_owned(),
        ],
        corroboration_tasks_percentage: 25.0,
        corroboration_tasks: vec!["dataset/t4".to_owned()],
        environment_image_repository: "cladenc-environments-0123456789abcdef".to_owned(),
        seed: 7,
    }
}

fn said(settings: &Settings) -> String {
    let mut written: Vec<u8> = Vec::new();
    directed_evolution::say_the_holdout(settings, &mut written);
    String::from_utf8(written).unwrap()
}

#[test]
fn the_holdout_of_a_dataset_is_said_with_its_selection_and_its_corroboration_tasks_counted() {
    let directory = Memory::default();
    let settings = settings(&directory);
    assert_eq!(
        said(&settings),
        format!(
            "cladenc: holding out 3 of the 4 tasks of the dataset {} from every progenitor: 2 \
             selection tasks and 1 corroboration task\n",
            settings.datasets[0].directory.display()
        )
    );
}

#[test]
fn the_holdout_is_said_for_each_dataset_in_turn_one_with_no_holdout_task_among_them() {
    let directory = Memory::default();
    let mut settings = settings(&directory);
    let other = test_inputs::dataset_named_in(&directory, directory.path(), "other", &["t1"]);
    settings
        .datasets
        .push(Dataset::read(&directory, &other).unwrap());
    let written = said(&settings);
    let of_the_other = format!(
        "cladenc: holding out 0 of the 1 task of the dataset {} from every progenitor: 0 \
         selection tasks and 0 corroboration tasks",
        settings.datasets[1].directory.display()
    );
    assert_eq!(written.lines().last(), Some(of_the_other.as_str()));
}
