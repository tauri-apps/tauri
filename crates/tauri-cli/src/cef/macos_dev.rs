//! Running a cef app from `tauri dev` on macOS.
//!
//! CEF resolves its framework relative to the running executable
//! (`<exe>/../Frameworks/Chromium Embedded Framework.framework`) and launches
//! its renderer/GPU/utility subprocesses through helper executables that must
//! themselves be nested `.app`s — so a cef app only runs from inside an `.app`
//! bundle, and `tauri dev` cannot just execute the binary it built. Both dev
//! flows therefore bundle an `.app` first and run the executable inside it:
//!
//! - a Rust app builds with cargo and bundles the result ([`run_dev_cef_macos`]);
//! - a bindings app (node/deno/python) has no binary to build — its executable is
//!   the language runtime — so it bundles an `.app` *around* a copy of that
//!   runtime ([`stage_bindings_dev_app`]).
//!
//! Both go through [`bundle_dev_app`], which is what stages the CEF framework and
//! the helper `.app`s (the bundler embeds the helper binary, so neither flow has
//! to build or ship one).

use crate::helpers::app_paths::Dirs;
use crate::helpers::config::Config;
use crate::interface::{
  AppInterface, AppSettings, ExitReason, Options,
  rust::{DevChild, RustAppSettings, RustupTarget, tauri_config_to_bundle_settings},
};
use crate::{CommandExt, error::Context, error::ErrorExt};

use shared_child::SharedChild;
use std::fs;
use std::io::{BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri_bundler::{AppCategory, BundleBinary, BundleSettings, PackageSettings};

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
  let app_bundle_path = bundle_dev_app(&settings)?;

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

/// Bundles the `.app` a dev run executes from, returning its path.
///
/// This is where CEF actually gets staged: with `cef_path` set on the bundle
/// settings, the macOS bundler copies the framework into `Contents/Frameworks`
/// and creates the helper `.app`s next to it from the helper binary it embeds at
/// build time.
fn bundle_dev_app(settings: &tauri_bundler::Settings) -> crate::Result<PathBuf> {
  let bundles = tauri_bundler::bundle_project(settings)
    .map_err(Box::new)
    .context("failed to bundle app")?;

  bundles
    .first()
    .and_then(|b| b.bundle_paths.first())
    .cloned()
    .ok_or_else(|| crate::Error::GenericError("no bundle created".into()))
}

/// The dev `.app` staged for a bindings app, and the two paths its runner is
/// started with.
pub struct BindingsDevApp {
  /// The executable to spawn instead of the runner: a copy of the runner inside
  /// `Contents/MacOS`, so CEF (and AppKit) see a bundled app.
  pub executable: PathBuf,
  /// The helper executable to point CEF at through `TAURI_CEF_SUBPROCESS_PATH`.
  pub subprocess_helper: PathBuf,
}

/// Stages the dev `.app` a cef bindings app (node/deno/python) runs from.
///
/// A bindings app has no binary of its own — the app *is* the language runtime
/// executing a script — so there is nothing to build here, only an `.app` to
/// bundle around it. `runner_exe` (the resolved `node`/`deno`/`python`) is copied
/// in as the bundle's main binary and spawned from there with the same script
/// arguments, which is what puts the CEF framework and the helper `.app`s within
/// reach of the running process.
///
/// The bundle is staged in its own `dev-app/` directory so it never collides with
/// the standalone binary and bundles `tauri build` writes to the same out dir.
/// It is deliberately *not* the app's real bundle: no frontend assets, config or
/// capabilities are staged into it (dev reads those from disk and the config env)
/// and `beforeBundleCommand` is not run, since nothing here ships.
pub fn stage_bindings_dev_app(
  package_settings: PackageSettings,
  binaries: Vec<BundleBinary>,
  config: &Config,
  runner_exe: &Path,
  cef_dir: &Path,
  out_dir: &Path,
  target: String,
) -> crate::Result<BindingsDevApp> {
  let bin_name = binaries
    .iter()
    .find(|bin| bin.main())
    .map(|bin| bin.name().to_string())
    .ok_or_else(|| crate::Error::GenericError("no main binary to bundle".into()))?;

  let project_out_directory = out_dir.join("dev-app");
  fs::create_dir_all(&project_out_directory).fs_context(
    "failed to create the dev app directory",
    project_out_directory.clone(),
  )?;

  // The bundler bundles what it finds at `<project out>/<main binary>`, so the
  // runner takes that place. Copied rather than symlinked: the bundle's own copy
  // is what gets signed, and a language runtime resolves its standard library
  // from its compiled-in prefix, not from where it is executed.
  //
  // A previous run's copy is removed rather than overwritten, and the fresh one
  // made writable, because `fs::copy` carries the source's mode over and package
  // managers install their runtimes read-only (Homebrew's `deno` is `r-xr-xr-x`):
  // overwriting that copy — here or when the bundler copies it into the `.app`,
  // or when codesign rewrites it — fails with a permission error.
  let staged_runner = project_out_directory.join(&bin_name);
  let _ = fs::remove_file(&staged_runner);
  fs::copy(runner_exe, &staged_runner).fs_context(
    "failed to stage the runner executable for the dev app bundle",
    runner_exe.to_path_buf(),
  )?;
  fs::set_permissions(&staged_runner, fs::Permissions::from_mode(0o755)).fs_context(
    "failed to make the staged runner executable writable",
    staged_runner.clone(),
  )?;

  let bundle_settings = BundleSettings {
    identifier: Some(config.identifier.clone()),
    publisher: config.bundle.publisher.clone(),
    icon: (!config.bundle.icon.is_empty()).then(|| config.bundle.icon.clone()),
    category: config
      .bundle
      .category
      .as_deref()
      .and_then(|category| AppCategory::from_str(category).ok()),
    short_description: config
      .bundle
      .short_description
      .clone()
      .or_else(|| config.product_name.clone()),
    long_description: config.bundle.long_description.clone(),
    // What routes the bundler through its CEF staging: the framework is copied
    // out of this directory and the helper `.app`s are created beside it.
    cef_path: Some(cef_dir.to_path_buf()),
    ..Default::default()
  };

  let mut settings = tauri_bundler::bundle::SettingsBuilder::new()
    .package_settings(package_settings)
    .bundle_settings(bundle_settings)
    .binaries(binaries)
    .project_out_directory(&project_out_directory)
    .target(target)
    .package_types(vec![tauri_bundler::bundle::PackageType::MacOsBundle])
    .build()
    .context("failed to build bundler settings")?;
  settings.set_log_level(log::Level::Info);

  let app_bundle_path = bundle_dev_app(&settings)?;

  // The helper `.app`s the bundler creates are named after the main binary; CEF
  // finds them by that convention, but the bindings point it at one explicitly so
  // the dev run does not depend on the bundle's naming matching what CEF derives.
  let helper_name = format!("{bin_name} Helper");
  Ok(BindingsDevApp {
    executable: app_bundle_path.join("Contents/MacOS").join(&bin_name),
    subprocess_helper: app_bundle_path
      .join("Contents/Frameworks")
      .join(format!("{helper_name}.app"))
      .join("Contents/MacOS")
      .join(&helper_name),
  })
}
