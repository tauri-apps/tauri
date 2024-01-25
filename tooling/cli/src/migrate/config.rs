// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;

use serde_json::{Map, Value};
use tauri_utils::{
  acl::capability::{Capability, CapabilityContext},
  platform::Target,
};

use std::{
  fs::{create_dir_all, write},
  path::Path,
};

macro_rules! move_allowlist_object {
  ($plugins: ident, $value: expr, $plugin: literal, $field: literal) => {{
    if $value != Default::default() {
      $plugins
        .entry($plugin)
        .or_insert_with(|| Value::Object(Default::default()))
        .as_object_mut()
        .unwrap()
        .insert($field.into(), serde_json::to_value($value.clone())?);
    }
  }};
}

pub fn migrate(tauri_dir: &Path) -> Result<()> {
  if let Ok((mut config, config_path)) =
    tauri_utils_v1::config::parse::parse_value(tauri_dir.join("tauri.conf.json"))
  {
    let migrated = migrate_config(&mut config)?;
    write(&config_path, serde_json::to_string_pretty(&config)?)?;

    let mut permissions = vec![
      "path:default",
      "event:default",
      "window:default",
      "app:default",
      "resources:default",
      "menu:default",
      "tray:default",
    ];
    permissions.extend(migrated.permissions);

    let capabilities_path = config_path.parent().unwrap().join("capabilities");
    create_dir_all(&capabilities_path)?;
    write(
      capabilities_path.join("migrated.json"),
      serde_json::to_string_pretty(&Capability {
        identifier: "migrated".to_string(),
        description: "permissions that were migrated from v1".into(),
        context: CapabilityContext::Local,
        windows: vec!["main".into()],
        permissions: permissions
          .into_iter()
          .map(|p| p.to_string().try_into().unwrap())
          .collect(),
        platforms: vec![
          Target::Linux,
          Target::MacOS,
          Target::Windows,
          Target::Android,
          Target::Ios,
        ],
      })?,
    )?;
  }

  Ok(())
}

struct MigratedConfig {
  permissions: Vec<&'static str>,
}

fn migrate_config(config: &mut Value) -> Result<MigratedConfig> {
  let mut migrated = MigratedConfig {
    permissions: Vec::new(),
  };

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
        let allowlist = process_allowlist(tauri_config, &mut plugins, allowlist)?;
        let permissions = allowlist_to_permissions(&allowlist);
        migrated.permissions = permissions;
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

  Ok(migrated)
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
) -> Result<tauri_utils_v1::config::AllowlistConfig> {
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
      serde_json::to_value(allowlist.protocol.asset_scope.clone())?,
    );
    if allowlist.protocol.asset {
      asset_protocol.insert("enable".into(), true.into());
    }
    security.insert("assetProtocol".into(), asset_protocol.into());
  }

  Ok(allowlist)
}

