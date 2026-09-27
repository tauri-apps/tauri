// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Build-time helpers for Tauri applications.
//!
//! Every Tauri application must run [`build()`] (or [`try_build()`]) from its `build.rs`,
//! on every target platform. It sets up everything the [`tauri`] crate expects to find at
//! compile time and at runtime:
//!
//! - emits `cargo:rerun-if-changed` instructions for the Tauri configuration files;
//! - defines the `dev`, `desktop` and `mobile` [cfg aliases] used across the Tauri crates;
//! - parses the permissions of the application and of its plugins, resolves the capabilities
//!   (from the `capabilities` directory and from `app > security > capabilities` in the
//!   configuration file) and validates them, writing the resulting Access Control List to the
//!   `OUT_DIR` so [`tauri::generate_context!`] can embed it;
//!   it also writes the capability JSON schemas to `gen/schemas`;
//! - copies the configured `bundle > externalBin` sidecars and `bundle > resources` next to
//!   the compiled binary so they are available when running the app with `cargo run`;
//! - on macOS and iOS, sets the deployment target from the configuration and links the
//!   configured frameworks;
//! - on Windows, compiles a resource file with the application icon, version information and
//!   the [application manifest], and optionally statically links the Visual C++ runtime
//!   (see [`WindowsAttributes`]);
//! - on Android, generates the Gradle files of the mobile project and updates the Android
//!   manifest with the configured file associations;
//! - optionally runs the context code generation at build time instead of at macro expansion
//!   time (see `Attributes::codegen`, requires the `codegen` Cargo feature).
//!
//! # Examples
//!
//! The default `build.rs` of a Tauri application:
//!
//! ```rust,ignore
//! // build.rs
//! fn main() {
//!   tauri_build::build()
//! }
//! ```
//!
//! Customizing the build with [`Attributes`]:
//!
//! ```rust,ignore
//! // build.rs
//! fn main() {
//!   let attributes = tauri_build::Attributes::new()
//!     // the app commands that get a `allow-$command`/`deny-$command` permission generated
//!     .app_manifest(tauri_build::AppManifest::new().commands(&["my_command"]))
//!     // a plugin that lives in the app crate instead of its own crate
//!     .plugin(
//!       "my-plugin",
//!       tauri_build::InlinedPlugin::new().commands(&["do_something"]),
//!     )
//!     .windows_attributes(
//!       tauri_build::WindowsAttributes::new().window_icon_path("icons/icon.ico"),
//!     );
//!   tauri_build::try_build(attributes).expect("failed to run tauri-build");
//! }
//! ```
//!
//! See [`Attributes`] for the complete list of options: [`Attributes::config_path`],
//! [`Attributes::capabilities_path_pattern`], [`Attributes::plugin`] / [`InlinedPlugin`],
//! [`Attributes::app_manifest`] / [`AppManifest`], [`Attributes::windows_attributes`] /
//! [`WindowsAttributes`] and `Attributes::codegen` / `CodegenContext` (`codegen` feature).
//!
//! # Environment variables
//!
//! In addition to the [environment variables set by cargo] for build scripts, the following
//! variables are read:
//!
//! - `TAURI_CONFIG`: a JSON string that is merged into the parsed configuration file.
//!   Set by the Tauri CLI when the configuration is changed from the command line
//!   (e.g. `tauri build --config`). The build script reruns when it changes.
//! - `TAURI_ANDROID_PROJECT_PATH`: path to the Android Studio project of the application,
//!   set by the Tauri CLI on Android builds. When set, the Gradle files of the project are
//!   regenerated and the Android manifest is updated with the configured file associations.
//!   (the `tauri-plugin` crate reads the matching `TAURI_IOS_PROJECT_PATH` for iOS projects).
//! - `STATIC_VCRUNTIME`: **deprecated**, use `build > windows > staticVCRuntime` in the Tauri
//!   configuration or [`WindowsAttributes::static_vc_runtime`] instead. Any value other than
//!   `false` statically links the Visual C++ runtime.
//!
//! [`tauri`]: https://docs.rs/tauri/latest/tauri/
//! [`tauri::generate_context!`]: https://docs.rs/tauri/latest/tauri/macro.generate_context.html
//! [cfg aliases]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-cfgkeyvalue
//! [application manifest]: https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests
//! [environment variables set by cargo]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

use anyhow::Context;
pub use anyhow::Result;
use cargo_toml::Manifest;

use tauri_utils::{
  config::{BundleResources, Config, WebviewInstallMode},
  resources::{ResourcePaths, external_binaries},
};

use std::{
  collections::HashMap,
  env,
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
};

mod acl;
#[cfg(feature = "codegen")]
mod codegen;
mod manifest;
mod mobile;
mod static_vcruntime;

