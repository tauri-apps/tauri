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
val implementation = configurations.getByName(\"implementation\")
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

/// Identifier of the auto-generated Android manifest block for activity embedding.
const MANIFEST_BLOCK_ID: &str = "tauri-activity-embedding";
/// First line of every Kotlin source generated by this module, used to recognize stale files.
const GENERATED_HEADER: &str =
  "// THIS IS AN AUTOGENERATED FILE BY TAURI ACTIVITY EMBEDDING. DO NOT EDIT THIS FILE DIRECTLY.";
const SPLIT_INITIALIZER_CLASS: &str = "TauriSplitInitializer";
/// Same `configChanges` set the `MainActivity` template uses, so secondary activities
/// (and their webviews) are not recreated on dark mode, keyboard or locale changes.
const ACTIVITY_CONFIG_CHANGES: &str =
  "orientation|keyboardHidden|keyboard|screenSize|locale|smallestScreenSize|screenLayout|uiMode";

/// The app's Android package name as used by Kotlin sources and by the manifest.
struct AndroidPackage {
  /// Package with Kotlin-only keywords escaped with backticks (`package` statements, class references).
  kotlin: String,
  /// Raw package name (manifest entries, file system paths).
  unescaped: String,
}

impl AndroidPackage {
  /// Uses the package names exported by the Tauri CLI, deriving them from the identifier when unset.
  fn resolve(identifier: &str) -> Self {
    let kotlin = std::env::var("WRY_ANDROID_PACKAGE").ok();
    let unescaped = std::env::var("TAURI_ANDROID_PACKAGE_UNESCAPED")
      .ok()
      .or_else(|| kotlin.as_ref().map(|p| p.replace('`', "")))
      .unwrap_or_else(|| identifier.replace('-', "_"));
    Self {
      kotlin: kotlin.unwrap_or_else(|| unescaped.clone()),
      unescaped,
    }
  }
}

/// Synchronizes the Android project with the `bundle > android > activityEmbedding` configuration.
///
/// When `config` is enabled and has split rules, this validates it, writes
/// `TauriSplitInitializer.kt` plus a default `TauriActivity` subclass for each secondary
/// activity the app sources do not define, and declares the secondary activities in the manifest.
/// Otherwise every previously generated file and manifest entry is removed.
pub fn sync_activity_embedding(
  project_dir: &Path,
  config: Option<&ActivityEmbeddingConfig>,
  identifier: &str,
) -> Result<()> {
  let config = config.filter(|c| c.enabled && !c.split_rules.is_empty());
  let package = AndroidPackage::resolve(identifier);
  let out_dir = kotlin_out_dir(project_dir, &package);

  let mut sources: Vec<(String, String)> = Vec::new();
  let mut manifest_xml = String::new();

  if let Some(config) = config {
    validate_activity_embedding(config)?;

    let manifest_path = project_dir.join("app/src/main/AndroidManifest.xml");
    let manifest = if manifest_path.exists() {
      std::fs::read_to_string(&manifest_path).context("failed to read AndroidManifest.xml")?
    } else {
      String::new()
    };
    let user_manifest = tauri_utils::build::remove_xml_block(&manifest, MANIFEST_BLOCK_ID);

    manifest_xml = activity_embedding_manifest_xml(config, &package.unescaped, &user_manifest);
    sources.push((
      format!("{SPLIT_INITIALIZER_CLASS}.kt"),
      split_initializer_kotlin(config, &package.kotlin),
    ));
    for name in unique_secondaries(config) {
      if !name.contains('.') && !activity_source_exists(project_dir, &package.unescaped, name) {
        sources.push((
          format!("{name}.kt"),
          default_activity_kotlin(name, &package.kotlin),
        ));
      }
    }
  }

  write_generated_sources(&out_dir, &sources)?;
  tauri_utils::build::update_android_manifest(MANIFEST_BLOCK_ID, "application", manifest_xml)
}

fn kotlin_out_dir(project_dir: &Path, package: &AndroidPackage) -> PathBuf {
  std::env::var_os("WRY_ANDROID_KOTLIN_FILES_OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| {
      package_source_dir(project_dir, "java", &package.unescaped).join("generated")
    })
}

