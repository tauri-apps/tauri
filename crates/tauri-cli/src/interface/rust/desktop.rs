// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{AppSettings, DevProcess, ExitReason, Options, RustAppSettings, RustupTarget};
use crate::{
  error::{Context, ErrorExt},
  CommandExt, Error,
};

use std::{
  io::ErrorKind,
  path::{Path, PathBuf},
  process::{ExitStatus, Stdio},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};
use tauri_utils::platform::Target as TargetPlatform;
use tokio::{
  io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader},
  process::Command,
  sync::{watch, Notify},
};

pub struct DevChild {
  manually_killed_app: Arc<AtomicBool>,
  kill_signal: Arc<Notify>,
  exit_status: watch::Receiver<Option<ExitStatus>>,
}

impl Drop for DevChild {
  fn drop(&mut self) {
    // signal the task owning the child process to kill it.
    // the OS process would also be killed by tokio's kill_on_drop
    // once the owning task is dropped with the runtime.
    self.kill_signal.notify_one();
  }
}

#[async_trait::async_trait]
impl DevProcess for DevChild {
  async fn kill(&self) -> std::io::Result<()> {
    // set the flag before killing so the exit watcher reports TriggeredKill
    self.manually_killed_app.store(true, Ordering::SeqCst);
    self.kill_signal.notify_one();
    let mut exit_status = self.exit_status.clone();
    exit_status
      .wait_for(Option::is_some)
      .await
      .map_err(|_| std::io::Error::other("app process watcher task is gone"))?;
    Ok(())
  }

  async fn wait(&self) -> std::io::Result<ExitStatus> {
    let mut exit_status = self.exit_status.clone();
    let status = *exit_status
      .wait_for(Option::is_some)
      .await
      .map_err(|_| std::io::Error::other("app process watcher task is gone"))?;
    Ok(status.expect("checked Some above"))
  }

  fn manually_killed_process(&self) -> bool {
    self.manually_killed_app.load(Ordering::SeqCst)
  }
}

/// Read all bytes until a newline (the `0xA` byte) or a carriage return (`\r`) is reached,
/// and append them to the provided buffer.
///
/// Async port of `tauri_utils::io::read_line`, needed to forward cargo's progress bars.
async fn read_line<R: AsyncBufRead + Unpin + ?Sized>(
  r: &mut R,
  buf: &mut Vec<u8>,
) -> std::io::Result<usize> {
  let mut read = 0;
  loop {
    let (done, used) = {
      let available = match r.fill_buf().await {
        Ok(n) => n,
        Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
        Err(e) => return Err(e),
      };
      match memchr::memchr(b'\n', available) {
        Some(i) => {
          let end = i + 1;
          buf.extend_from_slice(&available[..end]);
          (true, end)
        }
        None => match memchr::memchr(b'\r', available) {
          Some(i) => {
            let end = i + 1;
            buf.extend_from_slice(&available[..end]);
            (true, end)
          }
          None => {
            buf.extend_from_slice(available);
            (false, available.len())
          }
        },
      }
    };
    r.consume(used);
    read += used;
    if done || used == 0 {
      return Ok(read);
    }
  }
}

