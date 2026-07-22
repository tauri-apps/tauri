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

    // Marker the bindings' loaders use to recognize this staged resource dir
    // among the several places each packaging format can put it, and to tell it
    // apart from an unrelated `resources/` directory a user's own bundle
    // resources might create next to the binary (see `bundledResourceDir` in the
    // bindings). It rides along into the bundle like every other staged file.
    let marker = resources_dir.join(RESOURCE_MARKER);
    fs::write(&marker, []).fs_context("failed to write the resource marker", marker.clone())?;

    // 1. stage the tauri-ffi cdylib the compiled app loads over FFI (the only
    //    sibling resource — a native library can't be embedded and dlopen'd).
    //    The library is per-runtime (`app > runtime`): `libtauri_<runtime>`.
    let runtime = self.app_settings.runtime;
    let lib_name = cdylib_name(&self.app_settings.target_triple, runtime);
    // Resolved by asking the app's own bindings for the library it loads (see
    // `resolve_cdylib`), so it is the exact `libtauri_<runtime>` the app runs
    // against — no need to inspect the file to tell wry from cef.
    let lib_src = resolve_cdylib(
      &self.app_settings.target_triple,
      runtime,
      dirs.tauri,
      &runner_config,
    )?;
    let lib_dest = resources_dir.join(&lib_name);
    fs::copy(&lib_src, &lib_dest)
      .fs_context("failed to stage tauri-ffi library", lib_src.clone())?;
    log::info!(action = "Bundling"; "tauri-ffi ({runtime}) library from {}", display_path(&lib_src));

    // 1b. a cef cdylib needs the CEF distribution (libcef plus the pak/locale
    //     data it loads) and a subprocess helper alongside it — see
    //     `stage_cef_runtime`. The distribution is downloaded on demand when the
    //     library ships without it (an installed app); the helper always ships
    //     beside the library, so it comes from `lib_dir` regardless.
    if runtime == WebviewRuntime::Cef {
      let lib_dir = lib_src
        .parent()
        .expect("resolved cdylib path has no parent directory");
      let cef_dir = resolve_cef_distribution(&lib_src, &self.app_settings.target_triple)?;
      stage_cef_runtime(
        lib_dir,
        &cef_dir,
        &resources_dir,
        &self.app_settings.target_triple,
      )?;
    }

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
      &self.app_settings.package_settings.product_name,
    )?;

    // 3. compile the self-contained binary, embedding the payload
    let bin_path = self.app_settings.app_binary_path(&options, dirs.tauri)?;
    log::info!(action = "Compiling"; "{} app into {}", runner_label(runner), display_path(&bin_path));
    compile_binary(
      runner,
      &runner_config,
      &entry,
      &bin_path,
      dirs.tauri,
      &payload,
      runtime,
    )?;

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

    // Ensure the CEF runtime is available before spawning (downloading it once
    // when the installed library ships without it), so every restart just reuses
    // the resolved distribution.
    let cef_env = self.cef_dev_env(&options, dirs)?;

    if options.no_watch {
      let (tx, rx) = sync_channel(1);
      let _child = self.spawn_app(config, &options, &cef_env, dirs, move |status, reason| {
        on_exit(status, reason);
        tx.send(()).unwrap();
      })?;
      rx.recv().unwrap();
      Ok(())
    } else {
      self.run_dev_watcher(config, options, &cef_env, on_exit, dirs)
    }
  }
}

impl Bindings {
  fn spawn_app<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &self,
    config: &ConfigMetadata,
    options: &Options,
    cef_env: &HashMap<&'static str, String>,
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

