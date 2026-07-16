// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Interface for tauri-ffi bindings projects (Node.js, Deno, Python, …).
//!
//! These projects have no `Cargo.toml` — the application is a script executed
//! by the `build > runner` command from the Tauri config (e.g.
//! `{ "cmd": "node", "args": ["main.js"] }`), and the Tauri runtime lives in
//! the prebuilt `tauri-ffi` cdylib loaded by the language bindings.
//!
//! - `dev` spawns the runner with the fully merged config in the
//!   `TAURI_CONFIG` environment variable (plus `TAURI_DEV=true`), restarting
//!   it when project files change. The `beforeDevCommand`/dev-server/devUrl
//!   handling is shared with the Rust interface (`crate::dev::setup`).
//! - `build` compiles the app into a self-contained native binary using the
//!   runner's native compiler (`deno compile`, PyInstaller, Node SEA) so it
//!   runs on a machine without the language runtime installed — the
//!   embedded-binary equivalent of a Rust `tauri build`. The frontend assets,
//!   merged config and capabilities are embedded *inside* the binary through
//!   the runner's own mechanism (Deno `--include`, Node SEA assets, PyInstaller
//!   data); only the `tauri-ffi` cdylib is staged as a sibling bundle resource,
//!   since a native library must live on disk to be `dlopen`'d. Everything is
//!   then handed to `tauri-bundler` for `.app`/`.msi`/`.appimage` packaging.
//!
//! Output goes under `<tauri_dir>/dist/` (not `target/`). The compiled app
//! reads its assets/config/capabilities from inside the executable and loads
//! the cdylib from the bundle's resource dir, so `dist/<name>` and the packaged
//! bundle both run standalone.

use std::{
  collections::HashMap,
  fs,
  io::Write,
  path::{Path, PathBuf},
  process::{Command, ExitStatus},
  str::FromStr,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::sync_channel,
  },
  time::Duration,
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use shared_child::SharedChild;
use tauri_bundler::{AppCategory, BundleBinary, BundleSettings, PackageSettings};
use tauri_utils::{
  config::{FrontendDist, WebviewRuntime, parse::is_configuration_file},
  display_path,
  platform::Target as TargetPlatform,
};

use super::{AppSettings, DevProcess, ExitReason, Interface, Options};
use crate::{
  CommandExt,
  error::{Context, ErrorExt, bail},
  helpers::{
    app_paths::Dirs,
    command_env,
    config::{BundleResources, Config, ConfigMetadata, get_config, reload_config},
  },
};

/// Magic bytes prefixing a tauri-ffi assets archive. Format (all
/// little-endian): magic, `u64` index length, index JSON
/// (`{"files":{"/path":[offset,len]}}`, offsets relative to the blob region
/// that follows the index), concatenated file contents.
const ASSETS_ARCHIVE_MAGIC: &[u8; 8] = b"TAURIPK1";

pub struct Bindings {
  app_settings: Arc<BindingsAppSettings>,
  target_triple: String,
}

pub struct BindingsAppSettings {
  package_settings: PackageSettings,
  frontend_dist: Option<FrontendDist>,
  target_triple: String,
  /// The webview runtime (`app > runtime`) whose `tauri-ffi` library is staged.
  runtime: WebviewRuntime,
}

/// The native compiler used to produce the self-contained binary, detected
/// from the `build > runner` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Runner {
  /// `deno compile`
  Deno,
  /// Node.js Single Executable Application (`--experimental-sea-config`)
  Node,
  /// PyInstaller
  Python,
}

impl Runner {
  fn detect(cmd: &str) -> crate::Result<Self> {
    let stem = Path::new(cmd)
      .file_stem()
      .map(|s| s.to_string_lossy().to_lowercase())
      .unwrap_or_default();
    if stem.starts_with("deno") {
      Ok(Runner::Deno)
    } else if stem.starts_with("node") {
      Ok(Runner::Node)
    } else if stem.starts_with("python") || stem == "py" {
      Ok(Runner::Python)
    } else {
      bail!(
        "cannot determine how to build a standalone binary for runner `{cmd}` — supported runners are deno, node and python"
      )
    }
  }
}

impl Interface for Bindings {
  type AppSettings = BindingsAppSettings;

  fn new(config: &Config, target: Option<String>, tauri_dir: &Path) -> crate::Result<Self> {
    let product_name = config.product_name.clone().unwrap_or_else(|| {
      tauri_dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "app".into())
    });
    let package_settings = PackageSettings {
      product_name,
      version: config.version.clone().unwrap_or_else(|| "0.1.0".into()),
      description: String::new(),
      homepage: None,
      authors: None,
      default_run: None,
    };
    let target_triple = if let Some(target) = target {
      target
    } else {
      tauri_utils::platform::target_triple().context("failed to get target triple")?
    };

