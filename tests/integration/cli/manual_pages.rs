use std::fs;
use std::iter;
use std::path::Path;

use cladenc::cli::Cli;
use clap::CommandFactory;

#[test]
fn the_manual_has_a_page_for_cladenc_and_for_every_sub_command_and_no_other() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/man/man1");
    let mut written: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    written.sort();
    let mut expected: Vec<String> = iter::once("cladenc.1".to_owned())
        .chain(
            Cli::command()
                .get_subcommands()
                .map(|sub_command| format!("cladenc-{}.1", sub_command.get_name())),
        )
        .collect();
    expected.sort();
    assert_eq!(written, expected);
}
