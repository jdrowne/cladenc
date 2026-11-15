mod checks;
mod contract;

use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clap::builder::NonEmptyStringValueParser;
use clap::{Args, Parser, Subcommand};

use crate::search::{self, Found, SearchError};

pub use checks::Refused;

#[derive(Clone, Debug, Parser)]
#[command(
    name = "cladenc",
    version,
    about = "Searches for a better agentic coding harness by letting harnesses modify themselves",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Start a new search and print the directory of its best harness",
        after_long_help = contract::CONTRACT
    )]
    Start(Box<StartOptions>),
    #[command(about = "Continue a search that stopped before its end")]
    Resume(ResumeOptions),
    #[command(about = "Print the state of a search without changing it")]
    Report(ReportOptions),
}

#[derive(Clone, Debug, Args)]
pub struct StartOptions {
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The number of evaluations to start"
    )]
    pub budget: u64,
    #[arg(
        long,
        value_name = "PERCENT",
        value_parser = checks::confidence,
        help = "The probability behind the lower bound that picks the best harness"
    )]
    pub confidence: f64,
    #[arg(
        long,
        value_name = "IMAGE",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The Docker image every container is started from"
    )]
    pub docker_image: String,
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The time limit of one evaluation"
    )]
    pub evaluation_timeout: u64,
    #[arg(
        long,
        value_name = "FILE",
        help = "The program that carries out an evaluation"
    )]
    pub evaluator: PathBuf,
    #[arg(
        long,
        value_name = "NUMBER",
        value_parser = checks::genealogy_growth,
        help = "How fast the genealogy may grow, from 0 to 1"
    )]
    pub genealogy_growth: f64,
    #[arg(
        long,
        value_name = "COMMAND",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The shell command that builds a harness in a container, where one has to be built"
    )]
    pub harness_build_command: Option<String>,
    #[arg(
        long,
        value_name = "COMMAND",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The shell command that runs a harness in a container"
    )]
    pub harness_run_command: String,
    #[arg(
        long,
        value_name = "COUNT",
        help = "Procreations of the progenitor made before the first evaluation"
    )]
    pub initial_procreations: u64,
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The time limit of one procreation"
    )]
    pub procreation_timeout: u64,
    #[arg(
        long,
        value_name = "DIR",
        help = "The directory of source code copied to make the progenitor"
    )]
    pub progenitor: PathBuf,
    #[arg(long, help = "Print nothing on standard error")]
    pub quiet: bool,
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory, which start creates"
    )]
    pub run_dir: PathBuf,
    #[arg(long, value_name = "INTEGER", help = "The seed of every random draw")]
    pub seed: u64,
    #[arg(long, value_name = "FILE", help = "The task ids, one per line")]
    pub tasks: PathBuf,
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The most jobs running at once"
    )]
    pub workers: u64,
}

#[derive(Clone, Debug, Args)]
pub struct ResumeOptions {
    #[arg(long, help = "Print nothing on standard error")]
    pub quiet: bool,
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory of the search to continue"
    )]
    pub run_dir: PathBuf,
}

#[derive(Clone, Debug, Args)]
pub struct ReportOptions {
    #[arg(long, help = "Print one JSON object in place of the table")]
    pub json: bool,
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory of the search to report on"
    )]
    pub run_dir: PathBuf,
}

pub fn execute(cli: &Cli, stop: &AtomicBool, output: &mut dyn Write, errors: &mut dyn Write) -> u8 {
    let quiet = match &cli.command {
        Command::Start(options) => options.quiet,
        Command::Resume(options) => options.quiet,
        Command::Report(_) => false,
    };
    let mut errors = (!quiet).then_some(errors);
    let writer = errors
        .as_mut()
        .map(|errors| &mut **errors as &mut dyn Write);
    let printed = match &cli.command {
        Command::Start(options) => start(options, stop, writer).map(directory),
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

fn start(
    options: &StartOptions,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, (u8, String)> {
    let settings = options
        .settings()
        .map_err(|refused| (2, refused.to_string()))?;
    let run_directory = mountable(&options.run_dir)?;
    search::search(&settings, &run_directory, stop, writer).map_err(stopped)
}

fn resume(
    options: &ResumeOptions,
    stop: &AtomicBool,
    writer: Option<&mut dyn Write>,
) -> Result<Found, (u8, String)> {
    let run_directory = mountable(&options.run_dir)?;
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
        .map_err(|error| stopped(SearchError::Unusable(error)))
}

fn directory(found: Found) -> String {
    format!("{}\n", found.directory.display())
}

fn absolute(run_dir: &Path) -> Result<PathBuf, (u8, String)> {
    path::absolute(run_dir).map_err(|error| (2, format!("--run-dir takes a path: {error}")))
}

fn mountable(run_dir: &Path) -> Result<PathBuf, (u8, String)> {
    absolute(run_dir).and_then(|run_directory| {
        if run_directory.to_string_lossy().contains(':') {
            Err((
                2,
                "--run-dir takes a path that holds no colon, because Docker cannot mount a \
                 directory whose path holds one"
                    .to_owned(),
            ))
        } else {
            Ok(run_directory)
        }
    })
}

fn stopped(error: SearchError) -> (u8, String) {
    let status = match error {
        SearchError::Io(_) | SearchError::Unusable(_) => 2,
        SearchError::Journal(_)
        | SearchError::Asked
        | SearchError::Build(_)
        | SearchError::Job { .. } => 1,
    };
    (status, error.to_string())
}
