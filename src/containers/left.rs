use std::ffi::OsString;
use std::fs;
use std::io;
use std::process::Command;

use super::docker::Host;
use crate::outputs_directory::OutputsDirectory;

pub(super) fn label(outputs_directory: &OutputsDirectory) -> OsString {
    let path = outputs_directory.path();
    let mut label = OsString::from("cladenc.directed-evolution=");
    label.push(fs::canonicalize(path).as_deref().unwrap_or(path));
    label
}

pub fn remove_left_behind(host: Host<'_>) -> io::Result<u64> {
    let mut filter = OsString::from("label=");
    filter.push(label(host.outputs_directory));
    let mut list = Command::new("docker");
    list.args(["ps", "--all", "--quiet", "--filter"])
        .arg(filter);
    let listed = host.docker.print(list)?;
    let left: Vec<&str> = listed.lines().collect();
    if left.is_empty() {
        return Ok(0);
    }
    let mut remove = Command::new("docker");
    remove.args(["rm", "--force"]).args(&left);
    host.docker.print(remove).map(|_| left.len() as u64)
}