fn package_source_dir(project_dir: &Path, source_root: &str, package: &str) -> PathBuf {
  project_dir
    .join("app/src/main")
    .join(source_root)
    .join(package.replace('.', "/"))
}

/// Whether the app sources define the activity class in the app package.
fn activity_source_exists(project_dir: &Path, package: &str, name: &str) -> bool {
  ["java", "kotlin"].iter().any(|root| {
    let dir = package_source_dir(project_dir, root, package);
    dir.join(format!("{name}.kt")).exists() || dir.join(format!("{name}.java")).exists()
  })
}

/// Writes the generated Kotlin sources and deletes previously generated ones that are no longer needed.
fn write_generated_sources(out_dir: &Path, sources: &[(String, String)]) -> Result<()> {
  if sources.is_empty() && !out_dir.exists() {
    return Ok(());
  }

  std::fs::create_dir_all(out_dir)
    .context("failed to create Android generated sources directory")?;

  for (file_name, contents) in sources {
    write_if_changed(out_dir.join(file_name), contents)
      .with_context(|| format!("failed to write {file_name}"))?;
  }

  for entry in
    std::fs::read_dir(out_dir).context("failed to read Android generated sources directory")?
  {
    let path = entry?.path();
    let is_current = path
      .file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|file_name| sources.iter().any(|(name, _)| name == file_name));
    if is_current || !path.extension().is_some_and(|ext| ext == "kt") {
      continue;
    }
    let generated = std::fs::read_to_string(&path)
      .is_ok_and(|contents| contents.lines().next() == Some(GENERATED_HEADER));
    if generated {
      std::fs::remove_file(&path)
        .with_context(|| format!("failed to remove stale {}", path.display()))?;
    }
  }

  Ok(())
}

/// Secondary activity names in configuration order, without duplicates.
fn unique_secondaries(config: &ActivityEmbeddingConfig) -> Vec<&str> {
  let mut seen = HashSet::new();
  config
    .split_rules
    .iter()
    .map(|rule| rule.secondary.as_str())
    .filter(|name| seen.insert(*name))
    .collect()
}

fn validate_activity_embedding(config: &ActivityEmbeddingConfig) -> Result<()> {
  for (i, rule) in config.split_rules.iter().enumerate() {
    let context = || format!("invalid `bundle > android > activityEmbedding > splitRules[{i}]`");

    validate_class_name(&rule.primary, "primary").with_context(context)?;
    validate_class_name(&rule.secondary, "secondary").with_context(context)?;

    if let Some(SplitType::Ratio(ratio)) = &rule.split_type {
      anyhow::ensure!(
        *ratio > 0.0 && *ratio < 1.0,
        "{}: `splitType.ratio` must be greater than 0.0 and less than 1.0, got {ratio}",
        context()
      );
    }

    for (field, value) in [
      (
        "maxAspectRatioInPortrait",
        &rule.max_aspect_ratio_in_portrait,
      ),
      (
        "maxAspectRatioInLandscape",
        &rule.max_aspect_ratio_in_landscape,
      ),
    ] {
      if let Some(EmbeddingAspectRatio::Ratio(ratio)) = value {
        anyhow::ensure!(
          *ratio > 1.0,
          "{}: `{field}.ratio` must be greater than 1.0, got {ratio}",
          context()
        );
      }
    }

    for (field, value) in [
      ("minWidthDp", rule.min_width_dp),
      ("minHeightDp", rule.min_height_dp),
      ("minSmallestWidthDp", rule.min_smallest_width_dp),
    ] {
      if let Some(dp) = value {
        anyhow::ensure!(
          i32::try_from(dp).is_ok(),
          "{}: `{field}` must not exceed {}, got {dp}",
          context(),
          i32::MAX
        );
      }
    }
  }

  Ok(())
}

