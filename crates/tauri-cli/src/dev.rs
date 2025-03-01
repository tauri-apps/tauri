// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{frontend_dir, tauri_dir},
    command_env,
    config::{
      get as get_config, reload as reload_config, BeforeDevCommand, ConfigHandle, FrontendDist,
    },
  },
  interface::{AppInterface, ExitReason, Interface},
  CommandExt, ConfigValue, Result,
};

use anyhow::{bail, Context};
use clap::{ArgAction, Parser};
use shared_child::SharedChild;
use tauri_utils::platform::Target;

use std::{
  env::set_current_dir,
  net::{IpAddr, Ipv4Addr},
  process::{exit, Command, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
  },
};

mod builtin_dev_server;

static BEFORE_DEV: OnceLock<Mutex<Arc<SharedChild>>> = OnceLock::new();
static KILL_BEFORE_DEV_FLAG: OnceLock<AtomicBool> = OnceLock::new();

#[cfg(unix)]
const KILL_CHILDREN_SCRIPT: &[u8] = include_bytes!("../scripts/kill-children.sh");

pub const TAURI_CLI_BUILTIN_WATCHER_IGNORE_FILE: &[u8] =
  include_bytes!("../tauri-dev-watcher.gitignore");

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Run your app in development mode",
  long_about = "Run your app in development mode with hot-reloading for the Rust code. It makes use of the `build.devUrl` property from your `tauri.conf.json` file. It also runs your `build.beforeDevCommand` which usually starts your frontend devServer.",
  trailing_var_arg(true)
)]
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
  pub exit_on_panic: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<ConfigValue>,
  /// Run the code in release mode
  #[clap(long = "release")]
  pub release_mode: bool,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments. Arguments after a second `--` are passed to the application
  /// e.g. `tauri dev -- [runnerArgs] -- [appArgs]`.
  pub args: Vec<String>,
  /// Skip waiting for the frontend dev server to start before building the tauri application.
  #[clap(long, env = "TAURI_CLI_NO_DEV_SERVER_WAIT")]
  pub no_dev_server_wait: bool,
  /// Disable the file watcher.
  #[clap(long)]
  pub no_watch: bool,

  /// Disable the built-in dev server for static files.
  #[clap(long)]
  pub no_dev_server: bool,
  /// Specify port for the built-in dev server for static files. Defaults to 1430.
  #[clap(long, env = "TAURI_CLI_PORT")]
  pub port: Option<u16>,

  #[clap(skip)]
  pub host: Option<IpAddr>,
}

pub fn command(options: Options) -> Result<()> {
  crate::helpers::app_paths::resolve();

  let r = command_internal(options);
  if r.is_err() {
    kill_before_dev_process();
  }
  r
}

fn command_internal(mut options: Options) -> Result<()> {
  let target = options
    .target
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(Target::current);

  let config = get_config(target, options.config.as_ref().map(|c| &c.0))?;

  let mut interface = AppInterface::new(
    config.lock().unwrap().as_ref().unwrap(),
    options.target.clone(),
  )?;

  setup(&interface, &mut options, config)?;

  let exit_on_panic = options.exit_on_panic;
  let no_watch = options.no_watch;
  interface.dev(options.into(), move |status, reason| {
    on_app_exit(status, reason, exit_on_panic, no_watch)
  })
}

pub fn setup(interface: &AppInterface, options: &mut Options, config: ConfigHandle) -> Result<()> {
  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

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
    let cwd = script_cwd.unwrap_or_else(|| frontend_dir().clone());
    if let Some(before_dev) = script {
      log::info!(action = "Running"; "BeforeDevCommand (`{}`)", before_dev);
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
            log::error!("The \"beforeDevCommand\" terminated with a non-zero status code.");
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
    options
      .runner
      .clone_from(&config.lock().unwrap().as_ref().unwrap().build.runner);
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

  let mut dev_url = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .dev_url
    .clone();
  let frontend_dist = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .frontend_dist
    .clone();
  if !options.no_dev_server && dev_url.is_none() {
    if let Some(FrontendDist::Directory(path)) = &frontend_dist {
      if path.exists() {
        let path = path.canonicalize()?;

        let ip = options
          .host
          .unwrap_or_else(|| Ipv4Addr::new(127, 0, 0, 1).into());

        let server_url = builtin_dev_server::start(path, ip, options.port)?;
        let server_url = format!("http://{server_url}");
        dev_url = Some(server_url.parse().unwrap());

        if let Some(c) = &mut options.config {
          if let Some(build) = c
            .0
            .as_object_mut()
            .and_then(|root| root.get_mut("build"))
            .and_then(|build| build.as_object_mut())
          {
            build.insert("devUrl".into(), server_url.into());
          }
        } else {
          options
            .config
            .replace(crate::ConfigValue(serde_json::json!({
              "build": {
                "devUrl": server_url
              }
            })));
        }

        reload_config(options.config.as_ref().map(|c| &c.0))?;
      }
    }
  }

  if !options.no_dev_server_wait {
    if let Some(url) = dev_url {
      let host = url
        .host()
        .unwrap_or_else(|| panic!("No host name in the URL"));
      let port = url
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
          log::warn!("Waiting for your frontend dev server to start on {url}...",);
        }
        i += 1;
        if i == max_attempts {
          log::error!("Could not connect to `{url}` after {}s. Please make sure that is the URL to your dev server.", i * sleep_interval.as_secs());
          exit(1);
        }
        std::thread::sleep(sleep_interval);
      }
    }
  }

  Ok(())
}

pub fn on_app_exit(code: Option<i32>, reason: ExitReason, exit_on_panic: bool, no_watch: bool) {
  if no_watch
    || (!matches!(reason, ExitReason::TriggeredKill)
      && (exit_on_panic || matches!(reason, ExitReason::NormalExit)))
  {
    kill_before_dev_process();
    exit(code.unwrap_or(0));
  }
}

pub fn kill_before_dev_process() {
  if let Some(child) = BEFORE_DEV.get() {
    let child = child.lock().unwrap();
    let kill_before_dev_flag = KILL_BEFORE_DEV_FLAG.get().unwrap();
    if kill_before_dev_flag.load(Ordering::Relaxed) {
      return;
    }
    kill_before_dev_flag.store(true, Ordering::Relaxed);
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
      kill_children_script_path.push("tauri-stop-dev-processes.sh");

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
