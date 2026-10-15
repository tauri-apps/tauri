// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Result, error::Context, error::ErrorExt};

use regex::Regex;
use serde_json::Value;

use std::{fs, path::Path};

/// Migrates the main and the platform-specific configuration files in the given directory.
pub fn migrate(tauri_dir: &Path) -> Result<()> {
  let entries = fs::read_dir(tauri_dir).fs_context("failed to read directory", tauri_dir)?;
  for entry in entries {
    let path = entry
      .fs_context("failed to read directory entry", tauri_dir)?
      .path();
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
      continue;
    };

    let migrated = if is_json_config(file_name, "json") {
      let contents = fs::read_to_string(&path).fs_context("failed to read config", &path)?;
      let mut config: Value = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))?;
      migrate_json(&mut config).then(|| {
        edit_json_text(&contents, &config, |s| serde_json::from_str(s).ok())
          .unwrap_or_else(|| serialize_json(&config, &contents))
      })
    } else if is_json_config(file_name, "json5") {
      let contents = fs::read_to_string(&path).fs_context("failed to read config", &path)?;
      let mut config: Value = json5::from_str(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))?;
      migrate_json(&mut config).then(|| {
        edit_json_text(&contents, &config, |s| json5::from_str(s).ok()).unwrap_or_else(|| {
          log::warn!(
            "{} was rewritten as JSON, so its comments were not preserved",
            path.display()
          );
          serialize_json(&config, &contents)
        })
      })
    } else if is_toml_config(file_name) {
      let contents = fs::read_to_string(&path).fs_context("failed to read config", &path)?;
      let mut config: toml_edit::DocumentMut = contents
        .parse()
        .with_context(|| format!("failed to parse {}", path.display()))?;
      migrate_toml(&mut config).then(|| config.to_string())
    } else {
      None
    };

    if let Some(migrated) = migrated {
      fs::write(&path, migrated).fs_context("failed to write config", &path)?;
    }
  }

  Ok(())
}

/// `tauri.conf.json` and `tauri.<platform>.conf.json` (or `.json5`).
fn is_json_config(file_name: &str, extension: &str) -> bool {
  file_name
    .strip_prefix("tauri.")
    .and_then(|n| n.strip_suffix(extension))
    .and_then(|n| n.strip_suffix('.'))
    .is_some_and(|n| n == "conf" || n.strip_suffix(".conf").is_some_and(is_platform))
}

/// `Tauri.toml` and `Tauri.<platform>.toml`.
fn is_toml_config(file_name: &str) -> bool {
  file_name
    .strip_prefix("Tauri")
    .and_then(|n| n.strip_suffix(".toml"))
    .is_some_and(|n| n.is_empty() || n.strip_prefix('.').is_some_and(is_platform))
}

fn is_platform(name: &str) -> bool {
  ["macos", "windows", "linux", "android", "ios"].contains(&name)
}

/// Applies the migration as text edits to keep the file formatting and comments.
///
/// Returns `None` if the edited text does not parse to the `migrated` config,
/// e.g. when a key matched somewhere else in the file or the formatting is unusual.
fn edit_json_text(
  original: &str,
  migrated: &Value,
  parse: impl Fn(&str) -> Option<Value>,
) -> Option<String> {
  let mut edited = original.to_string();
  for key in [
    r#"(?:"macOSPrivateApi"|'macOSPrivateApi'|macOSPrivateApi)"#,
    r#"(?:"macos-private-api"|'macos-private-api')"#,
  ] {
    let entry = format!(r"{key}[ \t]*:[ \t]*(?:true|false)");
    for (pattern, replacement) in [
      // an entry on its own line, followed by another one (or a JSON5 trailing comma)
      (format!(r"(?m)^[ \t]*{entry}[ \t]*,[ \t]*\r?\n"), ""),
      // the last entry of an object
      (format!(r",(\s*){entry}"), ""),
      // an entry followed by another one on the same line
      (format!(r"{entry}\s*,\s*"), ""),
      // the only entry of an object
      (format!(r"\{{\s*{entry}\s*\}}"), "{}"),
    ] {
      let re = Regex::new(&pattern).expect("invalid regex");
      edited = re.replace_all(&edited, replacement).into_owned();
    }
  }
  for quote in ['"', '\''] {
    edited = edited.replace(
      &format!("{quote}install-icon{quote}"),
      &format!("{quote}installer-icon{quote}"),
    );
  }

  (parse(&edited).as_ref() == Some(migrated)).then_some(edited)
}