#[cfg(feature = "codegen")]
#[cfg_attr(docsrs, doc(cfg(feature = "codegen")))]
pub use codegen::context::CodegenContext;

pub use acl::{AppManifest, DefaultPermissionRule, InlinedPlugin};

fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let from = from.as_ref();
  let to = to.as_ref();
  if !from.exists() {
    return Err(anyhow::anyhow!("{:?} does not exist", from));
  }
  if !from.is_file() {
    return Err(anyhow::anyhow!("{:?} is not a file", from));
  }
  let dest_dir = to.parent().expect("No data in parent");
  fs::create_dir_all(dest_dir)?;
  fs::copy(from, to)?;
  Ok(())
}

fn copy_binaries(
  binaries: ResourcePaths,
  target_triple: &str,
  path: &Path,
  package_name: Option<&str>,
) -> Result<()> {
  for src in binaries {
    let src = src?;
    println!("cargo:rerun-if-changed={}", src.display());
    let file_name = src
      .file_name()
      .expect("failed to extract external binary filename")
      .to_string_lossy()
      .replace(&format!("-{target_triple}"), "");

    if package_name == Some(&file_name) {
      return Err(anyhow::anyhow!(
        "Cannot define a sidecar with the same name as the Cargo package name `{}`. Please change the sidecar name in the filesystem and the Tauri configuration.",
        file_name
      ));
    }

    let dest = path.join(file_name);
    if dest.exists() {
      fs::remove_file(&dest).unwrap();
    }
    copy_file(&src, &dest)?;
  }
  Ok(())
}

/// Copies resources to a path.
fn copy_resources(resources: ResourcePaths<'_>, path: &Path) -> Result<()> {
  let path = path.canonicalize()?;
  let mut resources = resources.iter();
  for resource in resources.by_ref() {
    let resource = resource?;

    // avoid copying the resource if target is the same as source
    let src = resource.path().canonicalize()?;
    let target = path.join(resource.target());
    if src != target {
      copy_file(src, target)?;
    }
  }

  for path in resources.rerun_if_changed() {
    println!("cargo:rerun-if-changed={}", path.display());
  }

  Ok(())
}

#[cfg(unix)]
fn symlink_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
  std::os::unix::fs::symlink(src, dst)
}

/// Makes a symbolic link to a directory.
#[cfg(windows)]
fn symlink_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
  std::os::windows::fs::symlink_dir(src, dst)
}

/// Makes a symbolic link to a file.
#[cfg(unix)]
fn symlink_file(src: &Path, dst: &Path) -> std::io::Result<()> {
  std::os::unix::fs::symlink(src, dst)
}

/// Makes a symbolic link to a file.
#[cfg(windows)]
fn symlink_file(src: &Path, dst: &Path) -> std::io::Result<()> {
  std::os::windows::fs::symlink_file(src, dst)
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
  for entry in walkdir::WalkDir::new(from) {
    let entry = entry?;
    debug_assert!(entry.path().starts_with(from));
    let rel_path = entry.path().strip_prefix(from)?;
    let dest_path = to.join(rel_path);
    if entry.file_type().is_symlink() {
      let target = fs::read_link(entry.path())?;
      if entry.path().is_dir() {
        symlink_dir(&target, &dest_path)?;
      } else {
        symlink_file(&target, &dest_path)?;
      }
    } else if entry.file_type().is_dir() {
      fs::create_dir(dest_path)?;
    } else {
      fs::copy(entry.path(), dest_path)?;
    }
  }
  Ok(())
}

// Copies the framework under `{src_dir}/{framework}.framework` to `{dest_dir}/{framework}.framework`.
fn copy_framework_from(src_dir: &Path, framework: &str, dest_dir: &Path) -> Result<bool> {
  let src_name = format!("{framework}.framework");
  let src_path = src_dir.join(&src_name);
  if src_path.exists() {
    copy_dir(&src_path, &dest_dir.join(&src_name))?;
    Ok(true)
  } else {
    Ok(false)
  }
}

// Copies the macOS application bundle frameworks to the target folder
fn copy_frameworks(dest_dir: &Path, frameworks: &[String]) -> Result<()> {
  fs::create_dir_all(dest_dir)
    .with_context(|| format!("Failed to create frameworks output directory at {dest_dir:?}"))?;
  for framework in frameworks.iter() {
    if framework.ends_with(".framework") {
      let src_path = Path::new(framework);
      let src_name = src_path
        .file_name()
        .expect("Couldn't get framework filename");
      let dest_path = dest_dir.join(src_name);
      copy_dir(src_path, &dest_path)?;
      continue;
    } else if framework.ends_with(".dylib") {
      let src_path = Path::new(framework);
      if !src_path.exists() {
        return Err(anyhow::anyhow!("Library not found: {}", framework));
      }
      let src_name = src_path.file_name().expect("Couldn't get library filename");
      let dest_path = dest_dir.join(src_name);
      copy_file(src_path, &dest_path)?;
      continue;
    } else if framework.contains('/') {
      return Err(anyhow::anyhow!(
        "Framework path should have .framework extension: {}",
        framework
      ));
    }
    if let Some(home_dir) = dirs::home_dir() {
      if copy_framework_from(&home_dir.join("Library/Frameworks/"), framework, dest_dir)? {
        continue;
      }
    }
    if copy_framework_from("/Library/Frameworks/".as_ref(), framework, dest_dir)?
      || copy_framework_from("/Network/Library/Frameworks/".as_ref(), framework, dest_dir)?
    {
      continue;
    }
  }
  Ok(())
}

