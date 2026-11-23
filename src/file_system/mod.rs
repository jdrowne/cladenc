mod disk;

use std::ffi::OsString;
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};

pub use disk::Disk;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Directory,
    File,
    SymbolicLink,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Found {
    pub kind: Kind,
    pub mode: u32,
    pub length: u64,
}

pub trait FileSystem: Debug + Send + Sync {
    fn found(&self, path: &Path) -> io::Result<Found>;

    fn followed(&self, path: &Path) -> io::Result<Found>;

    fn resolved(&self, path: &Path) -> io::Result<PathBuf>;

    fn list(&self, directory: &Path) -> io::Result<Vec<OsString>>;

    fn read(&self, file: &Path) -> io::Result<Vec<u8>>;

    fn read_text(&self, file: &Path) -> io::Result<String>;

    fn target(&self, link: &Path) -> io::Result<PathBuf>;

    fn write(&self, file: &Path, contents: &[u8]) -> io::Result<()>;

    fn make_directory(&self, directory: &Path) -> io::Result<()>;

    fn make_directories(&self, directory: &Path) -> io::Result<()>;

    fn link(&self, target: &Path, link: &Path) -> io::Result<()>;

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()>;

    fn remove_file(&self, file: &Path) -> io::Result<()>;

    fn remove_directory(&self, directory: &Path) -> io::Result<()>;

    fn remove_tree(&self, directory: &Path) -> io::Result<()>;

    fn open(&self, file: &Path, make: bool) -> io::Result<Box<dyn Opened>>;
}

pub trait Opened: Debug + Send {
    fn lock(&mut self) -> bool;

    fn contents(&mut self) -> io::Result<Vec<u8>>;

    fn cut(&mut self, length: u64) -> io::Result<()>;

    fn append(&mut self, line: &[u8]) -> io::Result<()>;
}

#[cfg(test)]
pub(crate) mod tests;