    Ok(Self {
      app_settings: Arc::new(BindingsAppSettings {
        package_settings,
        frontend_dist: config.build.frontend_dist.clone(),
        target_triple: target_triple.clone(),
        runtime: config.app.runtime,
      }),
      target_triple,
    })
  }

  fn app_settings(&self) -> Arc<BindingsAppSettings> {
    self.app_settings.clone()
  }

  fn env(&self) -> HashMap<&str, String> {
    let mut env = HashMap::new();
    env.insert("TAURI_ENV_TARGET_TRIPLE", self.target_triple.clone());

    let target_components: Vec<&str> = self.target_triple.split('-').collect();
    let (arch, host) = match target_components.as_slice() {
      [arch, _, host] | [arch, _, host, _] => (*arch, *host),
      _ => {
        log::warn!("Invalid target triple: {}", self.target_triple);
        return env;
      }
    };
    env.insert("TAURI_ENV_ARCH", arch.into());
    env.insert("TAURI_ENV_PLATFORM", host.into());
    env.insert(
      "TAURI_ENV_FAMILY",
      match host {
        "windows" => "windows".into(),
        _ => "unix".into(),
      },
    );

    env
  }

  /// Compiles the app into a self-contained binary in `dist/`. The frontend
  /// assets, config and capabilities are embedded *inside* the binary through
  /// the runner's native mechanism (Deno `--include`, Node SEA assets,
  /// PyInstaller data); only the `tauri-ffi` cdylib remains a sibling resource,
  /// since a native library must live on disk to be `dlopen`'d.
  fn build(&mut self, options: Options, dirs: &Dirs) -> crate::Result<PathBuf> {
    let Some(runner_config) = options.runner.clone() else {
      bail!(
        "no runner configured — set `build > runner` in your Tauri config, e.g. {}",
        r#"{ "cmd": "node", "args": ["main.js"] }"#
      );
    };
    let runner = Runner::detect(runner_config.cmd())?;
    let entry = runner_entry(&runner_config, dirs.tauri)?;

    let out_dir = self.app_settings.out_dir(&options, dirs.tauri)?;
    let resources_dir = out_dir.join("resources");
    // Start clean so a prior build's staged files (older assets/config that are
    // now embedded, not resources) don't linger and get bundled. Everything the
    // bundler picks up from here is re-staged below (cdylib) or during compile
    // (Node's koffi.node).
    let _ = fs::remove_dir_all(&resources_dir);
    fs::create_dir_all(&resources_dir).fs_context(
      "failed to create resources directory",
      resources_dir.clone(),
    )?;

    // 1. stage the tauri-ffi cdylib the compiled app loads over FFI (the only
    //    sibling resource — a native library can't be embedded and dlopen'd).
    //    The library is per-runtime (`app > runtime`): `libtauri_<runtime>`.
    let runtime = self.app_settings.runtime;
    let lib_name = cdylib_name(&self.app_settings.target_triple, runtime);
    let lib_src = resolve_cdylib(&self.app_settings.target_triple, runtime, dirs.tauri)?;
    let lib_dest = resources_dir.join(&lib_name);
    fs::copy(&lib_src, &lib_dest)
      .fs_context("failed to stage tauri-ffi library", lib_src.clone())?;
    log::info!(action = "Bundling"; "tauri-ffi ({runtime}) library from {}", display_path(&lib_src));

    // 2. build the payload embedded *into* the binary: the packed frontend
    //    archive, the merged config (frontendDist rewritten to the archive) and
    //    the packed capabilities — the standalone equivalent of the assets and
    //    resolved ACL a Rust `tauri build` bakes into the executable.
    let embed_dir = out_dir.join(".embed");
    let payload = stage_embed_payload(
      &self.app_settings.frontend_dist,
      dirs.tauri,
      &options,
      &embed_dir,
    )?;

    // 3. compile the self-contained binary, embedding the payload
    let bin_path = self.app_settings.app_binary_path(&options, dirs.tauri)?;
    log::info!(action = "Compiling"; "{} app into {}", runner_label(runner), display_path(&bin_path));
    compile_binary(runner, &entry, &bin_path, dirs.tauri, &payload)?;

    Ok(bin_path)
  }

  fn dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    config: &mut ConfigMetadata,
    options: Options,
    on_exit: F,
    dirs: &Dirs,
  ) -> crate::Result<()> {
    let on_exit = Arc::new(on_exit);

    if options.no_watch {
      let (tx, rx) = sync_channel(1);
      let _child = self.spawn_app(config, &options, dirs, move |status, reason| {
        on_exit(status, reason);
        tx.send(()).unwrap();
      })?;
      rx.recv().unwrap();
      Ok(())
    } else {
      self.run_dev_watcher(config, options, on_exit, dirs)
    }
  }
}

