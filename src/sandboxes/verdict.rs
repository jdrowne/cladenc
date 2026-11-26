use std::path::Path;

use serde_json::{Map, Value};

use crate::file_system::{FileSystem, Kind};
use crate::genealogy::TrialResult;

const VERDICTS: &str = "reward.json";
const VERDICT: &str = "reward.txt";

pub(super) fn result(files: &dyn FileSystem, verifier: &Path) -> TrialResult {
    let labelled = verifier.join(VERDICTS);
    let verdicts = if files.found(&labelled).is_ok() {
        read(files, &labelled).and_then(|text| of_each_label(&text))
    } else {
        read(files, &verifier.join(VERDICT)).and_then(|text| single(&text))
    };
    match verdicts {
        None => TrialResult::NoVerdict,
        Some(verdicts) if verdicts.iter().all(|verdict| *verdict == 1.0) => TrialResult::Completed,
        Some(_) => TrialResult::NotCompleted,
    }
}

fn of_each_label(text: &str) -> Option<Vec<f64>> {
    serde_json::from_str::<Map<String, Value>>(text)
        .ok()
        .filter(|verdicts| !verdicts.is_empty())
        .and_then(|verdicts| verdicts.values().map(Value::as_f64).collect())
}

fn single(text: &str) -> Option<Vec<f64>> {
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|verdict| verdict.is_finite())
        .map(|verdict| vec![verdict])
}

fn read(files: &dyn FileSystem, file: &Path) -> Option<String> {
    files
        .found(file)
        .ok()
        .filter(|found| found.kind == Kind::File)
        .and_then(|_| files.read_text(file).ok())
}
