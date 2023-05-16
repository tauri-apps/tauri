// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::{var, var_os},
  fs::{copy, create_dir, create_dir_all, read_dir, read_to_string, remove_dir_all, write, File},
  io::Write,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub(crate) struct PluginMetadata {
  pub path: PathBuf,
}

#[derive(Default)]
pub struct PluginBuilder {
  android_path: Option<PathBuf>,
  ios_path: Option<PathBuf>,
}

impl PluginBuilder {
  /// Creates a new builder for mobile plugin functionality.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the Android project path.
  pub fn android_path<P: Into<PathBuf>>(mut self, android_path: P) -> Self {
    self.android_path.replace(android_path.into());
    self
  }

  /// Sets the iOS project path.
  pub fn ios_path<P: Into<PathBuf>>(mut self, ios_path: P) -> Self {
    self.ios_path.replace(ios_path.into());
    self
  }

  /// Injects the mobile templates in the given path relative to the manifest root.
  pub fn run(self) -> Result<()> {
    let target_os = var("CARGO_CFG_TARGET_OS").unwrap();
    let mobile = target_os == "android" || target_os == "ios";
    crate::cfg_alias("mobile", mobile);
    crate::cfg_alias("desktop", !mobile);

    match target_os.as_str() {
      "android" => {
        if let Some(path) = self.android_path {
          let manifest_dir = var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
          let source = manifest_dir.join(path);

          let tauri_library_path = std::env::var("DEP_TAURI_ANDROID_LIBRARY_PATH")
            .expect("missing `DEP_TAURI_ANDROID_LIBRARY_PATH` environment variable. Make sure `tauri` is a dependency of the plugin.");
          println!("cargo:rerun-if-env-changed=DEP_TAURI_ANDROID_LIBRARY_PATH");

          create_dir_all(source.join(".tauri")).context("failed to create .tauri directory")?;
          copy_folder(
            Path::new(&tauri_library_path),
            &source.join(".tauri").join("tauri-api"),
            &[],
          )
          .context("failed to copy tauri-api to the plugin project")?;

          if let Some(project_dir) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
            let pkg_name = var("CARGO_PKG_NAME").unwrap();
            println!("cargo:rerun-if-env-changed=TAURI_ANDROID_PROJECT_PATH");

            let plugins_json_path = project_dir.join(".tauri").join("plugins").join(pkg_name);

            let mut file = File::create(&plugins_json_path)?;
            file.write_all(source.display().to_string().as_bytes())?;
            file.flush()?;
            file.sync_data()?;

            println!("cargo:rerun-if-changed={}", plugins_json_path.display());
          }
        }
      }
      #[cfg(target_os = "macos")]
      "ios" => {
        if let Some(path) = self.ios_path {
          let manifest_dir = var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
          let tauri_library_path = std::env::var("DEP_TAURI_IOS_LIBRARY_PATH")
            .expect("missing `DEP_TAURI_IOS_LIBRARY_PATH` environment variable. Make sure `tauri` is a dependency of the plugin.");

          let tauri_dep_path = path.parent().unwrap().join(".tauri");
          create_dir_all(&tauri_dep_path).context("failed to create .tauri directory")?;
          copy_folder(
            Path::new(&tauri_library_path),
            &tauri_dep_path.join("tauri-api"),
            &[".build", "Package.resolved", "Tests"],
          )
          .context("failed to copy tauri-api to the plugin project")?;
          link_swift_library(&var("CARGO_PKG_NAME").unwrap(), manifest_dir.join(path));
        }
      }
      _ => (),
    }

    Ok(())
  }
}

#[cfg(target_os = "macos")]
#[doc(hidden)]
pub fn link_swift_library(name: &str, source: impl AsRef<Path>) {
  let source = source.as_ref();

  let sdk_root = std::env::var_os("SDKROOT");
  std::env::remove_var("SDKROOT");

  swift_rs::SwiftLinker::new(
    &std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "10.13".into()),
  )
  .with_ios(&std::env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".into()))
  .with_package(name, source)
  .link();

  if let Some(root) = sdk_root {
    std::env::set_var("SDKROOT", root);
  }
}

fn copy_folder(source: &Path, target: &Path, ignore_paths: &[&str]) -> Result<()> {
  let _ = remove_dir_all(target);

  for entry in walkdir::WalkDir::new(source) {
    let entry = entry?;
    let rel_path = entry.path().strip_prefix(source)?;
    let rel_path_str = rel_path.to_string_lossy();
    if ignore_paths
      .iter()
      .any(|path| rel_path_str.starts_with(path))
    {
      continue;
    }
    let dest_path = target.join(rel_path);

    if entry.file_type().is_dir() {
      create_dir(&dest_path)
        .with_context(|| format!("failed to create directory {}", dest_path.display()))?;
    } else {
      copy(entry.path(), &dest_path).with_context(|| {
        format!(
          "failed to copy {} to {}",
          entry.path().display(),
          dest_path.display()
        )
      })?;
      println!("cargo:rerun-if-changed={}", entry.path().display());
    }
  }

  Ok(())
}

pub(crate) fn generate_gradle_files(project_dir: PathBuf) -> Result<()> {
  let gradle_settings_path = project_dir.join("tauri.settings.gradle");
  let app_build_gradle_path = project_dir.join("app").join("tauri.build.gradle.kts");

  let mut gradle_settings =
    "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n".to_string();
  let mut app_build_gradle = "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
val implementation by configurations
dependencies {"
    .to_string();

  let plugin_output_folder = project_dir.join(".tauri").join("plugins");

  for entry in read_dir(&plugin_output_folder)? {
    let entry = entry?;
    let plugin_name = entry
      .path()
      .file_name()
      .unwrap()
      .to_string_lossy()
      .into_owned();
    let plugin_path = read_to_string(entry.path())?;

    gradle_settings.push_str(&format!("include ':{plugin_name}'"));
    gradle_settings.push('\n');
    gradle_settings.push_str(&format!(
      "project(':{plugin_name}').projectDir = new File({:?})",
      tauri_utils::display_path(plugin_path)
    ));
    gradle_settings.push('\n');

    app_build_gradle.push('\n');
    app_build_gradle.push_str(&format!(r#"  implementation(project(":{plugin_name}"))"#));
  }

  app_build_gradle.push_str("\n}");

  write(&gradle_settings_path, gradle_settings).context("failed to write tauri.settings.gradle")?;

  write(&app_build_gradle_path, app_build_gradle)
    .context("failed to write tauri.build.gradle.kts")?;

  println!("cargo:rerun-if-changed={}", gradle_settings_path.display());
  println!("cargo:rerun-if-changed={}", app_build_gradle_path.display());
  println!("cargo:rerun-if-changed={}", plugin_output_folder.display());

  Ok(())
}
