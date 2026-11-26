use std::path::Path;

use serde_json::{Map, Value};

use crate::file_system::{FileSystem, Kind};

const SCORES: &str = "reward.json";
const SCORE: &str = "reward.txt";

pub(super) fn completed(files: &dyn FileSystem, verifier: &Path) -> bool {
    let scores = verifier.join(SCORES);
    if files.found(&scores).is_ok() {
        read(files, &scores)
            .and_then(|text| serde_json::from_str::<Map<String, Value>>(&text).ok())
            .is_some_and(|scores| {
                !scores.is_empty() && scores.values().all(|score| score.as_f64() == Some(1.0))
            })
    } else {
        read(files, &verifier.join(SCORE)).is_some_and(|text| text.trim().parse::<f64>() == Ok(1.0))
    }
}

fn read(files: &dyn FileSystem, file: &Path) -> Option<String> {
    files
        .found(file)
        .ok()
        .filter(|found| found.kind == Kind::File)
        .and_then(|_| files.read_text(file).ok())
}
