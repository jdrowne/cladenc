use std::io;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli};
use clap::Parser;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [SIGINT, SIGTERM] {
        flag::register(signal, Arc::clone(&stop)).expect("a signal can be listened for");
    }
    ExitCode::from(cli::execute(
        &cli,
        &stop,
        &mut io::stdout(),
        &mut io::stderr(),
    ))
}
