// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashSet,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use tauri_utils::{
  config::{
    ActivityEmbeddingConfig, AndroidIntentAction, EmbeddingAspectRatio, SplitFinishBehavior,
    SplitLayoutDirection, SplitPairRule, SplitType,
  },
  write_if_changed,
};

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

pub fn generate_gradle_files(
  project_dir: PathBuf,
  activity_embedding: Option<&ActivityEmbeddingConfig>,
) -> Result<()> {
  let gradle_settings_path = project_dir.join("tauri.settings.gradle");
  let app_build_gradle_path = project_dir.join("app").join("tauri.build.gradle.kts");

  let mut gradle_settings =
    "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n".to_string();
  let mut app_build_gradle = "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
val implementation by configurations
dependencies {
  implementation(\"androidx.lifecycle:lifecycle-process:2.10.0\")"
    .to_string();

  if activity_embedding.is_some() {
    app_build_gradle.push_str("\n  implementation(\"androidx.window:window:1.5.0\")");
    app_build_gradle.push_str("\n  implementation(\"androidx.startup:startup-runtime:1.2.0\")");
  }

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

  write_if_changed(&gradle_settings_path, gradle_settings)
    .context("failed to write tauri.settings.gradle")?;

  write_if_changed(&app_build_gradle_path, app_build_gradle)
    .context("failed to write tauri.build.gradle.kts")?;

  println!("cargo:rerun-if-changed={}", gradle_settings_path.display());
  println!("cargo:rerun-if-changed={}", app_build_gradle_path.display());

  Ok(())
}

/// Configures Android Activity Embedding: updates the manifest, generates Gradle
/// dependencies, and writes the `TauriSplitInitializer.kt` file.
pub fn setup_activity_embedding(
  project_dir: &Path,
  config: &ActivityEmbeddingConfig,
  identifier: &str,
) -> Result<()> {
  if config.split_rules.is_empty() {
    return Ok(());
  }

  let package_name = identifier_to_android_package(identifier);

  update_android_manifest_activity_embedding(config, &package_name)?;
  generate_split_initializer(project_dir, config, &package_name)?;

  Ok(())
}

fn identifier_to_android_package(identifier: &str) -> String {
  identifier
    .split('.')
    .map(|s| s.replace('-', "_"))
    .collect::<Vec<_>>()
    .join(".")
}

fn update_android_manifest_activity_embedding(
  config: &ActivityEmbeddingConfig,
  package_name: &str,
) -> Result<()> {
  let mut xml = String::new();

  xml.push_str(
    "<property\n    \
       android:name=\"android.window.PROPERTY_ACTIVITY_EMBEDDING_SPLITS_ENABLED\"\n    \
       android:value=\"true\" />\n",
  );

  xml.push_str(&format!(
    "<provider\n    \
       android:name=\"androidx.startup.InitializationProvider\"\n    \
       android:authorities=\"${{applicationId}}.androidx-startup\"\n    \
       android:exported=\"false\">\n    \
       <meta-data\n        \
         android:name=\"{package_name}.TauriSplitInitializer\"\n        \
         android:value=\"androidx.startup\" />\n\
     </provider>\n"
  ));

  let mut declared = HashSet::new();
  for rule in &config.split_rules {
    let name = if rule.secondary.contains('.') {
      rule.secondary.clone()
    } else {
      format!(".{}", rule.secondary)
    };
    if !declared.insert(name.clone()) {
      continue;
    }
    xml.push_str(&format!(
      "<activity\n    \
         android:name=\"{name}\"\n    \
         android:exported=\"false\"\n    \
         android:configChanges=\"orientation|screenSize|screenLayout|smallestScreenSize\" />\n"
    ));
  }

  tauri_utils::build::update_android_manifest("tauri-activity-embedding", "application", xml)
}

fn generate_split_initializer(
  project_dir: &Path,
  config: &ActivityEmbeddingConfig,
  package_name: &str,
) -> Result<()> {
  let package_dir = package_name.replace('.', "/");
  let source_dir = project_dir
    .join("app/src/main/java")
    .join(&package_dir)
    .join("generated");
  std::fs::create_dir_all(&source_dir)
    .context("failed to create Android source directory for split initializer")?;

  let mut kt = String::new();
  kt.push_str(&format!(
    "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\npackage {package_name}\n\n"
  ));
  kt.push_str(
    "import android.content.ComponentName\n\
     import android.content.Context\n\
     import androidx.startup.Initializer\n\
     import androidx.window.embedding.EmbeddingAspectRatio\n\
     import androidx.window.embedding.RuleController\n\
     import androidx.window.embedding.SplitAttributes\n\
     import androidx.window.embedding.SplitPairFilter\n\
     import androidx.window.embedding.SplitPairRule\n\
     import androidx.window.embedding.SplitRule\n\n",
  );
  kt.push_str(
    "class TauriSplitInitializer : Initializer<RuleController> {\n    \
       override fun create(context: Context): RuleController {\n        \
         val ruleController = RuleController.getInstance(context)\n\n",
  );

  for (i, rule) in config.split_rules.iter().enumerate() {
    append_split_rule(&mut kt, i, rule, package_name);
  }

  kt.push_str(
    "        return ruleController\n    \
     }\n\n    \
     override fun dependencies(): List<Class<out Initializer<*>>> = emptyList()\n\
     }\n",
  );

  let initializer_path = source_dir.join("TauriSplitInitializer.kt");
  write_if_changed(&initializer_path, kt).context("failed to write TauriSplitInitializer.kt")?;

  Ok(())
}

fn append_split_rule(kt: &mut String, i: usize, rule: &SplitPairRule, package_name: &str) {
  let primary = qualify_activity(&rule.primary, package_name);
  let secondary = qualify_activity(&rule.secondary, package_name);
  let secondary_intent_action = rule
    .secondary_intent_action
    .as_deref()
    .map(|s| format!("{s:?}"))
    .unwrap_or_else(|| "null".to_string());

  kt.push_str(&format!(
    "        val filter{i} = SplitPairFilter(\n            \
       ComponentName(context, {primary}::class.java),\n            \
       ComponentName(context, {secondary}::class.java),\n            \
       {secondary_intent_action}\n        \
     )\n"
  ));

  kt.push_str(&format!(
    "        val attrsBuilder{i} = SplitAttributes.Builder()\n"
  ));
  if let Some(split_type) = &rule.split_type {
    kt.push_str(&format!(
      "        attrsBuilder{i}.setSplitType({})\n",
      kotlin_split_type(split_type)
    ));
  }
  if let Some(direction) = rule.layout_direction {
    kt.push_str(&format!(
      "        attrsBuilder{i}.setLayoutDirection({})\n",
      kotlin_layout_direction(direction)
    ));
  }
  kt.push_str(&format!("        val attrs{i} = attrsBuilder{i}.build()\n"));

  kt.push_str(&format!(
    "        val ruleBuilder{i} = SplitPairRule.Builder(setOf(filter{i}))\n        \
       ruleBuilder{i}.setDefaultSplitAttributes(attrs{i})\n"
  ));

  if let Some(dp) = rule.min_width_dp {
    kt.push_str(&format!("        ruleBuilder{i}.setMinWidthDp({dp})\n"));
  }
  if let Some(dp) = rule.min_height_dp {
    kt.push_str(&format!("        ruleBuilder{i}.setMinHeightDp({dp})\n"));
  }
  if let Some(dp) = rule.min_smallest_width_dp {
    kt.push_str(&format!(
      "        ruleBuilder{i}.setMinSmallestWidthDp({dp})\n"
    ));
  }
  if let Some(ar) = &rule.max_aspect_ratio_in_portrait {
    kt.push_str(&format!(
      "        ruleBuilder{i}.setMaxAspectRatioInPortrait({})\n",
      kotlin_aspect_ratio(ar)
    ));
  }
  if let Some(ar) = &rule.max_aspect_ratio_in_landscape {
    kt.push_str(&format!(
      "        ruleBuilder{i}.setMaxAspectRatioInLandscape({})\n",
      kotlin_aspect_ratio(ar)
    ));
  }
  if let Some(b) = rule.finish_primary_with_secondary {
    kt.push_str(&format!(
      "        ruleBuilder{i}.setFinishPrimaryWithSecondary({})\n",
      kotlin_finish_behavior(b)
    ));
  }
  if let Some(b) = rule.finish_secondary_with_primary {
    kt.push_str(&format!(
      "        ruleBuilder{i}.setFinishSecondaryWithPrimary({})\n",
      kotlin_finish_behavior(b)
    ));
  }
  if let Some(clear_top) = rule.clear_top {
    kt.push_str(&format!(
      "        ruleBuilder{i}.setClearTop({clear_top})\n"
    ));
  }
  if let Some(tag) = &rule.tag {
    kt.push_str(&format!("        ruleBuilder{i}.setTag({tag:?})\n"));
  }

  kt.push_str(&format!(
    "        val rule{i} = ruleBuilder{i}.build()\n        \
       ruleController.addRule(rule{i})\n\n"
  ));
}

fn qualify_activity(name: &str, package_name: &str) -> String {
  if name.contains('.') {
    name.to_string()
  } else {
    format!("{package_name}.{name}")
  }
}

fn kotlin_split_type(split_type: &SplitType) -> String {
  match split_type {
    SplitType::Ratio(r) => format!("SplitAttributes.SplitType.ratio({r}f)"),
    SplitType::Expand => "SplitAttributes.SplitType.SPLIT_TYPE_EXPAND".to_string(),
    SplitType::Hinge => "SplitAttributes.SplitType.SPLIT_TYPE_HINGE".to_string(),
  }
}

fn kotlin_layout_direction(direction: SplitLayoutDirection) -> &'static str {
  match direction {
    SplitLayoutDirection::Locale => "SplitAttributes.LayoutDirection.LOCALE",
    SplitLayoutDirection::LeftToRight => "SplitAttributes.LayoutDirection.LEFT_TO_RIGHT",
    SplitLayoutDirection::RightToLeft => "SplitAttributes.LayoutDirection.RIGHT_TO_LEFT",
    SplitLayoutDirection::TopToBottom => "SplitAttributes.LayoutDirection.TOP_TO_BOTTOM",
    SplitLayoutDirection::BottomToTop => "SplitAttributes.LayoutDirection.BOTTOM_TO_TOP",
  }
}

fn kotlin_finish_behavior(behavior: SplitFinishBehavior) -> &'static str {
  match behavior {
    SplitFinishBehavior::Never => "SplitRule.FinishBehavior.NEVER",
    SplitFinishBehavior::Always => "SplitRule.FinishBehavior.ALWAYS",
    SplitFinishBehavior::Adjacent => "SplitRule.FinishBehavior.ADJACENT",
  }
}

fn kotlin_aspect_ratio(ar: &EmbeddingAspectRatio) -> String {
  match ar {
    EmbeddingAspectRatio::AlwaysAllow => "EmbeddingAspectRatio.ALWAYS_ALLOW".to_string(),
    EmbeddingAspectRatio::AlwaysDisallow => "EmbeddingAspectRatio.ALWAYS_DISALLOW".to_string(),
    EmbeddingAspectRatio::Ratio(r) => format!("EmbeddingAspectRatio.ratio({r}f)"),
  }
}
