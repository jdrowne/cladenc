use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli};
use cladenc::containers::{Outside, Program};
use cladenc::directed_evolution::{self, DirectedEvolutionError, Found};
use cladenc::file_system::{Disk, FileSystem};
use cladenc::journal::Settings;

pub fn disk() -> Arc<dyn FileSystem> {
    Arc::new(Disk)
}

pub fn start(
    settings: &Settings,
    path: &Path,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, DirectedEvolutionError> {
    let files = disk();
    let outside = Outside {
        docker: &Program,
        files: &files,
    };
    directed_evolution::start(outside, settings, path, stop, writer)
}

pub fn resume(
    path: &Path,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, DirectedEvolutionError> {
    let files = disk();
    let outside = Outside {
        docker: &Program,
        files: &files,
    };
    directed_evolution::resume(outside, path, stop, writer)
}

pub fn execute(cli: &Cli, stop: &AtomicBool, output: &mut dyn Write, errors: &mut dyn Write) -> u8 {
    let files = disk();
    let outside = Outside {
        docker: &Program,
        files: &files,
    };
    cli::execute(outside, cli, stop, output, errors)
}
