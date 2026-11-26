use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use tar::{Archive, EntryType};

pub(super) fn unpack(printed: &mut dyn Read, into: &Path) -> io::Result<()> {
    let mut archive = Archive::new(printed);
    archive.set_preserve_permissions(true);
    let mut directories = Vec::new();
    let mut left_out = HashSet::new();
    for entry in archive.entries().into_iter().flatten() {
        let mut entry = entry?;
        let kind = entry.header().entry_type();
        let linked_to = entry.link_name_bytes().map(Cow::into_owned);
        let linked_to = linked_to.as_ref();
        fs::create_dir_all(into)?;
        match kind {
            EntryType::Directory => directories.push(entry),
            EntryType::Fifo | EntryType::Char | EntryType::Block => {
                left_out.insert(entry.path_bytes().into_owned());
            }
            EntryType::Link if linked_to.is_some_and(|name| left_out.contains(name)) => {}
            _ => {
                entry.unpack_in(into)?;
            }
        }
    }
    directories.sort_by(|one, other| other.path_bytes().cmp(&one.path_bytes()));
    for mut directory in directories {
        directory.unpack_in(into)?;
    }
    Ok(())
}