impl Bindings {
  fn spawn_app<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &self,
    config: &ConfigMetadata,
    options: &Options,
    dirs: &Dirs,
    on_exit: F,
  ) -> crate::Result<BindingsDevChild> {
    let Some(runner) = options.runner.clone() else {
      bail!(
        "no runner configured for this bindings project — set `build > runner` in your Tauri config, e.g. {}, or pass `--runner`",
        r#"{ "cmd": "node", "args": ["main.js"] }"#
      );
    };

    let mut command = std::process::Command::new(runner.cmd());
    let cwd = runner
      .cwd()
      .map(|cwd| dirs.tauri.join(cwd))
      .unwrap_or_else(|| dirs.tauri.to_path_buf());
    command.current_dir(cwd);
    if let Some(args) = runner.args() {
      command.args(args);
    }
    command.args(&options.args);

    command.envs(command_env(options.debug));
    command.envs(self.env());
    // The bindings read the fully merged config from TAURI_CONFIG — the
    // runtime equivalent of what tauri-build does at compile time — and
    // TAURI_DEV switches the tauri-ffi runtime into dev mode (devUrl serving).
    command.env(
      "TAURI_CONFIG",
      serde_json::to_string(&**config).context("failed to serialize config")?,
    );
    command.env("TAURI_DEV", "true");

    // A cef tauri-ffi build dynamically links libcef (and CEF loads its resource
    // files from next to it); cef-dll-sys stages both next to the build output.
    // The prebuilt cdylib has no rpath, so the runner's `dlopen` fails with
    // `libcef.so: cannot open shared object file` unless we point its dynamic
    // loader at that directory. Detect a cef build by the staged libcef (rather
    // than `app.runtime`) so it also works when the config wasn't set — in dev
    // the runner loads whatever was built — and is a no-op for a wry build.
    if cfg!(target_os = "macos") {
      // macOS CEF needs the .app framework/helper layout (and SIP strips DYLD_*
      // from a signed runner like node), so a library-search path is not enough
      // — dev must bundle-and-run like `crate::cef::macos_dev`.
      if self.app_settings.runtime == WebviewRuntime::Cef {
        log::warn!(
          "`tauri dev` with the cef runtime is not yet supported for bindings apps on macOS (the CEF framework must be staged in an .app bundle)"
        );
      }
    } else if let Ok(lib) = resolve_cdylib(&self.target_triple, self.app_settings.runtime, dirs.tauri)
    {
      let libcef = if self.target_triple.contains("windows") {
        "libcef.dll"
      } else {
        "libcef.so"
      };
      if let Some(dir) = lib.parent() {
        if dir.join(libcef).exists() {
          prepend_library_search_path(&mut command, dir);
        }
      }
    }

    log::info!(
      action = "Running";
      "`{} {}`",
      runner.cmd(),
      command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ")
    );

    let child = SharedChild::spawn(&mut command).map_err(|error| crate::Error::CommandFailed {
      command: runner.cmd().into(),
      error,
    })?;
    let child = Arc::new(child);
    let manually_killed = Arc::new(AtomicBool::default());

    let child_ = child.clone();
    let manually_killed_ = manually_killed.clone();
    std::thread::spawn(move || {
      let status = child_.wait().expect("failed to wait on app process");
      on_exit(
        status.code(),
        if manually_killed_.load(Ordering::SeqCst) {
          ExitReason::TriggeredKill
        } else {
          ExitReason::NormalExit
        },
      );
    });

    Ok(BindingsDevChild {
      child,
      manually_killed,
    })
  }

  fn run_dev_watcher<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    config: &mut ConfigMetadata,
    options: Options,
    on_exit: Arc<F>,
    dirs: &Dirs,
  ) -> crate::Result<()> {
    let on_exit_ = on_exit.clone();
    let mut child = self.spawn_app(config, &options, dirs, move |status, reason| {
      on_exit_(status, reason)
    })?;

    let ignore_matcher = build_ignore_matcher(dirs.tauri, &self.app_settings.frontend_dist);

    let (tx, rx) = sync_channel(1);
    let mut watcher = new_debouncer(Duration::from_secs(1), None, move |r| {
      if let Ok(events) = r {
        tx.send(events).unwrap()
      }
    })
    .unwrap();

    // Watch the non-ignored top-level entries of the Tauri directory (so
    // node_modules & friends are never watched) plus any additional folders.
    log::info!("Watching {} for changes...", display_path(dirs.tauri));
    for entry in fs::read_dir(dirs.tauri)
      .fs_context("failed to read directory", dirs.tauri.to_path_buf())?
      .flatten()
    {
      let path = entry.path();
      let is_dir = path.is_dir();
      if !is_ignored(&ignore_matcher, &path, is_dir) {
        let _ = watcher.watch(
          &path,
          if is_dir {
            RecursiveMode::Recursive
          } else {
            RecursiveMode::NonRecursive
          },
        );
      }
    }
    for folder in &options.additional_watch_folders {
      let path = if folder.is_absolute() {
        folder.clone()
      } else {
        dirs.tauri.join(folder)
      };
      if let Ok(path) = dunce::canonicalize(&path) {
        log::info!("Watching {} for changes...", display_path(&path));
        let _ = watcher.watch(&path, RecursiveMode::Recursive);
      } else {
        log::warn!(
          "Additional watch folder '{}' not found, ignoring",
          path.display()
        );
      }
    }

    let merge_configs = options
      .config
      .iter()
      .map(|c| c.0.clone())
      .collect::<Vec<_>>();
    while let Ok(events) = rx.recv() {
      let paths: Vec<PathBuf> = events
        .into_iter()
        .filter(|event| !event.kind.is_access())
        .flat_map(|event| event.event.paths)
        .filter(|path| !is_ignored(&ignore_matcher, path, path.is_dir()))
        .collect();

      let config_file_changed = paths
        .iter()
        .any(|path| is_configuration_file(TargetPlatform::current(), path));
      if config_file_changed {
        let merge_configs = merge_configs.iter().collect::<Vec<_>>();
        if let Err(error) = reload_config(config, &merge_configs, dirs.tauri) {
          log::error!("failed to reload config: {error}");
          continue;
        }
      }

      let Some(first_changed_path) = paths.first() else {
        continue;
      };

      log::info!(
        "File {} changed. Restarting application...",
        display_path(
          first_changed_path
            .strip_prefix(dirs.frontend)
            .unwrap_or(first_changed_path)
        )
      );

      child.kill().context("failed to kill app process")?;
      let _ = child.wait();
      let on_exit_ = on_exit.clone();
      child = self.spawn_app(config, &options, dirs, move |status, reason| {
        on_exit_(status, reason)
      })?;
    }
    bail!("File watcher exited unexpectedly")
  }
}

pub struct BindingsDevChild {
  child: Arc<SharedChild>,
  manually_killed: Arc<AtomicBool>,
}

impl DevProcess for BindingsDevChild {
  fn kill(&self) -> std::io::Result<()> {
    self.manually_killed.store(true, Ordering::SeqCst);
    self.child.kill()
  }

  fn wait(&self) -> std::io::Result<ExitStatus> {
    self.child.wait()
  }

  fn manually_killed_process(&self) -> bool {
    self.manually_killed.load(Ordering::SeqCst)
  }
}

impl AppSettings for BindingsAppSettings {
  fn get_package_settings(&self) -> PackageSettings {
    self.package_settings.clone()
  }

  fn get_bundle_settings(
    &self,
    options: &Options,
    config: &Config,
    _features: &[String],
    tauri_dir: &Path,
  ) -> crate::Result<BundleSettings> {
    let bundle = config.bundle.clone();
    let resources_dir = self.out_dir(options, tauri_dir)?.join("resources");

    // The cdylib is the only staged resource (assets/config/capabilities live
    // inside the binary); copy it — and any user-declared resources — into the
    // bundle's resource dir, where the compiled app loads the cdylib at startup.
    let mut resources_map = HashMap::new();
    for entry in fs::read_dir(&resources_dir)
      .fs_context("failed to read staged resources", resources_dir.clone())?
      .flatten()
    {
      let path = entry.path();
      if let Some(name) = path.file_name() {
        resources_map.insert(
          path.to_string_lossy().into_owned(),
          name.to_string_lossy().into_owned(),
        );
      }
    }
    // extra resources declared by the user, copied verbatim
    if let Some(BundleResources::List(list)) = &bundle.resources {
      for resource in list {
        resources_map.insert(
          resource.clone(),
          Path::new(resource).to_string_lossy().into_owned(),
        );
      }
    } else if let Some(BundleResources::Map(map)) = &bundle.resources {
      resources_map.extend(map.clone());
    }

    let icon = if bundle.icon.is_empty() {
      None
    } else {
      Some(bundle.icon.clone())
    };

    Ok(BundleSettings {
      identifier: Some(config.identifier.clone()),
      publisher: bundle.publisher.clone(),
      homepage: bundle.homepage.clone(),
      icon,
      resources: None,
      resources_map: Some(resources_map),
      copyright: bundle.copyright.clone(),
      category: bundle
        .category
        .as_deref()
        .and_then(|c| AppCategory::from_str(c).ok()),
      short_description: bundle
        .short_description
        .clone()
        .or_else(|| config.product_name.clone()),
      long_description: bundle.long_description.clone(),
      ..Default::default()
    })
  }

