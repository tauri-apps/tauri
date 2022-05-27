// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub use anyhow::Result;
use heck::ToSnakeCase;
use tauri_utils::resources::{external_binaries, resource_relpath, ResourcePaths};

use std::path::{Path, PathBuf};

#[cfg(feature = "codegen")]
mod codegen;
#[cfg(windows)]
mod static_vcruntime;

#[cfg(feature = "codegen")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "codegen")))]
pub use codegen::context::CodegenContext;

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
  std::fs::create_dir_all(dest_dir)?;
  std::fs::copy(from, to)?;
  Ok(())
}

fn copy_binaries<'a>(binaries: ResourcePaths<'a>, target_triple: &str, path: &Path) -> Result<()> {
  for src in binaries {
    let src = src?;
    println!("cargo:rerun-if-changed={}", src.display());
    let dest = path.join(
      src
        .file_name()
        .expect("failed to extract external binary filename")
        .to_string_lossy()
        .replace(&format!("-{}", target_triple), ""),
    );
    if dest.exists() {
      std::fs::remove_file(&dest).unwrap();
    }
    copy_file(&src, &dest)?;
  }
  Ok(())
}

/// Copies resources to a path.
fn copy_resources(resources: ResourcePaths<'_>, path: &Path) -> Result<()> {
  for src in resources {
    let src = src?;
    println!("cargo:rerun-if-changed={}", src.display());
    let dest = path.join(resource_relpath(&src));
    copy_file(&src, &dest)?;
  }
  Ok(())
}

// checks if the given Cargo feature is enabled.
fn has_feature(feature: &str) -> bool {
  // when a feature is enabled, Cargo sets the `CARGO_FEATURE_<name` env var to 1
  // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
  std::env::var(format!(
    "CARGO_FEATURE_{}",
    feature.to_snake_case().to_uppercase()
  ))
  .map(|x| x == "1")
  .unwrap_or(false)
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn cfg_alias(alias: &str, has_feature: bool) {
  if has_feature {
    println!("cargo:rustc-cfg={}", alias);
  }
}

/// Attributes used on Windows.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct WindowsAttributes {
  window_icon_path: Option<PathBuf>,
  /// The path to the sdk location. This can be a absolute or relative path. If not supplied
  /// this defaults to whatever `winres` crate determines is the best. See the
  /// [winres documentation](https://docs.rs/winres/*/winres/struct.WindowsResource.html#method.set_toolkit_path)
  sdk_dir: Option<PathBuf>,
}

