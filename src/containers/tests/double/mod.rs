use std::io;
use std::iter;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::containers::program::{Ending, Limits};
use crate::containers::{ContainerError, Docker, Outside};
use crate::file_system::FileSystem;
use crate::file_system::tests::double::Memory;
use crate::outputs_directory::OutputsDirectory;

const MADE: &str = ":/var/opt/cladenc/work";
const VERIFIER: &str = ":/logs/verifier/.";

#[derive(Clone)]
pub(crate) struct Asked {
    pub(crate) arguments: Vec<String>,
    pub(crate) variables: Vec<(String, String)>,
    pub(crate) input: String,
    pub(crate) kept: PathBuf,
}

#[derive(Clone, Copy)]
pub(crate) enum Answer {
    Exited(i32),
    TimedOut,
    Stopped,
    UntilStopped,
    Signal(i32),
    Refused(&'static str),
    Printed(&'static str),
}

struct Rule {
    needles: Vec<String>,
    answers: Vec<Answer>,
    given: usize,
}

type Effect = Box<dyn Fn(&Memory, &Asked) + Send + Sync>;

pub(crate) struct Double {
    asked: Mutex<Vec<Asked>>,
    rules: Mutex<Vec<Rule>>,
    effects: Vec<(Vec<String>, Effect)>,
    files: Memory,
    shared: Arc<dyn FileSystem>,
}

impl Default for Double {
    fn default() -> Self {
        let files = Memory::default();
        Self {
            asked: Mutex::default(),
            rules: Mutex::default(),
            effects: Vec::new(),
            shared: files.shared(),
            files,
        }
    }
}

impl Asked {
    fn of(command: &Command, input: &str, kept: &Path) -> Self {
        Self {
            arguments: iter::once(command.get_program())
                .chain(command.get_args())
                .map(|argument| argument.to_string_lossy().into_owned())
                .collect(),
            variables: command
                .get_envs()
                .filter_map(|(name, value)| {
                    value.map(|value| {
                        (
                            name.to_string_lossy().into_owned(),
                            value.to_string_lossy().into_owned(),
                        )
                    })
                })
                .collect(),
            input: input.to_owned(),
            kept: kept.to_owned(),
        }
    }

    fn fits(&self, needles: &[String]) -> bool {
        needles.iter().all(|needle| {
            self.arguments.contains(needle) || self.kept.to_string_lossy().ends_with(needle)
        })
    }

    fn name(&self) -> Option<&String> {
        self.arguments
            .iter()
            .position(|argument| argument == "--name")
            .and_then(|place| self.arguments.get(place + 1))
    }
}

impl Double {
    pub(crate) fn on(mut self, files: &Memory) -> Self {
        self.shared = files.shared();
        self.files = files.clone();
        self
    }

    pub(crate) fn outside(&self) -> Outside<'_> {
        Outside {
            docker: self,
            files: &self.shared,
        }
    }

    pub(crate) fn answering(self, needles: &[&str], answers: &[Answer]) -> Self {
        self.rules.lock().unwrap().push(Rule {
            needles: owned(needles),
            answers: answers.to_vec(),
            given: 0,
        });
        self
    }

    pub(crate) fn doing(
        mut self,
        needles: &[&str],
        effect: impl Fn(&Memory, &Asked) + Send + Sync + 'static,
    ) -> Self {
        self.effects.push((owned(needles), Box::new(effect)));
        self
    }

    pub(crate) fn changing(self) -> Self {
        self.changing_when(|_| true)
    }

    pub(crate) fn changing_when(
        self,
        changes: impl Fn(&Asked) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.doing(&["run"], move |files, asked| {
            let made = asked
                .arguments
                .iter()
                .find_map(|argument| argument.strip_suffix(MADE));
            if let Some(made) = made.filter(|_| changes(asked)) {
                let change = asked.kept.to_string_lossy().into_owned();
                files
                    .write(Path::new(made).join("changed"), change)
                    .unwrap();
            }
        })
    }

    pub(crate) fn grading(self, completes: impl Fn(&str) -> bool + Send + Sync + 'static) -> Self {
        self.doing(&["cp"], move |files, asked| {
            let from_the_verifier = asked
                .arguments
                .iter()
                .any(|argument| argument.ends_with(VERIFIER));
            let to = asked.arguments.last().map(PathBuf::from);
            if let Some(to) = to.filter(|_| from_the_verifier) {
                let reward = reward(completes(&attempt_of(&to)));
                files.write(to.join("reward.txt"), reward).unwrap();
            }
        })
    }

    pub(crate) fn asked(&self) -> Vec<Asked> {
        self.asked.lock().unwrap().clone()
    }

    pub(crate) fn commands(&self, outputs_directory: &OutputsDirectory) -> Vec<String> {
        let path = outputs_directory.path();
        let resolved = self
            .files
            .resolved(path)
            .unwrap_or_else(|_| path.to_owned());
        let asked = self.asked();
        let names: Vec<&String> = asked.iter().filter_map(Asked::name).collect();
        asked
            .iter()
            .map(|asked| {
                let command = asked
                    .arguments
                    .join(" ")
                    .replace(&*resolved.to_string_lossy(), "OUTPUTS")
                    .replace(&*path.to_string_lossy(), "OUTPUTS");
                names
                    .iter()
                    .fold(command, |command, name| command.replace(*name, "NAME"))
            })
            .collect()
    }

    fn answer(&self, command: &Command, input: &str, kept: &Path) -> Option<Answer> {
        let asked = Asked::of(command, input, kept);
        self.asked.lock().unwrap().push(asked.clone());
        self.effects
            .iter()
            .filter(|(needles, _)| asked.fits(needles))
            .for_each(|(_, effect)| effect(&self.files, &asked));
        let mut rules = self.rules.lock().unwrap();
        rules
            .iter_mut()
            .find(|rule| asked.fits(&rule.needles))
            .map(|rule| {
                let answer = rule.answers[rule.given.min(rule.answers.len() - 1)];
                rule.given += 1;
                answer
            })
    }
}

impl Docker for Double {
    fn run(
        &self,
        command: Command,
        input: &str,
        running: &Path,
        limits: Limits<'_>,
    ) -> Result<Ending, ContainerError> {
        let answer = self.answer(&command, input, running);
        while matches!(answer, Some(Answer::UntilStopped)) && !limits.stopped() {
            thread::yield_now();
        }
        ended(answer)
    }