fn allowlist_to_permissions(
  allowlist: &tauri_utils_v1::config::AllowlistConfig,
) -> Vec<&'static str> {
  macro_rules! permissions {
    ($allowlist: ident, $permissions_list: ident, $object: ident, $field: ident => $associated_permission: expr) => {
      if $allowlist.all || $allowlist.$object.all || $allowlist.$object.$field {
        $permissions_list.push($associated_permission);
      }
    };
  }

  let mut permissions = Vec::new();

  // fs
  permissions!(allowlist, permissions, fs, read_file => "fs:allow-read-file");
  permissions!(allowlist, permissions, fs, write_file => "fs:allow-write-file");
  permissions!(allowlist, permissions, fs, read_dir => "fs:allow-read-dir");
  permissions!(allowlist, permissions, fs, copy_file => "fs:allow-copy-file");
  permissions!(allowlist, permissions, fs, create_dir => "fs:allow-mkdir");
  permissions!(allowlist, permissions, fs, remove_dir => "fs:allow-remove");
  permissions!(allowlist, permissions, fs, remove_file => "fs:allow-remove");
  permissions!(allowlist, permissions, fs, rename_file => "fs:allow-rename");
  permissions!(allowlist, permissions, fs, exists => "fs:allow-exists");
  // window
  permissions!(allowlist, permissions, window, create => "window:allow-create");
  permissions!(allowlist, permissions, window, center => "window:allow-center");
  permissions!(allowlist, permissions, window, request_user_attention => "window:allow-request-user-attention");
  permissions!(allowlist, permissions, window, set_resizable => "window:allow-set-resizable");
  permissions!(allowlist, permissions, window, set_maximizable => "window:allow-set-maximizable");
  permissions!(allowlist, permissions, window, set_minimizable => "window:allow-set-minimizable");
  permissions!(allowlist, permissions, window, set_closable => "window:allow-set-closable");
  permissions!(allowlist, permissions, window, set_title => "window:allow-set-title");
  permissions!(allowlist, permissions, window, maximize => "window:allow-maximize");
  permissions!(allowlist, permissions, window, unmaximize => "window:allow-unmaximize");
  permissions!(allowlist, permissions, window, minimize => "window:allow-minimize");
  permissions!(allowlist, permissions, window, unminimize => "window:allow-unminimize");
  permissions!(allowlist, permissions, window, show => "window:allow-show");
  permissions!(allowlist, permissions, window, hide => "window:allow-hide");
  permissions!(allowlist, permissions, window, close => "window:allow-close");
  permissions!(allowlist, permissions, window, set_decorations => "window:allow-set-decorations");
  permissions!(allowlist, permissions, window, set_always_on_top => "window:allow-set-always-on-top");
  permissions!(allowlist, permissions, window, set_content_protected => "window:allow-set-content-protected");
  permissions!(allowlist, permissions, window, set_size => "window:allow-set-size");
  permissions!(allowlist, permissions, window, set_min_size => "window:allow-set-min-size");
  permissions!(allowlist, permissions, window, set_max_size => "window:allow-set-max-size");
  permissions!(allowlist, permissions, window, set_position => "window:allow-set-position");
  permissions!(allowlist, permissions, window, set_fullscreen => "window:allow-set-fullscreen");
  permissions!(allowlist, permissions, window, set_focus => "window:allow-set-focus");
  permissions!(allowlist, permissions, window, set_icon => "window:allow-set-icon");
  permissions!(allowlist, permissions, window, set_skip_taskbar => "window:allow-set-skip-taskbar");
  permissions!(allowlist, permissions, window, set_cursor_grab => "window:allow-set-cursor-grab");
  permissions!(allowlist, permissions, window, set_cursor_visible => "window:allow-set-cursor-visible");
  permissions!(allowlist, permissions, window, set_cursor_icon => "window:allow-set-cursor-icon");
  permissions!(allowlist, permissions, window, set_cursor_position => "window:allow-set-cursor-position");
  permissions!(allowlist, permissions, window, set_ignore_cursor_events => "window:allow-set-ignore-cursor-events");
  permissions!(allowlist, permissions, window, start_dragging => "window:allow-start-dragging");
  permissions!(allowlist, permissions, window, print => "webview:allow-print");
  // shell
  permissions!(allowlist, permissions, shell, execute => "shell:allow-execute");
  permissions!(allowlist, permissions, shell, sidecar => "shell:allow-execute");
  if allowlist.all
    || allowlist.shell.all
    || !matches!(
      allowlist.shell.open,
      tauri_utils_v1::config::ShellAllowlistOpen::Flag(false)
    )
  {
    permissions.push("shell:allow-open");
  }
  // dialog
  permissions!(allowlist, permissions, dialog, open => "dialog:allow-open");
  permissions!(allowlist, permissions, dialog, save => "dialog:allow-save");
  permissions!(allowlist, permissions, dialog, message => "dialog:allow-message");
  permissions!(allowlist, permissions, dialog, ask => "dialog:allow-ask");
  permissions!(allowlist, permissions, dialog, confirm => "dialog:allow-confirm");
  // http
  permissions!(allowlist, permissions, http, request => "http:default");
  // notification
  permissions!(allowlist, permissions, notification, all => "notification:default");
  // global-shortcut
  permissions!(allowlist, permissions, global_shortcut, all => "global-shortcut:allow-is-registered");
  permissions!(allowlist, permissions, global_shortcut, all => "global-shortcut:allow-register");
  permissions!(allowlist, permissions, global_shortcut, all => "global-shortcut:allow-register-all");
  permissions!(allowlist, permissions, global_shortcut, all => "global-shortcut:allow-unregister");
  permissions!(allowlist, permissions, global_shortcut, all => "global-shortcut:allow-unregister-all");
  // os
  permissions!(allowlist, permissions, os, all => "os:allow-platform");
  permissions!(allowlist, permissions, os, all => "os:allow-version");
  permissions!(allowlist, permissions, os, all => "os:allow-os-type");
  permissions!(allowlist, permissions, os, all => "os:allow-family");
  permissions!(allowlist, permissions, os, all => "os:allow-arch");
  permissions!(allowlist, permissions, os, all => "os:allow-exe-extension");
  permissions!(allowlist, permissions, os, all => "os:allow-locale");
  permissions!(allowlist, permissions, os, all => "os:allow-hostname");
  // process
  permissions!(allowlist, permissions, process, relaunch => "process:allow-restart");
  permissions!(allowlist, permissions, process, exit => "process:allow-exit");
  // clipboard
  permissions!(allowlist, permissions, clipboard, read_text => "clipboard-manager:allow-read");
  permissions!(allowlist, permissions, clipboard, write_text => "clipboard-manager:allow-write");
  // app
  permissions!(allowlist, permissions, app, show => "app:allow-app-show");
  permissions!(allowlist, permissions, app, hide => "app:allow-app-hide");

  permissions
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