    // A cef app is multi-process: the bindings' library loader preloads the
    // `libcef` staged beside it and points CEF at the sibling `tauri-cef-helper`
    // (via TAURI_CEF_SUBPROCESS_PATH) for its renderer/GPU/utility subprocesses —
    // both resolved from the same `_native/` the runner loads the library from.
    // An installed app ships the library without the ~1.4GB CEF runtime, so the
    // loader can't find `libcef` beside it; `cef_dev_env` downloaded it and points
    // the loader at the shared cache through TAURI_CEF_PATH (empty for wry, or when
    // CEF is already staged beside the library in a repo checkout).
    command.envs(cef_env.iter().map(|(k, v)| (*k, v)));
    // Only macOS is called out, where the CEF framework must live in an .app and
    // dev can't just run the runner (see `crate::cef::macos_dev`).
    if cfg!(target_os = "macos") && self.app_settings.runtime == WebviewRuntime::Cef {
      log::warn!(
        "`tauri dev` with the cef runtime is not yet supported for bindings apps on macOS (the CEF framework must be staged in an .app bundle)"
      );
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
    cef_env: &HashMap<&'static str, String>,
    on_exit: Arc<F>,
    dirs: &Dirs,
  ) -> crate::Result<()> {
    let on_exit_ = on_exit.clone();
    let mut child = self.spawn_app(config, &options, cef_env, dirs, move |status, reason| {
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
      child = self.spawn_app(config, &options, cef_env, dirs, move |status, reason| {
        on_exit_(status, reason)
      })?;
    }
    bail!("File watcher exited unexpectedly")
  }

  /// Ensures the CEF runtime is available for a cef bindings app in dev and
  /// returns the env pointing the loader at it. Empty for a wry app, a
  /// non-linux target (cef bindings are linux-only — see [`stage_cef_runtime`]),
  /// a project with no runner (`spawn_app` reports that), or when CEF is already
  /// staged beside the library in a repo checkout (the loader finds it there).
  ///
  /// Otherwise it downloads the CEF distribution the installed library links but
  /// does not ship (libcef is ~1.4GB) via [`resolve_cef_distribution`] and sets
  /// `TAURI_CEF_PATH` to it, which the loaders preload libcef from — a dev-only
  /// channel a compiled bundle ignores (it loads only its own staged libcef).
  fn cef_dev_env(
    &self,
    options: &Options,
    dirs: &Dirs,
  ) -> crate::Result<HashMap<&'static str, String>> {
    let runtime = self.app_settings.runtime;
    let target = &self.app_settings.target_triple;
    let mut env = HashMap::new();
    if runtime != WebviewRuntime::Cef || !target.contains("linux") {
      return Ok(env);
    }
    let Some(runner) = options.runner.clone() else {
      // No runner to probe the library with; `spawn_app` raises the clear error.
      return Ok(env);
    };
    let lib = resolve_cdylib(target, runtime, dirs.tauri, &runner)?;
    let cef_dir = resolve_cef_distribution(&lib, target)?;
    // Skip the env when the distribution already sits beside the library (a repo
    // checkout): the loader finds libcef there without it.
    if Some(cef_dir.as_path()) != lib.parent() {
      env.insert("TAURI_CEF_PATH", cef_dir.to_string_lossy().into_owned());
    }
    Ok(env)
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
      // `stage_cef_runtime` already put a full CEF distribution in the resource
      // dir, so it doubles as the bundler's `cef_path` — there is no CEF cache
      // directory to resolve by version the way the Rust interface does (a
      // bindings project has no Cargo.lock to read the `cef` version from).
      //
      // Setting this is what routes the AppImage through `sharun_cef`, and only
      // sharun produces a working CEF AppImage: linuxdeploy's AppRun exports an
      // LD_LIBRARY_PATH over the AppDir's bundled system libraries that CEF dies
      // initializing under. The cost is that libcef is then staged twice — once
      // here next to the cdylib that links it, once by the bundler next to the
      // binary. See `stage_cef_runtime` for why deduplicating needs sharun_cef
      // to change.
      cef_path: (config.app.runtime == WebviewRuntime::Cef).then(|| resources_dir.clone()),
      // The `__TAURI_BUNDLE_TYPE` marker the bundler patches is not in the
      // compiled binary (that's the language runtime) but in the embedded
      // `tauri-ffi` cdylib staged as a resource — point the bundler at it.
      bundle_type_binary: Some(resources_dir.join(cdylib_name(&self.target_triple, self.runtime))),
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

/// The CEF distribution files a packaged app needs beside `libcef`, mirroring
/// the list the bundler ships for a Rust cef app (see the `cef_files` in
/// `tauri-bundler`'s `linux/debian.rs`). `locales/` is copied separately.
const CEF_LINUX_FILES: &[&str] = &[
  // required
  "libcef.so",
  "icudtl.dat",
  "v8_context_snapshot.bin",
  // "optional" — but not really, since we want support for all of this
  "chrome_100_percent.pak",
  "chrome_200_percent.pak",
  "resources.pak",
  // ANGLE support
  "libEGL.so",
  "libGLESv2.so",
  // SwANGLE support
  "libvk_swiftshader.so",
  "vk_swiftshader_icd.json",
  "libvulkan.so.1",
  // sandbox
  "chrome-sandbox",
];

/// Locales CEF loads from `locales/` next to its module. Only en-US is shipped,
/// matching what the bundler stages for a Rust cef app.
const CEF_LOCALES: &[&str] = &[
  "en-US.pak",
  "en-US_FEMININE.pak",
  "en-US_MASCULINE.pak",
  "en-US_NEUTER.pak",
];

/// The `libcef` file name for a target. Bindings cef is linux-only for now (see
/// [`stage_cef_runtime`]), but this keeps the platform mapping in one place.
fn cef_library_name(target_triple: &str) -> &'static str {
  if target_triple.contains("windows") {
    "libcef.dll"
  } else if target_triple.contains("apple") {
    "libcef.dylib"
  } else {
    "libcef.so"
  }
}

/// The ELF section a cef tauri-ffi library embeds its linked CEF distribution
/// version in (see `crates/tauri-ffi/src/lib.rs`).
const CEF_VERSION_SECTION: &str = ".tauri_cef_version";

/// Reads the CEF distribution version (e.g. `150.0.10`) a cef tauri-ffi library
/// links against, from the [`CEF_VERSION_SECTION`] ELF section the library
/// build embeds. `None` when the library carries no such section — not a cef
/// library, or a build predating the section — leaving the caller to report a
/// clear error, since the version is what lets it download the matching CEF.
fn read_linked_cef_version(lib_path: &Path) -> Option<String> {
  use object::{Object, ObjectSection};
  let bytes = fs::read(lib_path).ok()?;
  let file = object::File::parse(&*bytes).ok()?;
  let data = file.section_by_name(CEF_VERSION_SECTION)?.data().ok()?;
  // The section is a fixed 32-byte, null-padded UTF-8 buffer.
  let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
  let version = std::str::from_utf8(&data[..end]).ok()?.trim();
  (!version.is_empty()).then(|| version.to_string())
}

/// Resolves the directory holding the CEF distribution (`libcef` plus its
/// pak/locale data) a cef bindings app links, given its resolved tauri-ffi
/// library.
///
/// A Rust cef app's `cargo build` downloads CEF through `cef-dll-sys`; a
/// bindings app has no such build step, and its prebuilt library ships *without*
/// the ~1.4GB CEF runtime it links. So:
/// - in a repo checkout `cef-dll-sys` (via `stage-dev.mjs`) already staged the
///   distribution next to the library — use it in place; otherwise
/// - download the version the library embeds (see [`read_linked_cef_version`])
///   into the shared cache, the same one a Rust cef app fills, via
///   [`ensure_cef_distribution`](crate::interface::rust::ensure_cef_distribution).
fn resolve_cef_distribution(lib_path: &Path, target_triple: &str) -> crate::Result<PathBuf> {
  let lib_dir = lib_path
    .parent()
    .expect("resolved cdylib path has no parent directory");
  if lib_dir.join(cef_library_name(target_triple)).is_file() {
    return Ok(lib_dir.to_path_buf());
  }
  let Some(version) = read_linked_cef_version(lib_path) else {
    bail!(
      "the cef runtime needs the CEF distribution, but no `{}` is staged next to {} and the library carries no `{CEF_VERSION_SECTION}` version to download it by — reinstall the app's dependencies (or rebuild the library in a repo checkout) so it provides a cef library",
      cef_library_name(target_triple),
      display_path(lib_path),
    );
  };
  crate::interface::rust::ensure_cef_distribution(target_triple, &version)
}

/// Stages the CEF distribution and the subprocess helper into a bindings app's
/// resource dir. The distribution files (libcef + pak/locale data) come from
/// `cef_dir` — either staged next to the library in a repo checkout or the
/// downloaded shared cache (see [`resolve_cef_distribution`]) — while the
/// subprocess helper comes from `lib_dir`, where it ships beside the library.
///
/// Unlike a Rust cef app — whose *executable* links libcef, so the bundler ships
/// CEF next to the binary — a bindings app's executable is a stock
/// node/deno/python interpreter and it is the *cdylib* that links libcef. So CEF
/// belongs next to the cdylib instead: the cdylib carries a `$ORIGIN` RUNPATH
/// (see `tauri-ffi`'s build script) to resolve `libcef` there, and CEF finds its
/// own pak/locale data by looking next to its module. That makes the resource
/// dir self-contained, and is what deb/rpm ship.
///
/// The resource dir is *also* handed to the bundler as `cef_path`, so libcef
/// ends up in a bundle twice: once here, once staged next to the binary. That is
/// deliberate for now — `cef_path` is what routes the AppImage through
/// `sharun_cef` rather than linuxdeploy, whose AppRun CEF cannot start under.
/// Dropping the second copy needs `sharun_cef` to change, and neither obvious
/// route works today:
/// - handing quick-sharun this directory as its lib dir gets it deployed, but
///   quick-sharun `ldd`s everything it is given, and the cdylib pulls a system
///   ICU into `shared/lib` that Chromium then loads over its own ("Invalid file
///   descriptor to ICU data received");
/// - not handing it over leaves it out of the AppDir entirely — sharun builds
///   the AppDir from what it is given, it does not copy the deb data tree.
///
/// Fixing it likely means splitting quick-sharun's deploy and pack steps so the
/// resource dir can be copied in between, untouched.
fn stage_cef_runtime(
  lib_dir: &Path,
  cef_dir: &Path,
  resources_dir: &Path,
  target_triple: &str,
) -> crate::Result<()> {
  if !target_triple.contains("linux") {
    // Windows needs its own file list, and macOS needs the CEF framework staged
    // into an .app (plus a helper .app per subprocess type) — neither is wired
    // up yet, matching the `dev` restriction in `spawn_app`.
    bail!(
      "the cef runtime is not yet supported for bindings apps on this target ({target_triple}) — only linux is"
    );
  }

  for name in CEF_LINUX_FILES {
    let src = cef_dir.join(name);
    if !src.is_file() {
      bail!(
        "the cef runtime needs `{name}` in the CEF distribution, but {} does not exist",
        display_path(&src)
      );
    }
    let dest = resources_dir.join(name);
    // libcef is ~1.4GB unstripped and ~250MB stripped, and it ships in every
    // bundle — strip it straight to the destination (one pass, ~250MB written)
    // like the bundler does for a Rust cef app. Best effort: fall back to a
    // plain copy when `strip` is missing (a missing strip only costs size).
    let stripped = name.ends_with(".so")
      && Command::new("strip")
        .arg("-o")
        .arg(&dest)
        .arg(&src)
        .output_ok()
        .is_ok();
    if !stripped {
      fs::copy(&src, &dest).fs_context("failed to stage CEF file", src.clone())?;
    }
  }

  let locales_dest = resources_dir.join("locales");
  fs::create_dir_all(&locales_dest).fs_context(
    "failed to create CEF locales directory",
    locales_dest.clone(),
  )?;
  for name in CEF_LOCALES {
    let src = cef_dir.join("locales").join(name);
    fs::copy(&src, locales_dest.join(name)).fs_context("failed to stage CEF locale", src)?;
  }
  log::info!(action = "Bundling"; "CEF distribution from {}", display_path(cef_dir));

  // CEF re-executes a helper for its renderer/GPU/utility subprocesses; the
  // bindings point it at ours through TAURI_CEF_SUBPROCESS_PATH. It ships beside
  // the library (in the platform package, and staged into `_native/` for dev by
  // `stage-dev.mjs`), so it is already here in `lib_dir` — just copy it in.
  let helper_name = if target_triple.contains("windows") {
    "tauri-cef-helper.exe"
  } else {
    "tauri-cef-helper"
  };
  let helper = lib_dir.join(helper_name);
  if !helper.is_file() {
    bail!(
      "the cef runtime needs the `{helper_name}` subprocess helper next to the tauri-ffi library, but {} does not exist — it ships in the cef platform package (and `bindings/scripts/stage-dev.mjs` stages it for a repo checkout)",
      display_path(&helper)
    );
  }
  fs::copy(&helper, resources_dir.join(helper_name))
    .fs_context("failed to stage the CEF subprocess helper", helper.clone())?;
  log::info!(action = "Bundling"; "CEF subprocess helper from {}", display_path(&helper));

  Ok(())
}

/// Locates the prebuilt tauri-ffi cdylib to stage, erroring when there is none.
/// See [`locate_cdylib`] for how it is resolved.
fn resolve_cdylib(
  target_triple: &str,
  runtime: WebviewRuntime,
  tauri_dir: &Path,
  runner: &tauri_utils::config::RunnerConfig,
) -> crate::Result<PathBuf> {
  match locate_cdylib(target_triple, runtime, tauri_dir, runner)? {
    Some(path) => Ok(path),
    None => bail!(
      "could not find a `{runtime}` tauri-ffi library for {target_triple}.\n  The library comes from your bindings package, so it is resolved by asking the runner for the very library your app loads — check that the project's dependencies are installed and provide a `{runtime}` library for this target.\n  In a repo checkout, stage it with `node bindings/scripts/stage-dev.mjs --lang <node|deno|python> --runtime {runtime}`; otherwise point `TAURI_FFI_LIB` at a library path."
    ),
  }
}

/// Locates the tauri-ffi cdylib to stage, in precedence order:
///
/// 1. `TAURI_FFI_LIB` — an explicit override, always wins.
/// 2. The app's own bindings package, via [`discover_cdylib_from_bindings`] — the
///    library the app actually loads, resolved by running the package's own
///    lookup in the configured runner. This is deliberately the *only* discovery
///    path: it is identical in a repo checkout (where `stage-dev.mjs` populates
///    the package's `_native/`) and a published app (where the platform package
///    carries the library), so `tauri build` can never stage a different
///    library — or a different runtime — than the app runs against, and the CLI
///    never has to inspect a file to tell wry from cef.
///
/// `None` means neither source held a library; [`resolve_cdylib`] turns that
/// into an error.
fn locate_cdylib(
  target_triple: &str,
  runtime: WebviewRuntime,
  tauri_dir: &Path,
  runner: &tauri_utils::config::RunnerConfig,
) -> crate::Result<Option<PathBuf>> {
  if let Some(env) = std::env::var_os("TAURI_FFI_LIB") {
    let path = PathBuf::from(env);
    if path.is_file() {
      return Ok(Some(path));
    }
    bail!(
      "TAURI_FFI_LIB points at {}, which does not exist",
      path.display()
    );
  }

  Ok(discover_cdylib_from_bindings(
    runner,
    runtime,
    target_triple,
    tauri_dir,
  ))
}

/// Asks the app's own bindings package for the `tauri_<runtime>` library it
/// would load, by running the package's own library resolution — the very code
/// the app runs at startup — in the configured runner.
///
/// The library ships *with the bindings* (the
/// `@tauri-apps/node-<runtime>-<platform>-<arch>` npm package, the `tauri_ffi`
/// wheel's `_native/`, Deno's download cache), so the app has the exact copy it
/// loads as soon as its dependencies are installed. Asking it — rather than
/// fetching a library of our own — is what keeps `tauri build` from staging a
/// *differently versioned* library than the one the app runs against, and is why
/// the CLI needs no downloader: the bindings are the only thing that has to know
/// how to obtain a library.
///
/// Skipped when cross-compiling: the runner only ever answers for its own host.
/// Best-effort otherwise — a missing package, a runner we have no script for, or
/// a package too old to export the entry point all yield `None`, with the
/// runner's own diagnosis logged for the error the caller then reports.
fn discover_cdylib_from_bindings(
  runner: &tauri_utils::config::RunnerConfig,
  runtime: WebviewRuntime,
  target_triple: &str,
  tauri_dir: &Path,
) -> Option<PathBuf> {
  if tauri_utils::platform::target_triple().ok()? != target_triple {
    return None;
  }

  let kind = runtime.as_str();
  // Each script prints the resolved path and nothing else, and exits non-zero
  // when the package cannot resolve a library. The specifiers resolve from the
  // project (node_modules / the active virtualenv / the deno import map), which
  // is why these run in the runner's own working directory.
  let runner_kind = Runner::detect(runner.cmd()).ok()?;
  let args = match runner_kind {
    Runner::Node => vec![
      "-e".to_string(),
      node_probe_expr(runtime, &format!("m.libraryPath('{kind}')")),
    ],
    Runner::Deno => vec![
      "eval".to_string(),
      // `ensureLibrary` rather than `libraryPath`: the npm and PyPI packages
      // *ship* the library, so installing the project's dependencies is enough,
      // but the deno package installs it on first use from its npm platform
      // package. Probing with `libraryPath` would make a `tauri build` that never
      // ran `launch()` — i.e. every CI job — fail on a library the app would have
      // installed for itself. `deno eval` runs with all permissions, so the npm
      // install it triggers needs no `--allow-*` flags (which `deno eval`
      // rejects). `console.log` writes the whole path (no partial `stdout.write`).
      format!(
        "const m = await import('@tauri-apps/deno/ffi'); console.log(await m.ensureLibrary('{kind}'))"
      ),
    ],
    Runner::Python => vec![
      "-c".to_string(),
      format!("import sys, tauri_ffi; sys.stdout.write(str(tauri_ffi.library_path('{kind}')))"),
    ],
  };

  let out = probe_bindings_runner(runner, tauri_dir, runner_kind, args)?;
  let path = PathBuf::from(out);
  path.is_file().then_some(path)
}

/// The `@tauri-apps/node` base-package specifiers to probe for `runtime`, most
/// specific first. A non-default runtime republishes the base package under a
/// suffixed name (`@tauri-apps/node-cef`) — which is what such an app depends on
/// — while a repo checkout / dev always uses the plain `@tauri-apps/node`, so
/// trying the suffixed one then falling back covers both.
fn node_ffi_specifiers(runtime: WebviewRuntime) -> Vec<String> {
  let mut specs = Vec::new();
  if runtime != WebviewRuntime::Wry {
    specs.push(format!("@tauri-apps/node-{}/ffi", runtime.as_str()));
  }
  specs.push("@tauri-apps/node/ffi".to_string());
  specs
}

/// A `node -e` expression that imports the first resolvable `@tauri-apps/node`
/// base package (see [`node_ffi_specifiers`]) and writes `String(<call>)` — an
/// expression using the imported module `m` — to stdout, exiting non-zero when
/// none resolve.
fn node_probe_expr(runtime: WebviewRuntime, call: &str) -> String {
  let specs = node_ffi_specifiers(runtime)
    .iter()
    .map(|s| format!("'{s}'"))
    .collect::<Vec<_>>()
    .join(",");
  format!(
    "(async()=>{{for(const s of [{specs}]){{try{{const m=await import(s);process.stdout.write(String({call}));return}}catch{{}}}}process.exit(1)}})()"
  )
}

/// Runs `args` in the app's configured runner and returns its trimmed stdout on
/// success, or `None` when the runner exits non-zero (its stderr is logged so a
/// failing `tauri build` names the cause rather than only "not found").
///
/// Honors `build > runner`'s command and working directory, not the canonical
/// interpreter name: the runner may be `python3`, a venv/nvm path, or configured
/// with a `cwd` subdirectory, and the bindings package the app loads only
/// resolves from *that* interpreter and directory.
fn probe_bindings_runner(
  runner: &tauri_utils::config::RunnerConfig,
  tauri_dir: &Path,
  runner_kind: Runner,
  args: Vec<String>,
) -> Option<String> {
  let mut command = Command::new(runner.cmd());
  command.args(args);
  command.current_dir(
    runner
      .cwd()
      .map(|cwd| tauri_dir.join(cwd))
      .unwrap_or_else(|| tauri_dir.to_path_buf()),
  );

  let output = command.output().ok()?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if !stderr.is_empty() {
      log::warn!(
        "the {} bindings could not answer a `tauri build` probe:\n{stderr}",
        runner_label(runner_kind)
      );
    }
    return None;
  }
  Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
  product_name: &str,
) -> crate::Result<EmbedPayload> {
  let _ = fs::remove_dir_all(embed_dir);
  fs::create_dir_all(embed_dir)
    .fs_context("failed to create embed directory", embed_dir.to_path_buf())?;

  let assets = pack_assets(frontend_dist, tauri_dir, embed_dir)?;
  let config = write_embed_config(tauri_dir, options, embed_dir, assets.is_some(), product_name)?;
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
  product_name: &str,
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
  // The Linux loaders derive their resource dir from `productName`
  // (`/usr/lib/<productName>`), the same name the bundler stages resources
  // under. When the config omits it, both fall back to the crate's resolved
  // name (the tauri dir's folder name) — but only the bundler knows that, so
  // bake it into the embedded config too, or a deb/rpm install resolves
  // `/usr/bin` and never finds its library.
  if value
    .get("productName")
    .and_then(|v| v.as_str())
    .filter(|s| !s.is_empty())
    .is_none()
  {
    value["productName"] = serde_json::Value::String(product_name.to_string());
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
  runner_config: &tauri_utils::config::RunnerConfig,
  entry: &Path,
  bin_path: &Path,
  tauri_dir: &Path,
  payload: &EmbedPayload,
  runtime: WebviewRuntime,
) -> crate::Result<()> {
  match runner {
    Runner::Deno => compile_deno(entry, bin_path, tauri_dir, payload),
    Runner::Python => compile_python(entry, bin_path, tauri_dir, payload),
    Runner::Node => compile_node(runner_config, entry, bin_path, tauri_dir, payload, runtime),
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

/// The Node-API addon the bindings run the app's event loop through.
const RUN_ADDON_NAME: &str = "tauri_node.node";

/// Empty marker file staged into the resource dir so the bindings' loaders can
/// recognize it among the candidate locations each packaging format uses (and
/// not mistake a user's own `resources/` directory for it). Kept in sync with
/// the loaders (`bindings/*/…`) and the SEA shim below.
const RESOURCE_MARKER: &str = ".tauri-resources";

/// Locates the run-loop addon (`tauri_node.node`) to stage, by asking the app's
/// own bindings package where it would load it from — the same
/// `@tauri-apps/node` lookup [`discover_cdylib_from_bindings`] uses for the
/// library, so the app ships the addon it was developed against.
///
/// The addon exists because koffi runs calls on a private stack that WebKit
/// aborts on; see `bindings/node/native/tauri_node.c`. A Node bindings app
/// cannot run a wry webview without it, so this is a hard error rather than a
/// best-effort lookup.
fn run_addon_path(
  runner: &tauri_utils::config::RunnerConfig,
  tauri_dir: &Path,
  runtime: WebviewRuntime,
) -> crate::Result<PathBuf> {
  let kind = runtime.as_str();
  // Same probe as the cdylib lookup: honor the configured runner command/cwd so
  // the addon is resolved from the very `@tauri-apps/node` the app loads, not a
  // stray `node` on the CLI's PATH (see `discover_cdylib_from_bindings`).
  let expr = node_probe_expr(runtime, &format!("(m.runAddonPath('{kind}') ?? '')"));
  let path = probe_bindings_runner(runner, tauri_dir, Runner::Node, vec!["-e".to_string(), expr])
    .filter(|s| !s.is_empty())
    .map(PathBuf::from);
  match path {
    Some(path) if path.is_file() => Ok(path),
    _ => bail!(
      "the `{RUN_ADDON_NAME}` run-loop addon was not found — the app cannot drive a webview without it.\n  In this repo, build it with `node bindings/node/native/build.mjs`; otherwise reinstall so the `@tauri-apps/node-{kind}-<platform>-<arch>` package is present."
    ),
  }
}

/// Replaces the `koffi` import in the SEA bundles: a native addon cannot be
/// embedded in a Node Single Executable Application, so the compiled app
/// dlopens the `koffi.node` staged in its bundle resource dir. The bindings
/// only use koffi's native surface (`load`/`func`/`decode`), so the raw
/// addon exports are a drop-in for the package.
///
/// It resolves the resource dir itself — it cannot import the bindings'
/// `bundledResourceDir()` (config.js), being the very module they load koffi
/// through — so the two must stay in sync; both pick the candidate holding the
/// `.tauri-resources` marker (see `RESOURCE_MARKER`).
const KOFFI_SEA_SHIM: &str = r#"// Generated by `tauri build` for the Node SEA bundle — do not edit.
// Keep in sync with bundledResourceDir() in @tauri-apps/node's config.js.
'use strict'
const fs = require('node:fs')
const path = require('node:path')

function resourceDir() {
  const exec = process.execPath
  const candidates = []
  const macos = exec.indexOf('/Contents/MacOS/')
  if (macos !== -1) candidates.push(exec.slice(0, macos) + '/Contents/Resources')
  const dir = path.dirname(exec)
  candidates.push(path.join(dir, 'resources')) // unpackaged `tauri build` output
  candidates.push(dir) // installers that stage flat next to the exe (Windows)
  if (process.platform === 'linux') {
    let name = null
    try {
      name = JSON.parse(require('node:sea').getAsset('config.json', 'utf8')).productName
    } catch {}
    if (name) {
      candidates.push(path.resolve(dir, '..', 'lib', name)) // deb/rpm, AppImage AppDir
      candidates.push(path.resolve(dir, '..', '..', 'lib', name)) // cef deb/rpm: binary in /usr/share/<name>/
      if (process.env.APPDIR) candidates.push(path.join(process.env.APPDIR, 'usr', 'lib', name))
      candidates.push(path.join('/usr', 'lib', name))
    }
  }
  for (const candidate of candidates) {
    if (fs.existsSync(path.join(candidate, '.tauri-resources'))) return candidate
  }
  return macos !== -1 ? candidates[0] : dir // best guess: a flat layout with no marker
}

const mod = { exports: {} }
process.dlopen(mod, path.join(resourceDir(), 'koffi.node'))
module.exports = mod.exports
"#;

/// Prints the path of koffi's native addon (`koffi.node`) for the host, to be
/// staged next to the compiled binary.
///
/// It loads `koffi` — resolved the way the bindings would at runtime: from
/// `@tauri-apps/node` when installed, else from the working directory — and
/// reports the addon koffi itself ended up mapping. koffi picks between several
/// builds shipped for one platform (`@koromix/koffi-<platform>-<arch>` carries
/// both a glibc and a musl `x64` addon, and a local cnoke build may sit in
/// `koffi/build/koffi/<triplet>`), so letting it choose and observing the result
/// is what keeps this in step with its selection rules rather than guessing at
/// them — picking by directory order stages the musl addon on a glibc host,
/// which then fails to `dlopen` in the built app.
///
/// This resolves for the *host*: cross-compiling a Node bindings app to another
/// platform is not supported (the SEA is built from the host `node` too).
const KOFFI_NATIVE_RESOLVE_SCRIPT: &str = r#"
const path = require('node:path')
const fs = require('node:fs')
const { createRequire } = require('node:module')
const anchor = (dir) => createRequire(path.join(dir, '_resolve_.js'))
// koffi's require entry (index.cjs) sits at its package root; resolve it the
// way the bindings would — from inside @tauri-apps/node when installed, else
// from the working directory
let koffiEntry = null
for (const via of ['@tauri-apps/node', null]) {
  try {
    let dir = process.cwd()
    if (via) dir = path.dirname(fs.realpathSync(anchor(dir).resolve(via)))
    koffiEntry = anchor(dir).resolve('koffi')
    break
  } catch {}
}
if (!koffiEntry) {
  console.error('cannot resolve the koffi package from ' + process.cwd())
  process.exit(1)
}
require(koffiEntry)
// the diagnostic report lists every shared object mapped into this process —
// after the require above, exactly one of them is koffi's addon
const file = process.report
  .getReport()
  .sharedObjects.find((entry) => /[\\/]koffi\.node$/.test(entry))
if (!file) {
  console.error('koffi native module (koffi.node) not found')
  process.exit(1)
}
process.stdout.write(fs.realpathSync(file))
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
  runner_config: &tauri_utils::config::RunnerConfig,
  entry: &Path,
  bin_path: &Path,
  tauri_dir: &Path,
  payload: &EmbedPayload,
  runtime: WebviewRuntime,
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

  // 2b. stage the run-loop addon beside it. The compiled app loads it by path
  //     from its resource dir (`runAddonPath`), the same way it loads the
  //     cdylib — a native addon can no more live inside the SEA than koffi's can.
  let addon = run_addon_path(runner_config, tauri_dir, runtime)?;
  fs::copy(&addon, resources_dir.join(RUN_ADDON_NAME))
    .fs_context("failed to stage the tauri-ffi run-loop addon", addon.clone())?;
  log::info!(action = "Bundling"; "run-loop addon from {}", display_path(&addon));

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