/// Ensures the value is a plain or fully qualified class name, since it is
/// inserted verbatim into Kotlin sources and the manifest.
fn validate_class_name(name: &str, field: &str) -> Result<()> {
  let valid = !name.is_empty()
    && name.split('.').all(|segment| {
      let mut chars = segment.chars();
      chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
    });
  anyhow::ensure!(
    valid,
    "`{field}` must be a class name such as `DetailActivity` or `com.example.DetailActivity`, got {name:?}"
  );
  Ok(())
}

/// Manifest entries for the `<application>` element: the embedding property, the
/// startup provider and one `<activity>` per secondary the app manifest does not declare itself.
fn activity_embedding_manifest_xml(
  config: &ActivityEmbeddingConfig,
  package: &str,
  user_manifest: &str,
) -> String {
  let mut xml = String::new();

  xml.push_str("<property\n");
  xml.push_str("    android:name=\"android.window.PROPERTY_ACTIVITY_EMBEDDING_SPLITS_ENABLED\"\n");
  xml.push_str("    android:value=\"true\" />\n");

  xml.push_str("<provider\n");
  xml.push_str("    android:name=\"androidx.startup.InitializationProvider\"\n");
  xml.push_str("    android:authorities=\"${applicationId}.androidx-startup\"\n");
  xml.push_str("    android:exported=\"false\">\n");
  xml.push_str("    <meta-data\n");
  xml.push_str(&format!(
    "        android:name=\"{package}.{SPLIT_INITIALIZER_CLASS}\"\n"
  ));
  xml.push_str("        android:value=\"androidx.startup\" />\n");
  xml.push_str("</provider>\n");

  for secondary in unique_secondaries(config) {
    let name = if secondary.contains('.') {
      secondary.to_string()
    } else {
      format!(".{secondary}")
    };
    if manifest_declares_activity(user_manifest, &name, package) {
      continue;
    }
    xml.push_str("<activity\n");
    xml.push_str(&format!("    android:name=\"{name}\"\n"));
    xml.push_str("    android:exported=\"false\"\n");
    xml.push_str(&format!(
      "    android:configChanges=\"{ACTIVITY_CONFIG_CHANGES}\" />\n"
    ));
  }

  xml
}

/// Whether the manifest (without the generated block) already declares the activity,
/// under either its relative (`.Name`) or fully qualified spelling.
fn manifest_declares_activity(manifest: &str, name: &str, package: &str) -> bool {
  let qualified = match name.strip_prefix('.') {
    Some(relative) => format!("{package}.{relative}"),
    None => name.to_string(),
  };
  let relative = qualified
    .strip_prefix(package)
    .and_then(|rest| rest.strip_prefix('.'))
    .map(|rest| format!(".{rest}"));

  std::iter::once(qualified).chain(relative).any(|n| {
    manifest.contains(&format!("android:name=\"{n}\""))
      || manifest.contains(&format!("android:name='{n}'"))
  })
}

fn split_initializer_kotlin(config: &ActivityEmbeddingConfig, package: &str) -> String {
  let mut kt = format!("{GENERATED_HEADER}\npackage {package}\n\n");
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
  kt.push_str(&format!(
    "class {SPLIT_INITIALIZER_CLASS} : Initializer<RuleController> {{\n"
  ));
  kt.push_str("    override fun create(context: Context): RuleController {\n");
  kt.push_str("        val ruleController = RuleController.getInstance(context)\n\n");

  for (i, rule) in config.split_rules.iter().enumerate() {
    append_split_rule(&mut kt, i, rule, package);
  }

  kt.push_str("        return ruleController\n");
  kt.push_str("    }\n\n");
  kt.push_str("    override fun dependencies(): List<Class<out Initializer<*>>> = emptyList()\n");
  kt.push_str("}\n");
  kt
}

