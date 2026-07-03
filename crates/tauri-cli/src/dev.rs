// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::{Context, ErrorExt},
  helpers::{
    app_paths::Dirs,
    command_env,
    config::{get_config, reload_config, BeforeDevCommand, ConfigMetadata, FrontendDist},
  },
  info::plugins::check_mismatched_packages,
  interface::{AppInterface, ExitReason},
  CommandExt, ConfigValue, Error, Result,
};

use clap::{ArgAction, Parser};
use tauri_utils::{config::RunnerConfig, platform::Target};
use tokio::sync::Notify;

use std::{
  env::set_current_dir,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  path::PathBuf,
  process::{exit, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
  },
};

mod builtin_dev_server;

/// Handle to the spawned `beforeDevCommand` process.
///
/// The process itself is owned by a monitor task; it is killed when
/// [`kill_before_dev_process`] is called and, as a last resort, by
/// tokio's kill-on-drop when the runtime shuts down.
struct BeforeDevChild {
  pid: u32,
  kill_signal: Arc<Notify>,
}

static BEFORE_DEV: OnceLock<BeforeDevChild> = OnceLock::new();
static KILL_BEFORE_DEV_FLAG: AtomicBool = AtomicBool::new(false);

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
  pub runner: Option<RunnerConfig>,
  /// Target triple to build against
  #[clap(short, long)]
  pub target: Option<String>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub features: Vec<String>,
  /// Exit on panic
  #[clap(short, long)]
  pub exit_on_panic: bool,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json and tauri.ios.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
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
  /// Additional paths to watch for changes.
  #[clap(long)]
  pub additional_watch_folders: Vec<PathBuf>,

  /// Disable the built-in dev server for static files.
  #[clap(long)]
  pub no_dev_server: bool,
  /// Specify port for the built-in dev server for static files. Defaults to 1430.
  #[clap(long, env = "TAURI_CLI_PORT")]
  pub port: Option<u16>,

  #[clap(skip)]
  pub host: Option<IpAddr>,
}

pub async fn command(options: Options) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();

  let r = command_internal(options, dirs).await;
  if r.is_err() {
    kill_before_dev_process();
  }
  r
}

async fn command_internal(mut options: Options, dirs: Dirs) -> Result<()> {
  let target = options
    .target
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(Target::current);

  let mut config = get_config(
    target,
    &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
    dirs.tauri,
  )?;

  let mut interface = AppInterface::new(&config, options.target.clone(), dirs.tauri).await?;

  setup(&interface, &mut options, &mut config, &dirs).await?;

  let exit_on_panic = options.exit_on_panic;
  let no_watch = options.no_watch;
  interface
    .dev(
      &mut config,
      options.into(),
      move |status, reason| on_app_exit(status, reason, exit_on_panic, no_watch),
      &dirs,
    )
    .await
}