pub fn run_dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
  options: Options,
  run_args: &[String],
  available_targets: &Option<Vec<RustupTarget>>,
  config_features: Vec<String>,
  on_exit: F,
) -> crate::Result<DevChild> {
  let mut dev_cmd = cargo_command(true, options, available_targets, config_features)?;
  let runner = dev_cmd
    .as_std()
    .get_program()
    .to_string_lossy()
    .into_owned();

  dev_cmd
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
  dev_cmd.arg("--color");
  dev_cmd.arg("always");

  dev_cmd.stdout(os_pipe::dup_stdout().unwrap());
  dev_cmd.stderr(Stdio::piped());

  dev_cmd.arg("--");
  dev_cmd.args(run_args);

  // ensure the app process dies with the CLI even if we don't get
  // a chance to kill it explicitly
  dev_cmd.kill_on_drop(true);

  let manually_killed_app = Arc::new(AtomicBool::default());
  let manually_killed_app_ = manually_killed_app.clone();

  log::info!(action = "Running"; "DevCommand (`{} {}`)", &dev_cmd.as_std().get_program().to_string_lossy(), dev_cmd.as_std().get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{acc} {arg}")));

  let mut dev_child = match dev_cmd.spawn() {
    Ok(c) => Ok(c),
    Err(e) if e.kind() == ErrorKind::NotFound => crate::error::bail!(
      "`{runner}` command not found.{}",
      if runner == "cargo" {
        " Please follow the Tauri setup guide: https://v2.tauri.app/start/prerequisites/"
      } else {
        ""
      }
    ),
    Err(e) => Err(Error::CommandFailed {
      command: runner,
      error: e,
    }),
  }?;

  let dev_child_stderr = dev_child.stderr.take().unwrap();
  let mut stderr = BufReader::new(dev_child_stderr);
  let stderr_lines = Arc::new(Mutex::new(Vec::new()));
  let stderr_lines_ = stderr_lines.clone();
  tokio::spawn(async move {
    let mut buf = Vec::new();
    let mut io_stderr = tokio::io::stderr();
    loop {
      buf.clear();
      match read_line(&mut stderr, &mut buf).await {
        Ok(0) | Err(_) => break,
        Ok(_) => {
          let _ = io_stderr.write_all(&buf).await;
          let _ = io_stderr.flush().await;
          stderr_lines_
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(&buf).into_owned());
        }
      }
    }
  });

  let kill_signal = Arc::new(Notify::new());
  let kill_signal_ = kill_signal.clone();
  let (exit_status_tx, exit_status_rx) = watch::channel(None);
  tokio::spawn(async move {
    let status = loop {
      tokio::select! {
        status = dev_child.wait() => break status,
        _ = kill_signal_.notified() => {
          let _ = dev_child.start_kill();
        }
      }
    };
    let status = status.expect("failed to wait on app process");
    let _ = exit_status_tx.send(Some(status));

    if status.success() {
      on_exit(status.code(), ExitReason::NormalExit);
    } else {
      let is_cargo_compile_error = stderr_lines
        .lock()
        .unwrap()
        .last()
        .map(|l| l.contains("could not compile"))
        .unwrap_or_default();
      stderr_lines.lock().unwrap().clear();

      on_exit(
        status.code(),
        if status.code() == Some(101) && is_cargo_compile_error {
          ExitReason::CompilationFailed
        } else if manually_killed_app_.load(Ordering::SeqCst) {
          ExitReason::TriggeredKill
        } else {
          ExitReason::NormalExit
        },
      );
    }
  });

  Ok(DevChild {
    manually_killed_app,
    kill_signal,
    exit_status: exit_status_rx,
  })
}

pub async fn build(
  options: Options,
  app_settings: &RustAppSettings,
  available_targets: &mut Option<Vec<RustupTarget>>,
  config_features: Vec<String>,
  main_binary_name: Option<&str>,
  tauri_dir: &Path,
) -> crate::Result<PathBuf> {
  let out_dir = app_settings.out_dir(&options, tauri_dir)?;
  let bin_path = app_settings.app_binary_path(&options, tauri_dir)?;

  if std::env::var_os("STATIC_VCRUNTIME").is_none_or(|v| v != "false") {
    std::env::set_var("STATIC_VCRUNTIME", "true");
  }

  if options.target.is_some() && available_targets.is_none() {
    *available_targets = fetch_available_targets().await;
  }

  if options.target == Some("universal-apple-darwin".into()) {
    tokio::fs::create_dir_all(&out_dir)
      .await
      .fs_context("failed to create project out directory", out_dir.clone())?;

    let bin_name = bin_path.file_stem().unwrap();

    let mut lipo_cmd = Command::new("lipo");
    lipo_cmd
      .arg("-create")
      .arg("-output")
      .arg(out_dir.join(bin_name));
    for triple in ["aarch64-apple-darwin", "x86_64-apple-darwin"] {
      let mut options = options.clone();
      options.target.replace(triple.into());

      let triple_out_dir = app_settings
        .out_dir(&options, tauri_dir)
        .with_context(|| format!("failed to get {triple} out dir"))?;

      build_production_app(options, available_targets, config_features.clone())
        .await
        .with_context(|| format!("failed to build {triple} binary"))?;

      lipo_cmd.arg(triple_out_dir.join(bin_name));
    }

    let lipo_status = lipo_cmd.output_ok().await?.status;
    if !lipo_status.success() {
      crate::error::bail!(
        "Result of `lipo` command was unsuccessful: {lipo_status}. (Is `lipo` installed?)"
      );
    }
  } else {
    build_production_app(options, available_targets, config_features)
      .await
      .with_context(|| "failed to build app")?;
  }

  rename_app(bin_path, main_binary_name, &app_settings.target_platform).await
}

