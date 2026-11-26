use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

use nix::errno::Errno;

use crate::file_system::{Found, Kind};

const ROOT: &str = "/";
const LINKS_FOLLOWED_AT_MOST: usize = 40;
const DIRECTORY_MODE: u32 = 0o755;
const FILE_MODE: u32 = 0o644;
const LINK_MODE: u32 = 0o777;
const PERMISSION_BITS: u32 = 0o7777;
const READ_BY_ITS_OWNER: u32 = 0o400;

#[derive(Clone, Debug)]
pub(super) enum Node {
    Directory { mode: u32 },
    File { mode: u32, contents: Vec<u8> },
    Link { target: PathBuf },
    Other,
}

#[derive(Debug)]
pub(super) struct Tree {
    nodes: BTreeMap<PathBuf, Node>,
    locked: BTreeSet<PathBuf>,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::from([(
                PathBuf::from(ROOT),
                Node::Directory {
                    mode: DIRECTORY_MODE,
                },
            )]),
            locked: BTreeSet::new(),
        }
    }
}

impl Tree {
    fn at(&self, path: &Path, follow: bool) -> io::Result<PathBuf> {
        let mut left = parts_of(path);
        let mut reached = PathBuf::from(ROOT);
        let mut links = LINKS_FOLLOWED_AT_MOST;
        while let Some(part) = left.pop() {
            if part == ".." {
                reached.pop();
                continue;
            }
            self.entered(&reached)?;
            reached.push(part);
            let target = self
                .target_at(&reached)
                .filter(|_| follow || !left.is_empty());
            if let Some(target) = target {
                links = links.checked_sub(1).ok_or_else(too_many_links)?;
                reached = above(&reached, &target);
                left.extend(parts_of(&target));
            }
        }
        Ok(reached)
    }

    fn entered(&self, directory: &Path) -> io::Result<()> {
        match self.nodes.get(directory) {
            Some(Node::Directory { .. }) => Ok(()),
            Some(_) => Err(Errno::ENOTDIR.into()),
            None => Err(Errno::ENOENT.into()),
        }
    }

    fn target_at(&self, path: &Path) -> Option<PathBuf> {
        match self.nodes.get(path) {
            Some(Node::Link { target }) => Some(target.clone()),
            _ => None,
        }
    }

    fn node(&self, path: &Path, follow: bool) -> io::Result<(PathBuf, &Node)> {
        let at = self.at(path, follow)?;
        match self.nodes.get(&at) {
            Some(node) => Ok((at, node)),
            None => Err(Errno::ENOENT.into()),
        }
    }

    fn free(&self, path: &Path) -> io::Result<PathBuf> {
        let at = self.at(path, false)?;
        if self.nodes.contains_key(&at) {
            Err(Errno::EEXIST.into())
        } else {
            Ok(at)
        }
    }

    fn inside(&self, directory: &Path) -> Vec<PathBuf> {
        self.nodes
            .keys()
            .filter(|path| path.starts_with(directory))
            .cloned()
            .collect()
    }

    fn names(&self, directory: &Path) -> Vec<OsString> {
        self.nodes
            .keys()
            .filter(|path| path.parent() == Some(directory))
            .filter_map(|path| path.file_name())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(super) fn found(&self, path: &Path, follow: bool) -> io::Result<Found> {
        self.node(path, follow).map(|(_, node)| found_of(node))
    }

    pub(super) fn resolved(&self, path: &Path) -> io::Result<PathBuf> {
        self.node(path, true).map(|(at, _)| at)
    }

    pub(super) fn list(&self, directory: &Path) -> io::Result<Vec<OsString>> {
        let (at, node) = self.node(directory, true)?;
        match node {
            Node::Directory { mode } if mode & READ_BY_ITS_OWNER == 0 => Err(Errno::EACCES.into()),
            _ => self.entered(&at).map(|()| self.names(&at)),
        }
    }

    pub(super) fn read(&self, file: &Path) -> io::Result<Vec<u8>> {
        match self.node(file, true)?.1 {
            Node::File { mode, .. } if mode & READ_BY_ITS_OWNER == 0 => Err(Errno::EACCES.into()),
            Node::File { contents, .. } => Ok(contents.clone()),
            Node::Directory { .. } => Err(Errno::EISDIR.into()),
            Node::Link { .. } | Node::Other => Err(Errno::EINVAL.into()),
        }
    }

    pub(super) fn target(&self, link: &Path) -> io::Result<PathBuf> {
        match self.node(link, false)?.1 {
            Node::Link { target } => Ok(target.clone()),
            _ => Err(Errno::EINVAL.into()),
        }
    }

    pub(super) fn write(&mut self, file: &Path, contents: &[u8]) -> io::Result<()> {
        let at = self.at(file, true)?;
        match self.nodes.get_mut(&at) {
            Some(Node::File { contents: kept, .. }) => {
                contents.clone_into(kept);
                Ok(())
            }
            Some(Node::Directory { .. }) => Err(Errno::EISDIR.into()),
            Some(_) => Err(Errno::EINVAL.into()),
            None => {
                let contents = contents.to_owned();
                let mode = FILE_MODE;
                self.nodes.insert(at, Node::File { mode, contents });
                Ok(())
            }
        }
    }

    pub(super) fn append(&mut self, file: &Path, line: &[u8]) -> io::Result<()> {
        self.read(file).and_then(|mut contents| {
            contents.extend_from_slice(line);
            self.write(file, &contents)
        })
    }

    pub(super) fn cut(&mut self, file: &Path, length: u64) -> io::Result<()> {
        self.read(file).and_then(|mut contents| {
            contents.truncate(usize::try_from(length).unwrap_or(usize::MAX));
            self.write(file, &contents)
        })
    }

    pub(super) fn put(&mut self, path: &Path, node: Node) -> io::Result<()> {
        self.free(path).map(|at| {
            self.nodes.insert(at, node);
        })
    }

    pub(super) fn make_directory(&mut self, directory: &Path) -> io::Result<()> {
        self.put(
            directory,
            Node::Directory {
                mode: DIRECTORY_MODE,
            },
        )
    }

    pub(super) fn make_directories(&mut self, directory: &Path) -> io::Result<()> {
        let mut reached = PathBuf::new();
        for part in directory.components() {
            reached.push(part);
            let made = self.make_directory(&reached);
            if !self
                .found(&reached, true)
                .is_ok_and(|found| found.kind == Kind::Directory)
            {
                return made;
            }
        }
        Ok(())
    }

    pub(super) fn copy(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let mode = match self.node(from, true)?.1 {
            Node::File { mode, .. } => *mode,
            _ => return Err(Errno::EINVAL.into()),
        };
        let contents = self.read(from)?;
        self.write(to, &contents)
            .and_then(|()| self.set_mode(to, mode))
    }

    pub(super) fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let (from, moved) = self.node(from, false)?;
        let to = self.at(to, false)?;
        let a_directory = matches!(moved, Node::Directory { .. });
        self.can_take(&to, a_directory)?;
        if to.starts_with(&from) && to != from {
            return Err(Errno::EINVAL.into());
        }
        for path in self.inside(&from) {
            let below = path.strip_prefix(&from).unwrap_or(Path::new(""));
            let moved_to = below
                .components()
                .fold(to.clone(), |to, part| to.join(part));
            if let Some(node) = self.nodes.remove(&path) {
                self.nodes.insert(moved_to, node);
            }
        }
        Ok(())
    }