    fn detach(&self, command: Command, input: &str, said: &Path) -> Result<Ending, ContainerError> {
        ended(self.answer(&command, input, said))
    }

    fn print(&self, command: Command) -> io::Result<String> {
        match self.answer(&command, "", Path::new("")) {
            None => Ok(String::new()),
            Some(Answer::Printed(text)) => Ok(text.to_owned()),
            Some(Answer::Refused(reason)) => Err(io::Error::other(reason)),
            Some(_) => Err(io::Error::other("the command did not succeed")),
        }
    }
}

fn attempt_of(to: &Path) -> String {
    to.ancestors()
        .nth(3)
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn reward(completed: bool) -> &'static str {
    if completed { "1" } else { "0" }
}

fn owned(needles: &[&str]) -> Vec<String> {
    needles.iter().map(|needle| (*needle).to_owned()).collect()
}

fn ended(answer: Option<Answer>) -> Result<Ending, ContainerError> {
    match answer {
        None | Some(Answer::Printed(_)) => Ok(Ending::Exited(0)),
        Some(Answer::Exited(status)) => Ok(Ending::Exited(status)),
        Some(Answer::TimedOut) => Ok(Ending::TimedOut),
        Some(Answer::Stopped | Answer::UntilStopped) => Err(ContainerError::Stopped),
        Some(Answer::Signal(signal)) => Err(ContainerError::Signal(signal)),
        Some(Answer::Refused(reason)) => Err(ContainerError::Io(io::Error::other(reason))),
    }
}
