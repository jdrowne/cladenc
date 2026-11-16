use std::ffi::OsString;
use std::fs;
use std::io;
use std::process::{Command, Stdio};

use crate::workspace::Workspace;

pub(super) fn label(workspace: &Workspace) -> OsString {
    let run_directory = workspace.run_directory();
    let mut label = OsString::from("cladenc.search=");
    label.push(
        fs::canonicalize(run_directory)
            .as_deref()
            .unwrap_or(run_directory),
    );
    label
}

pub fn remove_left_behind(workspace: &Workspace) -> io::Result<u64> {
    let mut filter = OsString::from("label=");
    filter.push(label(workspace));
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

fn printed_by(mut docker: Command) -> io::Result<String> {
    let output = docker.stdin(Stdio::null()).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}
