use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use super::SandboxError;
use super::program::{Ending, Limits};
use crate::file_system::FileSystem;
use crate::outputs_directory::OutputsDirectory;

pub trait Docker: Sync {
    fn run(
        &self,
        command: Command,
        input: &str,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, SandboxError>;

    fn detach(&self, command: Command, input: &str, said: &Path) -> Result<Ending, SandboxError>;

    fn print(&self, command: Command) -> io::Result<String>;
}

#[derive(Clone, Copy)]
pub struct Outside<'a> {
    pub docker: &'a dyn Docker,
    pub files: &'a Arc<dyn FileSystem>,
}

#[derive(Clone, Copy)]
pub struct Host<'a> {
    pub docker: &'a dyn Docker,
    pub outputs_directory: &'a OutputsDirectory,
}
