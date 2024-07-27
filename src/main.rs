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

  let config: Config = serde_yml::from_str(&contents).unwrap();

  println!("Found {} packages", config.packages.len());

  for package in config.packages {
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
