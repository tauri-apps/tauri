// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, AppUrl, WindowUrl},
  },
  interface::{AppInterface, ExitReason, Interface},
  Result,
};
use clap::Parser;

use anyhow::Context;
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use shared_child::SharedChild;

use std::{
  env::set_current_dir,
  io::Write,
  process::{exit, Command, ExitStatus, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

static BEFORE_DEV: OnceCell<Mutex<Arc<SharedChild>>> = OnceCell::new();
static KILL_BEFORE_DEV_FLAG: OnceCell<AtomicBool> = OnceCell::new();

#[cfg(unix)]
const KILL_CHILDREN_SCRIPT: &[u8] = include_bytes!("../scripts/kill-children.sh");

pub const TAURI_DEV_WATCHER_GITIGNORE: &[u8] = include_bytes!("../tauri-dev-watcher.gitignore");

#[derive(Debug, Clone, Parser)]
#[clap(about = "Tauri dev", trailing_var_arg(true))]
pub struct Options {
  /// Binary to use to run the application
  #[clap(short, long)]
  pub runner: Option<String>,
  /// Target triple to build against
  #[clap(short, long)]
  pub target: Option<String>,
  /// List of cargo features to activate
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub features: Option<Vec<String>>,
  /// Exit on panic
  #[clap(short, long)]
  exit_on_panic: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Run the code in release mode
  #[clap(long = "release")]
  pub release_mode: bool,
  /// Command line arguments passed to the runner
  pub args: Vec<String>,
}

pub fn command(options: Options) -> Result<()> {
  let r = command_internal(options);
  if r.is_err() {
    kill_before_dev_process();
    #[cfg(not(debug_assertions))]
    let _ = check_for_updates();
  }
  r
}

fn command_internal(mut options: Options) -> Result<()> {
  let tauri_path = tauri_dir();
  options.config = if let Some(config) = &options.config {
    Some(if config.starts_with('{') {
      config.to_string()
    } else {
      std::fs::read_to_string(&config).with_context(|| "failed to read custom configuration")?
    })
  } else {
    None
  };

  set_current_dir(&tauri_path).with_context(|| "failed to change current working directory")?;

  let config = get_config(options.config.as_deref())?;

  if let Some(before_dev) = &config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .before_dev_command
  {
    if !before_dev.is_empty() {
      info!(action = "Running"; "BeforeDevCommand (`{}`)", before_dev);
      #[cfg(target_os = "windows")]
      let mut command = {
        let mut command = Command::new("cmd");
        command
          .arg("/S")
          .arg("/C")
          .arg(before_dev)
          .current_dir(app_dir())
          .envs(command_env(true));
        command
      };
      #[cfg(not(target_os = "windows"))]
      let mut command = {
        let mut command = Command::new("sh");
        command
          .arg("-c")
          .arg(before_dev)
          .current_dir(app_dir())
          .envs(command_env(true));
        command
      };
      command.stdin(Stdio::piped());
      command.stdout(os_pipe::dup_stdout()?);
      command.stderr(os_pipe::dup_stderr()?);

      let child = SharedChild::spawn(&mut command)
        .unwrap_or_else(|_| panic!("failed to run `{}`", before_dev));
      let child = Arc::new(child);
      let child_ = child.clone();

      std::thread::spawn(move || {
        let status = child_
          .wait()
          .expect("failed to wait on \"beforeDevCommand\"");
        if !(status.success() || KILL_BEFORE_DEV_FLAG.get().unwrap().load(Ordering::Relaxed)) {
          error!("The \"beforeDevCommand\" terminated with a non-zero status code.");
          exit(status.code().unwrap_or(1));
        }
      });

      BEFORE_DEV.set(Mutex::new(child)).unwrap();
      KILL_BEFORE_DEV_FLAG.set(AtomicBool::default()).unwrap();

      let _ = ctrlc::set_handler(move || {
        kill_before_dev_process();
        #[cfg(not(debug_assertions))]
        let _ = check_for_updates();
        exit(130);
      });
    }
  }

  if options.runner.is_none() {
    options.runner = config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .runner
      .clone();
  }

  let mut cargo_features = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .features
    .clone()
    .unwrap_or_default();
  if let Some(features) = &options.features {
    cargo_features.extend(features.clone());
  }

  if std::env::var_os("TAURI_SKIP_DEVSERVER_CHECK") != Some("true".into()) {
    if let AppUrl::Url(WindowUrl::External(dev_server_url)) = config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .dev_path
      .clone()
    {
      let host = dev_server_url
        .host()
        .unwrap_or_else(|| panic!("No host name in the URL"));
      let port = dev_server_url
        .port_or_known_default()
        .unwrap_or_else(|| panic!("No port number in the URL"));
      let addrs;
      let addr;
      let addrs = match host {
        url::Host::Domain(domain) => {
          use std::net::ToSocketAddrs;
          addrs = (domain, port).to_socket_addrs()?;
          addrs.as_slice()
        }
        url::Host::Ipv4(ip) => {
          addr = (ip, port).into();
          std::slice::from_ref(&addr)
        }
        url::Host::Ipv6(ip) => {
          addr = (ip, port).into();
          std::slice::from_ref(&addr)
        }
      };
      let mut i = 0;
      let sleep_interval = std::time::Duration::from_secs(2);
      let max_attempts = 90;
      loop {
        if std::net::TcpStream::connect(addrs).is_ok() {
          break;
        }
        if i % 3 == 0 {
          warn!(
            "Waiting for your frontend dev server to start on {}...",
            dev_server_url
          );
        }
        i += 1;
        if i == max_attempts {
          error!(
            "Could not connect to `{}` after {}s. Please make sure that is the URL to your dev server.",
            dev_server_url, i * sleep_interval.as_secs()
          );
          exit(1);
        }
        std::thread::sleep(sleep_interval);
      }
    }
  }

  let mut interface = AppInterface::new(config.lock().unwrap().as_ref().unwrap())?;

  let exit_on_panic = options.exit_on_panic;
  interface.dev(options.into(), move |status, reason| {
    on_dev_exit(status, reason, exit_on_panic)
  })
}

fn on_dev_exit(status: ExitStatus, reason: ExitReason, exit_on_panic: bool) {
  if !matches!(reason, ExitReason::TriggeredKill)
    && (exit_on_panic || matches!(reason, ExitReason::NormalExit))
  {
    kill_before_dev_process();
    #[cfg(not(debug_assertions))]
    let _ = check_for_updates();
    exit(status.code().unwrap_or(0));
  }
}

#[cfg(not(debug_assertions))]
fn check_for_updates() -> Result<()> {
  if std::env::var_os("TAURI_SKIP_UPDATE_CHECK") != Some("true".into()) {
    let current_version = crate::info::cli_current_version()?;
    let current = semver::Version::parse(&current_version)?;

    let upstream_version = crate::info::cli_upstream_version()?;
    let upstream = semver::Version::parse(&upstream_version)?;
    if current < upstream {
      println!(
        "🚀 A new version of Tauri CLI is available! [{}]",
        upstream.to_string()
      );
    };
  }
  Ok(())
}

fn kill_before_dev_process() {
  if let Some(child) = BEFORE_DEV.get() {
    let child = child.lock().unwrap();
    KILL_BEFORE_DEV_FLAG
      .get()
      .unwrap()
      .store(true, Ordering::Relaxed);
    #[cfg(windows)]
      let _ = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(format!("function Kill-Tree {{ Param([int]$ppid); Get-CimInstance Win32_Process | Where-Object {{ $_.ParentProcessId -eq $ppid }} | ForEach-Object {{ Kill-Tree $_.ProcessId }}; Stop-Process -Id $ppid -ErrorAction SilentlyContinue }}; Kill-Tree {}", child.id()))
        .status();
    #[cfg(unix)]
    {
      let mut kill_children_script_path = std::env::temp_dir();
      kill_children_script_path.push("kill-children.sh");

      if !kill_children_script_path.exists() {
        if let Ok(mut file) = std::fs::File::create(&kill_children_script_path) {
          use std::os::unix::fs::PermissionsExt;
          let _ = file.write_all(KILL_CHILDREN_SCRIPT);
          let mut permissions = file.metadata().unwrap().permissions();
          permissions.set_mode(0o770);
          let _ = file.set_permissions(permissions);
        }
      }
      let _ = Command::new(&kill_children_script_path)
        .arg(child.id().to_string())
        .output();
    }
    let _ = child.kill();
  }
}
