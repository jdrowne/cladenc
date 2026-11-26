use crate::file_system::tests::double::Memory;
use crate::sandboxes::reward;

fn completed_on(files: &[(&str, &str)]) -> bool {
    let directory = Memory::default();
    for (name, text) in files {
        directory.write(directory.path().join(name), text).unwrap();
    }
    reward::completed(&directory, directory.path())
}

#[test]
fn a_score_of_1_in_reward_txt_is_completed() {
    assert!(completed_on(&[("reward.txt", "1\n")]));
}

#[test]
fn a_score_of_1_written_with_a_decimal_point_is_completed() {
    assert!(completed_on(&[("reward.txt", "1.0")]));
}

#[test]
fn a_score_of_0_is_not_completed() {
    assert!(!completed_on(&[("reward.txt", "0\n")]));
}

#[test]
fn a_score_between_0_and_1_is_not_completed() {
    assert!(!completed_on(&[("reward.txt", "0.5\n")]));
}

#[test]
fn a_reward_txt_that_holds_no_number_is_not_completed() {
    assert!(!completed_on(&[("reward.txt", "passed\n")]));
}

#[test]
fn a_reward_txt_that_holds_two_numbers_is_not_completed() {
    assert!(!completed_on(&[("reward.txt", "1\n1\n")]));
}

#[test]
fn a_verifier_that_left_no_score_did_not_complete_the_task() {
    assert!(!completed_on(&[]));
}

#[test]
fn a_reward_txt_that_is_a_directory_is_not_completed() {
    let directory = Memory::default();
    directory
        .create_dir(directory.path().join("reward.txt"))
        .unwrap();
    assert!(!reward::completed(&directory, directory.path()));
}

#[test]
fn a_reward_txt_that_is_a_symbolic_link_to_a_score_of_1_is_not_completed() {
    let directory = Memory::default();
    directory
        .write(directory.path().join("score"), "1\n")
        .unwrap();
    directory
        .symlink("score", directory.path().join("reward.txt"))
        .unwrap();
    assert!(!reward::completed(&directory, directory.path()));
}

#[test]
fn a_reward_json_whose_every_label_scores_1_is_completed() {
    assert!(completed_on(&[(
        "reward.json",
        r#"{"tests": 1, "style": 1.0}"#
    )]));
}

#[test]
fn a_reward_json_with_a_label_that_scores_less_than_1_is_not_completed() {
    assert!(!completed_on(&[(
        "reward.json",
        r#"{"tests": 1, "style": 0.5}"#
    )]));
}

#[test]
fn a_reward_json_that_holds_no_label_is_not_completed() {
    assert!(!completed_on(&[("reward.json", "{}")]));
}

#[test]
fn a_reward_json_whose_score_is_not_a_number_is_not_completed() {
    assert!(!completed_on(&[("reward.json", r#"{"tests": "1"}"#)]));
}

#[test]
fn a_reward_json_that_is_a_list_of_scores_is_not_completed() {
    assert!(!completed_on(&[("reward.json", "[1]")]));
}

#[test]
fn a_reward_json_that_is_a_bare_number_is_not_completed() {
    assert!(!completed_on(&[("reward.json", "1")]));
}

#[test]
fn reward_json_is_read_in_place_of_a_reward_txt_that_scores_0() {
    assert!(completed_on(&[
        ("reward.txt", "0\n"),
        ("reward.json", r#"{"tests": 1}"#)
    ]));
}

#[test]
fn a_reward_json_that_cannot_be_read_is_not_made_up_for_by_a_reward_txt_that_scores_1() {
    assert!(!completed_on(&[
        ("reward.txt", "1\n"),
        ("reward.json", "{\"tests\": 1")
    ]));
}

#[test]
fn a_reward_json_that_is_a_symbolic_link_is_not_made_up_for_by_a_reward_txt() {
    let directory = Memory::default();
    directory
        .write(directory.path().join("reward.txt"), "1\n")
        .unwrap();
    directory
        .write(directory.path().join("scores"), r#"{"tests": 1}"#)
        .unwrap();
    directory
        .symlink("scores", directory.path().join("reward.json"))
        .unwrap();
    assert!(!reward::completed(&directory, directory.path()));
}
