use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

const SCORES: &str = "reward.json";
const SCORE: &str = "reward.txt";

pub(super) fn completed(verifier: &Path) -> bool {
    let scores = verifier.join(SCORES);
    if fs::symlink_metadata(&scores).is_ok() {
        read(&scores)
            .and_then(|text| serde_json::from_str::<Map<String, Value>>(&text).ok())
            .is_some_and(|scores| {
                !scores.is_empty() && scores.values().all(|score| score.as_f64() == Some(1.0))
            })
    } else {
        read(&verifier.join(SCORE)).is_some_and(|text| text.trim().parse::<f64>() == Ok(1.0))
    }
}

fn read(file: &Path) -> Option<String> {
    fs::symlink_metadata(file)
        .ok()
        .filter(|metadata| metadata.is_file())
        .and_then(|_| fs::read_to_string(file).ok())
}
