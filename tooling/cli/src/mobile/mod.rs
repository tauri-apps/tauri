// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};
use std::path::PathBuf;

pub mod android;
mod init;
#[cfg(target_os = "macos")]
pub mod ios;

#[derive(PartialEq, Eq)]
pub enum Target {
  Android,
  #[cfg(target_os = "macos")]
  Ios,
}

impl Target {
  fn ide_name(&self) -> &'static str {
    match self {
      Self::Android => "Android Studio",
      #[cfg(target_os = "macos")]
      Self::Ios => "Xcode",
    }
  }

  fn command_name(&self) -> &'static str {
    match self {
      Self::Android => "android",
      #[cfg(target_os = "macos")]
      Self::Ios => "ios",
    }
  }
}

fn ensure_init(project_dir: PathBuf, target: Target) -> Result<()> {
  if !project_dir.exists() {
    bail!(
      "{} project directory {} doesn't exist. Please run `tauri {} init` and try again.",
      target.ide_name(),
      project_dir.display(),
      target.command_name(),
    )
  } else {
    Ok(())
  }
}