fn serialize_json(config: &Value, original: &str) -> String {
  let mut serialized = serde_json::to_string_pretty(config).expect("failed to serialize config");
  if original.ends_with('\n') {
    serialized.push('\n');
  }
  serialized
}

/// Returns whether the config was changed.
pub fn migrate_json(config: &mut Value) -> bool {
  let mut changed = false;

  if let Some(app) = config.get_mut("app").and_then(|a| a.as_object_mut()) {
    for key in ["macOSPrivateApi", "macos-private-api"] {
      changed |= app.shift_remove(key).is_some();
    }
  }

  if let Some(nsis) = config
    .pointer_mut("/bundle/windows/nsis")
    .and_then(|n| n.as_object_mut())
    && let Some(index) = nsis.keys().position(|key| key == "install-icon")
  {
    let icon = nsis.shift_remove("install-icon").unwrap();
    nsis.shift_insert(index, "installer-icon".into(), icon);
    changed = true;
  }

  changed
}

/// Returns whether the config was changed.
fn migrate_toml(config: &mut toml_edit::DocumentMut) -> bool {
  let mut changed = false;

  if let Some(app) = config.get_mut("app").and_then(|a| a.as_table_like_mut()) {
    for key in ["macOSPrivateApi", "macos-private-api"] {
      changed |= app.remove(key).is_some();
    }
  }

  if let Some(nsis) = config
    .get_mut("bundle")
    .and_then(|b| b.get_mut("windows"))
    .and_then(|w| w.get_mut("nsis"))
  {
    if let Some(nsis) = nsis.as_table_mut() {
      changed |= rename_toml_key(nsis, "install-icon", "installer-icon");
    } else if let Some(nsis) = nsis.as_inline_table_mut() {
      changed |= rename_toml_key(nsis, "install-icon", "installer-icon");
    }
  }

  changed
}

trait TomlTable {
  fn remove_entry(&mut self, key: &str) -> Option<(toml_edit::Key, toml_edit::Item)>;
  fn insert_formatted(&mut self, key: &toml_edit::Key, item: toml_edit::Item);
  fn keys(&self) -> Vec<String>;
  fn sort_by_position(&mut self, positions: &[String]);
}

impl TomlTable for toml_edit::Table {
  fn remove_entry(&mut self, key: &str) -> Option<(toml_edit::Key, toml_edit::Item)> {
    toml_edit::Table::remove_entry(self, key)
  }

  fn insert_formatted(&mut self, key: &toml_edit::Key, item: toml_edit::Item) {
    toml_edit::Table::insert_formatted(self, key, item);
  }

  fn keys(&self) -> Vec<String> {
    self.iter().map(|(key, _)| key.to_string()).collect()
  }

  fn sort_by_position(&mut self, positions: &[String]) {
    self.sort_values_by(|a, _, b, _| position(positions, a).cmp(&position(positions, b)));
  }
}

impl TomlTable for toml_edit::InlineTable {
  fn remove_entry(&mut self, key: &str) -> Option<(toml_edit::Key, toml_edit::Item)> {
    toml_edit::InlineTable::remove_entry(self, key).map(|(k, v)| (k, toml_edit::Item::Value(v)))
  }

  fn insert_formatted(&mut self, key: &toml_edit::Key, item: toml_edit::Item) {
    if let toml_edit::Item::Value(value) = item {
      toml_edit::InlineTable::insert_formatted(self, key, value);
    }
  }

