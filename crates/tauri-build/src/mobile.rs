// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{BTreeMap, HashSet},
  path::PathBuf,
};

use anyhow::{Context, Result};
use tauri_utils::{config::AndroidIntentAction, write_if_changed};

/// Updates the Android manifest to add file association intent filters
pub fn update_android_manifest_file_associations(
  associations: &[tauri_utils::config::FileAssociation],
) -> Result<()> {
  if associations.is_empty() {
    return Ok(());
  }

  let intent_filters = generate_file_association_intent_filters(associations);
  tauri_utils::build::update_android_manifest("tauri-file-associations", "activity", intent_filters)
}

fn generate_file_association_intent_filters(
  associations: &[tauri_utils::config::FileAssociation],
) -> String {
  let mut filters = String::new();

  for association in associations {
    // Get mime types - use explicit mime_type, or infer from extensions
    let mut mime_types = HashSet::new();

    if let Some(mime_type) = &association.mime_type {
      mime_types.insert((
        mime_type.clone(),
        association.android_intent_action_filters.clone(),
      ));
    } else {
      // Infer mime types from extensions
      for ext in &association.ext {
        if let Some(mime) = extension_to_mime_type(&ext.0) {
          mime_types.insert((mime, association.android_intent_action_filters.clone()));
        }
      }
    }

    // If we have mime types, create intent filters
    if !mime_types.is_empty() {
      for (mime_type, actions) in &mime_types {
        filters.push_str("<intent-filter>\n");
        if let Some(actions) = actions {
          for action in actions {
            let action = match action {
              AndroidIntentAction::Send => "SEND",
              AndroidIntentAction::SendMultiple => "SEND_MULTIPLE",
              AndroidIntentAction::View => "VIEW",
              _ => unimplemented!(),
            };
            filters.push_str(&format!(
              "    <action android:name=\"android.intent.action.{action}\" />\n"
            ));
          }
        } else {
          filters.push_str("    <action android:name=\"android.intent.action.SEND\" />\n");
          filters.push_str("    <action android:name=\"android.intent.action.SEND_MULTIPLE\" />\n");
          filters.push_str("    <action android:name=\"android.intent.action.VIEW\" />\n");
        }
        filters.push_str("    <category android:name=\"android.intent.category.DEFAULT\" />\n");
        filters.push_str("    <category android:name=\"android.intent.category.BROWSABLE\" />\n");
        filters.push_str(&format!(
          "    <data android:mimeType=\"{}\" />\n",
          mime_type
        ));

        // Add file scheme and path patterns for extensions
        if !association.ext.is_empty() {
          // Create path patterns for each extension
          // Android's pathPattern needs \\. (double backslash-dot) in XML to match a literal dot
          let path_patterns: Vec<String> = association
            .ext
            .iter()
            .map(|ext| format!(".*\\\\.{}", ext.0))
            .collect();

          for pattern in &path_patterns {
            filters.push_str(&format!(
              "    <data android:pathPattern=\"{}\" />\n",
              pattern
            ));
          }
        }

        filters.push_str("</intent-filter>\n");
      }
    } else if !association.ext.is_empty() {
      // If no mime type but we have extensions, use a generic approach
      filters.push_str("<intent-filter>\n");
      filters.push_str("    <action android:name=\"android.intent.action.VIEW\" />\n");
      filters.push_str("    <category android:name=\"android.intent.category.DEFAULT\" />\n");
      filters.push_str("    <category android:name=\"android.intent.category.BROWSABLE\" />\n");

      for ext in &association.ext {
        // Android's pathPattern needs \\. (double backslash-dot) in XML to match a literal dot
        filters.push_str(&format!(
          "    <data android:pathPattern=\".*\\\\.{}\" />\n",
          ext.0
        ));
      }

      filters.push_str("</intent-filter>\n");
    }
  }

  filters
}

fn extension_to_mime_type(ext: &str) -> Option<String> {
  Some(
    match ext.to_lowercase().as_str() {
      "png" => "image/png",
      "jpg" | "jpeg" => "image/jpeg",
      "gif" => "image/gif",
      "bmp" => "image/bmp",
      "webp" => "image/webp",
      "svg" => "image/svg+xml",
      "ico" => "image/x-icon",
      "tiff" | "tif" => "image/tiff",
      "heic" | "heif" => "image/heic",
      "mp4" => "video/mp4",
      "mov" => "video/quicktime",
      "avi" => "video/x-msvideo",
      "mkv" => "video/x-matroska",
      "mp3" => "audio/mpeg",
      "wav" => "audio/wav",
      "aac" => "audio/aac",
      "m4a" => "audio/mp4",
      "pdf" => "application/pdf",
      "txt" => "text/plain",
      "html" | "htm" => "text/html",
      "json" => "application/json",
      "xml" => "application/xml",
      "rtf" => "application/rtf",
      _ => return None,
    }
    .to_string(),
  )
}

