use std::path::Path;

use serde_json::{Map, Value};

use crate::file_system::{FileSystem, Kind};

const VERDICTS: &str = "reward.json";
const VERDICT: &str = "reward.txt";

pub(super) fn completed(files: &dyn FileSystem, verifier: &Path) -> bool {
    let verdicts = verifier.join(VERDICTS);
    if files.found(&verdicts).is_ok() {
        read(files, &verdicts)
            .and_then(|text| serde_json::from_str::<Map<String, Value>>(&text).ok())
            .is_some_and(|verdicts| {
                !verdicts.is_empty()
                    && verdicts
                        .values()
                        .all(|verdict| verdict.as_f64() == Some(1.0))
            })
    } else {
        read(files, &verifier.join(VERDICT))
            .is_some_and(|text| text.trim().parse::<f64>() == Ok(1.0))
    }
}

fn read(files: &dyn FileSystem, file: &Path) -> Option<String> {
    files
        .found(file)
        .ok()
        .filter(|found| found.kind == Kind::File)
        .and_then(|_| files.read_text(file).ok())
}