  fn keys(&self) -> Vec<String> {
    self.iter().map(|(key, _)| key.to_string()).collect()
  }

  fn sort_by_position(&mut self, positions: &[String]) {
    self.sort_values_by(|a, _, b, _| position(positions, a).cmp(&position(positions, b)));
  }
}

fn position(positions: &[String], key: &toml_edit::Key) -> usize {
  positions
    .iter()
    .position(|k| k == key.get())
    .unwrap_or(usize::MAX)
}

/// Renames a key, keeping its formatting and position. Returns whether the key existed.
fn rename_toml_key(table: &mut impl TomlTable, from: &str, to: &str) -> bool {
  let mut positions = table.keys();
  let Some((key, item)) = table.remove_entry(from) else {
    return false;
  };
  for position in &mut positions {
    if position == from {
      *position = to.to_string();
    }
  }
  let mut new_key = toml_edit::Key::new(to);
  *new_key.leaf_decor_mut() = key.leaf_decor().clone();
  table.insert_formatted(&new_key, item);
  table.sort_by_position(&positions);
  true
}

#[cfg(test)]
mod tests {
  #[test]
  fn config_file_names() {
    for name in [
      "tauri.conf.json",
      "tauri.macos.conf.json",
      "tauri.conf.json5",
      "tauri.windows.conf.json5",
    ] {
      assert!(
        super::is_json_config(name, "json") || super::is_json_config(name, "json5"),
        "{name}"
      );
    }
    for name in ["Tauri.toml", "Tauri.linux.toml"] {
      assert!(super::is_toml_config(name), "{name}");
    }
    for name in [
      "tauri.config.json",
      "tauri.web.conf.json",
      "package.json",
      "Cargo.toml",
      "Tauri.web.toml",
      "Tauri.linux.conf.toml",
    ] {
      assert!(
        !super::is_json_config(name, "json")
          && !super::is_json_config(name, "json5")
          && !super::is_toml_config(name),
        "{name}"
      );
    }
  }

  fn migrate_json_text(original: &str) -> Option<String> {
    let mut config: serde_json::Value = json5::from_str(original).unwrap();
    assert!(super::migrate_json(&mut config));
    super::edit_json_text(original, &config, |s| json5::from_str(s).ok())
  }

  #[test]
  fn edits_json_text() {
    // last entry of the object
    assert_eq!(
      migrate_json_text(
        "{\n  \"app\": {\n    \"windows\": [],\n    \"macOSPrivateApi\": true\n  }\n}\n"
      )
      .as_deref(),
      Some("{\n  \"app\": {\n    \"windows\": []\n  }\n}\n")
    );
    // single line
    assert_eq!(
      migrate_json_text(r#"{ "app": { "macOSPrivateApi": false, "windows": [] } }"#).as_deref(),
      Some(r#"{ "app": { "windows": [] } }"#)
    );
    // only entry
    assert_eq!(
      migrate_json_text(r#"{ "app": { "macOSPrivateApi": true } }"#).as_deref(),
      Some(r#"{ "app": {} }"#)
    );
    // JSON5 comments, unquoted keys and trailing commas
    assert_eq!(
      migrate_json_text("{\n  // comment\n  app: {\n    macOSPrivateApi: true,\n  },\n}\n")
        .as_deref(),
      Some("{\n  // comment\n  app: {\n  },\n}\n")
    );
    // NSIS installer icon
    assert_eq!(
      migrate_json_text(r#"{ "bundle": { "windows": { "nsis": { "install-icon": "a.ico" } } } }"#)
        .as_deref(),
      Some(r#"{ "bundle": { "windows": { "nsis": { "installer-icon": "a.ico" } } } }"#)
    );
  }

  #[test]
  fn falls_back_when_text_edit_does_not_match() {
    // the key is also used by a plugin config, which must be kept
    let original =
      r#"{ "app": { "macOSPrivateApi": true }, "plugins": { "p": { "macOSPrivateApi": true } } }"#;
    assert_eq!(migrate_json_text(original), None);
  }
}
