// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, reload as reload_config, AppUrl, WindowUrl},
    manifest::{get_workspace_members, rewrite_manifest},
    Logger,
  },
  CommandExt, Result,
};
use clap::Parser;

use anyhow::Context;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;
use shared_child::SharedChild;

use std::{
  env::set_current_dir,
  ffi::OsStr,
  process::{exit, Command},
  sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
  },
  time::Duration,
};

static BEFORE_DEV: OnceCell<Mutex<Arc<SharedChild>>> = OnceCell::new();

#[derive(Debug, Parser)]
#[clap(about = "Tauri dev", trailing_var_arg(true))]
pub struct Options {
  /// Binary to use to run the application
  #[clap(short, long)]
  runner: Option<String>,
  /// Target triple to build against
  #[clap(short, long)]
  target: Option<String>,
  /// List of cargo features to activate
  #[clap(short, long)]
  features: Option<Vec<String>>,
  /// Exit on panic
  #[clap(short, long)]
  exit_on_panic: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  config: Option<String>,
  /// Run the code in release mode
  #[clap(long = "release")]
  release_mode: bool,
  /// Command line arguments passed to the runner
  args: Vec<String>,
}

pub fn command(options: Options) -> Result<()> {
  let logger = Logger::new("tauri:dev");

  let tauri_path = tauri_dir();
  set_current_dir(&tauri_path).with_context(|| "failed to change current working directory")?;
  let merge_config = if let Some(config) = &options.config {
    Some(if config.starts_with('{') {
      config.to_string()
    } else {
      std::fs::read_to_string(&config)?
    })
  } else {
    None
  };
  let config = get_config(merge_config.as_deref())?;

  if let Some(before_dev) = &config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .before_dev_command
  {
    if !before_dev.is_empty() {
      logger.log(format!("Running `{}`", before_dev));
      #[cfg(target_os = "windows")]
      let mut command = {
        let mut command = Command::new("cmd");
        command
          .arg("/S")
          .arg("/C")
          .arg(before_dev)
          .current_dir(app_dir())
          .envs(command_env(true))
          .pipe()?; // development build always includes debug information
        command
      };
      #[cfg(not(target_os = "windows"))]
      let mut command = {
        let mut command = Command::new("sh");
        command
          .arg("-c")
          .arg(before_dev)
          .current_dir(app_dir())
          .envs(command_env(true))
          .pipe()?; // development build always includes debug information
        command
      };

      let child = SharedChild::spawn(&mut command)
        .unwrap_or_else(|_| panic!("failed to run `{}`", before_dev));
      let child = Arc::new(child);
      let child_ = child.clone();
      let logger_ = logger.clone();
      std::thread::spawn(move || {
        let status = child_
          .wait()
          .expect("failed to wait on \"beforeDevCommand\"");
        if !status.success() {
          logger_.error("The \"beforeDevCommand\" terminated with a non-zero status code.");
          exit(status.code().unwrap_or(1));
        }
      });

      BEFORE_DEV.set(Mutex::new(child)).unwrap();
    }
  }

  let runner_from_config = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .runner
    .clone();
  let runner = options
    .runner
    .clone()
    .or(runner_from_config)
    .unwrap_or_else(|| "cargo".to_string());

  {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(tauri_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
    rewrite_manifest(config.clone())?;
    loop {
      if let Ok(DebouncedEvent::NoticeWrite(_)) = rx.recv() {
        break;
      }
    }
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

  let (child_wait_tx, child_wait_rx) = channel();
  let child_wait_rx = Arc::new(Mutex::new(child_wait_rx));

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
          logger.warn(format!(
            "Waiting for your frontend dev server to start on {}...",
            dev_server_url
          ));
        }
        i += 1;
        if i == max_attempts {
          logger.error(format!(
          "Could not connect to `{}` after {}s. Please make sure that is the URL to your dev server.",
          dev_server_url, i * sleep_interval.as_secs()
        ));
          exit(1);
        }
        std::thread::sleep(sleep_interval);
      }
    }
  }

  let mut process = start_app(&options, &runner, &cargo_features, child_wait_rx.clone())?;

  let (tx, rx) = channel();

  let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
  watcher.watch(tauri_path.join("src"), RecursiveMode::Recursive)?;
  watcher.watch(tauri_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
  watcher.watch(tauri_path.join("tauri.conf.json"), RecursiveMode::Recursive)?;

  for member in get_workspace_members()? {
    let workspace_path = tauri_path.join(member);
    watcher.watch(workspace_path.join("src"), RecursiveMode::Recursive)?;
    watcher.watch(workspace_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
  }

  loop {
    if let Ok(event) = rx.recv() {
      let event_path = match event {
        DebouncedEvent::Create(path) => Some(path),
        DebouncedEvent::Remove(path) => Some(path),
        DebouncedEvent::Rename(_, dest) => Some(dest),
        DebouncedEvent::Write(path) => Some(path),
        _ => None,
      };

      if let Some(event_path) = event_path {
        if event_path.file_name() == Some(OsStr::new("tauri.conf.json")) {
          reload_config(merge_config.as_deref())?;
          rewrite_manifest(config.clone())?;
        } else {
          // When tauri.conf.json is changed, rewrite_manifest will be called
          // which will trigger the watcher again
          // So the app should only be started when a file other than tauri.conf.json is changed
          let _ = child_wait_tx.send(());
          process
            .kill()
            .with_context(|| "failed to kill app process")?;
          // wait for the process to exit
          loop {
            if let Ok(Some(_)) = process.try_wait() {
              break;
            }
          }
          process = start_app(&options, &runner, &cargo_features, child_wait_rx.clone())?;
        }
      }
    }
  }
}

fn kill_before_dev_process() {
  if let Some(child) = BEFORE_DEV.get() {
    let child = child.lock().unwrap();
    #[cfg(windows)]
      let _ = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(format!("function Kill-Tree {{ Param([int]$ppid); Get-CimInstance Win32_Process | Where-Object {{ $_.ParentProcessId -eq $ppid }} | ForEach-Object {{ Kill-Tree $_.ProcessId }}; Stop-Process -Id $ppid }}; Kill-Tree {}", child.id()))
        .status();
    #[cfg(not(windows))]
    let _ = Command::new("pkill")
      .args(&["-TERM", "-P"])
      .arg(child.id().to_string())
      .status();
    let _ = child.kill();
  }
}

fn start_app(
  options: &Options,
  runner: &str,
  features: &[String],
  child_wait_rx: Arc<Mutex<Receiver<()>>>,
) -> Result<Arc<SharedChild>> {
  let mut command = Command::new(runner);
  command.args(&["run", "--no-default-features"]);

  if options.release_mode {
    command.args(&["--release"]);
  }

  if let Some(target) = &options.target {
    command.args(&["--target", target]);
  }

  if !features.is_empty() {
    command.args(&["--features", &features.join(",")]);
  }

  if !options.args.is_empty() {
    command.args(&options.args);
  }

  command.pipe().unwrap();

  let child =
    SharedChild::spawn(&mut command).with_context(|| format!("failed to run {}", runner))?;
  let child_arc = Arc::new(child);

  let child_clone = child_arc.clone();
  let exit_on_panic = options.exit_on_panic;
  std::thread::spawn(move || {
    let status = child_clone.wait().expect("failed to wait on child");
    if exit_on_panic {
      // we exit if the status is a success code (app closed) or code is 101 (compilation error)
      // if the process wasn't killed by the file watcher
      if (status.success() || status.code() == Some(101))
          // `child_wait_rx` indicates that the process was killed by the file watcher
          && child_wait_rx
          .lock()
          .expect("failed to get child_wait_rx lock")
          .try_recv()
          .is_err()
      {
        kill_before_dev_process();
        exit(0);
      }
    } else if status.success() {
      // if we're no exiting on panic, we only exit if the status is a success code (app closed)
      kill_before_dev_process();
      exit(0);
    }
  });

  Ok(child_arc)
}