// TODO: far from ideal, but there's no other way to get the target dir, see <https://github.com/rust-lang/cargo/issues/5457>
// resolves the target dir from `OUT_DIR`, which is `<target dir>/build/<pkg>-<hash>/out` on stable
// and `<target dir>/build/<pkg>/<hash>/out` on recent nightlies, so we walk up to the `build` dir
// and take its parent instead of assuming a fixed depth.
fn target_dir_from_out_dir(out_dir: &Path) -> Option<&Path> {
  out_dir
    .ancestors()
    .find(|path| path.file_name() == Some(OsStr::new("build")))
    .and_then(|build_dir| build_dir.parent())
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn cfg_alias(alias: &str, has_feature: bool) {
  println!("cargo:rustc-check-cfg=cfg({alias})");
  if has_feature {
    println!("cargo:rustc-cfg={alias}");
  }
}

/// Attributes used on Windows.
#[allow(dead_code)]
#[derive(Debug)]
pub struct WindowsAttributes {
  window_icon_path: Option<PathBuf>,
  /// Whether to statically link the Visual C++ runtime into the application binary on Windows MSVC targets
  static_vc_runtime: Option<bool>,
  /// A string containing an [application manifest] to be included with the application on Windows.
  ///
  /// Defaults to:
  /// ```text
  #[doc = include_str!("windows-app-manifest.xml")]
  /// ```
  ///
  /// ## Warning
  ///
  /// if you are using tauri's dialog APIs, you need to specify a dependency on Common Control v6 by adding the following to your custom manifest:
  /// ```text
  ///  <dependency>
  ///    <dependentAssembly>
  ///      <assemblyIdentity
  ///        type="win32"
  ///        name="Microsoft.Windows.Common-Controls"
  ///        version="6.0.0.0"
  ///        processorArchitecture="*"
  ///        publicKeyToken="6595b64144ccf1df"
  ///        language="*"
  ///      />
  ///    </dependentAssembly>
  ///  </dependency>
  /// ```
  ///
  /// [application manifest]: https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests
  app_manifest: Option<String>,
  /// A series of strings containing additional .rc content to be appended to the generated resource file on Windows.
  append_rc_content: Vec<String>,
}

impl Default for WindowsAttributes {
  fn default() -> Self {
    Self::new()
  }
}

impl WindowsAttributes {
  /// Creates the default attribute set.
  pub fn new() -> Self {
    Self {
      static_vc_runtime: None,
      app_manifest: Some(include_str!("windows-app-manifest.xml").into()),
      window_icon_path: None,
      append_rc_content: Vec::new(),
    }
  }

  /// Creates the default attribute set without the default app manifest.
  #[must_use]
  pub fn new_without_app_manifest() -> Self {
    Self {
      app_manifest: None,
      window_icon_path: None,
      static_vc_runtime: None,
      append_rc_content: Vec::new(),
    }
  }

  /// Sets the icon to use as the application icon and default window icon.
  /// It must be in `ico` format.
  ///
  /// If not set, we will search for a `.ico` from the `bundle > icon` in your tauri config file, then `icons/icon.ico`.
  #[must_use]
  pub fn window_icon_path<P: AsRef<Path>>(mut self, window_icon_path: P) -> Self {
    self
      .window_icon_path
      .replace(window_icon_path.as_ref().into());
    self
  }

  /// Sets whether to statically link the Visual C++ runtime into the application binary on Windows MSVC targets.
  ///
  /// If unset, this is read from `build > windows > staticVCRuntime` in the Tauri configuration.
  #[must_use]
  pub fn static_vc_runtime(mut self, static_vc_runtime: bool) -> Self {
    self.static_vc_runtime.replace(static_vc_runtime);
    self
  }

  /// Sets the [application manifest] to be included with the application on Windows.
  ///
  /// Defaults to:
  /// ```text
  #[doc = include_str!("windows-app-manifest.xml")]
  /// ```
  ///
  /// ## Warning
  ///
  /// if you are using tauri's dialog APIs, you need to specify a dependency on Common Control v6 by adding the following to your custom manifest:
  /// ```text
  ///  <dependency>
  ///    <dependentAssembly>
  ///      <assemblyIdentity
  ///        type="win32"
  ///        name="Microsoft.Windows.Common-Controls"
  ///        version="6.0.0.0"
  ///        processorArchitecture="*"
  ///        publicKeyToken="6595b64144ccf1df"
  ///        language="*"
  ///      />
  ///    </dependentAssembly>
  ///  </dependency>
  /// ```
  ///
  /// # Example
  ///
  /// The following manifest will brand the exe as requesting administrator privileges.
  /// Thus, every time it is executed, a Windows UAC dialog will appear.
  ///
  /// ```rust,no_run
  /// let mut windows = tauri_build::WindowsAttributes::new();
  /// windows = windows.app_manifest(r#"
  /// <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  ///   <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
  ///       <security>
  ///           <requestedPrivileges>
  ///               <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
  ///           </requestedPrivileges>
  ///       </security>
  ///   </trustInfo>
  /// </assembly>
  /// "#);
  /// let attrs =  tauri_build::Attributes::new().windows_attributes(windows);
  /// tauri_build::try_build(attrs).expect("failed to run build script");
  /// ```
  ///
  /// Note that you can move the manifest contents to a separate file and use `include_str!("manifest.xml")`
  /// instead of the inline string.
  ///
  /// [manifest]: https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests
  #[must_use]
  pub fn app_manifest<S: AsRef<str>>(mut self, manifest: S) -> Self {
    self.app_manifest = Some(manifest.as_ref().to_string());
    self
  }

  /// Append additional .rc content to the generated resource file on Windows.
  /// This can be called multiple times to append multiple contents.
  #[must_use]
  pub fn append_rc_content<S: Into<String>>(mut self, content: S) -> Self {
    self.append_rc_content.push(content.into());
    self
  }
}

/// The attributes used on the build.
#[derive(Debug, Default)]
pub struct Attributes {
  #[allow(dead_code)]
  windows_attributes: WindowsAttributes,
  capabilities_path_pattern: Option<&'static str>,
  config_path: Option<PathBuf>,
  #[cfg(feature = "codegen")]
  codegen: Option<codegen::context::CodegenContext>,
  inlined_plugins: HashMap<&'static str, InlinedPlugin>,
  app_manifest: AppManifest,
}

impl Attributes {
  /// Creates the default attribute set.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the [`WindowsAttributes`], the Windows-specific options of the build script.
  ///
  /// They are used to configure the [application manifest], the application icon, additional
  /// `.rc` content and the static linking of the Visual C++ runtime.
  /// Setting them has no effect when compiling for other platforms.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// let attributes = tauri_build::Attributes::new().windows_attributes(
  ///   tauri_build::WindowsAttributes::new()
  ///     .window_icon_path("icons/icon.ico")
  ///     .static_vc_runtime(true),
  /// );
  /// tauri_build::try_build(attributes).expect("failed to run tauri-build");
  /// ```
  ///
  /// [application manifest]: https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests
  #[must_use]
  pub fn windows_attributes(mut self, windows_attributes: WindowsAttributes) -> Self {
    self.windows_attributes = windows_attributes;
    self
  }

  /// Set the glob pattern to be used to find the capabilities.
  ///
  /// **WARNING:** The `removeUnusedCommands` option does not work with a custom capabilities path.
  ///
  /// **Note:** You must emit [rerun-if-changed] instructions for your capabilities directory.
  ///
  /// [rerun-if-changed]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed
  #[must_use]
  pub fn capabilities_path_pattern(mut self, pattern: &'static str) -> Self {
    self.capabilities_path_pattern.replace(pattern);
    self
  }

  /// Adds the given plugin to the list of inlined plugins (a plugin that is part of your application).
  ///
  /// See [`InlinedPlugin`] for more information.
  pub fn plugin(mut self, name: &'static str, plugin: InlinedPlugin) -> Self {
    self.inlined_plugins.insert(name, plugin);
    self
  }

  /// Adds the given list of plugins to the list of inlined plugins (a plugin that is part of your application).
  ///
  /// See [`InlinedPlugin`] for more information.
  pub fn plugins<I>(mut self, plugins: I) -> Self
  where
    I: IntoIterator<Item = (&'static str, InlinedPlugin)>,
  {
    self.inlined_plugins.extend(plugins);
    self
  }

  /// Set the path to the `tauri.conf.json` (relative to the crate's directory).
  ///
  /// This defaults to a file called `tauri.conf.json` inside of the current working directory of
  /// the crate compiling; does not need to be set manually if that config file is in the same
  /// directory as your `Cargo.toml`.
  pub fn config_path(mut self, config_path: impl Into<PathBuf>) -> Self {
    self.config_path = Some(config_path.into());
    self
  }

  /// Sets the application manifest for the Access Control List.
  ///
  /// See [`AppManifest`] for more information.
  pub fn app_manifest(mut self, manifest: AppManifest) -> Self {
    self.app_manifest = manifest;
    self
  }

  /// Generates the Tauri application context at build time instead of at macro expansion time.
  ///
  /// The context is written to a file in the `OUT_DIR` that the
  /// [`tauri::tauri_build_context!`] macro includes, so the application uses
  /// `tauri::Builder::run(tauri::tauri_build_context!())` instead of
  /// `tauri::Builder::run(tauri::generate_context!())`.
  ///
  /// This moves the asset embedding and the ACL resolution to the build script, which means the
  /// `cargo:rerun-if-changed` instructions emitted for the frontend assets, icons and
  /// configuration files are respected - with `generate_context!` the code is only regenerated
  /// when the crate itself is recompiled.
  ///
  /// See [`CodegenContext`] for the available options.
  ///
  /// Requires the `codegen` Cargo feature.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// let attributes = tauri_build::Attributes::new().codegen(tauri_build::CodegenContext::new());
  /// tauri_build::try_build(attributes).expect("failed to run tauri-build");
  /// ```
  ///
  /// [`tauri::tauri_build_context!`]: https://docs.rs/tauri/latest/tauri/macro.tauri_build_context.html
  #[cfg(feature = "codegen")]
  #[cfg_attr(docsrs, doc(cfg(feature = "codegen")))]
  #[must_use]
  pub fn codegen(mut self, codegen: codegen::context::CodegenContext) -> Self {
    self.codegen.replace(codegen);
    self
  }
}

/// Whether the app is being compiled for development (`tauri dev`) or not.
///
/// It reads the `DEP_TAURI_DEV` environment variable, which is set by the build script of the
/// `tauri` crate from the `cargo:dev` instruction, and is `true` when the `tauri` crate is
/// compiled without the `custom-protocol` Cargo feature.
///
/// [`try_build`] uses it to define the `dev` [cfg alias], so prefer `#[cfg(dev)]` on your app
/// code; this function is meant for `build.rs` code that must branch on the development build.
///
/// # Panics
///
/// Panics if the `DEP_TAURI_DEV` environment variable is not set, which means the crate calling
/// it does not depend on a `tauri` version that emits the `cargo:dev` instruction.
/// Update the `tauri` dependency to the latest version to fix it.
///
/// [cfg alias]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-cfgkeyvalue
pub fn is_dev() -> bool {
  env::var_os("DEP_TAURI_DEV")
    .expect("missing `cargo:dev` instruction, please update tauri to latest")
    == "true"
}

/// Run all build time helpers for your Tauri Application.
///
/// To provide extra configuration, such as [`AppManifest::commands`]
/// for fine-grained control over command permissions, see [`try_build`].
/// See [`Attributes`] for the complete list of configuration options.
///
/// # Platforms
///
/// [`build()`] should be called inside of `build.rs` regardless of the platform, so **DO NOT** use a [conditional compilation]
/// check that prevents it from running on any of your targets.
///
/// Platform specific code is handled by the helpers automatically.
///
/// A build script is required in order to activate some cargo environmental variables that are
/// used when generating code and embedding assets.
///
/// # Panics
///
/// If any of the build time helpers fail, they will [`std::panic!`] with the related error message.
/// This is typically desirable when running inside a build script; see [`try_build`] for no panics.
///
/// [conditional compilation]: https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/conditional-compilation.html
pub fn build() {
  if let Err(error) = try_build(Attributes::default()) {
    let error = format!("{error:#}");
    println!("{error}");
    if error.starts_with("unknown field") {
      print!(
        "found an unknown configuration field. This usually means that you are using a CLI version that is newer than `tauri-build` and is incompatible. "
      );
      println!(
        "Please try updating the Rust crates by running `cargo update` in the Tauri app folder."
      );
    }
    std::process::exit(1);
  }
}

/// Same as [`build()`], but takes an extra configuration argument, and does not panic.
#[allow(unused_variables)]
pub fn try_build(attributes: Attributes) -> Result<()> {
  use anyhow::anyhow;

  println!("cargo:rerun-if-env-changed=TAURI_CONFIG");

  let target_os = env::var_os("CARGO_CFG_TARGET_OS").unwrap();
  let mobile = target_os == "ios" || target_os == "android";
  cfg_alias("desktop", !mobile);
  cfg_alias("mobile", mobile);

  let target_triple = env::var("TARGET").unwrap();
  let target = tauri_utils::platform::Target::from_triple(&target_triple);

  let config_root = if let Some(config_path) = &attributes.config_path {
    config_path.parent().with_context(|| {
      format!(
        "`config_path` '{}' doesn't have a parent directory",
        config_path.display()
      )
    })?
  } else {
    &env::current_dir().unwrap()
  };

  let (mut config, config_paths) = tauri_utils::config::parse::read_from(target, config_root)?;

  for config_file_path in config_paths {
    println!("cargo:rerun-if-changed={}", config_file_path.display());
  }
  if let Ok(env) = env::var("TAURI_CONFIG") {
    let merge_config: serde_json::Value = serde_json::from_str(&env)?;
    json_patch::merge(&mut config, &merge_config);
  }
  let config: Config = serde_json::from_value(config)?;
  let static_vc_runtime = should_static_link_vc_runtime(&config, &attributes);

  let s = config.identifier.split('.');
  let last = s.clone().count() - 1;
  let mut android_package_prefix = String::new();
  for (i, w) in s.enumerate() {
    if i == last {
      println!(
        "cargo:rustc-env=TAURI_ANDROID_PACKAGE_NAME_APP_NAME={}",
        w.replace('-', "_")
      );
    } else {
      android_package_prefix.push_str(&w.replace(['_', '-'], "_1"));
      android_package_prefix.push('_');
    }
  }
  android_package_prefix.pop();
  println!("cargo:rustc-env=TAURI_ANDROID_PACKAGE_NAME_PREFIX={android_package_prefix}");

  if let Some(project_dir) = env::var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
    mobile::generate_gradle_files(project_dir)?;

    // Update Android manifest with file associations
    if let Some(associations) = config.bundle.file_associations.as_ref() {
      mobile::update_android_manifest_file_associations(associations)?;
    }
  }

  cfg_alias("dev", is_dev());

  let cargo_toml_path = Path::new("Cargo.toml").canonicalize()?;
  let mut manifest = Manifest::<cargo_toml::Value>::from_path_with_metadata(cargo_toml_path)?;

  let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

  manifest::check(&config, &mut manifest)?;

  acl::build(&out_dir, target, &config, &attributes)?;

  tauri_utils::plugin::save_global_api_scripts_paths(&out_dir, None);

  println!("cargo:rustc-env=TAURI_ENV_TARGET_TRIPLE={target_triple}");
  // when running codegen in this build script, we need to access the env var directly
  // FIXME: This can be accessed from multiple threads
  unsafe { env::set_var("TAURI_ENV_TARGET_TRIPLE", &target_triple) };

  let target_dir = target_dir_from_out_dir(&out_dir)
    .with_context(|| format!("failed to resolve the target directory from {out_dir:?}"))?;

  if let Some(paths) = &config.bundle.external_bin {
    copy_binaries(
      ResourcePaths::new(&external_binaries(paths, &target_triple, &target), true),
      &target_triple,
      target_dir,
      manifest.package.as_ref().map(|p| p.name.as_ref()),
    )?;
  }

  let mut resources = config
    .bundle
    .resources
    .clone()
    .unwrap_or(BundleResources::List(Vec::new()));
  if target_triple.contains("windows") {
    if let Some(fixed_webview2_runtime_path) = match &config.bundle.windows.webview_install_mode {
      WebviewInstallMode::FixedRuntime { path } => Some(path),
      _ => None,
    } {
      resources.push(fixed_webview2_runtime_path.display().to_string());
    }
  }
  match resources {
    BundleResources::List(res) => {
      copy_resources(ResourcePaths::new(res.as_slice(), true), target_dir)?
    }
    BundleResources::Map(map) => copy_resources(ResourcePaths::from_map(&map, true), target_dir)?,
  }

  if target_triple.contains("darwin") {
    if let Some(frameworks) = &config.bundle.macos.frameworks {
      if !frameworks.is_empty() {
        let frameworks_dir = target_dir.parent().unwrap().join("Frameworks");
        let _ = fs::remove_dir_all(&frameworks_dir);
        // copy frameworks to the root `target` folder (instead of `target/debug` for instance)
        // because the rpath is set to `@executable_path/../Frameworks`.
        copy_frameworks(&frameworks_dir, frameworks)?;

        // If we have frameworks, we need to set the @rpath
        // https://github.com/tauri-apps/tauri/issues/7710
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
      }
    }

    if !is_dev() {
      if let Some(version) = &config.bundle.macos.minimum_system_version {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET={version}");
      }
    }
  }

  if target_triple.contains("ios") {
    println!(
      "cargo:rustc-env=IPHONEOS_DEPLOYMENT_TARGET={}",
      config.bundle.ios.minimum_system_version
    );
  }

  if target_triple.contains("windows") {
    use semver::Version;
    use tauri_winres::{VersionInfo, WindowsResource};

    let window_icon_path = attributes
      .windows_attributes
      .window_icon_path
      .unwrap_or_else(|| {
        // icon paths in the config are relative to the config file
        config_root.join(
          config
            .bundle
            .icon
            .iter()
            .find(|i| i.ends_with(".ico"))
            .map(AsRef::as_ref)
            .unwrap_or("icons/icon.ico"),
        )
      });

    let mut res = WindowsResource::new();

    if let Some(manifest) = attributes.windows_attributes.app_manifest {
      res.set_manifest(&manifest);
    }

    for content in attributes.windows_attributes.append_rc_content {
      res.append_rc_content(&content);
    }

    if let Some(version_str) = &config.version {
      if let Ok(v) = Version::parse(version_str) {
        let version = to_winres_version(&v);
        res.set_version_info(VersionInfo::FILEVERSION, version);
        res.set_version_info(VersionInfo::PRODUCTVERSION, version);
        res.set("FileVersion", version_str);
        res.set("ProductVersion", version_str);
      }
    }

    if let Some(product_name) = &config.product_name {
      res.set("ProductName", product_name);
    }

    let company_name = config.bundle.publisher.unwrap_or_else(|| {
      config
        .identifier
        .split('.')
        .nth(1)
        .unwrap_or(&config.identifier)
        .to_string()
    });

    res.set("CompanyName", &company_name);

    let file_description = config
      .product_name
      .or_else(|| manifest.package.as_ref().map(|p| p.name.clone()))
      .or_else(|| std::env::var("CARGO_PKG_NAME").ok());

    res.set("FileDescription", &file_description.unwrap());

    if let Some(copyright) = &config.bundle.copyright {
      res.set("LegalCopyright", copyright);
    }

    if window_icon_path.exists() {
      res.set_icon_with_id(
        &window_icon_path.display().to_string(),
        &tauri_utils::platform::WINDOWS_APP_ICON_RESOURCE_ID.to_string(),
      );
    } else {
      return Err(anyhow!(format!(
        "`{}` not found; required for generating a Windows Resource file during tauri-build",
        window_icon_path.display()
      )));
    }

    res.compile().with_context(|| {
      format!(
        "failed to compile `{}` into a Windows Resource file during tauri-build",
        window_icon_path.display()
      )
    })?;

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    match target_env.as_str() {
      "gnu" => {
        let target_arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
          "x86_64" => Some("x64"),
          "x86" => Some("x86"),
          "aarch64" => Some("arm64"),
          arch => None,
        };
        if let Some(target_arch) = target_arch {
          for entry in fs::read_dir(target_dir.join("build"))? {
            let path = entry?.path();
            let webview2_loader_path = path
              .join("out")
              .join(target_arch)
              .join("WebView2Loader.dll");
            if path.to_string_lossy().contains("webview2-com-sys") && webview2_loader_path.exists()
            {
              fs::copy(webview2_loader_path, target_dir.join("WebView2Loader.dll"))?;
              break;
            }
          }
        }
      }
      "msvc" if static_vc_runtime => {
        static_vcruntime::build();
      }
      _ => (),
    }
  }

  #[cfg(feature = "codegen")]
  if let Some(mut codegen) = attributes.codegen {
    if codegen.config_path.is_none() {
      codegen.config_path = attributes.config_path;
    }
    codegen.try_build()?;
  }

  Ok(())
}

