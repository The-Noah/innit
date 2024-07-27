use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
  packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
  name: String,
  winget_id: String,
  cmd: Option<Vec<String>>,
  tags: Option<Vec<String>>,
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

  println!("Found {} packages", config.packages.len());

  for package in config.packages {
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

    let result = Command::new("winget")
      .arg("install")
      .arg(&package.winget_id)
      .arg("--exact")
      .arg("--silent")
      .arg("--accept-package-agreements")
      .arg("--disable-interactivity")
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
                    println!("Failed to execute command");
                    break;
                  }
                }
                Err(error) => {
                  println!("Failed to execute command: {}", error);
                  break;
                }
              }
            }
          }
        } else {
          println!("Failed to install package");
        }
      }
      Err(error) => {
        println!("Failed to install package: {}", error);
      }
    }
  }
}
