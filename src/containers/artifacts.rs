use std::fs;
use std::iter;
use std::path::Path;

use super::container::Up;
use crate::benchmark::Artifact;

const KEPT: &str = ".";

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

pub(super) fn keep(up: &Up, artifacts: &[Artifact], kept: &Path) {
    for artifact in artifacts {
        let place = artifact
            .destination
            .split('/')
            .filter(|part| !part.is_empty())
            .fold(kept.to_owned(), |place, part| place.join(part));
        let _ = place.parent().map(fs::create_dir_all);
        up.copy_what_is_at(&artifact.source, &place);
        leave_out(&place, &artifact.exclude);
        place
            .ancestors()
            .skip(1)
            .take_while(|above| above.starts_with(kept))
            .for_each(|above| {
                let _ = fs::remove_dir(above);
            });
    }
}

pub(super) fn leave_out(tree: &Path, exclude: &[String]) {
    let patterns: Vec<Vec<Token>> = exclude.iter().map(|pattern| tokens(pattern)).collect();
    if !patterns.is_empty() && is_directory(tree) {
        leave_out_of(tree, KEPT, &patterns, is_left_out(KEPT, &patterns));
    }
}

fn leave_out_of(directory: &Path, path: &str, patterns: &[Vec<Token>], everything: bool) {
    let entries: Vec<fs::DirEntry> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    for entry in entries {
        let found = entry.path();
        let path = format!("{path}/{}", entry.file_name().to_string_lossy());
        if everything || is_left_out(&path, patterns) {
            let _ = fs::remove_file(&found).or_else(|_| fs::remove_dir_all(&found));
        } else if is_directory(&found) {
            leave_out_of(&found, &path, patterns, false);
        }
    }
}

fn is_directory(found: &Path) -> bool {
    fs::symlink_metadata(found).is_ok_and(|found| found.is_dir())
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
