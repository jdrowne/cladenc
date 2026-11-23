use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::container::{Run, Shell, Up};
use super::program::Limits;
use super::{ContainerError, make_directory};
use crate::benchmark::CollectCommand;

pub(super) fn run(
    up: &Up<'_>,
    commands: &[CollectCommand],
    kept: &Path,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    commands
        .iter()
        .enumerate()
        .try_for_each(|(place, command)| {
            let kept = kept.join((place + 1).to_string());
            make_directory(up.files(), &kept);
            let shell = Shell {
                command: &command.command,
                environment: &[],
            };
            let run = Run {
                shell: &shell,
                user: command.user.as_deref().unwrap_or(up.user()),
                variables: &[],
            };
            let limits = Limits::within(command.time_limit, stop);
            up.run(&run, &kept, limits).map(drop)
        })
}
