use crate::helpers::app_paths::Dirs;
use crate::interface::{
  AppInterface, AppSettings, ExitReason, Options,
  rust::{DevChild, RustAppSettings, RustupTarget, tauri_config_to_bundle_settings},
};
use crate::{CommandExt, error::Context};

use shared_child::SharedChild;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub fn run_dev_cef_macos<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
  app_settings: &RustAppSettings,
  options: Options,
  run_args: &[String],
  available_targets: &mut Option<Vec<RustupTarget>>,
  config_features: Vec<String>,
  on_exit: F,
  interface: &AppInterface,
  dirs: &Dirs,
) -> crate::Result<DevChild> {
  // Build the app
  let mut build_cmd = crate::interface::rust::cargo_command(
    false,
    options.clone(),
    available_targets,
    config_features.clone(),
  )?;
  build_cmd.env("CARGO_TERM_PROGRESS_WIDTH", "80");
  build_cmd.env("CARGO_TERM_PROGRESS_WHEN", "always");
  match build_cmd.piped() {
    Ok(status) if status.success() => {}
    Ok(status) => {
      return Err(crate::Error::CommandFailed {
        command: build_cmd.get_program().to_string_lossy().into_owned(),
        error: std::io::Error::other(format!("exit with status {status}")),
      });
    }
    Err(e) => {
      return Err(crate::Error::CommandFailed {
        command: build_cmd.get_program().to_string_lossy().into_owned(),
        error: e,
      });
    }
  }

  // Bundle the .app using the bundler
  let out_dir = app_settings.out_dir(&options, dirs.tauri)?;
  let bin_path = app_settings.app_binary_path(&options, dirs.tauri)?;
  let exec_name = bin_path
    .file_name()
    .and_then(|s| s.to_str())
    .ok_or_else(|| crate::Error::GenericError("failed to determine executable name".into()))?;

  // Build bundler settings for dev mode using the shared helper
  let target = if let Some(target) = options.target.clone() {
    target
  } else {
    tauri_utils::platform::target_triple().context("failed to get target triple")?
  };

  // Merge features
  let mut merged_features = config_features.clone();
  merged_features.extend(options.features.clone());

  // Get minimal config for dev mode (we'll use defaults for most things)
  let config = crate::helpers::config::get_config(
    tauri_utils::platform::Target::MacOS,
    &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
    dirs.tauri,
  )?;

  if let Some(before_bundle) = config.build.before_bundle_command.clone() {
    crate::helpers::run_hook(
      "beforeBundleCommand",
      before_bundle,
      interface,
      options.debug,
      dirs.frontend,
    )?;
  }

  // Get bundle settings using the shared helper
  let arch64bits =
    target.starts_with("x86_64") || target.starts_with("aarch64") || target.starts_with("riscv64");

  let bundle_settings = tauri_config_to_bundle_settings(
    app_settings,
    &merged_features,
    &config,
    dirs.tauri,
    config.bundle.clone(),
    None, // No updater in dev mode
    arch64bits,
  )?;
  let mut settings = tauri_bundler::bundle::SettingsBuilder::new()
    .package_settings(app_settings.get_package_settings())
    .bundle_settings(bundle_settings)
    .binaries(app_settings.get_binaries(&options, dirs.tauri)?)
    .project_out_directory(out_dir.clone())
    .target(target)
    .package_types(vec![tauri_bundler::bundle::PackageType::MacOsBundle])
    .build()
    .context("failed to build bundler settings")?;

  settings.set_log_level(log::Level::Info);

  // Bundle the app
  let bundles = tauri_bundler::bundle_project(&settings)
    .map_err(Box::new)
    .context("failed to bundle app")?;

  let app_bundle_path = bundles
    .first()
    .and_then(|b| b.bundle_paths.first())
    .ok_or_else(|| crate::Error::GenericError("no bundle created".into()))?
    .clone();

  // Launch the app executable from inside the .app
  let mut exec_cmd = Command::new(app_bundle_path.join("Contents/MacOS").join(exec_name));
  exec_cmd.stdout(os_pipe::dup_stdout().unwrap());
  exec_cmd.stderr(Stdio::piped());
  exec_cmd.args(run_args);

  let child = SharedChild::spawn(&mut exec_cmd).map_err(|e| crate::Error::CommandFailed {
    command: exec_cmd.get_program().to_string_lossy().into_owned(),
    error: e,
  })?;

  let dev_child = Arc::new(child);
  let dev_child_stderr = dev_child.take_stderr().unwrap();
  let mut stderr = BufReader::new(dev_child_stderr);
  let stderr_lines = Arc::new(Mutex::new(Vec::new()));
  let stderr_lines_ = stderr_lines.clone();
  std::thread::spawn(move || {
    let mut buf = Vec::new();
    let mut lines = stderr_lines_.lock().unwrap();
    let mut io_stderr = std::io::stderr();
    loop {
      buf.clear();
      if let Ok(0) = tauri_utils::io::read_line(&mut stderr, &mut buf) {
        break;
      }
      let _ = io_stderr.write_all(&buf);
      lines.push(String::from_utf8_lossy(&buf).into_owned());
    }
  });

  let manually_killed_app = Arc::new(AtomicBool::default());
  let manually_killed_app_ = manually_killed_app.clone();
  let dev_child_ = dev_child.clone();
  std::thread::spawn(move || {
    let status = dev_child_.wait().expect("failed to run app");
    if status.success() {
      on_exit(status.code(), ExitReason::NormalExit);
    } else {
      stderr_lines.lock().unwrap().clear();
      on_exit(
        status.code(),
        if manually_killed_app_.load(Ordering::Relaxed) {
          ExitReason::TriggeredKill
        } else {
          ExitReason::NormalExit
        },
      );
    }
  });

  Ok(DevChild {
    manually_killed_app,
    dev_child,
  })
}
