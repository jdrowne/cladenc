use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

pub(super) fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
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

pub(crate) fn make_removable(tree: &Path) {
    if let Ok(found) = fs::symlink_metadata(tree)
        && found.is_dir()
    {
        let mut permissions = found.permissions();
        permissions.set_mode(permissions.mode() | 0o700);
        let _ = fs::set_permissions(tree, permissions);
        for entry in fs::read_dir(tree).into_iter().flatten().flatten() {
            make_removable(&entry.path());
        }
    }
}

pub(crate) fn holds_only_harness_files(tree: &Path) -> io::Result<bool> {
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

pub(crate) fn same_tree(one: &Path, other: &Path) -> io::Result<bool> {
    let found = fs::symlink_metadata(one)?;
    let other_found = fs::symlink_metadata(other)?;
    let kind = found.file_type();
    if kind != other_found.file_type() {
        Ok(false)
    } else if kind.is_dir() {
        same_entries(one, other)
    } else if !kind.is_file() {
        Ok(fs::read_link(one).ok() == fs::read_link(other).ok())
    } else if found.permissions() != other_found.permissions() || found.len() != other_found.len() {
        Ok(false)
    } else {
        fs::read(one).and_then(|contents| fs::read(other).map(|others| contents == others))
    }
}

fn same_entries(one: &Path, other: &Path) -> io::Result<bool> {
    names_in(one)
        .and_then(|names| names_in(other).map(|other_names| (names, other_names)))
        .and_then(|(names, other_names)| {
            if names == other_names {
                names
                    .iter()
                    .map(|name| same_tree(&one.join(name), &other.join(name)))
                    .find(|same| !matches!(same, Ok(true)))
                    .unwrap_or(Ok(true))
            } else {
                Ok(false)
            }
        })
}

fn names_in(directory: &Path) -> io::Result<BTreeSet<OsString>> {
    fs::read_dir(directory).and_then(|entries| {
        entries
            .map(|entry| entry.map(|found| found.file_name()))
            .collect()
    })
}
