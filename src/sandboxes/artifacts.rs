use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::iter;
use std::path::{Component, Path, PathBuf};

use super::sandbox::Up;
use super::{Copied, SandboxError};
use crate::dataset::Artifact;
use crate::file_system::{FileSystem, Found, Kind};

const KEPT: &str = ".";
const GIVEN: &str = "given-to-the-verifier";
const TAKEN: &str = "taken-out-of-the-sandbox";
const READ_WRITTEN_AND_RUN: u32 = 0o777;
const BITS_OF_A_DIRECTORY: u32 = 0o1777;

pub(super) type Bits = BTreeMap<PathBuf, u32>;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Given {
    pub(super) directory: bool,
    pub(super) bits: Vec<(PathBuf, u32)>,
}

struct Copying<'a> {
    files: &'a dyn FileSystem,
    left: &'a Bits,
    top: &'a Path,
}

enum Token {
    AnyRun,
    One(Set),
}

struct Set {
    negated: bool,
    ranges: Vec<(char, char)>,
}

impl Set {
    fn of(character: char) -> Self {
        Self {
            negated: false,
            ranges: vec![(character, character)],
        }
    }

    fn any() -> Self {
        Self {
            negated: true,
            ranges: Vec::new(),
        }
    }

    fn holds(&self, character: char) -> bool {
        self.ranges
            .iter()
            .any(|(from, to)| (*from..=*to).contains(&character))
            != self.negated
    }
}

pub(super) fn keep(up: &Up<'_>, artifacts: &[Artifact], kept: &Path) -> Result<Bits, SandboxError> {
    let files = up.files();
    let mut left = Bits::new();
    for artifact in artifacts {
        let place = keep_one(up, artifact, kept, &mut left)?;
        remove_above_where_empty(files, &place, kept);
    }
    if let Some(published) = Artifact::published(artifacts) {
        let place = keep_one(up, &published, kept, &mut left)?;
        let _ = files.remove_directory(&place);
        remove_above_where_empty(files, &place, kept);
    }
    Ok(left)
}

fn keep_one(
    up: &Up<'_>,
    artifact: &Artifact,
    kept: &Path,
    left: &mut Bits,
) -> Result<PathBuf, SandboxError> {
    let place = place_of(artifact, kept);
    let _ = place
        .parent()
        .map(|above| up.files().make_directories(above));
    let changed = up
        .copy_what_is_at(&artifact.source, &kept.with_file_name(TAKEN), &place)
        .map_err(|error| SandboxError::NotCopiedOut {
            left: Copied::Artifact(artifact.source.clone()),
            error,
        })?;
    left.extend(changed);
    leave_out(up.files(), &place, &artifact.exclude);
    Ok(place)
}

pub(super) fn remove_above_where_empty(files: &dyn FileSystem, place: &Path, kept: &Path) {
    place
        .ancestors()
        .skip(1)
        .take_while(|above| above.starts_with(kept))
        .for_each(|above| {
            let _ = files.remove_directory(above);
        });
}

pub(super) fn place_of(artifact: &Artifact, kept: &Path) -> PathBuf {
    artifact
        .destination
        .split('/')
        .filter(|part| !part.is_empty())
        .fold(kept.to_owned(), |place, part| place.join(part))
}

pub(super) fn give(
    up: &Up<'_>,
    artifacts: &[Artifact],
    kept: &Path,
    owner: &str,
    bits: &Bits,
) -> Result<(), SandboxError> {
    let files = up.files();
    let given = kept.with_file_name(GIVEN);
    let published = Artifact::published(artifacts);
    let gave = artifacts.iter().chain(&published).try_for_each(|artifact| {
        remove(files, &given);
        let copied = trusted(files, &place_of(artifact, kept), &given, bits).map_err(|error| {
            SandboxError::NotGivenToVerifier {
                left: artifact.source.clone(),
                error,
            }
        })?;
        let path = artifact.path();
        match copied {
            Some(copy) if copy.directory => up
                .put(&given, &path, owner)
                .and_then(|()| up.set_bits(&path, &copy.bits)),
            Some(copy) => up
                .put_file(&given, &path, owner)
                .and_then(|()| up.set_bits(&path, &copy.bits)),
            None => Ok(()),
        }
    });
    remove(files, &given);
    gave
}

pub(super) fn remove(files: &dyn FileSystem, given: &Path) {
    let _ = files
        .remove_file(given)
        .or_else(|_| files.remove_tree(given));
}

pub(super) fn trusted(
    files: &dyn FileSystem,
    from: &Path,
    to: &Path,
    left: &Bits,
) -> io::Result<Option<Given>> {
    match files.found(from) {
        Ok(found) if matches!(found.kind, Kind::Directory | Kind::File) => {
            let copying = Copying {
                files,
                left,
                top: from,
            };
            let directory = found.kind == Kind::Directory;
            copying
                .copy(from, found, to, 0)
                .map(|bits| Some(Given { directory, bits }))
        }
        Ok(_) => Ok(None),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(naming(from)(error)),
    }
}