/// A minimal `TauriActivity` subclass, mirroring the `MainActivity` template.
fn default_activity_kotlin(name: &str, package: &str) -> String {
  format!(
    "{GENERATED_HEADER}\n\
     package {package}\n\n\
     import android.os.Bundle\n\
     import androidx.activity.enableEdgeToEdge\n\n\
     class {name} : TauriActivity() {{\n  \
       override fun onCreate(savedInstanceState: Bundle?) {{\n    \
         enableEdgeToEdge()\n    \
         super.onCreate(savedInstanceState)\n  \
       }}\n\
     }}\n"
  )
}

fn append_split_rule(kt: &mut String, i: usize, rule: &SplitPairRule, package: &str) {
  let primary = qualify_activity(&rule.primary, package);
  let secondary = qualify_activity(&rule.secondary, package);
  let secondary_intent_action = rule
    .secondary_intent_action
    .as_deref()
    .map(kotlin_string_literal)
    .unwrap_or_else(|| "null".to_string());

  kt.push_str(&format!("        val filter{i} = SplitPairFilter(\n"));
  kt.push_str(&format!(
    "            ComponentName(context, {primary}::class.java),\n"
  ));
  kt.push_str(&format!(
    "            ComponentName(context, {secondary}::class.java),\n"
  ));
  kt.push_str(&format!("            {secondary_intent_action}\n"));
  kt.push_str("        )\n");

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
    "        val ruleBuilder{i} = SplitPairRule.Builder(setOf(filter{i}))\n"
  ));
  kt.push_str(&format!(
    "        ruleBuilder{i}.setDefaultSplitAttributes(attrs{i})\n"
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
    kt.push_str(&format!(
      "        ruleBuilder{i}.setTag({})\n",
      kotlin_string_literal(tag)
    ));
  }

  kt.push_str(&format!("        val rule{i} = ruleBuilder{i}.build()\n"));
  kt.push_str(&format!("        ruleController.addRule(rule{i})\n\n"));
}

fn qualify_activity(name: &str, package: &str) -> String {
  if name.contains('.') {
    name.to_string()
  } else {
    format!("{package}.{name}")
  }
}