    fn can_take(&self, to: &Path, a_directory: bool) -> io::Result<()> {
        match self.nodes.get(to) {
            Some(Node::Directory { .. }) if !a_directory => Err(Errno::EISDIR.into()),
            Some(Node::Directory { .. }) if !self.names(to).is_empty() => {
                Err(Errno::ENOTEMPTY.into())
            }
            Some(Node::Directory { .. }) | None => Ok(()),
            Some(_) if a_directory => Err(Errno::ENOTDIR.into()),
            Some(_) => Ok(()),
        }
    }

    pub(super) fn set_mode(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        let (at, _) = self.node(path, true)?;
        if let Some(Node::Directory { mode: kept } | Node::File { mode: kept, .. }) =
            self.nodes.get_mut(&at)
        {
            *kept = mode & PERMISSION_BITS;
        }
        Ok(())
    }

    pub(super) fn remove_file(&mut self, file: &Path) -> io::Result<()> {
        let (at, node) = self.node(file, false)?;
        if matches!(node, Node::Directory { .. }) {
            return Err(Errno::EISDIR.into());
        }
        self.nodes.remove(&at);
        Ok(())
    }

    pub(super) fn remove_directory(&mut self, directory: &Path) -> io::Result<()> {
        let (at, _) = self.node(directory, false)?;
        self.entered(&at)?;
        if !self.names(&at).is_empty() {
            return Err(Errno::ENOTEMPTY.into());
        }
        self.nodes.remove(&at);
        Ok(())
    }

    pub(super) fn remove_tree(&mut self, directory: &Path) -> io::Result<()> {
        let (at, node) = self.node(directory, false)?;
        match found_of(node).kind {
            Kind::Directory => {
                for path in self.inside(&at) {
                    self.nodes.remove(&path);
                }
            }
            Kind::SymbolicLink => {
                self.nodes.remove(&at);
            }
            Kind::File | Kind::Other => return Err(Errno::ENOTDIR.into()),
        }
        Ok(())
    }

    pub(super) fn open(&mut self, file: &Path, make: bool) -> io::Result<PathBuf> {
        let at = self.at(file, true)?;
        match self.nodes.get(&at) {
            Some(Node::File { .. }) => Ok(at),
            Some(Node::Directory { .. }) => Err(Errno::EISDIR.into()),
            Some(_) => Err(Errno::EINVAL.into()),
            None if make => self.write(&at, &[]).map(|()| at),
            None => Err(Errno::ENOENT.into()),
        }
    }

    pub(super) fn lock(&mut self, file: &Path) -> bool {
        self.locked.insert(file.to_owned())
    }

    pub(super) fn unlock(&mut self, file: &Path) {
        self.locked.remove(file);
    }
}

fn parts_of(path: &Path) -> Vec<OsString> {
    path.components()
        .rev()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name.to_owned()),
            Component::ParentDir => Some(OsString::from("..")),
            Component::RootDir | Component::CurDir | Component::Prefix(_) => None,
        })
        .collect()
}

fn too_many_links() -> io::Error {
    Errno::ELOOP.into()
}

fn above(link: &Path, target: &Path) -> PathBuf {
    if target.is_absolute() {
        PathBuf::from(ROOT)
    } else {
        link.parent().unwrap_or(Path::new(ROOT)).to_owned()
    }
}

fn found_of(node: &Node) -> Found {
    match node {
        Node::Directory { mode } => Found {
            kind: Kind::Directory,
            mode: *mode,
            length: 0,
        },
        Node::File { mode, contents } => Found {
            kind: Kind::File,
            mode: *mode,
            length: contents.len() as u64,
        },
        Node::Link { target } => Found {
            kind: Kind::SymbolicLink,
            mode: LINK_MODE,
            length: target.as_os_str().len() as u64,
        },
        Node::Other => Found {
            kind: Kind::Other,
            mode: FILE_MODE,
            length: 0,
        },
    }
}
