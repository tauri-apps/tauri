// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
    migrate_config(&mut config)?;
    write(config_path, serde_json::to_string_pretty(&config)?)?;
  }

  Ok(())
}

fn migrate_config(config: &mut Value) -> Result<()> {
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

      if let Some(security) = tauri_config
        .get_mut("security")
        .and_then(|c| c.as_object_mut())
      {
        process_security(security)?;
      }

      if let Some(tray) = tauri_config.remove("systemTray") {
        tauri_config.insert("trayIcon".into(), tray);
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

  Ok(())
}

fn process_security(security: &mut Map<String, Value>) -> Result<()> {
  // migrate CSP: add `ipc:` to `connect-src`
  if let Some(csp_value) = security.remove("csp") {
    let csp = if csp_value.is_null() {
      csp_value
    } else {
      let mut csp: tauri_utils_v1::config::Csp = serde_json::from_value(csp_value)?;
      match &mut csp {
        tauri_utils_v1::config::Csp::Policy(csp) => {
          if csp.contains("connect-src") {
            *csp = csp.replace("connect-src", "connect-src ipc: http://ipc.localhost");
          } else {
            *csp = format!("{csp}; connect-src ipc: http://ipc.localhost");
          }
        }
        tauri_utils_v1::config::Csp::DirectiveMap(csp) => {
          if let Some(connect_src) = csp.get_mut("connect-src") {
            if !connect_src.contains("ipc: http://ipc.localhost") {
              connect_src.push("ipc: http://ipc.localhost");
            }
          } else {
            csp.insert(
              "connect-src".into(),
              tauri_utils_v1::config::CspDirectiveSources::List(vec![
                "ipc: http://ipc.localhost".to_string()
              ]),
            );
          }
        }
      }
      serde_json::to_value(csp)?
    };

    security.insert("csp".into(), csp);
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

#[cfg(test)]
mod test {
  fn migrate(original: &serde_json::Value) -> serde_json::Value {
    let mut migrated = original.clone();
    super::migrate_config(&mut migrated).expect("failed to migrate config");

    if let Err(e) = serde_json::from_value::<tauri_utils::config::Config>(migrated.clone()) {
      panic!("migrated config is not valid: {e}");
    }

    migrated
  }

  #[test]
  fn migrate_full() {
    let original = serde_json::json!({
      "tauri": {
        "bundle": {
          "identifier": "com.tauri.test"
        },
        "cli": {
          "description": "Tauri TEST"
        },
        "updater": {
          "active": true,
          "dialog": false,
          "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDE5QzMxNjYwNTM5OEUwNTgKUldSWTRKaFRZQmJER1h4d1ZMYVA3dnluSjdpN2RmMldJR09hUFFlZDY0SlFqckkvRUJhZDJVZXAK",
          "endpoints": [
            "https://tauri-update-server.vercel.app/update/{{target}}/{{current_version}}"
          ],
          "windows": {
            "installerArgs": [],
            "installMode": "passive"
          }
        },
        "allowlist": {
          "all": true,
          "fs": {
            "scope": {
              "allow": ["$APPDATA/db/**", "$DOWNLOAD/**", "$RESOURCE/**"],
              "deny": ["$APPDATA/db/*.stronghold"]
            }
          },
          "shell": {
            "open": true,
            "scope": [
              {
                "name": "sh",
                "cmd": "sh",
                "args": ["-c", { "validator": "\\S+" }],
                "sidecar": false
              },
              {
                "name": "cmd",
                "cmd": "cmd",
                "args": ["/C", { "validator": "\\S+" }],
                "sidecar": false
              }
            ]
          },
          "protocol": {
            "asset": true,
            "assetScope": {
              "allow": ["$APPDATA/db/**", "$RESOURCE/**"],
              "deny": ["$APPDATA/db/*.stronghold"]
            }
          },
          "http": {
            "scope": ["http://localhost:3003/"]
          }
        },
        "security": {
          "csp": "default-src: 'self' tauri:"
        }
      }
    });

    let migrated = migrate(&original);

    // bundle > updater
    assert_eq!(
      migrated["tauri"]["bundle"]["updater"]["active"],
      original["tauri"]["updater"]["active"]
    );
    assert_eq!(
      migrated["tauri"]["bundle"]["updater"]["pubkey"],
      original["tauri"]["updater"]["pubkey"]
    );
    assert_eq!(
      migrated["tauri"]["bundle"]["updater"]["windows"]["installMode"],
      original["tauri"]["updater"]["windows"]["installMode"]
    );

    // plugins > updater
    assert_eq!(
      migrated["plugins"]["updater"]["endpoints"],
      original["tauri"]["updater"]["endpoints"]
    );
    assert_eq!(
      migrated["plugins"]["updater"]["windows"]["installerArgs"],
      original["tauri"]["updater"]["windows"]["installerArgs"]
    );

    // cli
    assert_eq!(migrated["plugins"]["cli"], original["tauri"]["cli"]);

    // fs scope
    assert_eq!(
      migrated["plugins"]["fs"]["scope"]["allow"],
      original["tauri"]["allowlist"]["fs"]["scope"]["allow"]
    );
    assert_eq!(
      migrated["plugins"]["fs"]["scope"]["deny"],
      original["tauri"]["allowlist"]["fs"]["scope"]["deny"]
    );

    // shell scope
    assert_eq!(
      migrated["plugins"]["shell"]["scope"],
      original["tauri"]["allowlist"]["shell"]["scope"]
    );
    assert_eq!(
      migrated["plugins"]["shell"]["open"],
      original["tauri"]["allowlist"]["shell"]["open"]
    );

    // http scope
    assert_eq!(
      migrated["plugins"]["http"]["scope"],
      original["tauri"]["allowlist"]["http"]["scope"]
    );

    // asset scope
    assert_eq!(
      migrated["tauri"]["security"]["assetProtocol"]["enable"],
      original["tauri"]["allowlist"]["protocol"]["asset"]
    );
    assert_eq!(
      migrated["tauri"]["security"]["assetProtocol"]["scope"]["allow"],
      original["tauri"]["allowlist"]["protocol"]["assetScope"]["allow"]
    );
    assert_eq!(
      migrated["tauri"]["security"]["assetProtocol"]["scope"]["deny"],
      original["tauri"]["allowlist"]["protocol"]["assetScope"]["deny"]
    );

    // security CSP
    assert_eq!(
      migrated["tauri"]["security"]["csp"],
      format!(
        "{}; connect-src ipc: http://ipc.localhost",
        original["tauri"]["security"]["csp"].as_str().unwrap()
      )
    );
  }

  #[test]
  fn migrate_csp_object() {
    let original = serde_json::json!({
      "tauri": {
        "security": {
          "csp": {
            "default-src": ["self", "tauri:"]
          }
        }
      }
    });

    let migrated = migrate(&original);

    assert_eq!(
      migrated["tauri"]["security"]["csp"]["default-src"],
      original["tauri"]["security"]["csp"]["default-src"]
    );
    assert!(migrated["tauri"]["security"]["csp"]["connect-src"]
      .as_array()
      .expect("connect-src isn't an array")
      .contains(&"ipc: http://ipc.localhost".into()));
  }

  #[test]
  fn migrate_csp_existing_connect_src_string() {
    let original = serde_json::json!({
      "tauri": {
        "security": {
          "csp": {
            "default-src": ["self", "tauri:"],
            "connect-src": "self"
          }
        }
      }
    });

    let migrated = migrate(&original);

    assert_eq!(
      migrated["tauri"]["security"]["csp"]["default-src"],
      original["tauri"]["security"]["csp"]["default-src"]
    );
    assert_eq!(
      migrated["tauri"]["security"]["csp"]["connect-src"]
        .as_str()
        .expect("connect-src isn't a string"),
      format!(
        "{} ipc: http://ipc.localhost",
        original["tauri"]["security"]["csp"]["connect-src"]
          .as_str()
          .unwrap()
      )
    );
  }

  #[test]
  fn migrate_csp_existing_connect_src_array() {
    let original = serde_json::json!({
      "tauri": {
        "security": {
          "csp": {
            "default-src": ["self", "tauri:"],
            "connect-src": ["self", "asset:"]
          }
        }
      }
    });

    let migrated = migrate(&original);

    assert_eq!(
      migrated["tauri"]["security"]["csp"]["default-src"],
      original["tauri"]["security"]["csp"]["default-src"]
    );

    let migrated_connect_src = migrated["tauri"]["security"]["csp"]["connect-src"]
      .as_array()
      .expect("connect-src isn't an array");
    let original_connect_src = original["tauri"]["security"]["csp"]["connect-src"]
      .as_array()
      .unwrap();
    assert!(
      migrated_connect_src
        .iter()
        .zip(original_connect_src.iter())
        .all(|(a, b)| a == b),
      "connect-src migration failed"
    );
  }
}
