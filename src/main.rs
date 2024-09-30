use std::{
  fs,
  path::{self, PathBuf},
  process::{exit, Command, Stdio},
};

use dirs::home_dir;
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
  #[serde(rename = "file.download")]
  FileDownload(ActionContainer<actions::file::Download>),
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
      Action::FileDownload(action) => action,
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
  let args = std::env::args().collect::<Vec<String>>();
  let args = args.split_at(1).1; // remove self from args list

  if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
    print_help();
    return;
  } else if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    return;
  }

  let config_name = if let Some(i) = args.iter().position(|arg| arg == "--config" || arg == "-c") {
    if args.len() <= i + 1 {
      eprintln!("You must specify a filename after config flag");
      exit(1);
    }

    Some(args.get(i + 1).unwrap())
  } else {
    None
  };

  let config_path = if let Some(config_name) = config_name {
    PathBuf::from(".").join(config_name)
  } else {
    let path = home_dir().unwrap().join(".config").join("innit.yaml");
    let dotfiles_path = home_dir().unwrap().join("dotfiles").join(".config").join("innit.yaml");

    if !path.exists() && dotfiles_path.exists() {
      dotfiles_path
    } else {
      path
    }
  };

  if !config_path.exists() {
    eprintln!("Could not find config file");
    exit(1);
  }

  let contents = fs::read_to_string(&config_path).expect("Something went wrong reading the file");

  let tags = if let Some(i) = args.iter().position(|arg| arg == "--tags" || arg == "-t") {
    if args.len() <= i + 1 {
      eprintln!("You must specify a value after tags flag");
      exit(1);
    }

    args.get(i + 1).unwrap().split(",").map(|s| s.to_string()).collect()
  } else {
    vec![]
  };

  println!("Config: {}", config_path.display());
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

fn print_help() {
  println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
  println!("Usage: {} [options]", env!("CARGO_PKG_NAME"));
  println!();
  println!("Options:");
  println!("  -v, --version           Print version information.");
  println!("  -h, --help              Print this help message.");
  println!("  -c, --config <path>     Use specified config file.");
  println!("  -t, --tags <tag1,tag2>  Only run actions with specified tag(s).");
  println!();
  println!("Config:");
  println!("  If you used -c or --config, the file you specified will be used.");
  println!("  Otherwise, $HOME/.config/innit.yaml will be used if it exists.");
  println!("  Otherwise, $HOME/dotfiles/.config/innit.yaml will be used if it exists.");
}