async fn build_production_app(
  options: Options,
  available_targets: &Option<Vec<RustupTarget>>,
  config_features: Vec<String>,
) -> crate::Result<()> {
  let mut build_cmd = cargo_command(false, options, available_targets, config_features)?;
  let runner = build_cmd
    .as_std()
    .get_program()
    .to_string_lossy()
    .into_owned();
  match build_cmd.piped().await {
    Ok(status) if status.success() => Ok(()),
    Ok(_) => crate::error::bail!("failed to build app"),
    Err(e) if e.kind() == ErrorKind::NotFound => crate::error::bail!(
      "`{}` command not found.{}",
      runner,
      if runner == "cargo" {
        " Please follow the Tauri setup guide: https://v2.tauri.app/start/prerequisites/"
      } else {
        ""
      }
    ),
    Err(e) => Err(Error::CommandFailed {
      command: runner,
      error: e,
    }),
  }
}

fn cargo_command(
  dev: bool,
  options: Options,
  available_targets: &Option<Vec<RustupTarget>>,
  config_features: Vec<String>,
) -> crate::Result<Command> {
  let runner_config = options.runner.unwrap_or_else(|| "cargo".into());

  let mut build_cmd = Command::new(runner_config.cmd());
  // kill the runner if the CLI drops the child without waiting for it to finish
  build_cmd.kill_on_drop(true);
  build_cmd.arg(if dev { "run" } else { "build" });

  // Set working directory if specified
  if let Some(cwd) = runner_config.cwd() {
    build_cmd.current_dir(cwd);
  }

  // Add runner-specific arguments first
  if let Some(runner_args) = runner_config.args() {
    build_cmd.args(runner_args);
  }

  if let Some(target) = &options.target {
    validate_target(available_targets, target)?;
  }

  build_cmd.args(&options.args);

  let mut features = config_features;
  features.extend(options.features);
  if !features.is_empty() {
    build_cmd.arg("--features");
    build_cmd.arg(features.join(","));
  }

  if !options.debug && !options.args.contains(&"--profile".to_string()) {
    build_cmd.arg("--release");
  }

  if let Some(target) = options.target {
    build_cmd.arg("--target");
    build_cmd.arg(target);
  }

  Ok(build_cmd)
}

pub(super) async fn fetch_available_targets() -> Option<Vec<RustupTarget>> {
  if let Ok(output) = Command::new("rustup")
    .args(["target", "list"])
    .output()
    .await
  {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    Some(
      stdout
        .split('\n')
        .map(|t| {
          let mut s = t.split(' ');
          let name = s.next().unwrap().to_string();
          let installed = s.next().map(|v| v == "(installed)").unwrap_or_default();
          RustupTarget { name, installed }
        })
        .filter(|t| !t.name.is_empty())
        .collect(),
    )
  } else {
    None
  }
}

fn validate_target(
  available_targets: &Option<Vec<RustupTarget>>,
  target: &str,
) -> crate::Result<()> {
  if let Some(available_targets) = available_targets {
    if let Some(target) = available_targets.iter().find(|t| t.name == target) {
      if !target.installed {
        crate::error::bail!(
            "Target {target} is not installed (installed targets: {installed}). Please run `rustup target add {target}`.",
            target = target.name,
            installed = available_targets.iter().filter(|t| t.installed).map(|t| t.name.as_str()).collect::<Vec<&str>>().join(", ")
          );
      }
    }
    if !available_targets.iter().any(|t| t.name == target) {
      crate::error::bail!("Target {target} does not exist. Please run `rustup target list` to see the available targets.", target = target);
    }
  }
  Ok(())
}

async fn rename_app(
  bin_path: PathBuf,
  main_binary_name: Option<&str>,
  target_platform: &TargetPlatform,
) -> crate::Result<PathBuf> {
  if let Some(main_binary_name) = main_binary_name {
    let extension = if matches!(target_platform, TargetPlatform::Windows) {
      ".exe"
    } else {
      ""
    };
    let new_path = bin_path.with_file_name(format!("{main_binary_name}{extension}"));
    tokio::fs::rename(&bin_path, &new_path)
      .await
      .fs_context("failed to rename app binary", bin_path)?;
    Ok(new_path)
  } else {
    Ok(bin_path)
  }
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

  use windows_sys::{
    core::PCSTR,
    Win32::{
      Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
      Storage::FileSystem::{CreateFileA, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
      System::Console::{
        GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO, STD_ERROR_HANDLE,
      },
    },
  };

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
        c"CONOUT$".as_ptr() as PCSTR,
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        ptr::null_mut(),
        OPEN_EXISTING,
        0,
        std::ptr::null_mut(),
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