  fn app_binary_path(&self, options: &Options, tauri_dir: &Path) -> crate::Result<PathBuf> {
    let name = binary_name(&self.package_settings.product_name);
    let mut path = self.out_dir(options, tauri_dir)?.join(name);
    if self.target_triple.contains("windows") {
      path.set_extension("exe");
    }
    Ok(path)
  }

  fn get_binaries(&self, options: &Options, tauri_dir: &Path) -> crate::Result<Vec<BundleBinary>> {
    let bin_path = self.app_binary_path(options, tauri_dir)?;
    let name = bin_path
      .file_name()
      .expect("binary path has no file name")
      .to_string_lossy()
      .into_owned();
    Ok(vec![BundleBinary::new(name, true)])
  }

  fn app_name(&self) -> Option<String> {
    Some(self.package_settings.product_name.clone())
  }

  fn lib_name(&self) -> Option<String> {
    None
  }

  fn out_dir(&self, options: &Options, tauri_dir: &Path) -> crate::Result<PathBuf> {
    let out_dir = tauri_dir
      .join("dist")
      .join(if options.debug { "debug" } else { "release" });
    fs::create_dir_all(&out_dir).fs_context("failed to create out directory", out_dir.clone())?;
    Ok(out_dir)
  }
}

/// The binary/product name, sanitized for use as a file name.
fn binary_name(product_name: &str) -> String {
  product_name
    .chars()
    .map(|c| {
      if c.is_alphanumeric() || c == '-' || c == '_' {
        c
      } else {
        '-'
      }
    })
    .collect()
}

/// The distributed tauri-ffi cdylib file name for a target triple and runtime:
/// `libtauri_<runtime>` (`tauri_<runtime>.dll` on Windows).
fn cdylib_name(target_triple: &str, runtime: WebviewRuntime) -> String {
  let kind = runtime.as_str();
  if target_triple.contains("windows") {
    format!("tauri_{kind}.dll")
  } else if target_triple.contains("apple") {
    format!("libtauri_{kind}.dylib")
  } else {
    format!("libtauri_{kind}.so")
  }
}

/// The crate's own cdylib output name (`cargo build -p tauri-ffi`), which is
/// runtime-agnostic — the local dev-build fallback in [`resolve_cdylib`].
fn crate_cdylib_name(target_triple: &str) -> String {
  if target_triple.contains("windows") {
    "tauri_ffi.dll".into()
  } else if target_triple.contains("apple") {
    "libtauri_ffi.dylib".into()
  } else {
    "libtauri_ffi.so".into()
  }
}

/// Prepends `dir` to the dynamic-library search path (`LD_LIBRARY_PATH`, or
/// `PATH` on Windows) of `command`, so a cef runner's `dlopen` of the tauri-ffi
/// library resolves the `libcef` staged next to it.
fn prepend_library_search_path(command: &mut Command, dir: &Path) {
  let var = if cfg!(windows) { "PATH" } else { "LD_LIBRARY_PATH" };
  let mut paths = vec![dir.to_path_buf()];
  if let Some(existing) = std::env::var_os(var) {
    paths.extend(std::env::split_paths(&existing));
  }
  if let Ok(joined) = std::env::join_paths(paths) {
    command.env(var, joined);
  }
}

/// Locates the prebuilt tauri-ffi cdylib to bundle. Checks `TAURI_FFI_LIB`, then
/// a `target/{release,debug}` build in this repo/workspace — first the
/// runtime-specific distribution name (`libtauri_<runtime>`), then the crate's
/// own output name (`libtauri_ffi`, what `cargo build -p tauri-ffi` emits).
fn resolve_cdylib(
  target_triple: &str,
  runtime: WebviewRuntime,
  tauri_dir: &Path,
) -> crate::Result<PathBuf> {
  if let Some(env) = std::env::var_os("TAURI_FFI_LIB") {
    let path = PathBuf::from(env);
    if path.is_file() {
      return Ok(path);
    }
    bail!(
      "TAURI_FFI_LIB points at {}, which does not exist",
      path.display()
    );
  }
  let dist_name = cdylib_name(target_triple, runtime);
  let crate_name = crate_cdylib_name(target_triple);
  let mut dir = Some(tauri_dir);
  while let Some(current) = dir {
    for profile in ["release", "debug"] {
      for name in [dist_name.as_str(), crate_name.as_str()] {
        let candidate = current.join("target").join(profile).join(name);
        if candidate.is_file() {
          return Ok(candidate);
        }
      }
    }
    dir = current.parent();
  }
  bail!(
    "could not find the tauri-ffi library `{dist_name}` for the `{runtime}` runtime — set the `TAURI_FFI_LIB` environment variable to its path"
  )
}

/// The runner script entry (the last argument of the runner command).
fn runner_entry(
  runner: &tauri_utils::config::RunnerConfig,
  tauri_dir: &Path,
) -> crate::Result<PathBuf> {
  let entry = runner
    .args()
    .and_then(|args| args.iter().rev().find(|a| !a.starts_with('-')))
    .context("could not determine the app entry from `build > runner > args`")?;
  Ok(tauri_dir.join(entry))
}

fn runner_label(runner: Runner) -> &'static str {
  match runner {
    Runner::Deno => "Deno",
    Runner::Node => "Node.js",
    Runner::Python => "Python",
  }
}

/// The frontend assets, merged config and capabilities the compiled binary
/// embeds inside itself. `assets`/`capabilities` are `None` when the project
/// has none (remote frontend / no `capabilities/` directory). File names are a
/// fixed contract the language runtimes read back (`app.assets`, `config.json`,
/// `capabilities.json`).
struct EmbedPayload {
  /// Packed frontend archive, or `None` for a remote (URL) frontend.
  assets: Option<PathBuf>,
  /// Merged config (`frontendDist` rewritten to `app.assets` when packed).
  config: PathBuf,
  /// Capabilities packed as a JSON array of capability-file strings, or `None`.
  capabilities: Option<PathBuf>,
}

/// Builds the [`EmbedPayload`] in `embed_dir` — the assets/config/capabilities
/// baked into the standalone binary, the equivalent of what a Rust `tauri
/// build` embeds into its executable.
fn stage_embed_payload(
  frontend_dist: &Option<FrontendDist>,
  tauri_dir: &Path,
  options: &Options,
  embed_dir: &Path,
) -> crate::Result<EmbedPayload> {
  let _ = fs::remove_dir_all(embed_dir);
  fs::create_dir_all(embed_dir)
    .fs_context("failed to create embed directory", embed_dir.to_path_buf())?;

  let assets = pack_assets(frontend_dist, tauri_dir, embed_dir)?;
  let config = write_embed_config(tauri_dir, options, embed_dir, assets.is_some())?;
  let capabilities = pack_capabilities(tauri_dir, embed_dir)?;

  Ok(EmbedPayload {
    assets,
    config,
    capabilities,
  })
}

