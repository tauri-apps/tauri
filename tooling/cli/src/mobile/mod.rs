// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{app_paths::tauri_dir, config::Config as TauriConfig},
  interface::DevProcess,
};
use anyhow::{bail, Result};
use cargo_mobile::{
  bossy,
  config::app::{App, Raw as RawAppConfig},
  env::Error as EnvError,
  opts::NoiseLevel,
};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};
use shared_child::SharedChild;
use std::{
  collections::HashMap,
  env::set_var,
  ffi::OsString,
  fmt::Write,
  io::{BufRead, BufReader, Write as _},
  path::PathBuf,
  process::ExitStatus,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

#[cfg(not(windows))]
use cargo_mobile::env::Env;
#[cfg(windows)]
use cargo_mobile::os::Env;

pub mod android;
mod init;
#[cfg(target_os = "macos")]
pub mod ios;

const MIN_DEVICE_MATCH_SCORE: isize = 0;

#[derive(Clone)]
pub struct DevChild {
  child: Arc<SharedChild>,
  manually_killed_process: Arc<AtomicBool>,
}

impl DevChild {
  fn new(handle: bossy::Handle) -> Self {
    Self {
      child: Arc::new(SharedChild::new(handle.into()).unwrap()),
      manually_killed_process: Default::default(),
    }
  }
}

impl DevProcess for DevChild {
  fn kill(&self) -> std::io::Result<()> {
    self.child.kill()?;
    self.manually_killed_process.store(true, Ordering::Relaxed);
    Ok(())
  }

  fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
    self.child.try_wait()
  }

  fn wait(&self) -> std::io::Result<ExitStatus> {
    self.child.wait()
  }

  fn manually_killed_process(&self) -> bool {
    self.manually_killed_process.load(Ordering::Relaxed)
  }

  fn is_building_app(&self) -> bool {
    false
  }
}

#[derive(PartialEq, Eq, Copy, Clone)]
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

  fn ide_build_script_name(&self) -> &'static str {
    match self {
      Self::Android => "android-studio-script",
      #[cfg(target_os = "macos")]
      Self::Ios => "xcode-script",
    }
  }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliOptions {
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub noise_level: NoiseLevel,
  pub vars: HashMap<String, OsString>,
}

fn options_local_socket_name(bundle_identifier: &str, target: Target) -> PathBuf {
  let out_dir = std::env::temp_dir();
  let out_dir = out_dir.join(".tauri").join(bundle_identifier);
  let _ = std::fs::create_dir_all(&out_dir);
  out_dir
    .join("cli-options")
    .with_extension(target.command_name())
}

fn env_vars() -> HashMap<String, OsString> {
  let mut vars = HashMap::new();
  for (k, v) in std::env::vars_os() {
    let k = k.to_string_lossy();
    if (k.starts_with("TAURI") && k != "TAURI_PRIVATE_KEY" && k != "TAURI_KEY_PASSWORD")
      || k.starts_with("WRY")
      || k == "TMPDIR"
    {
      vars.insert(k.into_owned(), v);
    }
  }
  vars
}

fn env() -> Result<Env, EnvError> {
  let env = Env::new()?.explicit_env_vars(env_vars());
  Ok(env)
}

/// Writes CLI options to be used later on the Xcode and Android Studio build commands
pub fn write_options(
  mut options: CliOptions,
  bundle_identifier: &str,
  target: Target,
) -> crate::Result<()> {
  options.vars.extend(env_vars());
  let name = options_local_socket_name(bundle_identifier, target);
  let _ = std::fs::remove_file(&name);
  let mut value = serde_json::to_string(&options)?;
  value.push('\n');

  std::thread::spawn(move || {
    let listener = LocalSocketListener::bind(name).expect("failed to start local socket");
    for mut conn in listener.incoming().flatten() {
      let _ = conn.write_all(value.as_bytes());
    }
  });

  Ok(())
}

fn read_options(config: &TauriConfig, target: Target) -> CliOptions {
  let name = options_local_socket_name(&config.tauri.bundle.identifier, target);
  let conn = LocalSocketStream::connect(name).unwrap_or_else(|_| {
    log::error!(
      "failed to connect to local socket. You must keep the Tauri CLI alive with the `{cmd} dev` or `{cmd} build --open` commands.",
      cmd = target.command_name()
    );
    std::process::exit(1);
  });
  conn
    .set_nonblocking(true)
    .expect("failed to set local socket stream to nonblocking");
  let mut conn = BufReader::new(conn);

  let mut attempt = 0;
  let max_tries = 5;
  let buffer = loop {
    let mut buffer = String::new();
    if conn.read_line(&mut buffer).is_ok() {
      break buffer;
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    attempt += 1;
    if attempt == max_tries {
      log::error!(
      "failed to connect to local socket. You must keep the Tauri CLI alive with the `{cmd} dev` or `{cmd} build --open` commands.",
      cmd = target.command_name()
    );
      std::process::exit(1);
    }
  };

  let options: CliOptions = serde_json::from_str(&buffer).expect("invalid CLI options");
  for (k, v) in &options.vars {
    set_var(k, v);
  }
  options
}

fn get_app(config: &TauriConfig) -> App {
  let mut s = config.tauri.bundle.identifier.rsplit('.');
  let app_name = s.next().unwrap_or("app").to_string();
  let mut domain = String::new();
  for w in s {
    domain.push_str(w);
    domain.push('.');
  }
  domain.pop();

  let s = config.tauri.bundle.identifier.split('.');
  let last = s.clone().count() - 1;
  let mut reverse_domain = String::new();
  for (i, w) in s.enumerate() {
    if i != last {
      reverse_domain.push_str(w);
      reverse_domain.push('.');
    }
  }
  reverse_domain.pop();

  let manifest_path = tauri_dir().join("Cargo.toml");
  let app_name = if let Ok(manifest) = crate::interface::manifest::read_manifest(&manifest_path) {
    manifest
      .as_table()
      .get("package")
      .and_then(|p| p.as_table())
      .and_then(|p| p.get("name"))
      .and_then(|n| n.as_str())
      .map(|n| n.to_string())
      .unwrap_or(app_name)
  } else {
    app_name
  };

  let raw = RawAppConfig {
    name: app_name,
    stylized_name: config.package.product_name.clone(),
    domain,
    asset_dir: None,
    template_pack: None,
  };
  App::from_raw(tauri_dir(), raw).unwrap()
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

fn log_finished(outputs: Vec<PathBuf>, kind: &str) {
  if !outputs.is_empty() {
    let mut printable_paths = String::new();
    for path in &outputs {
      writeln!(printable_paths, "        {}", path.display()).unwrap();
    }

    log::info!(action = "Finished"; "{} {}{} at:\n{}", outputs.len(), kind, if outputs.len() == 1 { "" } else { "s" }, printable_paths);
  }
}