fn to_winres_version(v: &semver::Version) -> u64 {
  let build = v.build.parse::<u16>().map(u64::from).unwrap_or(0);

  (v.major << 48) | (v.minor << 32) | (v.patch << 16) | build
}

fn should_static_link_vc_runtime(config: &Config, attributes: &Attributes) -> bool {
  if let Some(value) = env::var_os("STATIC_VCRUNTIME") {
    println!(
      "cargo:warning=STATIC_VCRUNTIME is deprecated; use build.windows.staticVCRuntime in tauri.conf.json or tauri_build::WindowsAttributes::static_vc_runtime instead."
    );
    value != "false"
  } else {
    attributes
      .windows_attributes
      .static_vc_runtime
      .unwrap_or(config.build.windows.static_vc_runtime)
  }
}

#[cfg(test)]
mod tests {
  use semver::Version;
  use std::path::Path;

  #[test]
  fn target_dir_from_stable_out_dir() {
    let out_dir = Path::new("/app/target/debug/build/app-63ba68eead531e35/out");

    assert_eq!(
      crate::target_dir_from_out_dir(out_dir),
      Some(Path::new("/app/target/debug"))
    );
  }

  #[test]
  fn target_dir_from_nightly_out_dir() {
    let out_dir = Path::new("/app/target/debug/build/app/63ba68eead531e35/out");

    assert_eq!(
      crate::target_dir_from_out_dir(out_dir),
      Some(Path::new("/app/target/debug"))
    );
  }

