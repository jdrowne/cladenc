mod checks;
mod contract;

use std::io::Write;
use std::path::{self, Path, PathBuf};
use std::sync::atomic::AtomicBool;

use clap::builder::NonEmptyStringValueParser;
use clap::{ArgGroup, Args, Parser, Subcommand};

use crate::directed_evolution::{self, DirectedEvolutionError, Found};
use crate::report::{self, Report};

const HAS_NOT_ENDED: &str = "the directed evolution has not ended: call `cladenc status` to read \
                             its state so far";

const HAS_ENDED: &str =
    "the directed evolution has ended: call `cladenc report` to read its report";

#[derive(Clone, Debug, Parser)]
#[command(
    name = "cladenc",
    version,
    about = "Carries out a directed evolution of an agentic coding harness, in which harnesses \
             procreate, each changing a copy of its own source code, and are benchmarked",
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
    #[command(
        about = "Print the report of a directed evolution that has ended, without changing it"
    )]
    Report(ReportOptions),
    #[command(
        about = "Print the state so far of a directed evolution that has not ended, without \
                 changing it"
    )]
    Status(StatusOptions),
}

#[derive(Clone, Debug, Args)]
#[command(group(ArgGroup::new("image").required(true).args(["procreation_docker_image", "procreation_dockerfile"])))]
pub struct StartOptions {
    #[arg(
        long,
        value_name = "DIR",
        required = true,
        help = "The directory of a benchmark, which holds a directory for each task, given once \
                for each benchmark"
    )]
    benchmark: Vec<PathBuf>,
    #[arg(
        long,
        value_name = "NUMBER",
        value_parser = checks::percentage,
        help = "The percentage of each benchmark's tasks that are corroboration tasks, above 0 \
                and below 100"
    )]
    corroboration_tasks_percentage: f64,
    #[arg(
        long,
        value_name = "COMMAND",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The shell command that builds a harness in a container, where one has to be built"
    )]
    harness_build_command: Option<String>,
    #[arg(
        long,
        value_name = "COMMAND",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The shell command that runs a harness in a container"
    )]
    harness_run_command: String,
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The most containers running at once"
    )]
    max_concurrent_containers: u64,
    #[arg(
        long,
        value_name = "COUNT",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The most procreations to start"
    )]
    max_procreations: u64,
    #[arg(
        long,
        value_name = "NAME",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The name of the model the harnesses call, as its server knows it"
    )]
    model: String,
    #[arg(
        long,
        value_name = "FILE",
        help = "The file that holds the key the model is called with, where its server asks for one"
    )]
    model_key_file: Option<PathBuf>,
    #[arg(
        long,
        value_name = "URL",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The address the model is served at"
    )]
    model_url: String,
    #[arg(
        long,
        value_name = "DIR",
        help = "The outputs directory, which start creates"
    )]
    outputs_dir: PathBuf,
    #[arg(
        long,
        value_name = "DIR",
        help = "The directory of source code copied to make the primogenitor"
    )]
    primogenitor: PathBuf,
    #[arg(
        long,
        value_name = "IMAGE",
        value_parser = NonEmptyStringValueParser::new(),
        help = "The Docker image every procreation and every build of a harness is carried out \
                in, where --procreation-dockerfile is not given"
    )]
    procreation_docker_image: Option<String>,
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        requires = "procreation_dockerfile",
        conflicts_with = "procreation_docker_image",
        help = "The time limit of the build of the image, given with --procreation-dockerfile"
    )]
    procreation_docker_image_build_timeout: Option<u64>,
    #[arg(
        long,
        value_name = "FILE",
        requires = "procreation_docker_image_build_timeout",
        help = "The Dockerfile cladenc builds the image of every procreation and every build of a \
                harness from, in place of --procreation-docker-image"
    )]
    procreation_dockerfile: Option<PathBuf>,
    #[arg(
        long,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..),
        help = "The time limit of one procreation"
    )]
    procreation_timeout: u64,
    #[arg(
        long,
        value_name = "NUMBER",
        value_parser = checks::percentage,
        help = "The percentage of each benchmark's tasks that are selection tasks, above 0 and \
                below 100"
    )]
    selection_tasks_percentage: f64,
}

#[derive(Clone, Debug, Args)]
pub struct ResumeOptions {
    #[arg(
        long,
        value_name = "DIR",
        help = "The outputs directory of the directed evolution to continue"
    )]
    outputs_dir: PathBuf,
}

