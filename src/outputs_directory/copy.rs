use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::file_system::{FileSystem, Kind};

pub(super) fn copy_tree(files: &dyn FileSystem, from: &Path, to: &Path) -> io::Result<()> {
    let kind = files.found(from)?.kind;
    if kind == Kind::Directory {
        files
            .make_directory(to)
            .and_then(|()| files.list(from))
            .and_then(|names| {
                names
                    .iter()
                    .try_for_each(|name| copy_tree(files, &from.join(name), &to.join(name)))
            })
    } else if kind == Kind::File {
        files.copy(from, to)
    } else if kind == Kind::SymbolicLink {
        files
            .target(from)
            .and_then(|target| files.link(&target, to))
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

pub(crate) fn make_removable(files: &dyn FileSystem, tree: &Path) {
    if let Ok(found) = files.found(tree)
        && found.kind == Kind::Directory
    {
        let _ = files.set_mode(tree, found.mode | 0o700);
        for name in files.list(tree).unwrap_or_default() {
            make_removable(files, &tree.join(name));
        }
    }
}

pub(crate) fn holds_only_harness_files(files: &dyn FileSystem, tree: &Path) -> io::Result<bool> {
    let kind = files.found(tree)?.kind;
    if kind == Kind::Directory {
        files.list(tree).and_then(|names| {
            names.iter().try_fold(true, |only, name| {
                holds_only_harness_files(files, &tree.join(name)).map(|inside| only && inside)
            })
        })
    } else {
        Ok(kind == Kind::File || kind == Kind::SymbolicLink)
    }
}

pub(crate) fn same_tree(files: &dyn FileSystem, one: &Path, other: &Path) -> io::Result<bool> {
    let found = files.found(one)?;
    let other_found = files.found(other)?;
    if found.kind != other_found.kind {
        Ok(false)
    } else if found.kind == Kind::Directory {
        same_entries(files, one, other)
    } else if found.kind != Kind::File {
        Ok(files.target(one).ok() == files.target(other).ok())
    } else if found.mode != other_found.mode || found.length != other_found.length {
        Ok(false)
    } else {
        files
            .read(one)
            .and_then(|contents| files.read(other).map(|others| contents == others))
    }
}

fn same_entries(files: &dyn FileSystem, one: &Path, other: &Path) -> io::Result<bool> {
    names_in(files, one)
        .and_then(|names| names_in(files, other).map(|other_names| (names, other_names)))
        .and_then(|(names, other_names)| {
            if names == other_names {
                names
                    .iter()
                    .map(|name| same_tree(files, &one.join(name), &other.join(name)))
                    .find(|same| !matches!(same, Ok(true)))
                    .unwrap_or(Ok(true))
            } else {
                Ok(false)
            }
        })
}

fn names_in(files: &dyn FileSystem, directory: &Path) -> io::Result<BTreeSet<OsString>> {
    files
        .list(directory)
        .map(|names| names.into_iter().collect())
}
