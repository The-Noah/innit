use std::{
  fs,
  path::{self, PathBuf},
  process::{Command, Stdio},
};

use serde::Deserialize;

enum ActionResult {
  Success,
  Failure(String),
  Skipped(String),
}

trait ActionHandler {
  fn run(&self, tags: &[String]) -> ActionResult;
}

#[derive(Deserialize)]
#[serde(tag = "action")]
enum Action {
  #[serde(rename = "package.install")]
  PackageInstall(PackageInstall),
  #[serde(rename = "file.link")]
  FileLink(FileLink),
  #[serde(rename = "github.repo")]
  GitHubRepo(GitHubRepo),
  #[serde(rename = "command.run")]
  CommandRun(CommandRun),
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

#[derive(Deserialize)]
struct PackageInstall {
  name: String,
  winget_id: String,
  tags: Option<Vec<String>>,
}

impl ActionHandler for PackageInstall {
  fn run(&self, tags: &[String]) -> ActionResult {
    if let Some(package_tags) = &self.tags {
      if !tags.is_empty() && !tags.iter().any(|tag| package_tags.contains(tag)) {
        return ActionResult::Skipped("no matching tags".to_string());
      }
    }

    println!();
    println!("Installing package: {}", self.name);
    println!("Winget ID: {}", self.winget_id);

    let result = Command::new("winget").arg("list").arg("-q").arg(&self.winget_id).stdout(Stdio::null()).status();
    match result {
      Ok(status) => {
        if status.success() {
          return ActionResult::Skipped("already installed".to_string());
        }
      }
      Err(error) => {
        eprintln!("Failed to find package info: {}", error);
      }
    }

    let result = Command::new("winget")
      .arg("install")
      .arg(&self.winget_id)
      .arg("--exact")
      .arg("--silent")
      .arg("--accept-package-agreements")
      .arg("--disable-interactivity")
      .stdout(Stdio::null())
      .status();

    match result {
      Ok(status) => {
        if !status.success() {
          return ActionResult::Failure("unknown error".to_string());
        }
      }
      Err(error) => {
        return ActionResult::Failure(error.to_string());
      }
    }

    ActionResult::Success
  }
}

#[derive(Deserialize)]
struct FileLink {
  src: String,
  dest: String,
  #[serde(default = "default_hard")]
  hard: bool,
}

fn default_hard() -> bool {
  false
}

impl ActionHandler for FileLink {
  fn run(&self, tags: &[String]) -> ActionResult {
    let src = path::absolute(PathBuf::from(evaluate_vars(&self.src))).unwrap();
    let dest = path::absolute(PathBuf::from(evaluate_vars(&self.dest))).unwrap();

    println!();
    println!("Symlinking file: {}", src.file_name().unwrap().to_string_lossy());

    if dest.exists() && !dest.is_symlink() {
      let mut new_dest = dest.clone();
      new_dest.set_file_name(format!("{}.bak", dest.file_name().unwrap().to_string_lossy()));

      fs::rename(dest.clone(), new_dest).unwrap();
    }

    fs::remove_file(&dest).unwrap();

    if self.hard {
      fs::hard_link(&src, &dest).unwrap();
    } else {
      #[cfg(unix)]
      {
        std::os::unix::fs::symlink(&src, &dest).unwrap();
      }

      #[cfg(windows)]
      {
        if dest.is_dir() {
          std::os::windows::fs::symlink_dir(&src, &dest).unwrap();
        } else {
          std::os::windows::fs::symlink_file(&src, &dest).unwrap();
        }
      }
    }

    ActionResult::Success
  }
}

#[derive(Deserialize)]
struct GitHubRepo {
  repo: String,
  dest: String,
}

impl ActionHandler for GitHubRepo {
  fn run(&self, tags: &[String]) -> ActionResult {
    println!();
    println!("Cloning GitHub repository {}...", self.repo);

    let dest = path::absolute(PathBuf::from(evaluate_vars(&self.dest))).unwrap();

    let mut final_dest = dest.clone();
    final_dest.push(self.repo.split("/").collect::<Vec<&str>>().last().unwrap());

    if final_dest.exists() {
      let mut git_dir = final_dest.clone();
      git_dir.push(".git");

      if git_dir.exists() {
        println!("Repository target already exists, pulling...");

        match Command::new("git").current_dir(final_dest).arg("pull").stdout(Stdio::null()).status() {
          Ok(status) => {
            if !status.success() {
              return ActionResult::Failure("unknown git error".to_string());
            }
          }
          Err(error) => {
            return ActionResult::Failure(error.to_string());
          }
        }
      } else {
        return ActionResult::Failure("directory exists but isn't git repository".to_string());
      }

      return ActionResult::Success;
    }

    match Command::new("git")
      .current_dir(dest)
      .arg("clone")
      .arg(format!("https://github.com/{}.git", self.repo))
      .stdout(Stdio::null())
      .status()
    {
      Ok(status) => {
        if !status.success() {
          return ActionResult::Failure("unknown git error".to_string());
        }
      }
      Err(error) => {
        return ActionResult::Failure(error.to_string());
      }
    }

    ActionResult::Success
  }
}

#[derive(Deserialize)]
struct CommandRun {
  command: String,
}

impl ActionHandler for CommandRun {
  fn run(&self, tags: &[String]) -> ActionResult {
    println!();
    println!("Running command: {}", self.command);

    let result = Command::new("cmd").arg("/C").arg(self.command.clone()).stdout(Stdio::null()).status();
    match result {
      Ok(status) => {
        if !status.success() {
          return ActionResult::Failure("failed to execute command for unknown reason".to_string());
        }
      }
      Err(error) => {
        return ActionResult::Failure(error.to_string());
      }
    }

    ActionResult::Success
  }
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