/// Renders `s` as a Kotlin string literal, escaping quotes, backslashes,
/// `$` (which would start a string template) and control characters.
fn kotlin_string_literal(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 2);
  out.push('"');
  for c in s.chars() {
    match c {
      '\\' => out.push_str("\\\\"),
      '"' => out.push_str("\\\""),
      '$' => out.push_str("\\$"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
      c => out.push(c),
    }
  }
  out.push('"');
  out
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

#[cfg(test)]
mod tests {
  use super::*;

  const PACKAGE: &str = "com.tauri.api";

  fn rule(primary: &str, secondary: &str) -> SplitPairRule {
    SplitPairRule {
      primary: primary.into(),
      secondary: secondary.into(),
      secondary_intent_action: None,
      split_type: None,
      layout_direction: None,
      min_width_dp: None,
      min_height_dp: None,
      min_smallest_width_dp: None,
      max_aspect_ratio_in_portrait: None,
      max_aspect_ratio_in_landscape: None,
      finish_primary_with_secondary: None,
      finish_secondary_with_primary: None,
      clear_top: None,
      tag: None,
    }
  }

  fn config(split_rules: Vec<SplitPairRule>) -> ActivityEmbeddingConfig {
    ActivityEmbeddingConfig {
      enabled: true,
      split_rules,
    }
  }

  fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
      "tauri-build-activity-embedding-{name}-{}",
      std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn kotlin_string_literal_escapes_special_characters() {
    assert_eq!(kotlin_string_literal("plain"), "\"plain\"");
    assert_eq!(
      kotlin_string_literal("a$b \"q\" \\ \n\t\u{7f} é"),
      "\"a\\$b \\\"q\\\" \\\\ \\n\\t\\u007F é\""
    );
  }

  #[test]
  fn split_initializer_renders_every_field() {
    let mut full = rule("MainActivity", "DetailActivity");
    full.secondary_intent_action = Some("android.intent.action.VIEW".into());
    full.split_type = Some(SplitType::Ratio(0.33));
    full.layout_direction = Some(SplitLayoutDirection::LeftToRight);
    full.min_width_dp = Some(840);
    full.min_height_dp = Some(600);
    full.min_smallest_width_dp = Some(600);
    full.max_aspect_ratio_in_portrait = Some(EmbeddingAspectRatio::Ratio(1.4));
    full.max_aspect_ratio_in_landscape = Some(EmbeddingAspectRatio::AlwaysAllow);
    full.finish_primary_with_secondary = Some(SplitFinishBehavior::Never);
    full.finish_secondary_with_primary = Some(SplitFinishBehavior::Always);
    full.clear_top = Some(true);
    full.tag = Some("detail$1".into());

    let kt = split_initializer_kotlin(&config(vec![full]), PACKAGE);

    let expected = r#"// THIS IS AN AUTOGENERATED FILE BY TAURI ACTIVITY EMBEDDING. DO NOT EDIT THIS FILE DIRECTLY.
package com.tauri.api

import android.content.ComponentName
import android.content.Context
import androidx.startup.Initializer
import androidx.window.embedding.EmbeddingAspectRatio
import androidx.window.embedding.RuleController
import androidx.window.embedding.SplitAttributes
import androidx.window.embedding.SplitPairFilter
import androidx.window.embedding.SplitPairRule
import androidx.window.embedding.SplitRule

class TauriSplitInitializer : Initializer<RuleController> {
    override fun create(context: Context): RuleController {
        val ruleController = RuleController.getInstance(context)

        val filter0 = SplitPairFilter(
            ComponentName(context, com.tauri.api.MainActivity::class.java),
            ComponentName(context, com.tauri.api.DetailActivity::class.java),
            "android.intent.action.VIEW"
        )
        val attrsBuilder0 = SplitAttributes.Builder()
        attrsBuilder0.setSplitType(SplitAttributes.SplitType.ratio(0.33f))
        attrsBuilder0.setLayoutDirection(SplitAttributes.LayoutDirection.LEFT_TO_RIGHT)
        val attrs0 = attrsBuilder0.build()
        val ruleBuilder0 = SplitPairRule.Builder(setOf(filter0))
        ruleBuilder0.setDefaultSplitAttributes(attrs0)
        ruleBuilder0.setMinWidthDp(840)
        ruleBuilder0.setMinHeightDp(600)
        ruleBuilder0.setMinSmallestWidthDp(600)
        ruleBuilder0.setMaxAspectRatioInPortrait(EmbeddingAspectRatio.ratio(1.4f))
        ruleBuilder0.setMaxAspectRatioInLandscape(EmbeddingAspectRatio.ALWAYS_ALLOW)
        ruleBuilder0.setFinishPrimaryWithSecondary(SplitRule.FinishBehavior.NEVER)
        ruleBuilder0.setFinishSecondaryWithPrimary(SplitRule.FinishBehavior.ALWAYS)
        ruleBuilder0.setClearTop(true)
        ruleBuilder0.setTag("detail\$1")
        val rule0 = ruleBuilder0.build()
        ruleController.addRule(rule0)

        return ruleController
    }

    override fun dependencies(): List<Class<out Initializer<*>>> = emptyList()
}
"#;
    assert_eq!(kt, expected);
  }

  #[test]
  fn minimal_rule_keeps_sdk_defaults_and_qualifies_names() {
    let kt = split_initializer_kotlin(
      &config(vec![rule("MainActivity", "com.example.lib.DetailActivity")]),
      "com.`in`.app",
    );
    assert!(kt.contains("package com.`in`.app\n"));
    assert!(kt.contains("ComponentName(context, com.`in`.app.MainActivity::class.java)"));
    assert!(kt.contains("ComponentName(context, com.example.lib.DetailActivity::class.java)"));
    assert!(kt.contains("            null\n"));
    assert!(!kt.contains("setSplitType"));
    assert!(!kt.contains("setMinWidthDp"));
    assert!(!kt.contains("setTag"));
  }

  #[test]
  fn manifest_xml_declares_only_undeclared_secondaries() {
    let cfg = config(vec![
      rule("MainActivity", "DetailActivity"),
      rule("DetailActivity", "SettingsActivity"),
      rule("MainActivity", "DetailActivity"),
      rule("MainActivity", "com.example.lib.LibActivity"),
      rule("MainActivity", "com.tauri.api.OwnActivity"),
    ]);
    let user_manifest = r#"<application>
        <activity android:name=".MainActivity" android:exported="true" />
        <activity android:name="com.tauri.api.SettingsActivity" />
        <activity android:name='.OwnActivity' />
    </application>"#;

    let xml = activity_embedding_manifest_xml(&cfg, PACKAGE, user_manifest);

    assert!(xml.contains("android.window.PROPERTY_ACTIVITY_EMBEDDING_SPLITS_ENABLED"));
    assert!(xml.contains("android:name=\"com.tauri.api.TauriSplitInitializer\""));
    assert_eq!(xml.matches("android:name=\".DetailActivity\"").count(), 1);
    assert!(xml.contains("android:name=\"com.example.lib.LibActivity\""));
    assert!(!xml.contains("SettingsActivity"));
    assert!(!xml.contains("OwnActivity"));
    assert!(xml.contains(&format!(
      "android:configChanges=\"{ACTIVITY_CONFIG_CHANGES}\""
    )));
  }

  #[test]
  fn validation_rejects_out_of_range_values_and_bad_names() {
    assert!(
      validate_activity_embedding(&config(vec![rule("MainActivity", "DetailActivity")])).is_ok()
    );

    let mut r = rule("MainActivity", "DetailActivity");
    r.split_type = Some(SplitType::Ratio(1.0));
    let err = validate_activity_embedding(&config(vec![r])).unwrap_err();
    assert!(err.to_string().contains("splitRules[0]"), "{err}");
    assert!(err.to_string().contains("splitType.ratio"), "{err}");

    let mut r = rule("MainActivity", "DetailActivity");
    r.max_aspect_ratio_in_landscape = Some(EmbeddingAspectRatio::Ratio(1.0));
    assert!(validate_activity_embedding(&config(vec![r])).is_err());

    let mut r = rule("MainActivity", "DetailActivity");
    r.min_width_dp = Some(u32::MAX);
    assert!(validate_activity_embedding(&config(vec![r])).is_err());

    for bad in [
      "",
      "Detail Activity",
      "Foo::class.java",
      "1Activity",
      "a..b",
      "\"/>",
    ] {
      assert!(
        validate_activity_embedding(&config(vec![rule("MainActivity", bad)])).is_err(),
        "{bad:?} should be rejected"
      );
    }
    assert!(
      validate_activity_embedding(&config(vec![rule("com.example.Main", "_Detail2")])).is_ok()
    );
  }

  #[test]
  fn generated_sources_are_written_and_stale_ones_removed() {
    let out_dir = temp_dir("sources").join("generated");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(
      out_dir.join("TauriActivity.kt"),
      "package x\nabstract class TauriActivity",
    )
    .unwrap();

    let initializer = (
      "TauriSplitInitializer.kt".to_string(),
      split_initializer_kotlin(
        &config(vec![rule("MainActivity", "DetailActivity")]),
        PACKAGE,
      ),
    );
    let detail = (
      "DetailActivity.kt".to_string(),
      default_activity_kotlin("DetailActivity", PACKAGE),
    );
    write_generated_sources(&out_dir, &[initializer.clone(), detail]).unwrap();
    assert!(out_dir.join("TauriSplitInitializer.kt").exists());
    assert!(out_dir.join("DetailActivity.kt").exists());

    // DetailActivity is no longer generated (e.g. the user now provides it): it must be removed,
    // while files generated by other build scripts stay untouched.
    write_generated_sources(&out_dir, &[initializer]).unwrap();
    assert!(!out_dir.join("DetailActivity.kt").exists());
    assert!(out_dir.join("TauriActivity.kt").exists());

    // disabling the feature removes everything we generated
    write_generated_sources(&out_dir, &[]).unwrap();
    assert!(!out_dir.join("TauriSplitInitializer.kt").exists());
    assert!(out_dir.join("TauriActivity.kt").exists());

    let _ = std::fs::remove_dir_all(out_dir.parent().unwrap());
  }

  #[test]
  fn user_provided_activity_sources_are_detected() {
    let project_dir = temp_dir("project");
    assert!(!activity_source_exists(
      &project_dir,
      PACKAGE,
      "DetailActivity"
    ));

    let kotlin_dir = project_dir.join("app/src/main/kotlin/com/tauri/api");
    std::fs::create_dir_all(&kotlin_dir).unwrap();
    std::fs::write(kotlin_dir.join("DetailActivity.kt"), "").unwrap();
    assert!(activity_source_exists(
      &project_dir,
      PACKAGE,
      "DetailActivity"
    ));
    assert!(!activity_source_exists(
      &project_dir,
      PACKAGE,
      "OtherActivity"
    ));

    let _ = std::fs::remove_dir_all(project_dir);
  }
  #[test]
  #[serial_test::serial]
  fn sync_generates_and_removes_project_files() {
    let project_dir = temp_dir("sync");
    let manifest_path = project_dir.join("app/src/main/AndroidManifest.xml");
    std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    let manifest = "<manifest>\n    <application>\n        <activity android:name=\".MainActivity\" />\n    </application>\n</manifest>";
    std::fs::write(&manifest_path, manifest).unwrap();

    // the CLI exports these when it runs the build script
    unsafe {
      std::env::set_var("TAURI_ANDROID_PROJECT_PATH", &project_dir);
      std::env::set_var("WRY_ANDROID_PACKAGE", "com.`in`.app");
      std::env::set_var("TAURI_ANDROID_PACKAGE_UNESCAPED", "com.in.app");
      std::env::remove_var("WRY_ANDROID_KOTLIN_FILES_OUT_DIR");
    }

    let cfg = config(vec![rule("MainActivity", "DetailActivity")]);
    sync_activity_embedding(&project_dir, Some(&cfg), "com.in.app").unwrap();

    let generated = project_dir.join("app/src/main/java/com/in/app/generated");
    let initializer = std::fs::read_to_string(generated.join("TauriSplitInitializer.kt")).unwrap();
    assert!(initializer.contains("package com.`in`.app\n"));
    let detail = std::fs::read_to_string(generated.join("DetailActivity.kt")).unwrap();
    assert!(detail.contains("class DetailActivity : TauriActivity()"));
    let rewritten = std::fs::read_to_string(&manifest_path).unwrap();
    assert!(rewritten.contains("android:name=\"com.in.app.TauriSplitInitializer\""));
    assert!(rewritten.contains("android:name=\".DetailActivity\""));

    // disabling the feature restores the project
    sync_activity_embedding(&project_dir, None, "com.in.app").unwrap();
    assert!(!generated.join("TauriSplitInitializer.kt").exists());
    assert!(!generated.join("DetailActivity.kt").exists());
    assert_eq!(std::fs::read_to_string(&manifest_path).unwrap(), manifest);

    unsafe {
      std::env::remove_var("TAURI_ANDROID_PROJECT_PATH");
      std::env::remove_var("WRY_ANDROID_PACKAGE");
      std::env::remove_var("TAURI_ANDROID_PACKAGE_UNESCAPED");
    }
    let _ = std::fs::remove_dir_all(project_dir);
  }
}