/// Packs `frontend_dist` into `<embed_dir>/app.assets`. Returns `None` for a
/// remote (URL) frontend or when no `frontendDist` is configured.
fn pack_assets(
  frontend_dist: &Option<FrontendDist>,
  tauri_dir: &Path,
  embed_dir: &Path,
) -> crate::Result<Option<PathBuf>> {
  let dir = match frontend_dist {
    Some(FrontendDist::Directory(dir)) => {
      if dir.is_absolute() {
        dir.clone()
      } else {
        tauri_dir.join(dir)
      }
    }
    Some(FrontendDist::Url(url)) => {
      log::info!("frontendDist is the URL {url}; no assets to pack");
      return Ok(None);
    }
    Some(FrontendDist::Files(_)) => {
      bail!("`frontendDist` file lists are not supported for bindings projects — use a directory")
    }
    Some(_) => bail!("unsupported frontendDist configuration for bindings projects"),
    None => {
      log::info!("no frontendDist configured; no assets to pack");
      return Ok(None);
    }
  };
  let archive = embed_dir.join("app.assets");
  let count = write_assets_archive(&dir, &archive)?;
  log::info!(action = "Packed"; "{count} assets into {}", display_path(&archive));
  Ok(Some(archive))
}

/// Writes `<embed_dir>/config.json` — the merged config with `frontendDist`
/// rewritten to `app.assets` when a frontend archive was packed (a sentinel
/// keeping the custom-protocol origin; the runtime serves the embedded archive
/// bytes directly, not this path).
fn write_embed_config(
  tauri_dir: &Path,
  options: &Options,
  embed_dir: &Path,
  packed_assets: bool,
) -> crate::Result<PathBuf> {
  let target = TargetPlatform::current();
  let config = get_config(
    target,
    &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
    tauri_dir,
  )?;
  let mut value = serde_json::to_value(&*config).context("failed to serialize config")?;
  if packed_assets {
    value["build"]["frontendDist"] = serde_json::Value::String("app.assets".into());
  }
  let out = embed_dir.join("config.json");
  fs::write(
    &out,
    serde_json::to_string(&value).context("failed to serialize config")?,
  )
  .fs_context("failed to write embedded config", out.clone())?;
  Ok(out)
}

/// Packs the `capabilities/` directory into `<embed_dir>/capabilities.json` — a
/// JSON array of each capability file's raw content (JSON or TOML), which the
/// runtime hands to `add_capability` one by one. This is the standalone-binary
/// equivalent of the resolved ACL a Rust app bakes into its executable. The
/// filtering mirrors the runtimes' dev-mode discovery (extensions `.json`,
/// `.json5`, `.toml`; the `schemas/` subfolder is skipped). Returns `None` when
/// there is no `capabilities/` directory, so the app falls back to the built-in
/// `core:default` grant.
fn pack_capabilities(tauri_dir: &Path, embed_dir: &Path) -> crate::Result<Option<PathBuf>> {
  let src = tauri_dir.join("capabilities");
  if !src.is_dir() {
    return Ok(None);
  }
  let mut files: Vec<PathBuf> = Vec::new();
  for entry in walkdir::WalkDir::new(&src)
    .into_iter()
    .filter_entry(|e| e.file_name() != "schemas")
  {
    let entry = entry.context("failed to read capabilities directory")?;
    if !entry.file_type().is_file() {
      continue;
    }
    let ext = entry
      .path()
      .extension()
      .and_then(|e| e.to_str())
      .map(str::to_lowercase);
    if matches!(ext.as_deref(), Some("json" | "json5" | "toml")) {
      files.push(entry.path().to_path_buf());
    }
  }
  if files.is_empty() {
    return Ok(None);
  }
  files.sort();

  let mut capabilities: Vec<String> = Vec::with_capacity(files.len());
  for file in &files {
    capabilities
      .push(fs::read_to_string(file).fs_context("failed to read capability file", file.clone())?);
  }
  let out = embed_dir.join("capabilities.json");
  fs::write(
    &out,
    serde_json::to_string(&capabilities).context("failed to serialize capabilities")?,
  )
  .fs_context("failed to write embedded capabilities", out.clone())?;
  log::info!(action = "Packed"; "{} capabilities from {}", files.len(), display_path(&src));
  Ok(Some(out))
}

/// Compiles the app into a self-contained binary at `bin_path`, embedding the
/// [`EmbedPayload`] via the runner's native mechanism.
fn compile_binary(
  runner: Runner,
  entry: &Path,
  bin_path: &Path,
  tauri_dir: &Path,
  payload: &EmbedPayload,
) -> crate::Result<()> {
  match runner {
    Runner::Deno => compile_deno(entry, bin_path, tauri_dir, payload),
    Runner::Python => compile_python(entry, bin_path, tauri_dir, payload),
    Runner::Node => compile_node(entry, bin_path, tauri_dir, payload),
  }
}

/// Copies the payload files into `<dir>` under their runtime-contract names
/// (`app.assets`, `config.json`, `capabilities.json`). Used by the runners that
/// embed a directory (Deno `--include`, PyInstaller `--add-data`).
fn copy_payload_into(payload: &EmbedPayload, dir: &Path) -> crate::Result<()> {
  fs::create_dir_all(dir).fs_context("failed to create embed staging dir", dir.to_path_buf())?;
  fs::copy(&payload.config, dir.join("config.json"))
    .fs_context("failed to stage embedded config", payload.config.clone())?;
  if let Some(assets) = &payload.assets {
    fs::copy(assets, dir.join("app.assets"))
      .fs_context("failed to stage embedded assets", assets.clone())?;
  }
  if let Some(capabilities) = &payload.capabilities {
    fs::copy(capabilities, dir.join("capabilities.json")).fs_context(
      "failed to stage embedded capabilities",
      capabilities.clone(),
    )?;
  }
  Ok(())
}

/// Fixed subdirectory (beside the app entry) holding the embedded payload in a
/// `deno compile` binary. The Deno runtime reads it back relative to
/// `Deno.mainModule` (see `bindings/deno/config.ts`).
const DENO_EMBED_SUBDIR: &str = ".tauri-embed";

