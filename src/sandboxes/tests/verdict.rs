use crate::file_system::tests::double::Memory;
use crate::genealogy::{Cause, TrialResult};
use crate::sandboxes::verdict;

fn result_on(files: &[(&str, &str)]) -> TrialResult {
    let directory = Memory::default();
    for (name, text) in files {
        directory.write(directory.path().join(name), text).unwrap();
    }
    verdict::result(&directory, directory.path())
}

#[test]
fn a_verdict_of_1_in_reward_txt_is_a_pass() {
    assert_eq!(result_on(&[("reward.txt", "1\n")]), TrialResult::Pass);
}

#[test]
fn a_verdict_of_1_written_with_a_decimal_point_is_a_pass() {
    assert_eq!(result_on(&[("reward.txt", "1.0")]), TrialResult::Pass);
}

#[test]
fn a_verdict_of_0_is_a_fail() {
    assert_eq!(result_on(&[("reward.txt", "0\n")]), TrialResult::Fail);
}

#[test]
fn a_verdict_between_0_and_1_is_a_fail() {
    assert_eq!(result_on(&[("reward.txt", "0.5\n")]), TrialResult::Fail);
}

#[test]
fn a_verdict_above_1_is_a_fail() {
    assert_eq!(result_on(&[("reward.txt", "2\n")]), TrialResult::Fail);
}

#[test]
fn a_reward_txt_that_holds_no_number_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.txt", "passed\n")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_txt_that_holds_two_numbers_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.txt", "1\n1\n")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_txt_that_holds_nothing_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.txt", "")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_txt_that_reads_nan_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.txt", "nan\n")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_verifier_that_left_no_file_left_no_verdict() {
    assert_eq!(result_on(&[]), TrialResult::Inconclusive(Cause::NoVerdict));
}

#[test]
fn a_reward_txt_that_is_a_directory_is_no_verdict() {
    let directory = Memory::default();
    directory
        .create_dir(directory.path().join("reward.txt"))
        .unwrap();
    assert_eq!(
        verdict::result(&directory, directory.path()),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_txt_that_is_a_symbolic_link_to_a_verdict_of_1_is_no_verdict() {
    let directory = Memory::default();
    directory
        .write(directory.path().join("verdict"), "1\n")
        .unwrap();
    directory
        .symlink("verdict", directory.path().join("reward.txt"))
        .unwrap();
    assert_eq!(
        verdict::result(&directory, directory.path()),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_json_whose_every_verdict_is_1_is_a_pass() {
    assert_eq!(
        result_on(&[("reward.json", r#"{"tests": 1, "style": 1.0}"#)]),
        TrialResult::Pass
    );
}

#[test]
fn a_reward_json_with_a_verdict_of_less_than_1_is_a_fail() {
    assert_eq!(
        result_on(&[("reward.json", r#"{"tests": 1, "style": 0.5}"#)]),
        TrialResult::Fail
    );
}

#[test]
fn a_reward_json_that_holds_no_label_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.json", "{}")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_json_whose_verdict_is_not_a_number_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.json", r#"{"tests": "1"}"#)]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_json_with_a_verdict_of_0_beside_one_that_is_not_a_number_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.json", r#"{"tests": 0, "style": "good"}"#)]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_json_that_is_a_list_of_verdicts_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.json", "[1]")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_json_that_is_a_bare_number_is_no_verdict() {
    assert_eq!(
        result_on(&[("reward.json", "1")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn reward_json_is_read_in_place_of_a_reward_txt_with_a_verdict_of_0() {
    assert_eq!(
        result_on(&[("reward.txt", "0\n"), ("reward.json", r#"{"tests": 1}"#)]),
        TrialResult::Pass
    );
}

#[test]
fn a_reward_json_that_cannot_be_read_is_not_made_up_for_by_a_reward_txt_with_a_verdict_of_1() {
    assert_eq!(
        result_on(&[("reward.txt", "1\n"), ("reward.json", "{\"tests\": 1")]),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}

#[test]
fn a_reward_json_that_is_a_symbolic_link_is_not_made_up_for_by_a_reward_txt() {
    let directory = Memory::default();
    directory
        .write(directory.path().join("reward.txt"), "1\n")
        .unwrap();
    directory
        .write(directory.path().join("verdicts"), r#"{"tests": 1}"#)
        .unwrap();
    directory
        .symlink("verdicts", directory.path().join("reward.json"))
        .unwrap();
    assert_eq!(
        verdict::result(&directory, directory.path()),
        TrialResult::Inconclusive(Cause::NoVerdict)
    );
}
