// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, reload as reload_config, AppUrl, BeforeDevCommand, WindowUrl},
  },
  interface::{rust::DevChild, AppInterface, ExitReason, Interface},
  CommandExt, Result,
};
use clap::{ArgAction, Parser};

use anyhow::{bail, Context};
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use shared_child::SharedChild;

use std::{
  env::set_current_dir,
  process::{exit, Command, ExitStatus, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

pub(crate) static DEV_CHILD: OnceCell<Arc<Mutex<DevChild>>> = OnceCell::new();
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
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
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
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments. Arguments after a second `--` are passed to the application
  /// e.g. `tauri dev -- [runnerArgs] -- [appArgs]`.
  pub args: Vec<String>,
  /// Disable the file watcher
  #[clap(long)]
  pub no_watch: bool,
  /// Disable the dev server for static files.
  #[clap(long)]
  pub no_dev_server: bool,
  /// Specify port for the dev server for static files. Defaults to 1430
  /// Can also be set using `TAURI_DEV_SERVER_PORT` env var.
  #[clap(long)]
  pub port: Option<u16>,
}

pub fn command(options: Options) -> Result<()> {
  let r = command_internal(options);
  if r.is_err() {
    kill_before_dev_process();
  }
  r
}

fn command_internal(mut options: Options) -> Result<()> {
  let tauri_path = tauri_dir();
  options.config = if let Some(config) = &options.config {
    Some(if config.starts_with('{') {
      config.to_string()
    } else {
      std::fs::read_to_string(config).with_context(|| "failed to read custom configuration")?
    })
  } else {
    None
  };

  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

  let config = get_config(options.config.as_deref())?;

  let mut interface = AppInterface::new(
    config.lock().unwrap().as_ref().unwrap(),
    options.target.clone(),
  )?;

  if let Some(before_dev) = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .before_dev_command
    .clone()
  {
    let (script, script_cwd, wait) = match before_dev {
      BeforeDevCommand::Script(s) if s.is_empty() => (None, None, false),
      BeforeDevCommand::Script(s) => (Some(s), None, false),
      BeforeDevCommand::ScriptWithOptions { script, cwd, wait } => {
        (Some(script), cwd.map(Into::into), wait)
      }
    };
    let cwd = script_cwd.unwrap_or_else(|| app_dir().clone());
    if let Some(before_dev) = script {
      info!(action = "Running"; "BeforeDevCommand (`{}`)", before_dev);
      let mut env = command_env(true);
      env.extend(interface.env());

      #[cfg(windows)]
      let mut command = {
        let mut command = Command::new("cmd");
        command
          .arg("/S")
          .arg("/C")
          .arg(&before_dev)
          .current_dir(cwd)
          .envs(env);
        command
      };
      #[cfg(not(windows))]
      let mut command = {
        let mut command = Command::new("sh");
        command
          .arg("-c")
          .arg(&before_dev)
          .current_dir(cwd)
          .envs(env);
        command
      };

      if wait {
        let status = command.piped().with_context(|| {
          format!(
            "failed to run `{}` with `{}`",
            before_dev,
            if cfg!(windows) { "cmd /S /C" } else { "sh -c" }
          )
        })?;
        if !status.success() {
          bail!(
            "beforeDevCommand `{}` failed with exit code {}",
            before_dev,
            status.code().unwrap_or_default()
          );
        }
      } else {
        command.stdin(Stdio::piped());
        command.stdout(os_pipe::dup_stdout()?);
        command.stderr(os_pipe::dup_stderr()?);

        let child = SharedChild::spawn(&mut command)
          .unwrap_or_else(|_| panic!("failed to run `{before_dev}`"));
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
          exit(130);
        });
      }
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

  let mut dev_path = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .dev_path
    .clone();
  if !options.no_dev_server {
    if let AppUrl::Url(WindowUrl::App(path)) = &dev_path {
      use crate::helpers::web_dev_server::start_dev_server;
      if path.exists() {
        let path = path.canonicalize()?;
        let server_url = start_dev_server(path, options.port)?;
        let server_url = format!("http://{server_url}");
        dev_path = AppUrl::Url(WindowUrl::External(server_url.parse().unwrap()));

        // TODO: in v2, use an env var to pass the url to the app context
        // or better separate the config passed from the cli internally and
        // config passed by the user in `--config` into to separate env vars
        // and the context merges, the user first, then the internal cli config
        if let Some(c) = options.config {
          let mut c: tauri_utils::config::Config = serde_json::from_str(&c)?;
          c.build.dev_path = dev_path.clone();
          options.config = Some(serde_json::to_string(&c).unwrap());
        } else {
          options.config = Some(format!(r#"{{ "build": {{ "devPath": "{server_url}" }} }}"#))
        }
      }
    }

    reload_config(options.config.as_deref())?;
  }

  if std::env::var_os("TAURI_SKIP_DEVSERVER_CHECK") != Some("true".into()) {
    if let AppUrl::Url(WindowUrl::External(dev_server_url)) = dev_path {
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
      let timeout_duration = std::time::Duration::from_secs(1);
      let max_attempts = 90;
      'waiting: loop {
        for addr in addrs.iter() {
          if std::net::TcpStream::connect_timeout(addr, timeout_duration).is_ok() {
            break 'waiting;
          }
        }

        if i % 3 == 1 {
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

  let exit_on_panic = options.exit_on_panic;
  let no_watch = options.no_watch;
  interface.dev(options.into(), move |status, reason| {
    on_dev_exit(status, reason, exit_on_panic, no_watch)
  })
}

fn on_dev_exit(status: ExitStatus, reason: ExitReason, exit_on_panic: bool, no_watch: bool) {
  if no_watch
    || (!matches!(reason, ExitReason::TriggeredKill)
      && (exit_on_panic || matches!(reason, ExitReason::NormalExit)))
  {
    kill_before_dev_process();
    exit(status.code().unwrap_or(0));
  }
}

fn kill_before_dev_process() {
  if let Some(child) = BEFORE_DEV.get() {
    let child = child.lock().unwrap();
    KILL_BEFORE_DEV_FLAG
      .get()
      .unwrap()
      .store(true, Ordering::Relaxed);
    #[cfg(windows)]
    {
      let powershell_path = std::env::var("SYSTEMROOT").map_or_else(
        |_| "powershell.exe".to_string(),
        |p| format!("{p}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
      );
      let _ = Command::new(powershell_path)
      .arg("-NoProfile")
      .arg("-Command")
      .arg(format!("function Kill-Tree {{ Param([int]$ppid); Get-CimInstance Win32_Process | Where-Object {{ $_.ParentProcessId -eq $ppid }} | ForEach-Object {{ Kill-Tree $_.ProcessId }}; Stop-Process -Id $ppid -ErrorAction SilentlyContinue }}; Kill-Tree {}", child.id()))
      .status();
    }
    #[cfg(unix)]
    {
      use std::io::Write;
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
