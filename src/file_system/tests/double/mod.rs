mod tree;

use std::ffi::OsString;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::file_system::{FileSystem, Found, Kind, Opened};

use tree::{Node, Tree};

const DIRECTORY: &str = "/directory";

#[derive(Clone, Debug)]
pub(crate) struct Memory {
    tree: Arc<Mutex<Tree>>,
}

#[derive(Debug)]
struct InMemory {
    tree: Arc<Mutex<Tree>>,
    file: PathBuf,
    locked: bool,
}

impl Default for Memory {
    fn default() -> Self {
        let memory = Self {
            tree: Arc::new(Mutex::new(Tree::default())),
        };
        memory.create_dir(DIRECTORY).unwrap();
        memory
    }
}

impl Memory {
    fn tree(&self) -> MutexGuard<'_, Tree> {
        self.tree.lock().unwrap()
    }

    pub(crate) fn shared(&self) -> Arc<dyn FileSystem> {
        Arc::new(self.clone())
    }

    pub(crate) fn path(&self) -> &'static Path {
        Path::new(DIRECTORY)
    }

    pub(crate) fn write(
        &self,
        file: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> io::Result<()> {
        self.tree().write(file.as_ref(), contents.as_ref())
    }

    pub(crate) fn read_to_string(&self, file: impl AsRef<Path>) -> io::Result<String> {
        self.read_text(file.as_ref())
    }

    pub(crate) fn create_dir(&self, directory: impl AsRef<Path>) -> io::Result<()> {
        self.tree().make_directory(directory.as_ref())
    }

    pub(crate) fn create_dir_all(&self, directory: impl AsRef<Path>) -> io::Result<()> {
        self.tree().make_directories(directory.as_ref())
    }

    pub(crate) fn symlink(
        &self,
        target: impl AsRef<Path>,
        link: impl AsRef<Path>,
    ) -> io::Result<()> {
        self.link(target.as_ref(), link.as_ref())
    }

    pub(crate) fn socket(&self, path: impl AsRef<Path>) -> io::Result<()> {
        self.tree().put(path.as_ref(), Node::Other)
    }

    pub(crate) fn read_link(&self, link: impl AsRef<Path>) -> io::Result<PathBuf> {
        self.target(link.as_ref())
    }

    pub(crate) fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        self.tree().rename(from.as_ref(), to.as_ref())
    }

    pub(crate) fn set_mode(&self, path: impl AsRef<Path>, mode: u32) -> io::Result<()> {
        self.tree().set_mode(path.as_ref(), mode)
    }

    pub(crate) fn mode(&self, path: impl AsRef<Path>) -> u32 {
        self.found(path.as_ref()).unwrap().mode
    }

    pub(crate) fn remove_file(&self, file: impl AsRef<Path>) -> io::Result<()> {
        self.tree().remove_file(file.as_ref())
    }

    pub(crate) fn remove_dir_all(&self, directory: impl AsRef<Path>) -> io::Result<()> {
        self.tree().remove_tree(directory.as_ref())
    }

    pub(crate) fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.followed(path.as_ref()).is_ok()
    }

    pub(crate) fn is_dir(&self, path: impl AsRef<Path>) -> bool {
        self.leads_to(path.as_ref(), Kind::Directory)
    }

    pub(crate) fn is_file(&self, path: impl AsRef<Path>) -> bool {
        self.leads_to(path.as_ref(), Kind::File)
    }

    fn leads_to(&self, path: &Path, kind: Kind) -> bool {
        self.followed(path).is_ok_and(|found| found.kind == kind)
    }

    pub(crate) fn names_in(&self, directory: impl AsRef<Path>) -> Vec<String> {
        self.list(directory.as_ref())
            .unwrap()
            .iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect()
    }
}

impl test_inputs::Files for Memory {
    fn make_directories(&self, directory: &Path) {
        self.create_dir_all(directory).unwrap();
    }

    fn write(&self, file: &Path, text: &str) {
        self.tree().write(file, text.as_bytes()).unwrap();
    }

    fn set_mode(&self, file: &Path, mode: u32) {
        self.tree().set_mode(file, mode).unwrap();
    }

    fn link(&self, target: &str, link: &Path) {
        self.symlink(target, link).unwrap();
    }
}

impl FileSystem for Memory {
    fn found(&self, path: &Path) -> io::Result<Found> {
        self.tree().found(path, false)
    }

    fn followed(&self, path: &Path) -> io::Result<Found> {
        self.tree().found(path, true)
    }

    fn resolved(&self, path: &Path) -> io::Result<PathBuf> {
        self.tree().resolved(path)
    }

    fn list(&self, directory: &Path) -> io::Result<Vec<OsString>> {
        self.tree().list(directory)
    }

    fn read(&self, file: &Path) -> io::Result<Vec<u8>> {
        self.tree().read(file)
    }

    fn read_text(&self, file: &Path) -> io::Result<String> {
        self.tree().read(file).and_then(|contents| {
            String::from_utf8(contents).map_err(|_| ErrorKind::InvalidData.into())
        })
    }

    fn target(&self, link: &Path) -> io::Result<PathBuf> {
        self.tree().target(link)
    }

    fn write(&self, file: &Path, contents: &[u8]) -> io::Result<()> {
        self.tree().write(file, contents)
    }

    fn make_directory(&self, directory: &Path) -> io::Result<()> {
        self.tree().make_directory(directory)
    }

    fn make_directories(&self, directory: &Path) -> io::Result<()> {
        self.tree().make_directories(directory)
    }

    fn link(&self, target: &Path, link: &Path) -> io::Result<()> {
        let target = target.to_owned();
        self.tree().put(link, Node::Link { target })
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.tree().copy(from, to)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.tree().rename(from, to)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        self.tree().set_mode(path, mode)
    }

    fn remove_file(&self, file: &Path) -> io::Result<()> {
        self.tree().remove_file(file)
    }

    fn remove_directory(&self, directory: &Path) -> io::Result<()> {
        self.tree().remove_directory(directory)
    }

    fn remove_tree(&self, directory: &Path) -> io::Result<()> {
        self.tree().remove_tree(directory)
    }

    fn open(&self, file: &Path, make: bool) -> io::Result<Box<dyn Opened>> {
        self.tree().open(file, make).map(|file| {
            Box::new(InMemory {
                tree: Arc::clone(&self.tree),
                file,
                locked: false,
            }) as Box<dyn Opened>
        })
    }
}

impl InMemory {
    fn tree(&self) -> MutexGuard<'_, Tree> {
        self.tree.lock().unwrap()
    }
}

impl Opened for InMemory {
    fn lock(&mut self) -> bool {
        let locked = self.locked || self.tree().lock(&self.file);
        self.locked = locked;
        locked
    }

    fn contents(&mut self) -> io::Result<Vec<u8>> {
        self.tree().read(&self.file)
    }

    fn cut(&mut self, length: u64) -> io::Result<()> {
        self.tree().cut(&self.file, length)
    }

    fn append(&mut self, line: &[u8]) -> io::Result<()> {
        self.tree().append(&self.file, line)
    }
}

impl Drop for InMemory {
    fn drop(&mut self) {
        if self.locked {
            self.tree().unlock(&self.file);
        }
    }
}
