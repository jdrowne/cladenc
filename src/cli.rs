mod checks;
mod contract;

use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clap::builder::NonEmptyStringValueParser;
use clap::{ArgGroup, Args, Parser, Subcommand};

use crate::directed_evolution::{self, DirectedEvolutionError, Found};

pub use checks::Refused;

#[derive(Clone, Debug, Parser)]
#[command(
    name = "cladenc",
    version,
    about = "Makes a better agentic coding harness by directed evolution, letting harnesses \
             modify themselves",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command(
        about = "Start a new directed evolution and print the directory of its best harness",
        after_long_help = contract::CONTRACT
    )]
    Start(Box<StartOptions>),
    #[command(
        about = "Continue a directed evolution that stopped before its end",
        after_long_help = contract::RESUMING
    )]
    Resume(ResumeOptions),
    #[command(about = "Print the state of a directed evolution without changing it")]
    Report(ReportOptions),
}

#[derive(Clone, Debug, Args)]
#[command(group(ArgGroup::new("image").required(true).args(["docker_image", "dockerfile"])))]
pub struct StartOptions {
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The time limit of one assessment"
    )]
    pub assessment_timeout: u64,
    #[arg(
        long,
        value_name = "DIR",
        required = true,
        help = "The directory of a benchmark, which holds a directory for each task, given once \
                for each benchmark"
    )]
    pub benchmark: Vec<PathBuf>,
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The number of assessments to start"
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
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        requires = "dockerfile",
        conflicts_with = "docker_image",
        help = "The time limit of the build of the image, given with --dockerfile"
    )]
    pub docker_build_timeout: Option<u64>,
    #[arg(
        long,
        value_name = "IMAGE",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The Docker image every container is started from, where --dockerfile is not given"
    )]
    pub docker_image: Option<String>,
    #[arg(
        long,
        value_name = "FILE",
        requires = "docker_build_timeout",
        help = "The Dockerfile cladenc builds the image of every container from, in place of \
                --docker-image"
    )]
    pub dockerfile: Option<PathBuf>,
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
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The most containers running at once"
    )]
    pub max_concurrent_containers: u64,
    #[arg(
        long,
        value_name = "NAME",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The name of the model the harnesses call, as its server knows it"
    )]
    pub model: String,
    #[arg(
        long,
        value_name = "FILE",
        help = "The file that holds the key the model is called with, where its server asks for one"
    )]
    pub model_key_file: Option<PathBuf>,
    #[arg(
        long,
        value_name = "URL",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The address the model is served at"
    )]
    pub model_url: String,
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
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory, which start creates"
    )]
    pub run_dir: PathBuf,
}

#[derive(Clone, Debug, Args)]
pub struct ResumeOptions {
    #[arg(
        long,
        value_name = "DIR",
        help = "The run directory of the directed evolution to continue"
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
        help = "The run directory of the directed evolution to report on"
    )]
    pub run_dir: PathBuf,
}

pub fn execute(cli: &Cli, stop: &AtomicBool, output: &mut dyn Write, errors: &mut dyn Write) -> u8 {
    let printed = match &cli.command {
        Command::Start(options) => start(options, stop, errors).map(directory),
        Command::Resume(options) => resume(options, stop, errors).map(directory),
        Command::Report(options) => report(options),
    };
    match printed {
        Ok(text) => {
            let _ = write!(output, "{text}");
            0
        }
        Err((status, reason)) => {
            let _ = writeln!(errors, "cladenc: {reason}");
            status
        }
    }
}

fn start(
    options: &StartOptions,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, (u8, String)> {
    let settings = options
        .settings()
        .map_err(|refused| (2, refused.to_string()))?;
    let run_directory = mountable(&options.run_dir)?;
    directed_evolution::start(&settings, &run_directory, stop, writer).map_err(stopped)
}

fn resume(
    options: &ResumeOptions,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<Found, (u8, String)> {
    let run_directory = mountable(&options.run_dir)?;
    directed_evolution::resume(&run_directory, stop, writer).map_err(stopped)
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
        .map_err(|error| stopped(DirectedEvolutionError::Unusable(error)))
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

fn stopped(error: DirectedEvolutionError) -> (u8, String) {
    let status = match error {
        DirectedEvolutionError::Io(_)
        | DirectedEvolutionError::Unusable(_)
        | DirectedEvolutionError::Key(_)
        | DirectedEvolutionError::Left(_)
        | DirectedEvolutionError::Image(_) => 2,
        DirectedEvolutionError::Journal(_)
        | DirectedEvolutionError::Asked
        | DirectedEvolutionError::Build(_)
        | DirectedEvolutionError::Action { .. } => 1,
    };
    (status, error.to_string())
}
