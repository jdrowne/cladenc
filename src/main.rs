use std::io;
use std::process::{self, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use cladenc::cli::{self, Cli, Command};
use cladenc::containers::Program;
use clap::Parser;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::flag;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let stop = Arc::new(AtomicBool::new(false));
    if matches!(cli.command, Command::Start(_) | Command::Resume(_)) {
        let hangup = (!a_hangup_is_ignored()).then_some(SIGHUP);
        for signal in [SIGINT, SIGQUIT, SIGTERM].into_iter().chain(hangup) {
            flag::register(signal, Arc::clone(&stop)).expect("a signal can be listened for");
        }
    }
    ExitCode::from(cli::execute(
        &Program,
        &cli,
        &stop,
        &mut io::stdout(),
        &mut io::stderr(),
    ))
}

fn a_hangup_is_ignored() -> bool {
    process::Command::new("sh")
        .args(["-c", "kill -HUP $$"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
