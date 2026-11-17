use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::fs::symlink;
use std::path::Path;

pub fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    let kind = fs::symlink_metadata(from)?.file_type();
    if kind.is_dir() {
        fs::create_dir(to)
            .and_then(|()| fs::read_dir(from))
            .and_then(|mut entries| {
                entries.try_for_each(|entry| {
                    entry.and_then(|entry| copy_tree(&entry.path(), &to.join(entry.file_name())))
                })
            })
    } else if kind.is_file() {
        fs::copy(from, to).map(|_| ())
    } else if kind.is_symlink() {
        fs::read_link(from).and_then(|target| symlink(target, to))
    } else {
        Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!(
                "{} is not a regular file, a directory or a symbolic link",
                from.display()
            ),
        ))
    }
}

pub fn holds_only_harness_files(tree: &Path) -> io::Result<bool> {
    let kind = fs::symlink_metadata(tree)?.file_type();
    if kind.is_dir() {
        fs::read_dir(tree).and_then(|mut entries| {
            entries.try_fold(true, |only, entry| {
                entry
                    .and_then(|entry| holds_only_harness_files(&entry.path()))
                    .map(|inside| only && inside)
            })
        })
    } else {
        Ok(kind.is_file() || kind.is_symlink())
    }
}
