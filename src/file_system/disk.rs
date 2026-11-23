use std::ffi::OsString;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::{FileSystem, Found, Kind, Opened};

const PERMISSION_BITS: u32 = 0o7777;

#[derive(Clone, Copy, Debug)]
pub struct Disk;

#[derive(Debug)]
struct OnDisk {
    file: File,
}

impl FileSystem for Disk {
    fn found(&self, path: &Path) -> io::Result<Found> {
        fs::symlink_metadata(path).map(|found| found_of(&found))
    }

    fn followed(&self, path: &Path) -> io::Result<Found> {
        fs::metadata(path).map(|found| found_of(&found))
    }

    fn resolved(&self, path: &Path) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    fn list(&self, directory: &Path) -> io::Result<Vec<OsString>> {
        fs::read_dir(directory).and_then(|entries| {
            entries
                .map(|entry| entry.map(|entry| entry.file_name()))
                .collect()
        })
    }

    fn read(&self, file: &Path) -> io::Result<Vec<u8>> {
        fs::read(file)
    }

    fn read_text(&self, file: &Path) -> io::Result<String> {
        fs::read_to_string(file)
    }

    fn target(&self, link: &Path) -> io::Result<PathBuf> {
        fs::read_link(link)
    }

    fn write(&self, file: &Path, contents: &[u8]) -> io::Result<()> {
        fs::write(file, contents)
    }

    fn make_directory(&self, directory: &Path) -> io::Result<()> {
        fs::create_dir(directory)
    }

    fn make_directories(&self, directory: &Path) -> io::Result<()> {
        fs::create_dir_all(directory)
    }

    fn link(&self, target: &Path, link: &Path) -> io::Result<()> {
        symlink(target, link)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::copy(from, to).map(drop)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn set_mode(&self, path: &Path, mode: u32) -> io::Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }

    fn remove_file(&self, file: &Path) -> io::Result<()> {
        fs::remove_file(file)
    }

    fn remove_directory(&self, directory: &Path) -> io::Result<()> {
        fs::remove_dir(directory)
    }

    fn remove_tree(&self, directory: &Path) -> io::Result<()> {
        fs::remove_dir_all(directory)
    }

    fn open(&self, file: &Path, make: bool) -> io::Result<Box<dyn Opened>> {
        OpenOptions::new()
            .read(true)
            .append(true)
            .create(make)
            .open(file)
            .map(|file| Box::new(OnDisk { file }) as Box<dyn Opened>)
    }
}

impl Opened for OnDisk {
    fn lock(&mut self) -> bool {
        File::try_lock(&self.file).is_ok()
    }

    fn contents(&mut self) -> io::Result<Vec<u8>> {
        let mut contents = Vec::new();
        File::read_to_end(&mut self.file, &mut contents).map(|_| contents)
    }

    fn cut(&mut self, length: u64) -> io::Result<()> {
        self.file.set_len(length)
    }

    fn append(&mut self, line: &[u8]) -> io::Result<()> {
        self.file
            .write_all(line)
            .and_then(|()| self.file.sync_all())
    }
}

impl Drop for OnDisk {
    fn drop(&mut self) {
        let _ = File::unlock(&self.file);
    }
}

fn found_of(found: &Metadata) -> Found {
    let kind = if found.is_dir() {
        Kind::Directory
    } else if found.is_file() {
        Kind::File
    } else if found.is_symlink() {
        Kind::SymbolicLink
    } else {
        Kind::Other
    };
    Found {
        kind,
        mode: found.permissions().mode() & PERMISSION_BITS,
        length: found.len(),
    }
}
