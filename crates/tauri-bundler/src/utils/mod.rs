// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::OsStr,
  io::{BufRead, BufReader},
  path::Path,
  process::{Command, ExitStatus, Output, Stdio},
  sync::{Arc, Mutex},
};

pub mod fs_utils;
pub mod http_utils;

/// Returns true if the path has a filename indicating that it is a high-density
/// "retina" icon.  Specifically, returns true the file stem ends with
/// "@2x" (a convention specified by the [Apple developer docs](
/// <https://developer.apple.com/library/mac/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html>)).
#[allow(dead_code)]
pub fn is_retina(path: &Path) -> bool {
  path
    .file_stem()
    .and_then(OsStr::to_str)
    .map(|stem| stem.ends_with("@2x"))
    .unwrap_or(false)
}

pub trait CommandExt {
  // The `pipe` function sets the stdout and stderr to properly
  // show the command output in the Node.js wrapper.
  fn piped(&mut self) -> std::io::Result<ExitStatus>;
  fn output_ok(&mut self) -> crate::Result<Output>;
}

impl CommandExt for Command {
  fn piped(&mut self) -> std::io::Result<ExitStatus> {
    self.stdin(os_pipe::dup_stdin()?);
    self.stdout(os_pipe::dup_stdout()?);
    self.stderr(os_pipe::dup_stderr()?);
    let program = self.get_program().to_string_lossy().into_owned();
    log::debug!(action = "Running"; "Command `{} {}`", program, self.get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{acc} {arg}")));

    self.status().map_err(Into::into)
  }

  fn output_ok(&mut self) -> crate::Result<Output> {
    let program = self.get_program().to_string_lossy().into_owned();
    log::debug!(action = "Running"; "Command `{} {}`", program, self.get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{acc} {arg}")));

    self.stdout(Stdio::piped());
    self.stderr(Stdio::piped());

    let mut child = self.spawn()?;

    let mut stdout = child.stdout.take().map(BufReader::new).unwrap();
    let stdout_lines = Arc::new(Mutex::new(Vec::new()));
    let stdout_lines_ = stdout_lines.clone();
    std::thread::spawn(move || {
      let mut line = String::new();
      let mut lines = stdout_lines_.lock().unwrap();
      loop {
        line.clear();
        match stdout.read_line(&mut line) {
          Ok(0) => break,
          Ok(_) => {
            log::debug!(action = "stdout"; "{}", line.trim_end());
            lines.extend(line.as_bytes().to_vec());
          }
          Err(_) => (),
        }
      }
    });

    let mut stderr = child.stderr.take().map(BufReader::new).unwrap();
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines_ = stderr_lines.clone();
    std::thread::spawn(move || {
      let mut line = String::new();
      let mut lines = stderr_lines_.lock().unwrap();
      loop {
        line.clear();
        match stderr.read_line(&mut line) {
          Ok(0) => break,
          Ok(_) => {
            log::debug!(action = "stderr"; "{}", line.trim_end());
            lines.extend(line.as_bytes().to_vec());
          }
          Err(_) => (),
        }
      }
    });

    let status = child.wait()?;
    let output = Output {
      status,
      stdout: std::mem::take(&mut *stdout_lines.lock().unwrap()),
      stderr: std::mem::take(&mut *stderr_lines.lock().unwrap()),
    };

    if output.status.success() {
      Ok(output)
    } else {
      Err(crate::Error::GenericError(format!(
        "failed to run {program}"
      )))
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use tauri_utils::resources::resource_relpath;

  use super::is_retina;

  #[test]
  fn retina_icon_paths() {
    assert!(!is_retina(Path::new("data/icons/512x512.png")));
    assert!(is_retina(Path::new("data/icons/512x512@2x.png")));
  }

  #[test]
  fn resource_relative_paths() {
    assert_eq!(
      resource_relpath(Path::new("./data/images/button.png")),
      PathBuf::from("data/images/button.png")
    );
    assert_eq!(
      resource_relpath(Path::new("../../images/wheel.png")),
      PathBuf::from("_up_/_up_/images/wheel.png")
    );
    assert_eq!(
      resource_relpath(Path::new("/home/ferris/crab.png")),
      PathBuf::from("_root_/home/ferris/crab.png")
    );
  }
}