  #[test]
  fn target_dir_from_out_dir_with_triple() {
    let out_dir =
      Path::new("/app/target/aarch64-apple-darwin/release/build/app/63ba68eead531e35/out");

    assert_eq!(
      crate::target_dir_from_out_dir(out_dir),
      Some(Path::new("/app/target/aarch64-apple-darwin/release"))
    );
  }

  #[test]
  fn version_uses_numeric_build_metadata() {
    let version = Version::parse("1.2.3+42").unwrap();

    assert_eq!(
      crate::to_winres_version(&version),
      (1 << 48) | (2 << 32) | (3 << 16) | 42
    );
  }

  #[test]
  fn version_ignores_non_numeric_composite_build_metadata() {
    let version = Version::parse("1.2.3+42.sha").unwrap();

    assert_eq!(
      crate::to_winres_version(&version),
      (1 << 48) | (2 << 32) | (3 << 16)
    );
  }

  #[test]
  fn version_ignores_non_numeric_build_metadata() {
    let version = Version::parse("1.2.3+abc").unwrap();

    assert_eq!(
      crate::to_winres_version(&version),
      (1 << 48) | (2 << 32) | (3 << 16)
    );
  }

  #[test]
  fn version_ignores_build_metadata_that_does_not_fit_in_u16() {
    let version = Version::parse("1.2.3+70000").unwrap();

    assert_eq!(
      crate::to_winres_version(&version),
      (1 << 48) | (2 << 32) | (3 << 16)
    );
  }

