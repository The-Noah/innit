use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct PackageInstall {
  name: String,
  winget_id: String,
}

impl ActionHandler for PackageInstall {
  fn run(&self) -> ActionResult {
    println!("Installing package: {}", self.name);
    println!("Winget ID: {}", self.winget_id);

    let result = Command::new("winget").arg("list").arg("--id").arg(&self.winget_id).stdout(Stdio::null()).status();
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
      .arg("--id") // use id for matching
      .arg(&self.winget_id)
      .arg("--exact") // only match the exact item
      .arg("--silent") // request silent installation
      .arg("--accept-package-agreements") // accept all license agreements
      .arg("--disable-interactivity") // disable interactive prompts
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
