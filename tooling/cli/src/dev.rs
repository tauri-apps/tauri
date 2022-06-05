// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, reload as reload_config, AppUrl, ConfigHandle, WindowUrl},
    manifest::{rewrite_manifest, Manifest},
  },
  CommandExt, Result,
};
use clap::Parser;

use anyhow::Context;
use log::{error, info, warn};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;
use shared_child::SharedChild;

use std::{
  env::set_current_dir,
  ffi::OsStr,
  fs::FileType,
  io::{BufReader, Write},
  path::{Path, PathBuf},
  process::{exit, Command, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::channel,
    Arc, Mutex,
  },
  time::Duration,
};

static BEFORE_DEV: OnceCell<Mutex<Arc<SharedChild>>> = OnceCell::new();
static KILL_BEFORE_DEV_FLAG: OnceCell<AtomicBool> = OnceCell::new();

#[cfg(unix)]
const KILL_CHILDREN_SCRIPT: &[u8] = include_bytes!("../scripts/kill-children.sh");

const TAURI_DEV_WATCHER_GITIGNORE: &[u8] = include_bytes!("../tauri-dev-watcher.gitignore");

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
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
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
  let r = command_internal(options);
  if r.is_err() {
    kill_before_dev_process();
    #[cfg(not(debug_assertions))]
    let _ = check_for_updates();
  }
  r
}

fn command_internal(options: Options) -> Result<()> {
  let tauri_path = tauri_dir();
  let merge_config = if let Some(config) = &options.config {
    Some(if config.starts_with('{') {
      config.to_string()
    } else {
      std::fs::read_to_string(&config).with_context(|| "failed to read custom configuration")?
    })
  } else {
    None
  };

  set_current_dir(&tauri_path).with_context(|| "failed to change current working directory")?;

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
      info!(action = "Running"; "BeforeDevCommand (`{}`)", before_dev);
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
      command.stdin(Stdio::piped());

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

  let manifest = {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(tauri_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
    let manifest = rewrite_manifest(config.clone())?;
    loop {
      if let Ok(DebouncedEvent::NoticeWrite(_)) = rx.recv() {
        break;
      }
    }
    manifest
  };

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

  let manually_killed_app = Arc::new(AtomicBool::default());

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

  let process = start_app(
    &options,
    &runner,
    &manifest,
    &cargo_features,
    manually_killed_app.clone(),
  )?;
  let shared_process = Arc::new(Mutex::new(process));
  if let Err(e) = watch(
    shared_process.clone(),
    manually_killed_app,
    tauri_path,
    merge_config,
    config,
    options,
    runner,
    manifest,
    cargo_features,
  ) {
    shared_process
      .lock()
      .unwrap()
      .kill()
      .with_context(|| "failed to kill app process")?;
    Err(e)
  } else {
    Ok(())
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

fn lookup<F: FnMut(FileType, PathBuf)>(dir: &Path, mut f: F) {
  let mut default_gitignore = std::env::temp_dir();
  default_gitignore.push(".tauri-dev");
  let _ = std::fs::create_dir_all(&default_gitignore);
  default_gitignore.push(".gitignore");
  if !default_gitignore.exists() {
    if let Ok(mut file) = std::fs::File::create(default_gitignore.clone()) {
      let _ = file.write_all(TAURI_DEV_WATCHER_GITIGNORE);
    }
  }

  let mut builder = ignore::WalkBuilder::new(dir);
  let _ = builder.add_ignore(default_gitignore);
  builder.require_git(false).ignore(false).max_depth(Some(1));

  for entry in builder.build().flatten() {
    f(entry.file_type().unwrap(), dir.join(entry.path()));
  }
}

#[allow(clippy::too_many_arguments)]
fn watch(
  process: Arc<Mutex<Arc<SharedChild>>>,
  manually_killed_app: Arc<AtomicBool>,
  tauri_path: PathBuf,
  merge_config: Option<String>,
  config: ConfigHandle,
  options: Options,
  runner: String,
  mut manifest: Manifest,
  cargo_features: Vec<String>,
) -> Result<()> {
  let (tx, rx) = channel();

  let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
  lookup(&tauri_path, |file_type, path| {
    if path != tauri_path {
      let _ = watcher.watch(
        path,
        if file_type.is_dir() {
          RecursiveMode::Recursive
        } else {
          RecursiveMode::NonRecursive
        },
      );
    }
  });

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
          manifest = rewrite_manifest(config.clone())?;
        } else {
          // When tauri.conf.json is changed, rewrite_manifest will be called
          // which will trigger the watcher again
          // So the app should only be started when a file other than tauri.conf.json is changed
          manually_killed_app.store(true, Ordering::Relaxed);
          let mut p = process.lock().unwrap();
          p.kill().with_context(|| "failed to kill app process")?;
          // wait for the process to exit
          loop {
            if let Ok(Some(_)) = p.try_wait() {
              break;
            }
          }
          *p = start_app(
            &options,
            &runner,
            &manifest,
            &cargo_features,
            manually_killed_app.clone(),
          )?;
        }
      }
    }
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

fn start_app(
  options: &Options,
  runner: &str,
  manifest: &Manifest,
  features: &[String],
  manually_killed_app: Arc<AtomicBool>,
) -> Result<Arc<SharedChild>> {
  let mut command = Command::new(runner);
  command
    .env(
      "CARGO_TERM_PROGRESS_WIDTH",
      terminal::stderr_width()
        .map(|width| {
          if cfg!(windows) {
            std::cmp::min(60, width)
          } else {
            width
          }
        })
        .unwrap_or(if cfg!(windows) { 60 } else { 80 })
        .to_string(),
    )
    .env("CARGO_TERM_PROGRESS_WHEN", "always");
  command.arg("run").arg("--color").arg("always");

  if !options.args.contains(&"--no-default-features".into()) {
    let manifest_features = manifest.features();
    let enable_features: Vec<String> = manifest_features
      .get("default")
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter(|feature| {
        if let Some(manifest_feature) = manifest_features.get(feature) {
          !manifest_feature.contains(&"tauri/custom-protocol".into())
        } else {
          feature != "tauri/custom-protocol"
        }
      })
      .collect();
    command.arg("--no-default-features");
    if !enable_features.is_empty() {
      command.args(&["--features", &enable_features.join(",")]);
    }
  }

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

  command.stdout(os_pipe::dup_stdout().unwrap());
  command.stderr(Stdio::piped());

  let child =
    SharedChild::spawn(&mut command).with_context(|| format!("failed to run {}", runner))?;
  let child_arc = Arc::new(child);
  let child_stderr = child_arc.take_stderr().unwrap();
  let mut stderr = BufReader::new(child_stderr);
  let stderr_lines = Arc::new(Mutex::new(Vec::new()));
  let stderr_lines_ = stderr_lines.clone();
  std::thread::spawn(move || {
    let mut buf = Vec::new();
    let mut lines = stderr_lines_.lock().unwrap();
    let mut io_stderr = std::io::stderr();
    loop {
      buf.clear();
      match tauri_utils::io::read_line(&mut stderr, &mut buf) {
        Ok(s) if s == 0 => break,
        _ => (),
      }
      let _ = io_stderr.write_all(&buf);
      if !buf.ends_with(&[b'\r']) {
        let _ = io_stderr.write_all(b"\n");
      }
      lines.push(String::from_utf8_lossy(&buf).into_owned());
    }
  });

  let child_clone = child_arc.clone();
  let exit_on_panic = options.exit_on_panic;
  std::thread::spawn(move || {
    let status = child_clone.wait().expect("failed to wait on child");

    if exit_on_panic {
      if !manually_killed_app.load(Ordering::Relaxed) {
        kill_before_dev_process();
        #[cfg(not(debug_assertions))]
        let _ = check_for_updates();
        exit(status.code().unwrap_or(0));
      }
    } else {
      let is_cargo_compile_error = stderr_lines
        .lock()
        .unwrap()
        .last()
        .map(|l| l.contains("could not compile"))
        .unwrap_or_default();
      stderr_lines.lock().unwrap().clear();

      // if we're no exiting on panic, we only exit if:
      // - the status is a success code (app closed)
      // - status code is the Cargo error code
      //    - and error is not a cargo compilation error (using stderr heuristics)
      if status.success() || (status.code() == Some(101) && !is_cargo_compile_error) {
        kill_before_dev_process();
        #[cfg(not(debug_assertions))]
        let _ = check_for_updates();
        exit(status.code().unwrap_or(1));
      }
    }
  });

  Ok(child_arc)
}

// taken from https://github.com/rust-lang/cargo/blob/78b10d4e611ab0721fc3aeaf0edd5dd8f4fdc372/src/cargo/core/shell.rs#L514
#[cfg(unix)]
mod terminal {
  use std::mem;

  pub fn stderr_width() -> Option<usize> {
    unsafe {
      let mut winsize: libc::winsize = mem::zeroed();
      // The .into() here is needed for FreeBSD which defines TIOCGWINSZ
      // as c_uint but ioctl wants c_ulong.
      #[allow(clippy::useless_conversion)]
      if libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ.into(), &mut winsize) < 0 {
        return None;
      }
      if winsize.ws_col > 0 {
        Some(winsize.ws_col as usize)
      } else {
        None
      }
    }
  }
}