  #[test]
  #[serial_test::serial]
  fn static_vc_runtime_chain() {
    // 1. Nothing is set, should default to true
    let config = tauri_utils::config::Config::default();
    let attributes = crate::Attributes::new();
    assert!(crate::should_static_link_vc_runtime(&config, &attributes));

    // 2. Set to anything but "false" in env, should be true
    unsafe { std::env::set_var("STATIC_VCRUNTIME", "qweqe") };
    let config = tauri_utils::config::Config::default();
    let attributes = crate::Attributes::new();
    assert!(crate::should_static_link_vc_runtime(&config, &attributes));
    unsafe { std::env::remove_var("STATIC_VCRUNTIME") };

    // 3. Set to "false" in env, should be false
    unsafe { std::env::set_var("STATIC_VCRUNTIME", "false") };
    let config = tauri_utils::config::Config::default();
    let attributes = crate::Attributes::new();
    assert!(!crate::should_static_link_vc_runtime(&config, &attributes));
    unsafe { std::env::remove_var("STATIC_VCRUNTIME") };

    // 4. Set to true in attributes, should be true
    let config = tauri_utils::config::Config::default();
    let attributes = crate::Attributes::new()
      .windows_attributes(crate::WindowsAttributes::new().static_vc_runtime(true));
    assert!(crate::should_static_link_vc_runtime(&config, &attributes));

    // 5. Set to false in attributes, should be false
    let config = tauri_utils::config::Config::default();
    let attributes = crate::Attributes::new()
      .windows_attributes(crate::WindowsAttributes::new().static_vc_runtime(false));
    assert!(!crate::should_static_link_vc_runtime(&config, &attributes));

    // 6. Set to true in config, should be true
    let config = tauri_utils::config::Config {
      build: tauri_utils::config::BuildConfig {
        windows: tauri_utils::config::WindowsBuildConfig {
          static_vc_runtime: true,
        },
        ..Default::default()
      },
      ..Default::default()
    };
    let attributes = crate::Attributes::new();
    assert!(crate::should_static_link_vc_runtime(&config, &attributes));

    // 7. Set to false in config, should be false
    let config = tauri_utils::config::Config {
      build: tauri_utils::config::BuildConfig {
        windows: tauri_utils::config::WindowsBuildConfig {
          static_vc_runtime: false,
        },
        ..Default::default()
      },
      ..Default::default()
    };
    let attributes = crate::Attributes::new();
    assert!(!crate::should_static_link_vc_runtime(&config, &attributes));

    // 8. Set to true in config and false in attributes, should be false because attributes takes precedence over config
    let config = tauri_utils::config::Config {
      build: tauri_utils::config::BuildConfig {
        windows: tauri_utils::config::WindowsBuildConfig {
          static_vc_runtime: true,
        },
        ..Default::default()
      },
      ..Default::default()
    };
    let attributes = crate::Attributes::new()
      .windows_attributes(crate::WindowsAttributes::new().static_vc_runtime(false));
    assert!(!crate::should_static_link_vc_runtime(&config, &attributes));

    // 9. Set to false in env and true in attributes, should be false because env takes precedence over attributes
    unsafe { std::env::set_var("STATIC_VCRUNTIME", "false") };
    let config = tauri_utils::config::Config::default();
    let attributes = crate::Attributes::new()
      .windows_attributes(crate::WindowsAttributes::new().static_vc_runtime(true));
    assert!(!crate::should_static_link_vc_runtime(&config, &attributes));
    unsafe { std::env::remove_var("STATIC_VCRUNTIME") };
  }
}
