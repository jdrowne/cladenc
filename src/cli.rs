mod checks;

use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clap::{Args, Parser, Subcommand};

use crate::search::{self, Found, SearchError};

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
    #[command(about = "Continue a search that stopped before its end")]
    Resume(ResumeOptions),
    #[command(about = "Print the state of a search without changing it")]
    Report(ReportOptions),
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

#[derive(Clone, Debug, Args)]
pub struct ResumeOptions {
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory of the search to continue"
    )]
    pub run_dir: PathBuf,
    #[arg(long, help = "Print nothing on standard error")]
    pub quiet: bool,
}

#[derive(Clone, Debug, Args)]
pub struct ReportOptions {
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory of the search to report on"
    )]
    pub run_dir: PathBuf,
    #[arg(long, help = "Print one JSON object in place of the table")]
    pub json: bool,
}

pub fn execute(cli: &Cli, stop: &AtomicBool, output: &mut dyn Write, errors: &mut dyn Write) -> u8 {
    let quiet = match &cli.command {
        Command::Run(options) => options.quiet,
        Command::Resume(options) => options.quiet,
        Command::Report(_) => false,
    };
    let mut errors = (!quiet).then_some(errors);
    let writer = errors
        .as_mut()
        .map(|errors| &mut **errors as &mut dyn Write);
    let printed = match &cli.command {
        Command::Run(options) => run(options, stop, writer).map(directory),
        Command::Resume(options) => resume(options, stop, writer).map(directory),
        Command::Report(options) => report(options),
    };
    match printed {
        Ok(text) => {
            let _ = write!(output, "{text}");
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

fn run(
    options: &RunOptions,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, (u8, String)> {
    let settings = options
        .settings()
        .map_err(|refused| (2, refused.to_string()))?;
    let run_directory = absolute(&options.run_dir)?;
    search::search(&settings, &run_directory, stop, writer).map_err(stopped)
}

fn resume(
    options: &ResumeOptions,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, (u8, String)> {
    let run_directory = absolute(&options.run_dir)?;
    search::resume(&run_directory, stop, writer).map_err(stopped)
}

fn report(options: &ReportOptions) -> Result<String, (u8, String)> {
    let run_directory = absolute(&options.run_dir)?;
    crate::report::read(&run_directory)
        .map(|report| {
            if options.json {
                report.json() + "\n"
            } else {
                report.table()
            }
        })
        .map_err(|error| (2, format!("the run directory cannot be used: {error:?}")))
}

fn directory(found: Found) -> String {
    format!("{}\n", found.directory.display())
}

fn absolute(run_dir: &Path) -> Result<PathBuf, (u8, String)> {
    path::absolute(run_dir).map_err(|error| (2, format!("--run-dir takes a path: {error}")))
}

fn stopped(error: SearchError) -> (u8, String) {
    match error {
        SearchError::Io(error) => (2, format!("the run directory cannot be made: {error}")),
        SearchError::Unusable(error) => (2, format!("the run directory cannot be used: {error:?}")),
        error => (1, format!("the search stopped before its end: {error:?}")),
    }
}
