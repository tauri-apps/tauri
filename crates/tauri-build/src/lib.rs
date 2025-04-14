// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This applies the macros at build-time in order to rig some special features needed by `cargo`.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use anyhow::Context;
pub use anyhow::Result;
use cargo_toml::Manifest;

use tauri_utils::{
  config::{BundleResources, Config, WebviewInstallMode},
  resources::{external_binaries, ResourcePaths},
};

use std::{
  collections::HashMap,
  env, fs,
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
  package_name: Option<&String>,
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
  for resource in resources.iter() {
    let resource = resource?;

    println!("cargo:rerun-if-changed={}", resource.path().display());

    // avoid copying the resource if target is the same as source
    let src = resource.path().canonicalize()?;
    let target = path.join(resource.target());
    if src != target {
      copy_file(src, target)?;
    }
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
      let src_path = PathBuf::from(framework);
      let src_name = src_path
        .file_name()
        .expect("Couldn't get framework filename");
      let dest_path = dest_dir.join(src_name);
      copy_dir(&src_path, &dest_path)?;
      continue;
    } else if framework.ends_with(".dylib") {
      let src_path = PathBuf::from(framework);
      if !src_path.exists() {
        return Err(anyhow::anyhow!("Library not found: {}", framework));
      }
      let src_name = src_path.file_name().expect("Couldn't get library filename");
      let dest_path = dest_dir.join(src_name);
      copy_file(&src_path, &dest_path)?;
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
    if copy_framework_from(&PathBuf::from("/Library/Frameworks/"), framework, dest_dir)?
      || copy_framework_from(
        &PathBuf::from("/Network/Library/Frameworks/"),
        framework,
        dest_dir,
      )?
    {
      continue;
    }
  }
  Ok(())
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
      window_icon_path: Default::default(),
      app_manifest: Some(include_str!("windows-app-manifest.xml").into()),
    }
  }

  /// Creates the default attriute set wihtou the default app manifest.
  #[must_use]
  pub fn new_without_app_manifest() -> Self {
    Self {
      app_manifest: None,
      window_icon_path: Default::default(),
    }
  }

  /// Sets the icon to use on the window. Currently only used on Windows.
  /// It must be in `ico` format. Defaults to `icons/icon.ico`.
  #[must_use]
  pub fn window_icon_path<P: AsRef<Path>>(mut self, window_icon_path: P) -> Self {
    self
      .window_icon_path
      .replace(window_icon_path.as_ref().into());
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
}

/// The attributes used on the build.
#[derive(Debug, Default)]
pub struct Attributes {
  #[allow(dead_code)]
  windows_attributes: WindowsAttributes,
  capabilities_path_pattern: Option<&'static str>,
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

  /// Sets the icon to use on the window. Currently only used on Windows.
  #[must_use]
  pub fn windows_attributes(mut self, windows_attributes: WindowsAttributes) -> Self {
    self.windows_attributes = windows_attributes;
    self
  }

  /// Set the glob pattern to be used to find the capabilities.
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

  /// Sets the application manifest for the Access Control List.
  ///
  /// See [`AppManifest`] for more information.
  pub fn app_manifest(mut self, manifest: AppManifest) -> Self {
    self.app_manifest = manifest;
    self
  }

  #[cfg(feature = "codegen")]
  #[cfg_attr(docsrs, doc(cfg(feature = "codegen")))]
  #[must_use]
  pub fn codegen(mut self, codegen: codegen::context::CodegenContext) -> Self {
    self.codegen.replace(codegen);
    self
  }
}

pub fn is_dev() -> bool {
  env::var("DEP_TAURI_DEV").expect("missing `cargo:dev` instruction, please update tauri to latest")
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
      print!("found an unknown configuration field. This usually means that you are using a CLI version that is newer than `tauri-build` and is incompatible. ");
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

  let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
  let mobile = target_os == "ios" || target_os == "android";
  cfg_alias("desktop", !mobile);
  cfg_alias("mobile", mobile);

  let target_triple = env::var("TARGET").unwrap();
  let target = tauri_utils::platform::Target::from_triple(&target_triple);

  let (mut config, config_paths) =
    tauri_utils::config::parse::read_from(target, &env::current_dir().unwrap())?;
  for config_file_path in config_paths {
    println!("cargo:rerun-if-changed={}", config_file_path.display());
  }
  if let Ok(env) = env::var("TAURI_CONFIG") {
    let merge_config: serde_json::Value = serde_json::from_str(&env)?;
    json_patch::merge(&mut config, &merge_config);
  }
  let config: Config = serde_json::from_value(config)?;

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
    mobile::generate_gradle_files(project_dir, &config)?;
  }

  cfg_alias("dev", is_dev());

  let cargo_toml_path = Path::new("Cargo.toml").canonicalize()?;
  let mut manifest = Manifest::<cargo_toml::Value>::from_path_with_metadata(cargo_toml_path)?;

  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  manifest::check(&config, &mut manifest)?;

  acl::build(&out_dir, target, &attributes)?;

  tauri_utils::plugin::save_global_api_scripts_paths(&out_dir, None);

  println!("cargo:rustc-env=TAURI_ENV_TARGET_TRIPLE={target_triple}");
  // when running codegen in this build script, we need to access the env var directly
  env::set_var("TAURI_ENV_TARGET_TRIPLE", &target_triple);

  // TODO: far from ideal, but there's no other way to get the target dir, see <https://github.com/rust-lang/cargo/issues/5457>
  let target_dir = out_dir
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap();

  if let Some(paths) = &config.bundle.external_bin {
    copy_binaries(
      ResourcePaths::new(external_binaries(paths, &target_triple).as_slice(), true),
      &target_triple,
      target_dir,
      manifest.package.as_ref().map(|p| &p.name),
    )?;
  }

  #[allow(unused_mut, clippy::redundant_clone)]
  let mut resources = config
    .bundle
    .resources
    .clone()
    .unwrap_or_else(|| BundleResources::List(Vec::new()));
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

    if let Some(version) = &config.bundle.macos.minimum_system_version {
      println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET={version}");
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

    fn find_icon<F: Fn(&&String) -> bool>(config: &Config, predicate: F, default: &str) -> PathBuf {
      let icon_path = config
        .bundle
        .icon
        .iter()
        .find(|i| predicate(i))
        .cloned()
        .unwrap_or_else(|| default.to_string());
      icon_path.into()
    }

    let window_icon_path = attributes
      .windows_attributes
      .window_icon_path
      .unwrap_or_else(|| find_icon(&config, |i| i.ends_with(".ico"), "icons/icon.ico"));

    let mut res = WindowsResource::new();

    if let Some(manifest) = attributes.windows_attributes.app_manifest {
      res.set_manifest(&manifest);
    }

    if let Some(version_str) = &config.version {
      if let Ok(v) = Version::parse(version_str) {
        let version = (v.major << 48) | (v.minor << 32) | (v.patch << 16);
        res.set_version_info(VersionInfo::FILEVERSION, version);
        res.set_version_info(VersionInfo::PRODUCTVERSION, version);
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
      res.set_icon_with_id(&window_icon_path.display().to_string(), "32512");
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
      "msvc" => {
        if env::var("STATIC_VCRUNTIME").is_ok_and(|v| v == "true") {
          static_vcruntime::build();
        }
      }
      _ => (),
    }
  }

  #[cfg(feature = "codegen")]
  if let Some(codegen) = attributes.codegen {
    codegen.try_build()?;
  }

  Ok(())
}
