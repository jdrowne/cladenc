mod checks;

use std::io::Write;
use std::path::{self, PathBuf};
use std::sync::atomic::AtomicBool;

use clap::{Args, Parser, Subcommand};

use crate::search::{self, SearchError};

pub use checks::Refused;

#[derive(Clone, Debug, Parser)]
#[command(
    name = "cladenc",
    about = "Searches for a better agentic coding harness by letting harnesses modify themselves"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command(about = "Start a new search and print the directory of its best harness")]
    Run(RunOptions),
}

#[derive(Clone, Debug, Args)]
pub struct RunOptions {
    #[arg(
        long,
        value_name = "DIR",
        help = "The directory copied to make the progenitor"
    )]
    pub progenitor: PathBuf,
    #[arg(long, value_name = "FILE", help = "The task ids, one per line")]
    pub tasks: PathBuf,
    #[arg(
        long,
        value_name = "FILE",
        help = "The program that carries out an expansion"
    )]
    pub expander: PathBuf,
    #[arg(
        long,
        value_name = "FILE",
        help = "The program that carries out an evaluation"
    )]
    pub evaluator: PathBuf,
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory, which run creates"
    )]
    pub run_dir: PathBuf,
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The number of evaluations to start"
    )]
    pub budget: u64,
    #[arg(
        long,
        value_name = "NUMBER",
        value_parser = checks::genealogy_growth,
        help = "How fast the genealogy may grow, from 0 to 1"
    )]
    pub genealogy_growth: f64,
    #[arg(
        long,
        value_name = "PERCENT",
        value_parser = checks::confidence,
        help = "The probability behind the lower bound that picks the best harness"
    )]
    pub confidence: f64,
    #[arg(
        long,
        value_name = "COUNT",
        help = "Expansions of the progenitor made before the first evaluation"
    )]
    pub initial_expansions: u64,
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The most jobs running at once"
    )]
    pub workers: u64,
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The time limit of one expansion"
    )]
    pub expansion_timeout: u64,
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The time limit of one evaluation"
    )]
    pub evaluation_timeout: u64,
    #[arg(long, value_name = "INTEGER", help = "The seed of every random draw")]
    pub seed: u64,
    #[arg(long, help = "Print nothing on standard error")]
    pub quiet: bool,
}

pub fn execute(cli: &Cli, stop: &AtomicBool, output: &mut dyn Write, errors: &mut dyn Write) -> u8 {
    match &cli.command {
        Command::Run(options) => run(options, stop, output, errors),
    }
}

fn run(
    options: &RunOptions,
    stop: &AtomicBool,
    output: &mut dyn Write,
    errors: &mut dyn Write,
) -> u8 {
    let mut errors = (!options.quiet).then_some(errors);
    let found = options
        .settings()
        .map_err(|refused| (2, refused.to_string()))
        .and_then(|settings| {
            path::absolute(&options.run_dir)
                .map(|run_directory| (settings, run_directory))
                .map_err(|error| (2, format!("--run-dir takes a path: {error}")))
        })
        .and_then(|(settings, run_directory)| {
            let writer = errors
                .as_mut()
                .map(|errors| &mut **errors as &mut dyn Write);
            search::search(&settings, &run_directory, stop, writer).map_err(stopped)
        });
    match found {
        Ok(found) => {
            let _ = writeln!(output, "{}", found.directory.display());
            0
        }
        Err((status, reason)) => {
            if let Some(errors) = errors {
                let _ = writeln!(errors, "cladenc: {reason}");
            }
            status
        }
    }
}

fn stopped(error: SearchError) -> (u8, String) {
    match error {
        SearchError::Io(error) => (2, format!("the run directory cannot be made: {error}")),
        error => (1, format!("the search stopped before its end: {error:?}")),
    }
}
