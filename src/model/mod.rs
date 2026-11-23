use std::path::Path;

use crate::file_system::FileSystem;

const NAME: &str = "CLADENC_MODEL";
const URL: &str = "CLADENC_MODEL_URL";
const KEY: &str = "CLADENC_MODEL_KEY";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Model<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub key: Option<&'a str>,
}

impl<'a> Model<'a> {
    pub(crate) fn environment(&self) -> impl Iterator<Item = (&'static str, &'a str)> {
        [(NAME, self.name), (URL, self.url)]
            .into_iter()
            .chain(self.key.map(|key| (KEY, key)))
    }
}

pub(crate) fn read_key(files: &dyn FileSystem, file: &Path) -> Option<String> {
    let text = files.read_text(file).ok()?;
    let mut lines = text.lines();
    lines
        .next()
        .filter(|key| !key.is_empty() && !key.contains('\0') && lines.next().is_none())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
