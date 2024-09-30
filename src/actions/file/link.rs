use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct FileLink {
  src: String,
  dest: String,
  #[serde(default = "default_hard")]
  hard: bool,
}

fn default_hard() -> bool {
  false
}

impl ActionHandler for FileLink {
  fn run(&self) -> ActionResult {
    let src = path::absolute(PathBuf::from(evaluate_vars(&self.src))).unwrap();
    let dest = path::absolute(PathBuf::from(evaluate_vars(&self.dest))).unwrap();

    println!("Symlinking file: {}", src.file_name().unwrap().to_string_lossy());

    if dest.exists() {
      if dest.is_symlink() {
        fs::remove_file(&dest).unwrap();
      } else {
        let mut new_dest = dest.clone();
        new_dest.set_file_name(format!("{}.bak", dest.file_name().unwrap().to_string_lossy()));

        fs::rename(dest.clone(), new_dest).unwrap();
      }
    }

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
