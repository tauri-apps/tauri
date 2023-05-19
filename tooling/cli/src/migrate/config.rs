use crate::Result;

use serde_json::{Map, Value};

use std::{fs::write, path::Path};

macro_rules! move_allowlist_object {
  ($plugins: ident, $value: expr, $plugin: literal, $field: literal) => {{
    if $value != Default::default() {
      $plugins
        .entry($plugin)
        .or_insert_with(|| Value::Object(Default::default()))
        .as_object_mut()
        .unwrap()
        .insert($field.into(), serde_json::to_value($value)?);
    }
  }};
}

pub fn migrate(tauri_dir: &Path) -> Result<()> {
  if let Ok((mut config, config_path)) =
    tauri_utils_v1::config::parse::parse_value(tauri_dir.join("tauri.conf.json"))
  {
    if let Some(config) = config.as_object_mut() {
      let mut plugins = config
        .entry("plugins")
        .or_insert_with(|| Value::Object(Default::default()))
        .as_object_mut()
        .unwrap()
        .clone();

      if let Some(tauri_config) = config.get_mut("tauri").and_then(|c| c.as_object_mut()) {
        // allowlist
        if let Some(allowlist) = tauri_config.remove("allowlist") {
          process_allowlist(tauri_config, &mut plugins, allowlist)?;
        }

        // cli
        if let Some(cli) = tauri_config.remove("cli") {
          process_cli(&mut plugins, cli)?;
        }

        // cli
        if let Some(updater) = tauri_config.remove("updater") {
          process_updater(tauri_config, &mut plugins, updater)?;
        }
      }

      config.insert("plugins".into(), plugins.into());
    }

    write(config_path, serde_json::to_string_pretty(&config)?)?;
  }
  Ok(())
}

fn process_allowlist(
  tauri_config: &mut Map<String, Value>,
  plugins: &mut Map<String, Value>,
  allowlist: Value,
) -> Result<()> {
  let allowlist: tauri_utils_v1::config::AllowlistConfig = serde_json::from_value(allowlist)?;

  move_allowlist_object!(plugins, allowlist.fs.scope, "fs", "scope");
  move_allowlist_object!(plugins, allowlist.shell.scope, "shell", "scope");
  move_allowlist_object!(plugins, allowlist.shell.open, "shell", "open");
  move_allowlist_object!(plugins, allowlist.http.scope, "http", "scope");

  if allowlist.protocol.asset_scope != Default::default() {
    let security = tauri_config
      .entry("security")
      .or_insert_with(|| Value::Object(Default::default()))
      .as_object_mut()
      .unwrap();

    let mut asset_protocol = Map::new();
    asset_protocol.insert(
      "scope".into(),
      serde_json::to_value(allowlist.protocol.asset_scope)?,
    );
    if allowlist.protocol.asset {
      asset_protocol.insert("enable".into(), true.into());
    }
    security.insert("assetProtocol".into(), asset_protocol.into());
  }

  Ok(())
}

fn process_cli(plugins: &mut Map<String, Value>, cli: Value) -> Result<()> {
  if let Some(cli) = cli.as_object() {
    plugins.insert("cli".into(), serde_json::to_value(cli)?);
  }
  Ok(())
}

fn process_updater(
  tauri_config: &mut Map<String, Value>,
  plugins: &mut Map<String, Value>,
  mut updater: Value,
) -> Result<()> {
  if let Some(updater) = updater.as_object_mut() {
    updater.remove("dialog");

    let endpoints = updater
      .remove("endpoints")
      .unwrap_or_else(|| Value::Array(Default::default()));

    let mut plugin_updater_config = Map::new();
    plugin_updater_config.insert("endpoints".into(), endpoints);
    if let Some(windows) = updater.get_mut("windows").and_then(|w| w.as_object_mut()) {
      if let Some(installer_args) = windows.remove("installerArgs") {
        let mut windows_updater_config = Map::new();
        windows_updater_config.insert("installerArgs".into(), installer_args);

        plugin_updater_config.insert("windows".into(), windows_updater_config.into());
      }
    }

    plugins.insert("updater".into(), plugin_updater_config.into());
  }

  tauri_config
    .get_mut("bundle")
    .unwrap()
    .as_object_mut()
    .unwrap()
    .insert("updater".into(), updater);

  Ok(())
}