pub fn generate_gradle_files(project_dir: PathBuf) -> Result<()> {
  let gradle_settings_path = project_dir.join("tauri.settings.gradle");
  let app_build_gradle_path = project_dir.join("app").join("tauri.build.gradle.kts");

  // Multiple build scripts write the same android project files: the app crate
  // AND any dependency crate that also calls tauri_build::build() (e.g. an
  // embedded reader lib like `readestlib`). Each script only sees its OWN
  // direct plugin deps via the DEP_*_ANDROID_LIBRARY_PATH env vars, so the last
  // writer would silently drop the other crate's plugin modules — the app then
  // crashes at startup with ClassNotFoundException for the missing plugin's
  // Kotlin class (e.g. app.tauri.dialog.DialogPlugin). MERGE with the existing
  // files instead, so the result is the union of every crate's plugins.
  let mut plugin_paths = read_android_plugin_paths(&gradle_settings_path);

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
      plugin_paths
        .entry(plugin_name)
        .or_insert_with(|| PathBuf::from(value));
    }
  }

  let mut gradle_settings =
    "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n".to_string();
  let mut app_build_gradle = "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
val implementation by configurations
dependencies {
  implementation(\"androidx.lifecycle:lifecycle-process:2.10.0\")"
    .to_string();

  for (plugin_name, plugin_path) in &plugin_paths {
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

  // Overwrite only if changed to not trigger rebuilds
  write_if_changed(&gradle_settings_path, gradle_settings)
    .context("failed to write tauri.settings.gradle")?;

  write_if_changed(&app_build_gradle_path, app_build_gradle)
    .context("failed to write tauri.build.gradle.kts")?;

  println!("cargo:rerun-if-changed={}", gradle_settings_path.display());
  println!("cargo:rerun-if-changed={}", app_build_gradle_path.display());

  Ok(())
}

/// Parse the plugin modules already listed in a generated `tauri.settings.gradle`
/// file, mapped from the Gradle module name to its android library directory.
///
/// Format of each entry:
/// ```text
/// include ':tauri-plugin-fs'
/// project(':tauri-plugin-fs').projectDir = new File("C:\\...\\android")
/// ```
fn read_android_plugin_paths(path: &PathBuf) -> BTreeMap<String, PathBuf> {
  let Ok(content) = std::fs::read_to_string(path) else {
    return BTreeMap::new();
  };
  parse_android_plugin_paths(&content)
}

/// Parse the `project(':name').projectDir = new File("path")` lines from the
/// content of a generated `tauri.settings.gradle` file. Lines that do not
/// match (headers, `include` lines, malformed entries) are ignored.
fn parse_android_plugin_paths(content: &str) -> BTreeMap<String, PathBuf> {
  let mut plugins = BTreeMap::new();

  for line in content.lines() {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("project(':") {
      // rest: NAME').projectDir = new File("PATH")
      if let Some((name, path)) = rest.split_once("').projectDir = new File(\"") {
        let path = path.strip_suffix("\")").unwrap_or(path);
        plugins.insert(name.to_string(), PathBuf::from(path));
      }
    }
  }

  plugins
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_plugin_entries_with_windows_paths() {
    let content = r#"
// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
include ':tauri-plugin-fs'
project(':tauri-plugin-fs').projectDir = new File("C:\work\vendor\tauri-plugins\plugins\fs\android")
include ':tauri-plugin-http'
project(':tauri-plugin-http').projectDir = new File("C:\work\vendor\tauri-plugins\plugins\http\android")
"#;

    let plugins = parse_android_plugin_paths(content);

    assert_eq!(plugins.len(), 2);
    assert_eq!(
      plugins.get("tauri-plugin-fs").unwrap(),
      &PathBuf::from(r"C:\work\vendor\tauri-plugins\plugins\fs\android")
    );
    assert_eq!(
      plugins.get("tauri-plugin-http").unwrap(),
      &PathBuf::from(r"C:\work\vendor\tauri-plugins\plugins\http\android")
    );
  }

  #[test]
  fn parses_unix_style_paths() {
    let content = "project(':tauri-plugin-shell').projectDir = new File(\"/home/user/tauri-plugins/shell/android\")\n";
    let plugins = parse_android_plugin_paths(content);
    assert_eq!(
      plugins.get("tauri-plugin-shell").unwrap(),
      &PathBuf::from("/home/user/tauri-plugins/shell/android")
    );
  }

  #[test]
  fn ignores_header_include_lines_and_malformed_entries() {
    let header = "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n";
    let include = "include ':tauri-android'\n";
    let malformed = "project(':x').projectDir = new File(unquoted)\n";
    let plugins = parse_android_plugin_paths(&format!("{header}{include}{malformed}"));
    assert!(plugins.is_empty());
  }

  #[test]
  fn returns_empty_map_for_empty_content() {
    assert!(parse_android_plugin_paths("").is_empty());
  }
}
