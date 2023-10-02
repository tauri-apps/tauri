// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::{var, var_os},
  fs::{copy, create_dir, create_dir_all, read_to_string, remove_dir_all, write},
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

          println!("cargo:android_library_path={}", source.display());
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

#[cfg(target_os = "macos")]
fn update_plist_file<P: AsRef<Path>, F: FnOnce(&mut plist::Dictionary)>(
  path: P,
  f: F,
) -> Result<()> {
  use std::io::Cursor;

  let path = path.as_ref();
  if path.exists() {
    let plist_str = read_to_string(path)?;
    let mut plist = plist::Value::from_reader(Cursor::new(&plist_str))?;
    if let Some(dict) = plist.as_dictionary_mut() {
      f(dict);
      let mut plist_buf = Vec::new();
      let writer = Cursor::new(&mut plist_buf);
      plist::to_writer_xml(writer, &plist)?;
      let new_plist_str = String::from_utf8(plist_buf)?;
      if new_plist_str != plist_str {
        write(path, new_plist_str)?;
      }
    }
  }

  Ok(())
}

#[cfg(target_os = "macos")]
pub fn update_entitlements<F: FnOnce(&mut plist::Dictionary)>(f: F) -> Result<()> {
  if let (Some(project_path), Ok(app_name)) = (
    var_os("TAURI_IOS_PROJECT_PATH").map(PathBuf::from),
    var("TAURI_IOS_APP_NAME"),
  ) {
    update_plist_file(
      project_path
        .join(format!("{app_name}_iOS"))
        .join(format!("{app_name}_iOS.entitlements")),
      f,
    )?;
  }

  Ok(())
}

fn xml_block_comment(id: &str) -> String {
  format!("<!-- {id}. AUTO-GENERATED. DO NOT REMOVE. -->")
}

fn insert_into_xml(xml: &str, block_identifier: &str, parent_tag: &str, contents: &str) -> String {
  let block_comment = xml_block_comment(block_identifier);

  let mut rewritten = Vec::new();
  let mut found_block = false;
  let parent_closing_tag = format!("</{parent_tag}>");
  for line in xml.split('\n') {
    if line.contains(&block_comment) {
      found_block = !found_block;
      continue;
    }

    // found previous block which should be removed
    if found_block {
      continue;
    }

    if let Some(index) = line.find(&parent_closing_tag) {
      let identation = " ".repeat(index + 4);
      rewritten.push(format!("{}{}", identation, block_comment));
      for l in contents.split('\n') {
        rewritten.push(format!("{}{}", identation, l));
      }
      rewritten.push(format!("{}{}", identation, block_comment));
    }

    rewritten.push(line.to_string());
  }

  rewritten.join("\n")
}

pub fn update_android_manifest(block_identifier: &str, parent: &str, insert: String) -> Result<()> {
  if let Some(project_path) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
    let manifest_path = project_path.join("app/src/main/AndroidManifest.xml");
    let manifest = read_to_string(&manifest_path)?;
    let rewritten = insert_into_xml(&manifest, block_identifier, parent, &insert);
    if rewritten != manifest {
      write(manifest_path, rewritten)?;
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

  for (env, value) in std::env::vars_os() {
    let env = env.to_string_lossy();
    if env.starts_with("DEP_") && env.ends_with("_ANDROID_LIBRARY_PATH") {
      let name_len = env.len() - "DEP_".len() - "_ANDROID_LIBRARY_PATH".len();
      let mut plugin_name = env
        .chars()
        .skip("DEP_".len())
        .take(name_len)
        .collect::<String>()
        .to_lowercase()
        .replace('_', "-");
      if plugin_name == "tauri" {
        plugin_name = "tauri-android".into();
      }
      let plugin_path = PathBuf::from(value);

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
  }

  app_build_gradle.push_str("\n}");

  write(&gradle_settings_path, gradle_settings).context("failed to write tauri.settings.gradle")?;

  write(&app_build_gradle_path, app_build_gradle)
    .context("failed to write tauri.build.gradle.kts")?;

  println!("cargo:rerun-if-changed={}", gradle_settings_path.display());
  println!("cargo:rerun-if-changed={}", app_build_gradle_path.display());

  Ok(())
}

#[cfg(test)]
mod tests {
  #[test]
  fn insert_into_xml() {
    let manifest = r#"<manifest>
    <application>
        <intent-filter>
        </intent-filter>
    </application>
</manifest>"#;
    let id = "tauritest";
    let new = super::insert_into_xml(manifest, id, "application", "<something></something>");

    let block_id_comment = super::xml_block_comment(id);
    let expected = format!(
      r#"<manifest>
    <application>
        <intent-filter>
        </intent-filter>
        {block_id_comment}
        <something></something>
        {block_id_comment}
    </application>
</manifest>"#
    );

    assert_eq!(new, expected);

    // assert it's still the same after an empty update
    let new = super::insert_into_xml(&expected, id, "application", "<something></something>");
    assert_eq!(new, expected);
  }
}