fn compile_deno(
  entry: &Path,
  bin_path: &Path,
  tauri_dir: &Path,
  payload: &EmbedPayload,
) -> crate::Result<()> {
  // `deno compile` maps `--include`d files into a virtual FS rooted at the main
  // module, so we stage the payload beside the entry (`<entry_dir>/.tauri-embed`)
  // and the runtime reads it relative to `Deno.mainModule`. Clean it up after.
  let entry_dir = entry.parent().expect("entry has no parent directory");
  let embed_in_tree = entry_dir.join(DENO_EMBED_SUBDIR);
  copy_payload_into(payload, &embed_in_tree)?;

  let result = (|| {
    // Embed the whole project directory so the dynamically-loaded worker module
    // is included (deno compile only auto-follows static imports). Exclude the
    // build output and the source config/capabilities — the runtime reads the
    // embedded payload, not those (potentially unmerged) copies.
    let mut cmd = Command::new("deno");
    cmd
      .current_dir(tauri_dir)
      .arg("compile")
      .args(["--output".as_ref(), bin_path.as_os_str()])
      .arg("--allow-all")
      .args(["--include", "."])
      .args(["--exclude", "dist"])
      .args(["--exclude", "tauri.conf.json"])
      .args(["--exclude", "capabilities"])
      .arg(entry);
    run_compiler("deno compile", &mut cmd)
  })();

  let _ = fs::remove_dir_all(&embed_in_tree);
  result
}

fn compile_python(
  entry: &Path,
  bin_path: &Path,
  tauri_dir: &Path,
  payload: &EmbedPayload,
) -> crate::Result<()> {
  let name = bin_path.file_name().expect("binary path has no file name");
  let dist = bin_path.parent().expect("binary path has no parent");
  let work = dist.join(".pyinstaller");
  let mut cmd = Command::new("pyinstaller");
  cmd
    .current_dir(tauri_dir)
    .arg("--onefile")
    .args(["--name".as_ref(), name])
    .args(["--distpath".as_ref(), dist.as_os_str()])
    .args(["--workpath".as_ref(), work.as_os_str()])
    .args(["--specpath".as_ref(), work.as_os_str()])
    .args(["--paths".as_ref(), tauri_dir.as_os_str()])
    .arg("--noconfirm");
  // Embed the payload as data files, extracted to `sys._MEIPASS` at runtime.
  // PyInstaller's `--add-data` uses `<src><os-sep><dest-dir>`; `.` places the
  // files at the _MEIPASS root, where the runtime reads them back.
  let sep = if cfg!(windows) { ";" } else { ":" };
  let mut add_data = |src: &Path| {
    let mut spec = src.as_os_str().to_os_string();
    spec.push(sep);
    spec.push(".");
    cmd.arg("--add-data").arg(spec);
  };
  add_data(&payload.config);
  if let Some(assets) = &payload.assets {
    add_data(assets);
  }
  if let Some(capabilities) = &payload.capabilities {
    add_data(capabilities);
  }
  // let PyInstaller resolve the tauri_ffi package (and any dev sys.path entries)
  if let Some(python_path) = std::env::var_os("PYTHONPATH") {
    for path in std::env::split_paths(&python_path) {
      cmd.args(["--paths".as_ref(), path.as_os_str()]);
    }
  }
  cmd.arg(entry);
  run_compiler("pyinstaller", &mut cmd)
}

/// The magic string postject looks for inside the node binary to locate the
/// injection point — a constant of Node's SEA implementation, not ours.
const NODE_SEA_FUSE: &str = "NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2";

/// Replaces the `koffi` import in the SEA bundles: a native addon cannot be
/// embedded in a Node Single Executable Application, so the compiled app
/// dlopens the `koffi.node` staged in its bundle resource dir. The bindings
/// only use koffi's native surface (`load`/`func`/`decode`), so the raw
/// addon exports are a drop-in for the package.
const KOFFI_SEA_SHIM: &str = r#"// Generated by `tauri build` for the Node SEA bundle — do not edit.
'use strict'
const path = require('node:path')
const exec = process.execPath
const macos = exec.indexOf('/Contents/MacOS/')
const dir = macos !== -1 ? exec.slice(0, macos) + '/Contents/Resources' : path.dirname(exec)
const mod = { exports: {} }
process.dlopen(mod, path.join(dir, 'koffi.node'))
module.exports = mod.exports
"#;

/// Prints the path of koffi's native addon (`koffi.node`), resolving the
/// `koffi` package the way the bindings would at runtime: from
/// `@tauri-apps/node` when installed, else from the working directory. The
/// addon comes from the `@koromix/koffi-<platform>-<arch>` prebuilt package
/// (koffi's optional dependency) or a local cnoke build inside koffi itself.
const KOFFI_NATIVE_RESOLVE_SCRIPT: &str = r#"
const path = require('node:path')
const fs = require('node:fs')
const { createRequire } = require('node:module')
const anchor = (dir) => createRequire(path.join(dir, '_resolve_.js'))
// koffi's require entry (index.cjs) sits at its package root; resolve it the
// way the bindings would — from inside @tauri-apps/node when installed, else
// from the working directory
let koffiDir = null
for (const via of ['@tauri-apps/node', null]) {
  try {
    let dir = process.cwd()
    if (via) dir = path.dirname(fs.realpathSync(anchor(dir).resolve(via)))
    koffiDir = path.dirname(fs.realpathSync(anchor(dir).resolve('koffi')))
    break
  } catch {}
}
if (!koffiDir) {
  console.error('cannot resolve the koffi package from ' + process.cwd())
  process.exit(1)
}
let file = null
try {
  const main = anchor(koffiDir).resolve(`@koromix/koffi-${process.platform}-${process.arch}`)
  const stack = [path.dirname(fs.realpathSync(main))]
  while (stack.length && !file) {
    const dir = stack.pop()
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory()) stack.push(path.join(dir, entry.name))
      else if (entry.name === 'koffi.node') file = path.join(dir, entry.name)
    }
  }
} catch {}
if (!file) {
  const build = path.join(koffiDir, 'build', 'koffi')
  for (const triplet of fs.existsSync(build) ? fs.readdirSync(build) : []) {
    const candidate = path.join(build, triplet, 'koffi.node')
    if (fs.existsSync(candidate)) { file = candidate; break }
  }
}
if (!file) {
  console.error('koffi native module (koffi.node) not found')
  process.exit(1)
}
process.stdout.write(file)
"#;

