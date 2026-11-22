use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::ContainerError;
use super::container::{Run, Shell, Up};
use super::program::Limits;
use crate::benchmark::CollectCommand;

pub(super) fn run(
    up: &Up,
    commands: &[CollectCommand],
    kept: &Path,
    stop: &AtomicBool,
) -> Result<(), ContainerError> {
    commands
        .iter()
        .enumerate()
        .try_for_each(|(place, command)| {
            let kept = kept.join((place + 1).to_string());
            let _ = fs::create_dir_all(&kept);
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
