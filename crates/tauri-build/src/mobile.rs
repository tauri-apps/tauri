// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, path::PathBuf};

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

  let mut gradle_settings =
    "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n".to_string();
  let mut app_build_gradle = "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
val implementation by configurations
dependencies {
  implementation(\"androidx.lifecycle:lifecycle-process:2.10.0\")"
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

  // Overwrite only if changed to not trigger rebuilds
  write_if_changed(&gradle_settings_path, gradle_settings)
    .context("failed to write tauri.settings.gradle")?;

  write_if_changed(&app_build_gradle_path, app_build_gradle)
    .context("failed to write tauri.build.gradle.kts")?;

  println!("cargo:rerun-if-changed={}", gradle_settings_path.display());
  println!("cargo:rerun-if-changed={}", app_build_gradle_path.display());

  Ok(())
}