/// Compiles the app into a Node.js Single Executable Application.
///
/// Node SEA embeds exactly one CommonJS script (plus opaque assets) into the
/// node binary, while the bindings rely on a separate `worker_threads`
/// module and koffi's native addon — neither can live inside the executable.
/// The pipeline works around both:
/// 1. run the entry in trace mode (`TAURI_SEA_TRACE`) so `launch()` reports
///    which module it would spawn as the worker;
/// 2. bundle the entry and the worker module into two self-contained CJS
///    scripts with esbuild, aliasing `koffi` to a shim that dlopens the
///    `koffi.node` staged as a bundle resource;
/// 3. build the SEA blob with the worker bundle embedded as the `worker.js`
///    asset — `launch()` runs it via `new Worker(source, { eval: true })` —
///    and inject it into a copy of the node executable with postject.
fn compile_node(
  entry: &Path,
  bin_path: &Path,
  tauri_dir: &Path,
  payload: &EmbedPayload,
) -> crate::Result<()> {
  let dist = bin_path.parent().expect("binary path has no parent");
  let work = dist.join(".sea");
  fs::create_dir_all(&work).fs_context("failed to create work directory", work.clone())?;
  let resources_dir = dist.join("resources");

  // SEA (with assets) needs node >= 20.12; check upfront for a clear error
  let output = Command::new("node")
    .args(["-p", "process.execPath + '\\n' + process.versions.node"])
    .output_ok()
    .context("failed to run `node` — is it installed?")?;
  let stdout = String::from_utf8_lossy(&output.stdout);
  let mut lines = stdout.lines();
  let node_exe = lines.next().unwrap_or_default().trim().to_string();
  let node_version = lines.next().unwrap_or_default().trim().to_string();
  let mut version_parts = node_version
    .split('.')
    .map(|p| p.parse::<u32>().unwrap_or(0));
  let (major, minor) = (
    version_parts.next().unwrap_or(0),
    version_parts.next().unwrap_or(0),
  );
  if major < 20 || (major == 20 && minor < 12) {
    bail!(
      "building a self-contained Node.js binary requires Node >= 20.12 (Single Executable Applications with assets); found {node_version}"
    );
  }

  // 1. discover the worker module: run the entry in trace mode — launch()
  //    writes the resolved path and exits before doing any FFI work
  let trace = work.join("entry.json");
  let _ = fs::remove_file(&trace);
  let mut cmd = Command::new("node");
  cmd
    .current_dir(tauri_dir)
    .env("TAURI_SEA_TRACE", &trace)
    .arg(entry);
  run_compiler("node (worker entry trace)", &mut cmd)?;
  let worker_entry = fs::read_to_string(&trace)
    .ok()
    .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
    .and_then(|json| json["entry"].as_str().map(PathBuf::from));
  let Some(worker_entry) = worker_entry else {
    bail!(
      "could not discover the worker module — the app entry {} must call launch() from '@tauri-apps/node'",
      display_path(entry)
    );
  };

  // 2. stage koffi's native addon next to the cdylib and generate the shim
  //    the bundles load it through
  let output = Command::new("node")
    .current_dir(tauri_dir)
    .args(["-e", KOFFI_NATIVE_RESOLVE_SCRIPT])
    .output_ok()
    .context("failed to locate the koffi native module")?;
  let koffi_node = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
  fs::copy(&koffi_node, resources_dir.join("koffi.node"))
    .fs_context("failed to stage koffi native module", koffi_node.clone())?;
  log::info!(action = "Bundling"; "koffi native module from {}", display_path(&koffi_node));
  let shim = work.join("koffi-sea.cjs");
  fs::write(&shim, KOFFI_SEA_SHIM).fs_context("failed to write koffi shim", shim.clone())?;

  // 3. bundle the main-thread entry and the worker module into two
  //    self-contained CJS scripts
  for (bundle_entry, outfile) in [(entry, "main.cjs"), (worker_entry.as_path(), "worker.cjs")] {
    let mut cmd = js_tool_command("esbuild", tauri_dir);
    cmd
      .current_dir(tauri_dir)
      .arg(bundle_entry)
      .args(["--bundle", "--platform=node", "--format=cjs", "--target=node20", "--log-level=warning"])
      .arg(format!("--outfile={}", work.join(outfile).display()))
      .arg(format!("--alias:koffi={}", shim.display()))
      // import.meta.url has no CJS equivalent; the bundles only ever run
      // inside the SEA binary, where the executable path is the module URL
      .arg("--define:import.meta.url=__tauriImportMetaUrl")
      .arg("--banner:js=const __tauriImportMetaUrl = require('node:url').pathToFileURL(process.execPath).href;");
    run_compiler("esbuild", &mut cmd)?;
  }

  // 4. build the SEA blob (paths in sea-config.json resolve from the cwd). The
  //    worker bundle plus the embedded payload (assets/config/capabilities) all
  //    ride along as SEA assets, read back at runtime via `sea.getAsset`.
  let mut assets = serde_json::Map::new();
  assets.insert("worker.js".into(), "worker.cjs".into());
  assets.insert(
    "config.json".into(),
    payload.config.to_string_lossy().into_owned().into(),
  );
  if let Some(app_assets) = &payload.assets {
    assets.insert(
      "app.assets".into(),
      app_assets.to_string_lossy().into_owned().into(),
    );
  }
  if let Some(capabilities) = &payload.capabilities {
    assets.insert(
      "capabilities.json".into(),
      capabilities.to_string_lossy().into_owned().into(),
    );
  }
  fs::write(
    work.join("sea-config.json"),
    serde_json::to_string_pretty(&serde_json::json!({
      "main": "main.cjs",
      "output": "sea-prep.blob",
      "disableExperimentalSEAWarning": true,
      "assets": assets
    }))
    .context("failed to serialize sea-config.json")?,
  )
  .fs_context(
    "failed to write sea-config.json",
    work.join("sea-config.json"),
  )?;
  let mut cmd = Command::new("node");
  cmd
    .current_dir(&work)
    .args(["--experimental-sea-config", "sea-config.json"]);
  run_compiler("node --experimental-sea-config", &mut cmd)?;

  // 5. inject the blob into a copy of the node executable
  let _ = fs::remove_file(bin_path);
  fs::copy(&node_exe, bin_path).fs_context(
    "failed to copy the node executable",
    PathBuf::from(node_exe),
  )?;
  if cfg!(target_os = "macos") {
    let mut cmd = Command::new("codesign");
    cmd.args(["--remove-signature".as_ref(), bin_path.as_os_str()]);
    run_compiler("codesign --remove-signature", &mut cmd)?;
  }
  let mut cmd = js_tool_command("postject", tauri_dir);
  cmd
    .current_dir(&work)
    .args([
      bin_path.as_os_str(),
      "NODE_SEA_BLOB".as_ref(),
      "sea-prep.blob".as_ref(),
    ])
    .args(["--sentinel-fuse", NODE_SEA_FUSE]);
  if cfg!(target_os = "macos") {
    cmd.args(["--macho-segment-name", "NODE_SEA"]);
  }
  run_compiler("postject", &mut cmd)?;
  if cfg!(target_os = "macos") {
    let mut cmd = Command::new("codesign");
    cmd.args(["--sign".as_ref(), "-".as_ref(), bin_path.as_os_str()]);
    run_compiler("codesign --sign", &mut cmd)?;
  }
  Ok(())
}

