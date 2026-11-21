use std::ffi::OsString;
use std::fs;
use std::io;
use std::process::{Command, Stdio};

use crate::outputs_directory::OutputsDirectory;

pub(super) fn label(outputs_directory: &OutputsDirectory) -> OsString {
    let path = outputs_directory.path();
    let mut label = OsString::from("cladenc.directed-evolution=");
    label.push(fs::canonicalize(path).as_deref().unwrap_or(path));
    label
}

pub fn remove_left_behind(outputs_directory: &OutputsDirectory) -> io::Result<u64> {
    let mut filter = OsString::from("label=");
    filter.push(label(outputs_directory));
    let mut list = Command::new("docker");
    list.args(["ps", "--all", "--quiet", "--filter"])
        .arg(filter);
    let listed = printed_by(list)?;
    let left: Vec<&str> = listed.lines().collect();
    if left.is_empty() {
        return Ok(0);
    }
    let mut remove = Command::new("docker");
    remove.args(["rm", "--force"]).args(&left);
    printed_by(remove).map(|_| left.len() as u64)
}

pub(super) fn printed_by(mut docker: Command) -> io::Result<String> {
    let output = docker.stdin(Stdio::null()).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}
