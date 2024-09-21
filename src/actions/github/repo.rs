use serde::Deserialize;

use crate::*;

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
