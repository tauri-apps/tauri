// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod app_paths;
pub mod cargo;
pub mod cargo_manifest;
pub mod config;
pub mod flock;
pub mod framework;
pub mod fs;
pub mod npm;
#[cfg(target_os = "macos")]
pub mod pbxproj;
pub mod plugins;
pub mod prompts;
pub mod template;
pub mod updater_signature;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Context;
use tauri_utils::config::HookCommand;

use crate::{
  interface::{AppInterface, Interface},
  CommandExt,
};

use self::app_paths::frontend_dir;

pub fn command_env(debug: bool) -> HashMap<&'static str, String> {
  let mut map = HashMap::new();

  map.insert(
    "TAURI_ENV_PLATFORM_VERSION",
    os_info::get().version().to_string(),
  );

  if debug {
    map.insert("TAURI_ENV_DEBUG", "true".into());
  }

  map
}

pub fn resolve_tauri_path<P: AsRef<Path>>(path: P, crate_name: &str) -> PathBuf {
  let path = path.as_ref();
  if path.is_absolute() {
    path.join(crate_name)
  } else {
    PathBuf::from("..").join(path).join(crate_name)
  }
}

pub fn cross_command(bin: &str) -> Command {
  #[cfg(target_os = "windows")]
  let cmd = {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg(bin);
    cmd
  };
  #[cfg(not(target_os = "windows"))]
  let cmd = Command::new(bin);
  cmd
}

pub fn run_hook(
  name: &str,
  hook: HookCommand,
  interface: &AppInterface,
  debug: bool,
) -> crate::Result<()> {
  let (script, script_cwd) = match hook {
    HookCommand::Script(s) if s.is_empty() => (None, None),
    HookCommand::Script(s) => (Some(s), None),
    HookCommand::ScriptWithOptions { script, cwd } => (Some(script), cwd.map(Into::into)),
  };
  let cwd = script_cwd.unwrap_or_else(|| frontend_dir().clone());
  if let Some(script) = script {
    log::info!(action = "Running"; "{} `{}`", name, script);

    let mut env = command_env(debug);
    env.extend(interface.env());

    log::debug!("Setting environment for hook {:?}", env);

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
      .arg("/S")
      .arg("/C")
      .arg(&script)
      .current_dir(cwd)
      .envs(env)
      .piped()
      .with_context(|| format!("failed to run `{script}` with `cmd /C`"))?;
    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh")
      .arg("-c")
      .arg(&script)
      .current_dir(cwd)
      .envs(env)
      .piped()
      .with_context(|| format!("failed to run `{script}` with `sh -c`"))?;

    if !status.success() {
      anyhow::bail!(
        "{} `{}` failed with exit code {}",
        name,
        script,
        status.code().unwrap_or_default()
      );
    }
  }

  Ok(())
}

#[cfg(target_os = "macos")]
pub fn strip_semver_prerelease_tag(version: &mut semver::Version) -> crate::Result<()> {
  if !version.pre.is_empty() {
    if let Some((_prerelease_tag, number)) = version.pre.as_str().to_string().split_once('.') {
      version.pre = semver::Prerelease::EMPTY;
      version.build = semver::BuildMetadata::new(&format!(
        "{prefix}{number}",
        prefix = if version.build.is_empty() {
          "".to_string()
        } else {
          format!(".{}", version.build.as_str())
        }
      ))
      .with_context(|| format!("bundle version {number:?} prerelease is invalid"))?;
    }
  }

  Ok(())
}
