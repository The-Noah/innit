use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct PackageInstall {
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
