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
  fn run(&self, tags: &[String]) -> ActionResult;
}

#[derive(Deserialize)]
#[serde(tag = "action")]
enum Action {
  #[serde(rename = "package.install")]
  PackageInstall(actions::package::Install),
  #[serde(rename = "file.link")]
  FileLink(actions::file::Link),
  #[serde(rename = "github.repo")]
  GitHubRepo(actions::github::Repo),
  #[serde(rename = "command.run")]
  CommandRun(actions::command::Run),
}

impl Action {
  fn inner_ref(&self) -> &dyn ActionHandler {
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