impl Copying<'_> {
    fn copy(
        &self,
        from: &Path,
        found: Found,
        to: &Path,
        depth: usize,
    ) -> io::Result<Vec<(PathBuf, u32)>> {
        let files = self.files;
        let copied = match found.kind {
            Kind::Directory => files
                .make_directory(to)
                .and_then(|()| files.set_mode(to, found.mode & BITS_OF_A_DIRECTORY))
                .and_then(|()| files.list(from)),
            Kind::File => files
                .copy(from, to)
                .and_then(|()| files.set_mode(to, found.mode & READ_WRITTEN_AND_RUN))
                .map(|()| Vec::new()),
            Kind::SymbolicLink => files
                .target(from)
                .and_then(|target| self.link_inside(&target, to, depth))
                .map(|()| Vec::new()),
            Kind::Other => Ok(Vec::new()),
        };
        copied
            .map_err(naming(from))
            .and_then(|names| {
                names.into_iter().try_fold(Vec::new(), |bits, name| {
                    let (from, to) = (from.join(&name), to.join(&name));
                    files
                        .found(&from)
                        .map_err(naming(&from))
                        .and_then(|found| self.copy(&from, found, &to, depth + 1))
                        .map(|below| with(bits, below))
                })
            })
            .map(|bits| with(bits, self.left_on(from, found)))
    }

    fn link_inside(&self, target: &Path, to: &Path, depth: usize) -> io::Result<()> {
        if stays_inside(target, depth) {
            self.files.link(target, to)
        } else {
            Ok(())
        }
    }

    fn left_on(&self, from: &Path, found: Found) -> Option<(PathBuf, u32)> {
        let below = from.strip_prefix(self.top).unwrap_or(Path::new(""));
        let (kept_with, all) = match found.kind {
            Kind::Directory => (
                below.as_os_str().is_empty().then_some(found.mode),
                BITS_OF_A_DIRECTORY,
            ),
            _ => (None, READ_WRITTEN_AND_RUN),
        };
        self.left
            .get(from)
            .copied()
            .or(kept_with)
            .map(|bits| (below.to_owned(), bits & all))
    }
}

fn with(
    mut bits: Vec<(PathBuf, u32)>,
    more: impl IntoIterator<Item = (PathBuf, u32)>,
) -> Vec<(PathBuf, u32)> {
    bits.extend(more);
    bits
}

fn naming(path: &Path) -> impl Fn(io::Error) -> io::Error {
    move |error| io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}

fn stays_inside(target: &Path, depth: usize) -> bool {
    depth
        .checked_sub(1)
        .and_then(|directory| {
            target
                .components()
                .try_fold(directory, |level, part| match part {
                    Component::Normal(_) => Some(level + 1),
                    Component::CurDir => Some(level),
                    Component::ParentDir => level.checked_sub(1),
                    Component::RootDir | Component::Prefix(_) => None,
                })
        })
        .is_some()
}

pub(super) fn leave_out(files: &dyn FileSystem, tree: &Path, exclude: &[String]) {
    let patterns: Vec<Vec<Token>> = exclude.iter().map(|pattern| tokens(pattern)).collect();
    if !patterns.is_empty() && is_directory(files, tree) {
        let everything = is_left_out(KEPT, &patterns);
        leave_out_of(files, tree, KEPT, &patterns, everything);
    }
}

fn leave_out_of(
    files: &dyn FileSystem,
    directory: &Path,
    path: &str,
    patterns: &[Vec<Token>],
    everything: bool,
) {
    for name in files.list(directory).unwrap_or_default() {
        let found = directory.join(&name);
        let path = format!("{path}/{}", name.to_string_lossy());
        if everything || is_left_out(&path, patterns) {
            remove(files, &found);
        } else if is_directory(files, &found) {
            leave_out_of(files, &found, &path, patterns, false);
        }
    }
}

fn is_directory(files: &dyn FileSystem, found: &Path) -> bool {
    files
        .found(found)
        .is_ok_and(|found| found.kind == Kind::Directory)
}

fn is_left_out(path: &str, patterns: &[Vec<Token>]) -> bool {
    let ends: Vec<Vec<char>> = iter::once(path)
        .chain(path.match_indices('/').map(|(slash, _)| &path[slash + 1..]))
        .map(|end| end.chars().collect())
        .collect();
    patterns
        .iter()
        .any(|pattern| ends.iter().any(|end| matches(pattern, end)))
}

fn matches(pattern: &[Token], text: &[char]) -> bool {
    let (mut token, mut character) = (0, 0);
    let mut run: Option<(usize, usize)> = None;
    while character < text.len() {
        match (pattern.get(token), run) {
            (Some(Token::AnyRun), _) => {
                run = Some((token, character));
                token += 1;
            }
            (Some(Token::One(set)), _) if set.holds(text[character]) => {
                token += 1;
                character += 1;
            }
            (_, Some((star, taken))) => {
                run = Some((star, taken + 1));
                token = star + 1;
                character = taken + 1;
            }
            (_, None) => return false,
        }
    }
    pattern[token..]
        .iter()
        .all(|token| matches!(token, Token::AnyRun))
}

fn tokens(pattern: &str) -> Vec<Token> {
    let pattern: Vec<char> = pattern.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < pattern.len() {
        let (token, taken) = match (pattern[at], pattern.get(at + 1)) {
            ('*', _) => (Token::AnyRun, 1),
            ('?', _) => (Token::One(Set::any()), 1),
            ('\\', Some(character)) => (Token::One(Set::of(*character)), 2),
            ('[', _) => set(&pattern[at + 1..]).unwrap_or((Token::One(Set::of('[')), 1)),
            (character, _) => (Token::One(Set::of(character)), 1),
        };
        tokens.push(token);
        at += taken;
    }
    tokens
}

fn set(after: &[char]) -> Option<(Token, usize)> {
    let negated = matches!(after.first(), Some('!' | '^'));
    let start = usize::from(negated);
    let end = (start + 1..after.len()).find(|place| after[*place] == ']')?;
    let members = &after[start..end];
    let mut ranges = Vec::new();
    let mut at = 0;
    while at < members.len() {
        let to = if at + 2 < members.len() && members[at + 1] == '-' {
            at + 2
        } else {
            at
        };
        ranges.push((members[at], members[to]));
        at = to + 1;
    }
    Some((Token::One(Set { negated, ranges }), end + 2))
}
