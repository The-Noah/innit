use std::{
  fs,
  path::PathBuf,
  process::{Command, Stdio},
};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "action")]
enum Action {
  #[serde(rename = "package.install")]
  PackageInstall(Package),
  #[serde(rename = "file.link")]
  FileLink(FileLink),
  #[serde(rename = "github.repo")]
  GitHubRepo(GitHubRepo),
}

#[derive(Deserialize)]
struct Config {
  actions: Vec<Action>,
}

#[derive(Deserialize)]
struct Package {
  name: String,
  winget_id: String,
  cmd: Option<Vec<String>>,
  tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct FileLink {
  src: String,
  dest: String,
  #[serde(default = "default_hard")]
  hard: bool,
}

#[derive(Deserialize)]
struct GitHubRepo {
  repo: String,
  dest: PathBuf,
}

fn default_hard() -> bool {
  false
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
    match action {
      Action::PackageInstall(package) => {
        if let Some(package_tags) = &package.tags {
          if !tags.is_empty() && !tags.iter().any(|tag| package_tags.contains(tag)) {
            continue;
          }
        }

        println!();
        println!("Installing package: {}", package.name);
        println!("Winget ID: {}", package.winget_id);

        if let Some(cmd) = &package.cmd {
          println!("Custom command(s): {}", cmd.join(", "));
        }

        let result = Command::new("winget").arg("list").arg("-q").arg(&package.winget_id).stdout(Stdio::null()).status();
        match result {
          Ok(status) => {
            if status.success() {
              println!("Package already installed. Skipping");
              continue;
            }
          }
          Err(error) => {
            eprintln!("Failed to find package info: {}", error);
          }
        }

        let result = Command::new("winget")
          .arg("install")
          .arg(&package.winget_id)
          .arg("--exact")
          .arg("--silent")
          .arg("--accept-package-agreements")
          .arg("--disable-interactivity")
          .stdout(Stdio::null())
          .status();

        match result {
          Ok(status) => {
            if status.success() {
              println!("Package installed successfully");

              if let Some(cmd) = package.cmd {
                for command in cmd {
                  println!("Running command: {}", command);

                  let result = Command::new("cmd").arg("/C").arg(command).status();
                  match result {
                    Ok(status) => {
                      if status.success() {
                        println!("Command executed successfully");
                      } else {
                        eprintln!("Failed to execute command");
                        break;
                      }
                    }
                    Err(error) => {
                      eprintln!("Failed to execute command: {}", error);
                      break;
                    }
                  }
                }
              }
            } else {
              eprintln!("Failed to install package");
            }
          }
          Err(error) => {
            eprintln!("Failed to install package: {}", error);
          }
        }
      }
      Action::FileLink(file) => {
        let src = PathBuf::from(file.src);
        let dest = PathBuf::from(file.dest);

        println!();
        println!("Symlinking file: {}", src.file_name().unwrap().to_string_lossy());

        if dest.exists() && !dest.is_symlink() {
          let mut new_dest = dest.clone();
          new_dest.set_file_name(format!("{}.bak", dest.file_name().unwrap().to_string_lossy()));

          fs::rename(dest.clone(), new_dest).unwrap();
        }

        fs::remove_file(&dest).unwrap();

        if file.hard {
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
      }
      Action::GitHubRepo(repo) => {
        println!();
        println!("Cloning GitHub repository {}...", repo.repo);

        let mut final_dest = repo.dest.clone();
        final_dest.push(repo.repo.split("/").collect::<Vec<&str>>().last().unwrap());

        if final_dest.exists() {
          let mut git_dir = final_dest.clone();
          git_dir.push(".git");

          if git_dir.exists() {
            println!("Repository target already exists, pulling...");

            match Command::new("git").current_dir(final_dest).arg("pull").stdout(Stdio::null()).status() {
              Ok(status) => {
                if status.success() {
                  println!("Repository updated");
                } else {
                  eprintln!("Failed to update repository")
                }
              }
              Err(error) => {
                eprintln!("Failed to update repository: {}", error);
              }
            }
          } else {
            eprintln!("Directory exists but isn't git repository");
          }

          continue;
        }

        match Command::new("git")
          .current_dir(repo.dest)
          .arg("clone")
          .arg(format!("https://github.com/{}.git", repo.repo))
          .stdout(Stdio::null())
          .status()
        {
          Ok(status) => {
            if status.success() {
              println!("Repository cloned");
            } else {
              eprintln!("Failed to clone repository")
            }
          }
          Err(error) => {
            eprintln!("Failed to clone repository: {}", error);
          }
        }
      }
    };
  }
}