#[derive(Clone, Debug, Args)]
pub struct ReportOptions {
    #[arg(long, help = "Print one JSON object in place of the table")]
    json: bool,
    #[arg(
        long,
        value_name = "DIR",
        help = "The outputs directory of the directed evolution to report on"
    )]
    outputs_dir: PathBuf,
}

#[derive(Clone, Debug, Args)]
pub struct StatusOptions {
    #[arg(long, help = "Print one JSON object in place of the table")]
    json: bool,
    #[arg(
        long,
        value_name = "DIR",
        help = "The outputs directory of the directed evolution to print the state of"
    )]
    outputs_dir: PathBuf,
}

pub fn execute(cli: &Cli, stop: &AtomicBool, output: &mut dyn Write, errors: &mut dyn Write) -> u8 {
    let printed = match &cli.command {
        Command::Start(options) => start(options, stop, errors),
        Command::Resume(options) => resume(options, stop, errors),
        Command::Report(options) => read(&options.outputs_dir)
            .and_then(ended)
            .map(|report| shown(&report, options.json)),
        Command::Status(options) => read(&options.outputs_dir)
            .and_then(not_ended)
            .map(|report| shown(&report, options.json)),
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
) -> Result<String, (u8, String)> {
    let settings = options
        .settings()
        .map_err(|refused| (2, refused.to_string()))?;
    let outputs_directory = mountable(&options.outputs_dir)?;
    directed_evolution::start(&settings, &outputs_directory, stop, writer)
        .map_err(stopped)
        .and_then(|found| finished(&outputs_directory, &found, writer))
}

fn resume(
    options: &ResumeOptions,
    stop: &AtomicBool,
    writer: &mut dyn Write,
) -> Result<String, (u8, String)> {
    let outputs_directory = mountable(&options.outputs_dir)?;
    directed_evolution::resume(&outputs_directory, stop, writer)
        .map_err(stopped)
        .and_then(|found| finished(&outputs_directory, &found, writer))
}

fn finished(
    outputs_directory: &Path,
    found: &Found,
    writer: &mut dyn Write,
) -> Result<String, (u8, String)> {
    read(outputs_directory).and_then(ended).map(|report| {
        let _ = write!(writer, "{}", report.table());
        format!("{}\n", found.directory.display())
    })
}

fn read(outputs_dir: &Path) -> Result<Report, (u8, String)> {
    absolute(outputs_dir).and_then(|outputs_directory| {
        report::read(&outputs_directory)
            .map_err(|error| stopped(DirectedEvolutionError::Unusable(error)))
    })
}

fn ended(report: Report) -> Result<Report, (u8, String)> {
    if report.has_ended() {
        Ok(report)
    } else {
        Err((2, HAS_NOT_ENDED.to_owned()))
    }
}

fn not_ended(report: Report) -> Result<Report, (u8, String)> {
    if report.has_ended() {
        Err((2, HAS_ENDED.to_owned()))
    } else {
        Ok(report)
    }
}

fn shown(report: &Report, json: bool) -> String {
    if json {
        report.json() + "\n"
    } else {
        report.table()
    }
}

fn absolute(outputs_dir: &Path) -> Result<PathBuf, (u8, String)> {
    path::absolute(outputs_dir).map_err(|error| (2, format!("--outputs-dir takes a path: {error}")))
}

fn mountable(outputs_dir: &Path) -> Result<PathBuf, (u8, String)> {
    absolute(outputs_dir).and_then(|outputs_directory| {
        if outputs_directory.to_string_lossy().contains(':') {
            Err((
                2,
                "--outputs-dir takes a path that holds no colon, because Docker cannot mount a \
                 directory whose path holds one"
                    .to_owned(),
            ))
        } else {
            Ok(outputs_directory)
        }
    })
}

fn stopped(error: DirectedEvolutionError) -> (u8, String) {
    let status = match error {
        DirectedEvolutionError::Io(_)
        | DirectedEvolutionError::Unusable(_)
        | DirectedEvolutionError::Key(_)
        | DirectedEvolutionError::Left(_)
        | DirectedEvolutionError::Image(_)
        | DirectedEvolutionError::Configuration { .. }
        | DirectedEvolutionError::NotOnTheMachine { .. }
        | DirectedEvolutionError::Machine(_)
        | DirectedEvolutionError::NotStarted
        | DirectedEvolutionError::TooLarge { .. }
        | DirectedEvolutionError::Environment { .. } => 2,
        DirectedEvolutionError::Journal(_)
        | DirectedEvolutionError::Asked
        | DirectedEvolutionError::Build(_)
        | DirectedEvolutionError::Action { .. } => 1,
    };
    (status, error.to_string())
}

#[cfg(test)]
mod tests;
