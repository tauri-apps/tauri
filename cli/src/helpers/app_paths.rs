use std::env::current_dir;
use std::path::PathBuf;

pub fn app_dir() -> PathBuf {
  let mut dir = current_dir().expect("failed to read cwd");

  let mut count = 0;

  // only go up three folders max
  while count <= 2 {
    let test_path = dir.join("src-tauri/tauri.conf.json");
    if test_path.exists() {
      return dir;
    }
    count += 1;
    match dir.parent() {
      Some(parent) => {
        dir = parent.to_path_buf();
      }
      None => break,
    }
  }

  panic!("Couldn't recognize the current folder as a Tauri project.")
}

pub fn tauri_dir() -> PathBuf {
  app_dir().join("src-tauri")
}
