use std::path::PathBuf;

use crate::file_system::Kind;
use crate::genealogy::HarnessId;
use crate::outputs_directory::OutputsDirectory;

const LINE: &str = "/var/log/cladenc/line";
const CHILDREN: &str = "/var/log/cladenc/children";

pub(super) fn to(
    outputs_directory: &OutputsDirectory,
    progenitor: HarnessId,
    before: Option<HarnessId>,
) -> Vec<(PathBuf, String)> {
    let line = outputs_directory
        .line_of(progenitor)
        .into_iter()
        .map(|harness| (LINE, harness));
    let children = outputs_directory
        .children_of(progenitor)
        .into_iter()
        .filter(|child| before.is_none_or(|later| *child < later))
        .map(|harness| (CHILDREN, harness));
    line.chain(children)
        .flat_map(|(relation, harness)| {
            [
                (outputs_directory.harness(harness), "harness"),
                (outputs_directory.trials(harness), "trials"),
                (outputs_directory.procreation(harness), "procreation"),
                (outputs_directory.build(harness), "build"),
            ]
            .map(|(kept, name)| (kept, format!("{relation}/{harness}/{name}")))
        })
        .filter(|(kept, _)| {
            outputs_directory
                .files()
                .followed(kept)
                .is_ok_and(|found| found.kind == Kind::Directory)
        })
        .collect()
}
