use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub fn temporary_directory() -> TempDir {
    tempfile::tempdir().expect("a temporary directory can be made")
}

pub fn progenitor(directory: &Path) -> PathBuf {
    let progenitor = directory.join("progenitor");
    let script = progenitor.join("harness.sh");
    fs::create_dir_all(progenitor.join("prompts")).expect("the progenitor can be made");
    fs::write(&script, "#!/bin/sh\necho harness\n").expect("the script can be written");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o750))
        .expect("the script can be given its permission bits");
    fs::write(progenitor.join("prompts/system.txt"), "Solve the task.\n")
        .expect("the prompt can be written");
    symlink("prompts/system.txt", progenitor.join("prompt.txt"))
        .expect("the symbolic link can be made");
    progenitor
}

pub fn sample(tier: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test")
        .join(tier)
        .join("samples")
        .join(name)
}
