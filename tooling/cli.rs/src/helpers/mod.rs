// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod app_paths;
pub mod config;
pub mod framework;
mod logger;
pub mod manifest;
pub mod updater_signature;

pub use logger::Logger;

use std::{
  collections::HashMap,
  io::{BufRead, BufReader},
  process::{Command, Stdio},
};

pub fn execute_with_output(cmd: &mut Command) -> crate::Result<()> {
  let mut child = cmd
    .stdout(Stdio::piped())
    .spawn()
    .expect("failed to spawn command");
  {
    let stdout = child.stdout.as_mut().expect("Failed to get stdout handle");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      println!("{}", line.expect("Failed to get line"));
    }
  }

  let status = child.wait()?;
  if status.success() {
    Ok(())
  } else {
    Err(anyhow::anyhow!("command failed"))
  }
}

pub fn command_env() -> HashMap<String, String> {
  let mut map = HashMap::new();
  map.insert("PLATFORM".into(), std::env::consts::OS.into());
  map.insert("ARCH".into(), std::env::consts::ARCH.into());
  map.insert("FAMILY".into(), std::env::consts::FAMILY.into());
  map.insert(
    "VERSION".into(),
    os_info::get().version().to_string().into(),
  );

  #[cfg(target_os = "linux")]
  map.insert("PLATFORM_TYPE".into(), "Linux".into());
  #[cfg(target_os = "windows")]
  map.insert("PLATFORM_TYPE".into(), "Windows_NT".into());
  #[cfg(target_os = "macos")]
  map.insert("PLATFORM_TYPE".into(), "Darwing".into());

  map
}
