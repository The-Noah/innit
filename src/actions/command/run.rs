use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct CommandRun {
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