// taken from https://github.com/rust-lang/cargo/blob/78b10d4e611ab0721fc3aeaf0edd5dd8f4fdc372/src/cargo/core/shell.rs#L543
#[cfg(windows)]
mod terminal {
  use std::{cmp, mem, ptr};
  use winapi::um::fileapi::*;
  use winapi::um::handleapi::*;
  use winapi::um::processenv::*;
  use winapi::um::winbase::*;
  use winapi::um::wincon::*;
  use winapi::um::winnt::*;

  pub fn stderr_width() -> Option<usize> {
    unsafe {
      let stdout = GetStdHandle(STD_ERROR_HANDLE);
      let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
      if GetConsoleScreenBufferInfo(stdout, &mut csbi) != 0 {
        return Some((csbi.srWindow.Right - csbi.srWindow.Left) as usize);
      }

      // On mintty/msys/cygwin based terminals, the above fails with
      // INVALID_HANDLE_VALUE. Use an alternate method which works
      // in that case as well.
      let h = CreateFileA(
        "CONOUT$\0".as_ptr() as *const CHAR,
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        ptr::null_mut(),
        OPEN_EXISTING,
        0,
        ptr::null_mut(),
      );
      if h == INVALID_HANDLE_VALUE {
        return None;
      }

      let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
      let rc = GetConsoleScreenBufferInfo(h, &mut csbi);
      CloseHandle(h);
      if rc != 0 {
        let width = (csbi.srWindow.Right - csbi.srWindow.Left) as usize;
        // Unfortunately cygwin/mintty does not set the size of the
        // backing console to match the actual window size. This
        // always reports a size of 80 or 120 (not sure what
        // determines that). Use a conservative max of 60 which should
        // work in most circumstances. ConEmu does some magic to
        // resize the console correctly, but there's no reasonable way
        // to detect which kind of terminal we are running in, or if
        // GetConsoleScreenBufferInfo returns accurate information.
        return Some(cmp::min(60, width));
      }

      None
    }
  }
}