/// A JS build tool command: the project-local installation
/// (`node_modules/.bin`, walking up from the app directory) when present,
/// otherwise `npx --yes <tool>` (fetched into the npx cache on first use).
fn js_tool_command(tool: &str, tauri_dir: &Path) -> Command {
  let bin = if cfg!(windows) {
    format!("{tool}.cmd")
  } else {
    tool.to_string()
  };
  let mut dir = Some(tauri_dir);
  while let Some(current) = dir {
    let candidate = current.join("node_modules").join(".bin").join(&bin);
    if candidate.is_file() {
      return Command::new(candidate);
    }
    dir = current.parent();
  }
  let mut cmd = Command::new(if cfg!(windows) { "npx.cmd" } else { "npx" });
  cmd.args(["--yes", tool]);
  cmd
}

fn run_compiler(name: &str, cmd: &mut Command) -> crate::Result<()> {
  log::debug!("running `{name}`: {cmd:?}");
  match cmd.piped() {
    Ok(status) if status.success() => Ok(()),
    Ok(status) => bail!("`{}` failed with status {}", name, status),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => bail!(
      "`{name}` command not found — install it to build a self-contained binary for this runner"
    ),
    Err(e) => Err(crate::Error::CommandFailed {
      command: name.into(),
      error: e,
    }),
  }
}

/// Ignore matcher for the dev watcher: builtin ignores, the frontend dist
/// directory (its changes are picked up by the dev server, not a restart)
/// and the project's `.taurignore`.
fn build_ignore_matcher(tauri_dir: &Path, frontend_dist: &Option<FrontendDist>) -> Gitignore {
  let mut builder = GitignoreBuilder::new(tauri_dir);
  for line in [
    "node_modules/",
    "target/",
    "dist/",
    "gen/",
    ".git/",
    "__pycache__/",
    "*.pyc",
    ".DS_Store",
  ] {
    let _ = builder.add_line(None, line);
  }
  if let Some(FrontendDist::Directory(dir)) = frontend_dist {
    if dir.is_relative() {
      let relative = dir.strip_prefix("./").unwrap_or(dir);
      let _ = builder.add_line(None, &format!("/{}/", relative.to_string_lossy()));
    }
  }
  let taurignore = tauri_dir.join(".taurignore");
  if taurignore.exists() {
    let _ = builder.add(taurignore);
  }
  builder.build().expect("failed to build ignore matcher")
}

fn is_ignored(matcher: &Gitignore, path: &Path, is_dir: bool) -> bool {
  path.starts_with(matcher.path())
    && matcher
      .matched_path_or_any_parents(path, is_dir)
      .is_ignore()
}

/// Writes the tauri-ffi assets archive (see [`ASSETS_ARCHIVE_MAGIC`]) for
/// every file inside `dist`, keyed like asset keys (`/index.html`).
fn write_assets_archive(dist: &Path, out: &Path) -> crate::Result<usize> {
  let dist = dunce::canonicalize(dist).fs_context("failed to resolve", dist.to_path_buf())?;

  let mut files: Vec<(String, PathBuf, u64)> = Vec::new();
  collect_assets(&dist, &dist, &mut files)?;
  files.sort();

  let mut index = serde_json::Map::new();
  let mut offset = 0u64;
  for (key, _, len) in &files {
    index.insert(key.clone(), serde_json::json!([offset, len]));
    offset += len;
  }
  let index = serde_json::to_vec(&serde_json::json!({ "files": index }))
    .context("failed to serialize assets index")?;

  let mut writer = std::io::BufWriter::new(
    fs::File::create(out).fs_context("failed to create assets archive", out.to_path_buf())?,
  );
  let write_context = || format!("failed to write assets archive {}", out.display());
  writer
    .write_all(ASSETS_ARCHIVE_MAGIC)
    .with_context(write_context)?;
  writer
    .write_all(&(index.len() as u64).to_le_bytes())
    .with_context(write_context)?;
  writer.write_all(&index).with_context(write_context)?;
  for (_, path, _) in &files {
    let mut file = fs::File::open(path).fs_context("failed to read asset", path.clone())?;
    std::io::copy(&mut file, &mut writer).with_context(write_context)?;
  }
  writer.flush().with_context(write_context)?;

  Ok(files.len())
}

fn collect_assets(
  root: &Path,
  dir: &Path,
  files: &mut Vec<(String, PathBuf, u64)>,
) -> crate::Result<()> {
  for entry in fs::read_dir(dir)
    .fs_context("failed to read assets directory", dir.to_path_buf())?
    .flatten()
  {
    let path = entry.path();
    if path.is_dir() {
      collect_assets(root, &path, files)?;
    } else {
      let len = entry
        .metadata()
        .fs_context("failed to read asset metadata", path.clone())?
        .len();
      let relative = path.strip_prefix(root).expect("asset outside dist root");
      let key = format!(
        "/{}",
        relative
          .components()
          .map(|c| c.as_os_str().to_string_lossy())
          .collect::<Vec<_>>()
          .join("/")
      );
      files.push((key, path, len));
    }
  }
  Ok(())
}
