use std::{
  fs,
  path::{self, PathBuf},
  process::{Command, Stdio},
};

use serde::Deserialize;

mod actions;

pub enum ActionResult {
  Success,
  Failure(String),
  Skipped(String),
}

pub trait ActionHandler {
  fn run(&self) -> ActionResult;
}

pub trait ActionRunner {
  fn run(&self, tags: &[String]) -> ActionResult;
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Platform {
  Windows,
  MacOS,
  Linux,
}

impl Platform {
  fn as_str(&self) -> &'static str {
    match self {
      Platform::Windows => "windows",
      Platform::MacOS => "macos",
      Platform::Linux => "linux",
    }
  }
}

#[derive(Deserialize)]
struct ActionContainer<T> {
  #[serde(flatten)]
  pub action: T,

  pub tags: Option<Vec<String>>,
  pub platforms: Option<Vec<Platform>>,
}

impl<T> ActionRunner for ActionContainer<T>
where
  T: ActionHandler,
{
  fn run(&self, tags: &[String]) -> ActionResult {
    println!();

    if !tags.is_empty() && !tags.iter().any(|tag| self.tags.as_ref().unwrap_or(&Vec::<String>::new()).contains(tag)) {
      return ActionResult::Skipped("no matching tags".to_string());
    }

    if self.platforms.is_some() && !self.platforms.as_ref().unwrap().iter().any(|platform| platform.as_str() == std::env::consts::OS) {
      return ActionResult::Skipped("not for this platform".to_string());
    }

    self.action.run()
  }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, tag = "action")]
enum Action {
  #[serde(rename = "package.install")]
  PackageInstall(ActionContainer<actions::package::Install>),
  #[serde(rename = "file.link")]
  FileLink(ActionContainer<actions::file::Link>),
  #[serde(rename = "github.repo")]
  GitHubRepo(ActionContainer<actions::github::Repo>),
  #[serde(rename = "command.run")]
  CommandRun(ActionContainer<actions::command::Run>),
}

impl Action {
  fn inner_ref(&self) -> &dyn ActionRunner {
    match self {
      Action::PackageInstall(action) => action,
      Action::FileLink(action) => action,
      Action::GitHubRepo(action) => action,
      Action::CommandRun(action) => action,
    }
  }
}

#[derive(Deserialize)]
struct Config {
  actions: Vec<Action>,
}

fn main() {
  println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

  let args: Vec<String> = std::env::args().collect();

  if args.len() < 2 {
    println!("Usage: {} <filename>", args[0]);
    std::process::exit(1);
  }

  let filename = &args[1];

  let contents = std::fs::read_to_string(filename).expect("Something went wrong reading the file");

  let tags = if args.len() > 3 && (args[2] == "--tags" || args[2] == "-t") {
    args[3].split(",").map(|s| s.to_string()).collect()
  } else {
    vec![]
  };

  println!("Tags: {}", tags.join(", "));

  let config: Config = serde_yml::from_str(&contents).expect("Failed to parse config");

  println!("Found {} actions", config.actions.len());

  for action in config.actions {
    match action.inner_ref().run(&tags) {
      ActionResult::Success => println!("Success"),
      ActionResult::Failure(reason) => eprintln!("Failure: {}", reason),
      ActionResult::Skipped(reason) => println!("Skipped: {}", reason),
    }
  }
}

fn evaluate_vars(text: &str) -> String {
  text.replace("{{ user.home }}", &dirs::home_dir().unwrap().display().to_string())
}
