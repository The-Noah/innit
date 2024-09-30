use std::io::Write;

use fs::File;
use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct FileDownload {
  url: String,
  dest: String,
}

impl ActionHandler for FileDownload {
  fn run(&self) -> ActionResult {
    let dest = path::absolute(PathBuf::from(evaluate_vars(&self.dest))).unwrap();

    println!("Downloading file {} -> {}", &self.url, dest.display());

    if dest.exists() {
      return ActionResult::Failure("target file already exists".to_string());
    }

    let response = if let Ok(response) = reqwest::blocking::get(&self.url) {
      response
    } else {
      return ActionResult::Failure("error downloading file".to_string());
    };

    let mut file = if let Ok(file) = File::create(dest) {
      file
    } else {
      return ActionResult::Failure("error creating file".to_string());
    };

    file.write_all(&response.bytes().unwrap()).unwrap();

    ActionResult::Success
  }
}