pub async fn setup(
  interface: &AppInterface,
  options: &mut Options,
  config: &mut ConfigMetadata,
  dirs: &Dirs,
) -> Result<()> {
  let (frontend_dir, tauri_dir) = (dirs.frontend, dirs.tauri);
  tokio::spawn(async move {
    if let Err(error) = check_mismatched_packages(frontend_dir, tauri_dir).await {
      log::error!("{error}");
    }
  });

  set_current_dir(dirs.tauri).context("failed to set current directory")?;

  if let Some(before_dev) = config.build.before_dev_command.clone() {
    let (script, script_cwd, wait) = match before_dev {
      BeforeDevCommand::Script(s) if s.is_empty() => (None, None, false),
      BeforeDevCommand::Script(s) => (Some(s), None, false),
      BeforeDevCommand::ScriptWithOptions { script, cwd, wait } => {
        (Some(script), cwd.map(Into::into), wait)
      }
    };
    let cwd = script_cwd.unwrap_or_else(|| dirs.frontend.to_owned());
    if let Some(before_dev) = script {
      log::info!(action = "Running"; "BeforeDevCommand (`{}`)", before_dev);
      let mut env = command_env(true);
      env.extend(interface.env());

      #[cfg(windows)]
      let mut command = {
        let mut command = tokio::process::Command::new("cmd");
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
        let mut command = tokio::process::Command::new("sh");
        command
          .arg("-c")
          .arg(&before_dev)
          .current_dir(cwd)
          .envs(env);
        command
      };

      if wait {
        let status = command
          .piped()
          .await
          .map_err(|error| Error::CommandFailed {
            command: format!(
              "`{before_dev}` with `{}`",
              if cfg!(windows) { "cmd /S /C" } else { "sh -c" }
            ),
            error,
          })?;
        if !status.success() {
          crate::error::bail!(
            "beforeDevCommand `{}` failed with exit code {}",
            before_dev,
            status.code().unwrap_or_default()
          );
        }
      } else {
        command.stdin(Stdio::piped());
        command.stdout(os_pipe::dup_stdout().unwrap());
        command.stderr(os_pipe::dup_stderr().unwrap());
        // ensure the process dies with the CLI even if we don't get
        // a chance to kill it explicitly
        command.kill_on_drop(true);

        let mut child = command
          .spawn()
          .unwrap_or_else(|_| panic!("failed to run `{before_dev}`"));
        let pid = child.id().expect("beforeDevCommand process already exited");

        let kill_signal = Arc::new(Notify::new());
        let kill_signal_ = kill_signal.clone();
        tokio::spawn(async move {
          let status = loop {
            tokio::select! {
              status = child.wait() => break status,
              _ = kill_signal_.notified() => {
                let _ = child.start_kill();
              }
            }
          };
          let status = status.expect("failed to wait on \"beforeDevCommand\"");
          if !(status.success() || KILL_BEFORE_DEV_FLAG.load(Ordering::SeqCst)) {
            log::error!("The \"beforeDevCommand\" terminated with a non-zero status code.");
            exit(status.code().unwrap_or(1));
          }
        });

        let _ = BEFORE_DEV.set(BeforeDevChild { pid, kill_signal });

        tokio::spawn(async {
          if tokio::signal::ctrl_c().await.is_ok() {
            kill_before_dev_process();
            exit(130);
          }
        });
      }
    }
  }

  if options.runner.is_none() {
    options.runner = config.build.runner.clone();
  }

  let mut cargo_features = config.build.features.clone().unwrap_or_default();
  cargo_features.extend(options.features.clone());

  let mut dev_url = config.build.dev_url.clone();
  let frontend_dist = config.build.frontend_dist.clone();
  if !options.no_dev_server && dev_url.is_none() {
    if let Some(FrontendDist::Directory(path)) = &frontend_dist {
      if path.exists() {
        let path = tokio::fs::canonicalize(path)
          .await
          .fs_context("failed to canonicalize path", path.to_path_buf())?;

        let ip = options.host.unwrap_or_else(|| Ipv4Addr::LOCALHOST.into());

        let server_url = builtin_dev_server::start(path, ip, options.port)
          .await
          .context("failed to start builtin dev server")?;
        let server_url = format!("http://{server_url}");
        dev_url = Some(server_url.parse().unwrap());

        options.config.push(crate::ConfigValue(serde_json::json!({
          "build": {
            "devUrl": server_url
          }
        })));

        reload_config(
          config,
          &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
          dirs.tauri,
        )?;
      }
    }
  }

  if !options.no_dev_server_wait {
    if let Some(url) = dev_url {
      let host = url.host().expect("No host name in the URL");
      let port = url
        .port_or_known_default()
        .expect("No port number in the URL");
      let addrs: Vec<SocketAddr> = match host {
        url::Host::Domain(domain) => tokio::net::lookup_host((domain, port))
          .await
          .unwrap()
          .collect(),
        url::Host::Ipv4(ip) => vec![(ip, port).into()],
        url::Host::Ipv6(ip) => vec![(ip, port).into()],
      };
      let mut i = 0;
      let sleep_interval = std::time::Duration::from_secs(2);
      let timeout_duration = std::time::Duration::from_secs(1);
      let max_attempts = 90;
      'waiting: loop {
        for addr in addrs.iter() {
          if matches!(
            tokio::time::timeout(timeout_duration, tokio::net::TcpStream::connect(addr)).await,
            Ok(Ok(_))
          ) {
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
        tokio::time::sleep(sleep_interval).await;
      }
    }
  }

  if options.additional_watch_folders.is_empty() {
    options
      .additional_watch_folders
      .extend(config.build.additional_watch_folders.clone());
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

// this needs to be callable from synchronous contexts right before the
// process exits (signal handlers, app exit callbacks), so it kills the
// process tree by pid using std APIs instead of the tokio child handle.
pub fn kill_before_dev_process() {
  if let Some(child) = BEFORE_DEV.get() {
    if KILL_BEFORE_DEV_FLAG.load(Ordering::SeqCst) {
      return;
    }
    KILL_BEFORE_DEV_FLAG.store(true, Ordering::SeqCst);
    #[cfg(windows)]
    {
      let powershell_path = std::env::var("SYSTEMROOT").map_or_else(
        |_| "powershell.exe".to_string(),
        |p| format!("{p}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
      );
      let _ = std::process::Command::new(powershell_path)
      .arg("-NoProfile")
      .arg("-Command")
      .arg(format!("function Kill-Tree {{ Param([int]$ppid); Get-CimInstance Win32_Process | Where-Object {{ $_.ParentProcessId -eq $ppid }} | ForEach-Object {{ Kill-Tree $_.ProcessId }}; Stop-Process -Id $ppid -ErrorAction SilentlyContinue }}; Kill-Tree {}", child.pid))
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
      let _ = std::process::Command::new(&kill_children_script_path)
        .arg(child.pid.to_string())
        .output();

      let _ = unsafe { libc::kill(child.pid as libc::pid_t, libc::SIGKILL) };
    }
    // let the monitor task reap the child handle
    child.kill_signal.notify_one();
  }
}
