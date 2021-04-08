use std::{
  env,
  path::PathBuf,
  process::{exit, Command},
};

/// Get the current binary
pub fn current_binary() -> Option<PathBuf> {
  let mut current_binary = None;

  // if we are running with an APP Image, we should return the app image path
  #[cfg(target_os = "linux")]
  if let Some(app_image_path) = env::var_os("APPIMAGE") {
    current_binary = Some(PathBuf::from(app_image_path));
  }

  // if we didn't extracted binary in previous step,
  // let use the current_exe from current environment
  if current_binary.is_none() {
    if let Ok(current_process) = env::current_exe() {
      current_binary = Some(current_process);
    }
  }

  current_binary
}

/// Restart application
pub fn restart_application(binary_to_start: Option<PathBuf>) {
  let mut binary_path = binary_to_start;
  // spawn new process
  if binary_path.is_none() {
    binary_path = current_binary();
  }

  if let Some(path) = binary_path {
    Command::new(path)
      .spawn()
      .expect("application failed to start");
  }

  exit(0);
}