impl WindowsAttributes {
  /// Creates the default attribute set.
  pub fn new() -> Self {
    Self::default()
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

  /// Sets the sdk dir for windows. Currently only used on Windows. This must be a vaild UTF-8
  /// path. Defaults to whatever the `winres` crate determines is best.
  #[must_use]
  pub fn sdk_dir<P: AsRef<Path>>(mut self, sdk_dir: P) -> Self {
    self.sdk_dir = Some(sdk_dir.as_ref().into());
    self
  }
}

/// The attributes used on the build.
#[derive(Debug, Default)]
pub struct Attributes {
  #[allow(dead_code)]
  windows_attributes: WindowsAttributes,
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
}

/// Run all build time helpers for your Tauri Application.
///
/// The current helpers include the following:
/// * Generates a Windows Resource file when targeting Windows.
///
/// # Platforms
///
/// [`build()`] should be called inside of `build.rs` regardless of the platform:
/// * New helpers may target more platforms in the future.
/// * Platform specific code is handled by the helpers automatically.
/// * A build script is required in order to activate some cargo environmental variables that are
///   used when generating code and embedding assets - so [`build()`] may as well be called.
///
/// In short, this is saying don't put the call to [`build()`] behind a `#[cfg(windows)]`.
///
/// # Panics
///
/// If any of the build time helpers fail, they will [`std::panic!`] with the related error message.
/// This is typically desirable when running inside a build script; see [`try_build`] for no panics.
pub fn build() {
  if let Err(error) = try_build(Attributes::default()) {
    panic!("error found during tauri-build: {:#?}", error);
  }
}

/// Non-panicking [`build()`].
#[allow(unused_variables)]
pub fn try_build(attributes: Attributes) -> Result<()> {
  use anyhow::anyhow;
  use cargo_toml::{Dependency, Manifest};
  use tauri_utils::config::{Config, TauriConfig};

  println!("cargo:rerun-if-env-changed=TAURI_CONFIG");
  println!("cargo:rerun-if-changed=tauri.conf.json");
  #[cfg(feature = "config-json5")]
  println!("cargo:rerun-if-changed=tauri.conf.json5");

  let config: Config = if let Ok(env) = std::env::var("TAURI_CONFIG") {
    serde_json::from_str(&env)?
  } else {
    serde_json::from_value(tauri_utils::config::parse::read_from(
      std::env::current_dir().unwrap(),
    )?)?
  };

  #[cfg(windows)]
  static_vcruntime::build();

  cfg_alias("dev", !has_feature("custom-protocol"));

  let mut manifest = Manifest::from_path("Cargo.toml")?;
  if let Some(tauri) = manifest.dependencies.remove("tauri") {
    let features = match tauri {
      Dependency::Simple(_) => Vec::new(),
      Dependency::Detailed(dep) => dep.features,
    };

    let all_cli_managed_features = TauriConfig::all_features();
    let diff = features_diff(
      &features
        .into_iter()
        .filter(|f| all_cli_managed_features.contains(&f.as_str()))
        .collect::<Vec<String>>(),
      &config
        .tauri
        .features()
        .into_iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>(),
    );

    let mut error_message = String::new();
    if !diff.remove.is_empty() {
      error_message.push_str("remove the `");
      error_message.push_str(&diff.remove.join(", "));
      error_message.push_str(if diff.remove.len() == 1 {
        "` feature"
      } else {
        "` features"
      });
      if !diff.add.is_empty() {
        error_message.push_str(" and ");
      }
    }
    if !diff.add.is_empty() {
      error_message.push_str("add the `");
      error_message.push_str(&diff.add.join(", "));
      error_message.push_str(if diff.add.len() == 1 {
        "` feature"
      } else {
        "` features"
      });
    }

    if !error_message.is_empty() {
      return Err(anyhow!("
      The `tauri` dependency features on the `Cargo.toml` file does not match the allowlist defined under `tauri.conf.json`.
      Please run `tauri dev` or `tauri build` or {}.
    ", error_message));
    }
  }

  let target_triple = std::env::var("TARGET").unwrap();
  let out_dir = std::env::var("OUT_DIR").unwrap();
  // TODO: far from ideal, but there's no other way to get the target dir, see <https://github.com/rust-lang/cargo/issues/5457>
  let target_dir = Path::new(&out_dir)
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap();

  if let Some(paths) = &config.tauri.bundle.external_bin {
    copy_binaries(
      ResourcePaths::new(external_binaries(paths, &target_triple).as_slice(), true),
      &target_triple,
      target_dir,
    )?;
  }

  #[allow(unused_mut)]
  let mut resources = config.tauri.bundle.resources.clone().unwrap_or_default();
  #[cfg(target_os = "linux")]
  if let Some(tray) = config.tauri.system_tray {
    resources.push(tray.icon_path.display().to_string());
  }
  copy_resources(ResourcePaths::new(resources.as_slice(), true), target_dir)?;

  #[cfg(target_os = "macos")]
  {
    if let Some(version) = config.tauri.bundle.macos.minimum_system_version {
      println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET={}", version);
    }
  }

  #[cfg(windows)]
  {
    use anyhow::Context;
    use semver::Version;
    use winres::{VersionInfo, WindowsResource};

    fn find_icon<F: Fn(&&String) -> bool>(config: &Config, predicate: F, default: &str) -> PathBuf {
      let icon_path = config
        .tauri
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

    if window_icon_path.exists() {
      let mut res = WindowsResource::new();
      if let Some(sdk_dir) = &attributes.windows_attributes.sdk_dir {
        if let Some(sdk_dir_str) = sdk_dir.to_str() {
          res.set_toolkit_path(sdk_dir_str);
        } else {
          return Err(anyhow!(
            "sdk_dir path is not valid; only UTF-8 characters are allowed"
          ));
        }
      }
      if let Some(version) = &config.package.version {
        if let Ok(v) = Version::parse(version) {
          let version = v.major << 48 | v.minor << 32 | v.patch << 16;
          res.set_version_info(VersionInfo::FILEVERSION, version);
          res.set_version_info(VersionInfo::PRODUCTVERSION, version);
        }
        res.set("FileVersion", version);
        res.set("ProductVersion", version);
      }
      if let Some(product_name) = &config.package.product_name {
        res.set("ProductName", product_name);
        res.set("FileDescription", product_name);
      }
      res.set_icon_with_id(&window_icon_path.display().to_string(), "32512");
      res.compile().with_context(|| {
        format!(
          "failed to compile `{}` into a Windows Resource file during tauri-build",
          window_icon_path.display()
        )
      })?;
    } else {
      return Err(anyhow!(format!(
        "`{}` not found; required for generating a Windows Resource file during tauri-build",
        window_icon_path.display()
      )));
    }
  }

  Ok(())
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Diff {
  remove: Vec<String>,
  add: Vec<String>,
}

fn features_diff(current: &[String], expected: &[String]) -> Diff {
  let mut remove = Vec::new();
  let mut add = Vec::new();
  for feature in current {
    if !expected.contains(feature) {
      remove.push(feature.clone());
    }
  }

  for feature in expected {
    if !current.contains(feature) {
      add.push(feature.clone());
    }
  }

  Diff { remove, add }
}

#[cfg(test)]
mod tests {
  use super::Diff;

  #[test]
  fn array_diff() {
    for (current, expected, result) in [
      (vec![], vec![], Default::default()),
      (
        vec!["a".into()],
        vec![],
        Diff {
          remove: vec!["a".into()],
          add: vec![],
        },
      ),
      (vec!["a".into()], vec!["a".into()], Default::default()),
      (
        vec!["a".into(), "b".into()],
        vec!["a".into()],
        Diff {
          remove: vec!["b".into()],
          add: vec![],
        },
      ),
      (
        vec!["a".into(), "b".into()],
        vec!["a".into(), "c".into()],
        Diff {
          remove: vec!["b".into()],
          add: vec!["c".into()],
        },
      ),
    ] {
      assert_eq!(super::features_diff(&current, &expected), result);
    }
  }
}
