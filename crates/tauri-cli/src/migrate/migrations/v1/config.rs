// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;

use serde_json::{Map, Value};
use tauri_utils::acl::{
  capability::{Capability, PermissionEntry},
  Scopes, Value as AclValue,
};

use std::{
  collections::{BTreeMap, HashSet},
  fs,
  path::Path,
};

pub fn migrate(tauri_dir: &Path) -> Result<MigratedConfig> {
  if let Ok((mut config, config_path)) =
    tauri_utils_v1::config::parse::parse_value(tauri_dir.join("tauri.conf.json"))
  {
    let migrated = migrate_config(&mut config)?;
    if config_path.extension().is_some_and(|ext| ext == "toml") {
      fs::write(&config_path, toml::to_string_pretty(&config)?)?;
    } else {
      fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    }

    let mut permissions: Vec<PermissionEntry> = vec!["core:default"]
      .into_iter()
      .map(|p| PermissionEntry::PermissionRef(p.to_string().try_into().unwrap()))
      .collect();
    permissions.extend(migrated.permissions.clone());

    let capabilities_path = config_path.parent().unwrap().join("capabilities");
    fs::create_dir_all(&capabilities_path)?;
    fs::write(
      capabilities_path.join("migrated.json"),
      serde_json::to_string_pretty(&Capability {
        identifier: "migrated".to_string(),
        description: "permissions that were migrated from v1".into(),
        local: true,
        remote: None,
        windows: vec!["main".into()],
        webviews: vec![],
        permissions,
        platforms: None,
      })?,
    )?;

    return Ok(migrated);
  }

  Ok(Default::default())
}

#[derive(Default)]
pub struct MigratedConfig {
  pub permissions: Vec<PermissionEntry>,
  pub plugins: HashSet<String>,
}

fn migrate_config(config: &mut Value) -> Result<MigratedConfig> {
  let mut migrated = MigratedConfig {
    permissions: Vec::new(),
    plugins: HashSet::new(),
  };

  if let Some(config) = config.as_object_mut() {
    process_package_metadata(config);
    process_build(config);

    let mut plugins = config
      .entry("plugins")
      .or_insert_with(|| Value::Object(Default::default()))
      .as_object_mut()
      .unwrap()
      .clone();

    if let Some(tauri_config) = config.get_mut("tauri").and_then(|c| c.as_object_mut()) {
      // allowlist
      if let Some(allowlist) = tauri_config.remove("allowlist") {
        let allowlist = process_allowlist(tauri_config, allowlist)?;
        let permissions = allowlist_to_permissions(allowlist);
        migrated.plugins = plugins_from_permissions(&permissions);
        migrated.permissions = permissions;
      }

      // dangerousUseHttpScheme/useHttpsScheme
      let dangerouse_use_http = tauri_config
        .get("security")
        .and_then(|w| w.as_object())
        .and_then(|w| {
          w.get("dangerousUseHttpScheme")
            .or_else(|| w.get("dangerous-use-http-scheme"))
        })
        .and_then(|v| v.as_bool())
        .unwrap_or_default();

      if let Some(windows) = tauri_config
        .get_mut("windows")
        .and_then(|w| w.as_array_mut())
      {
        for window in windows {
          if let Some(window) = window.as_object_mut() {
            window.insert("useHttpsScheme".to_string(), (!dangerouse_use_http).into());
          }
        }
      }

      // security
      if let Some(security) = tauri_config
        .get_mut("security")
        .and_then(|c| c.as_object_mut())
      {
        process_security(security)?;
      }

      // tauri > pattern
      if let Some(pattern) = tauri_config.remove("pattern") {
        tauri_config
          .entry("security")
          .or_insert_with(|| Value::Object(Default::default()))
          .as_object_mut()
          .map(|s| s.insert("pattern".into(), pattern));
      }

      // system tray
      if let Some((tray, key)) = tauri_config
        .remove("systemTray")
        .map(|v| (v, "trayIcon"))
        .or_else(|| tauri_config.remove("system-tray").map(|v| (v, "tray-icon")))
      {
        tauri_config.insert(key.into(), tray);
      }

      // cli
      if let Some(cli) = tauri_config.remove("cli") {
        process_cli(&mut plugins, cli)?;
      }

      // updater
      process_updater(tauri_config, &mut plugins, &mut migrated)?;
    }

    process_bundle(config, &migrated);

    // if we have migrated the updater config, let's ensure createUpdaterArtifacts is set
    if plugins.contains_key("updater") {
      let bundle_config = config
        .entry("bundle")
        .or_insert_with(|| Value::Object(Default::default()))
        .as_object_mut()
        .unwrap();
      if !bundle_config.contains_key("createUpdaterArtifacts") {
        bundle_config.insert("createUpdaterArtifacts".to_owned(), "v1Compatible".into());
      }
    }

    config.insert("plugins".into(), plugins.into());

    if let Some(tauri_config) = config.remove("tauri") {
      config.insert("app".into(), tauri_config);
    }
  }

  Ok(migrated)
}

fn process_package_metadata(config: &mut Map<String, Value>) {
  if let Some(mut package_config) = config.remove("package") {
    if let Some(package_config) = package_config.as_object_mut() {
      if let Some((product_name, key)) = package_config
        .remove("productName")
        .map(|v| (v, "productName"))
        .or_else(|| {
          package_config
            .remove("product-name")
            .map(|v| (v, "product-name"))
        })
      {
        config.insert(key.into(), product_name.clone());
        // keep main binary name unchanged
        config.insert("mainBinaryName".into(), product_name);
      }

      if let Some(version) = package_config.remove("version") {
        config.insert("version".into(), version);
      }
    }
  }

  if let Some(bundle_config) = config
    .get_mut("tauri")
    .and_then(|t| t.get_mut("bundle"))
    .and_then(|b| b.as_object_mut())
  {
    if let Some(identifier) = bundle_config.remove("identifier") {
      config.insert("identifier".into(), identifier);
    }
  }
}

fn process_build(config: &mut Map<String, Value>) {
  if let Some(build_config) = config.get_mut("build").and_then(|b| b.as_object_mut()) {
    if let Some((dist_dir, key)) = build_config
      .remove("distDir")
      .map(|v| (v, "frontendDist"))
      .or_else(|| {
        build_config
          .remove("dist-dir")
          .map(|v| (v, "frontend-dist"))
      })
    {
      build_config.insert(key.into(), dist_dir);
    }
    if let Some((dev_path, key)) = build_config
      .remove("devPath")
      .map(|v| (v, "devUrl"))
      .or_else(|| build_config.remove("dev-path").map(|v| (v, "dev-url")))
    {
      let is_url = url::Url::parse(dev_path.as_str().unwrap_or_default()).is_ok();
      if is_url {
        build_config.insert(key.into(), dev_path);
      }
    }
    if let Some((with_global_tauri, key)) = build_config
      .remove("withGlobalTauri")
      .map(|v| (v, "withGlobalTauri"))
      .or_else(|| {
        build_config
          .remove("with-global-tauri")
          .map(|v| (v, "with-global-tauri"))
      })
    {
      config
        .get_mut("tauri")
        .and_then(|t| t.as_object_mut())
        .map(|t| t.insert(key.into(), with_global_tauri));
    }
  }
}

fn process_bundle(config: &mut Map<String, Value>, migrated: &MigratedConfig) {
  let mut license_file = None;

  if let Some(mut bundle_config) = config
    .get_mut("tauri")
    .and_then(|b| b.as_object_mut())
    .and_then(|t| t.remove("bundle"))
  {
    if let Some(bundle_config) = bundle_config.as_object_mut() {
      if let Some(deb) = bundle_config.remove("deb") {
        bundle_config
          .entry("linux")
          .or_insert_with(|| Value::Object(Default::default()))
          .as_object_mut()
          .map(|l| l.insert("deb".into(), deb));
      }

      if let Some(appimage) = bundle_config.remove("appimage") {
        bundle_config
          .entry("linux")
          .or_insert_with(|| Value::Object(Default::default()))
          .as_object_mut()
          .map(|l| l.insert("appimage".into(), appimage));
      }

      if let Some(rpm) = bundle_config.remove("rpm") {
        bundle_config
          .entry("linux")
          .or_insert_with(|| Value::Object(Default::default()))
          .as_object_mut()
          .map(|l| l.insert("rpm".into(), rpm));
      }

      if let Some(dmg) = bundle_config.remove("dmg") {
        bundle_config
          .entry("macOS")
          .or_insert_with(|| Value::Object(Default::default()))
          .as_object_mut()
          .map(|l| l.insert("dmg".into(), dmg));
      }

      // license file
      if let Some(macos) = bundle_config
        .get_mut("macOS")
        .and_then(|v| v.as_object_mut())
      {
        if let Some(license) = macos.remove("license") {
          license_file = Some(license);
        }
      }

      // Windows
      if let Some(windows) = bundle_config.get_mut("windows") {
        if let Some(windows) = windows.as_object_mut() {
          if let Some(wix) = windows.get_mut("wix").and_then(|v| v.as_object_mut()) {
            if let Some(license_path) = wix.remove("license") {
              license_file = Some(license_path);
            }
          }
          if let Some(nsis) = windows.get_mut("nsis").and_then(|v| v.as_object_mut()) {
            if let Some(license_path) = nsis.remove("license") {
              license_file = Some(license_path);
            }
          }

          if let Some((fixed_runtime_path, key)) = windows
            .remove("webviewFixedRuntimePath")
            .map(|v| (v, "webviewInstallMode"))
            .or_else(|| {
              windows
                .remove("webview-fixed-runtime-path")
                .map(|v| (v, "webview-install-mode"))
            })
          {
            if !fixed_runtime_path.is_null() {
              windows.insert(
                key.into(),
                serde_json::json!({
                  "type": "fixedRuntime",
                  "path": fixed_runtime_path
                }),
              );
            }
          }
        }
      }
      if let Some(license_file) = license_file {
        bundle_config.insert("licenseFile".into(), license_file);
      }

      // Migrate updater from targets to update field
      if let Some(targets) = bundle_config.get_mut("targets") {
        let should_migrate = if let Some(targets) = targets.as_array_mut() {
          // targets: ["updater", ...]
          if let Some(index) = targets
            .iter()
            .position(|target| *target == serde_json::Value::String("updater".to_owned()))
          {
            targets.remove(index);
            true
          } else {
            false
          }
        } else if let Some(target) = targets.as_str() {
          // targets: "updater"
          if target == "updater" {
            bundle_config.remove("targets");
            true
          } else {
            // note that target == "all" is the default from the v1 tauri CLI
            // so we shouldn't bindly force updater bundles to be created
            // instead we only migrate if the updater has been migrated
            target == "all" && migrated.plugins.contains("updater")
          }
        } else {
          false
        };
        if should_migrate {
          bundle_config.insert("createUpdaterArtifacts".to_owned(), "v1Compatible".into());
        }
      }
    }

    config.insert("bundle".into(), bundle_config);
  }
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

  // dangerous_remote_domain_ipc_access no longer exists
  if let Some(dangerous_remote_domain_ipc_access) = security
    .remove("dangerousRemoteDomainIpcAccess")
    .or_else(|| security.remove("dangerous-remote-domain-ipc-access"))
  {
    println!("dangerous remote domain IPC access config ({dangerous_remote_domain_ipc_access:?}) no longer exists, see documentation for capabilities and remote access: https://v2.tauri.app/security/capabilities/#remote-api-access")
  }
  security
    .remove("dangerousUseHttpScheme")
    .or_else(|| security.remove("dangerous-use-http-scheme"));

  Ok(())
}

fn process_allowlist(
  tauri_config: &mut Map<String, Value>,
  allowlist: Value,
) -> Result<tauri_utils_v1::config::AllowlistConfig> {
  let allowlist: tauri_utils_v1::config::AllowlistConfig = serde_json::from_value(allowlist)?;

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
  allowlist: tauri_utils_v1::config::AllowlistConfig,
) -> Vec<PermissionEntry> {
  macro_rules! permissions {
    ($allowlist: ident, $permissions_list: ident, $object: ident, $field: ident => $associated_permission: expr) => {{
      if $allowlist.all || $allowlist.$object.all || $allowlist.$object.$field {
        $permissions_list.push(PermissionEntry::PermissionRef(
          $associated_permission.to_string().try_into().unwrap(),
        ));
        true
      } else {
        false
      }
    }};
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
  let (fs_allowed, fs_denied) = match allowlist.fs.scope {
    tauri_utils_v1::config::FsAllowlistScope::AllowedPaths(paths) => (paths, Vec::new()),
    tauri_utils_v1::config::FsAllowlistScope::Scope { allow, deny, .. } => (allow, deny),
  };
  if !(fs_allowed.is_empty() && fs_denied.is_empty()) {
    let fs_allowed = fs_allowed
      .into_iter()
      .map(|p| AclValue::String(p.to_string_lossy().into()))
      .collect::<Vec<_>>();
    let fs_denied = fs_denied
      .into_iter()
      .map(|p| AclValue::String(p.to_string_lossy().into()))
      .collect::<Vec<_>>();
    permissions.push(PermissionEntry::ExtendedPermission {
      identifier: "fs:scope".to_string().try_into().unwrap(),
      scope: Scopes {
        allow: if fs_allowed.is_empty() {
          None
        } else {
          Some(fs_allowed)
        },
        deny: if fs_denied.is_empty() {
          None
        } else {
          Some(fs_denied)
        },
      },
    });
  }

  // window
  permissions!(allowlist, permissions, window, create => "core:window:allow-create");
  permissions!(allowlist, permissions, window, center => "core:window:allow-center");
  permissions!(allowlist, permissions, window, request_user_attention => "core:window:allow-request-user-attention");
  permissions!(allowlist, permissions, window, set_resizable => "core:window:allow-set-resizable");
  permissions!(allowlist, permissions, window, set_maximizable => "core:window:allow-set-maximizable");
  permissions!(allowlist, permissions, window, set_minimizable => "core:window:allow-set-minimizable");
  permissions!(allowlist, permissions, window, set_closable => "core:window:allow-set-closable");
  permissions!(allowlist, permissions, window, set_title => "core:window:allow-set-title");
  permissions!(allowlist, permissions, window, maximize => "core:window:allow-maximize");
  permissions!(allowlist, permissions, window, unmaximize => "core:window:allow-unmaximize");
  permissions!(allowlist, permissions, window, minimize => "core:window:allow-minimize");
  permissions!(allowlist, permissions, window, unminimize => "core:window:allow-unminimize");
  permissions!(allowlist, permissions, window, show => "core:window:allow-show");
  permissions!(allowlist, permissions, window, hide => "core:window:allow-hide");
  permissions!(allowlist, permissions, window, close => "core:window:allow-close");
  permissions!(allowlist, permissions, window, set_decorations => "core:window:allow-set-decorations");
  permissions!(allowlist, permissions, window, set_always_on_top => "core:window:allow-set-always-on-top");
  permissions!(allowlist, permissions, window, set_content_protected => "core:window:allow-set-content-protected");
  permissions!(allowlist, permissions, window, set_size => "core:window:allow-set-size");
  permissions!(allowlist, permissions, window, set_min_size => "core:window:allow-set-min-size");
  permissions!(allowlist, permissions, window, set_max_size => "core:window:allow-set-max-size");
  permissions!(allowlist, permissions, window, set_position => "core:window:allow-set-position");
  permissions!(allowlist, permissions, window, set_fullscreen => "core:window:allow-set-fullscreen");
  permissions!(allowlist, permissions, window, set_focus => "core:window:allow-set-focus");
  permissions!(allowlist, permissions, window, set_icon => "core:window:allow-set-icon");
  permissions!(allowlist, permissions, window, set_skip_taskbar => "core:window:allow-set-skip-taskbar");
  permissions!(allowlist, permissions, window, set_cursor_grab => "core:window:allow-set-cursor-grab");
  permissions!(allowlist, permissions, window, set_cursor_visible => "core:window:allow-set-cursor-visible");
  permissions!(allowlist, permissions, window, set_cursor_icon => "core:window:allow-set-cursor-icon");
  permissions!(allowlist, permissions, window, set_cursor_position => "core:window:allow-set-cursor-position");
  permissions!(allowlist, permissions, window, set_ignore_cursor_events => "core:window:allow-set-ignore-cursor-events");
  permissions!(allowlist, permissions, window, start_dragging => "core:window:allow-start-dragging");
  permissions!(allowlist, permissions, window, print => "core:webview:allow-print");

  // shell
  if allowlist.shell.scope.0.is_empty() {
    let added = permissions!(allowlist, permissions, shell, execute => "shell:allow-execute");
    // prevent duplicated permission
    if !added {
      permissions!(allowlist, permissions, shell, sidecar => "shell:allow-execute");
    }
  } else {
    let allowed = allowlist
      .shell
      .scope
      .0
      .into_iter()
      .map(|p| serde_json::to_value(p).unwrap().into())
      .collect::<Vec<_>>();

    permissions.push(PermissionEntry::ExtendedPermission {
      identifier: "shell:allow-execute".to_string().try_into().unwrap(),
      scope: Scopes {
        allow: Some(allowed),
        deny: None,
      },
    });
  }

  if allowlist.all
    || allowlist.shell.all
    || !matches!(
      allowlist.shell.open,
      tauri_utils_v1::config::ShellAllowlistOpen::Flag(false)
    )
  {
    permissions.push(PermissionEntry::PermissionRef(
      "shell:allow-open".to_string().try_into().unwrap(),
    ));
  }
  // dialog
  permissions!(allowlist, permissions, dialog, open => "dialog:allow-open");
  permissions!(allowlist, permissions, dialog, save => "dialog:allow-save");
  permissions!(allowlist, permissions, dialog, message => "dialog:allow-message");
  permissions!(allowlist, permissions, dialog, ask => "dialog:allow-ask");
  permissions!(allowlist, permissions, dialog, confirm => "dialog:allow-confirm");

  // http
  if allowlist.http.scope.0.is_empty() {
    permissions!(allowlist, permissions, http, request => "http:default");
  } else {
    let allowed = allowlist
      .http
      .scope
      .0
      .into_iter()
      .map(|p| {
        let mut map = BTreeMap::new();
        map.insert("url".to_string(), AclValue::String(p.to_string()));
        AclValue::Map(map)
      })
      .collect::<Vec<_>>();

    permissions.push(PermissionEntry::ExtendedPermission {
      identifier: "http:default".to_string().try_into().unwrap(),
      scope: Scopes {
        allow: Some(allowed),
        deny: None,
      },
    });
  }

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
  permissions!(allowlist, permissions, clipboard, read_text => "clipboard-manager:allow-read-text");
  permissions!(allowlist, permissions, clipboard, write_text => "clipboard-manager:allow-write-text");
  // app
  permissions!(allowlist, permissions, app, show => "core:app:allow-app-show");
  permissions!(allowlist, permissions, app, hide => "core:app:allow-app-hide");

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
  migrated: &mut MigratedConfig,
) -> Result<()> {
  if let Some(mut updater) = tauri_config.remove("updater") {
    if let Some(updater) = updater.as_object_mut() {
      updater.remove("dialog");

      // we only migrate the updater config if it's active
      // since we now assume it's always active if the config object is set
      // we also migrate if pubkey is set so we do not lose that information on the migration
      // in this case, the user need to deal with the updater being inactive on their own
      if updater
        .remove("active")
        .and_then(|a| a.as_bool())
        .unwrap_or_default()
        || updater.get("pubkey").is_some()
      {
        plugins.insert("updater".into(), serde_json::to_value(updater)?);
        migrated.plugins.insert("updater".to_string());
      }
    }
  }

  Ok(())
}

const KNOWN_PLUGINS: &[&str] = &[
  "fs",
  "shell",
  "dialog",
  "http",
  "notification",
  "global-shortcut",
  "os",
  "process",
  "clipboard-manager",
];

fn plugins_from_permissions(permissions: &Vec<PermissionEntry>) -> HashSet<String> {
  let mut plugins = HashSet::new();

  for permission in permissions {
    let permission = permission.identifier().get();
    for plugin in KNOWN_PLUGINS {
      if permission.starts_with(plugin) {
        plugins.insert(plugin.to_string());
        break;
      }
    }
  }

  plugins
}

#[cfg(test)]
mod test {
  fn migrate(original: &serde_json::Value) -> serde_json::Value {
    let mut migrated = original.clone();
    super::migrate_config(&mut migrated).expect("failed to migrate config");

    if original.get("$schema").is_some() {
      if let Some(map) = migrated.as_object_mut() {
        map.insert(
          "$schema".to_string(),
          serde_json::Value::String("https://schema.tauri.app/config/2".to_string()),
        );
      }
    }

    if original
      .get("tauri")
      .and_then(|v| v.get("bundle"))
      .and_then(|v| v.get("identifier"))
      .is_none()
    {
      if let Some(map) = migrated.as_object_mut() {
        map.insert(
          "identifier".to_string(),
          serde_json::Value::String("com.tauri.test-injected".to_string()),
        );
      }
    }

    if let Err(e) = serde_json::from_value::<tauri_utils::config::Config>(migrated.clone()) {
      panic!("migrated config is not valid: {e}");
    }

    migrated
  }

  #[test]
  fn migrate_full() {
    let original = serde_json::json!({
      "$schema": "../node_modules/@tauri-apps/cli/schema.json",
      "build": {
        "distDir": "../dist",
        "devPath": "http://localhost:1240",
        "withGlobalTauri": true
      },
      "package": {
        "productName": "Tauri app",
        "version": "0.0.0"
      },
      "tauri": {
        "bundle": {
          "identifier": "com.tauri.test",
          "deb": {
            "depends": ["dep1"]
          },
          "appimage": {
            "bundleMediaFramework": true
          },
          "macOS": {
            "license": "license-file.txt"
          },
          "windows": {
            "wix": {
              "license": "license-file.txt"
            },
            "nsis": {
              "license": "license-file.txt"
            },
          },
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
            "installerArgs": ["arg1"],
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
        "pattern": { "use": "brownfield" },
        "security": {
          "csp": "default-src 'self' tauri:"
        },
        "windows": [{}]
      }
    });

    let migrated = migrate(&original);

    // $schema
    assert_eq!(migrated["$schema"], "https://schema.tauri.app/config/2");

    // plugins > updater
    assert_eq!(
      migrated["plugins"]["updater"]["endpoints"],
      original["tauri"]["updater"]["endpoints"]
    );
    assert_eq!(
      migrated["plugins"]["updater"]["pubkey"],
      original["tauri"]["updater"]["pubkey"]
    );
    assert_eq!(
      migrated["plugins"]["updater"]["windows"]["installMode"],
      original["tauri"]["updater"]["windows"]["installMode"]
    );
    assert_eq!(
      migrated["plugins"]["updater"]["windows"]["installerArgs"],
      original["tauri"]["updater"]["windows"]["installerArgs"]
    );

    // cli
    assert_eq!(migrated["plugins"]["cli"], original["tauri"]["cli"]);

    // asset scope
    assert_eq!(
      migrated["app"]["security"]["assetProtocol"]["enable"],
      original["tauri"]["allowlist"]["protocol"]["asset"]
    );
    assert_eq!(
      migrated["app"]["security"]["assetProtocol"]["scope"]["allow"],
      original["tauri"]["allowlist"]["protocol"]["assetScope"]["allow"]
    );
    assert_eq!(
      migrated["app"]["security"]["assetProtocol"]["scope"]["deny"],
      original["tauri"]["allowlist"]["protocol"]["assetScope"]["deny"]
    );

    // security CSP
    assert_eq!(
      migrated["app"]["security"]["csp"],
      format!(
        "{}; connect-src ipc: http://ipc.localhost",
        original["tauri"]["security"]["csp"].as_str().unwrap()
      )
    );

    // security pattern
    assert_eq!(
      migrated["app"]["security"]["pattern"],
      original["tauri"]["pattern"]
    );

    // license files
    assert_eq!(
      migrated["bundle"]["licenseFile"],
      original["tauri"]["bundle"]["macOS"]["license"]
    );
    assert_eq!(
      migrated["bundle"]["licenseFile"],
      original["tauri"]["bundle"]["windows"]["wix"]["license"]
    );
    assert_eq!(
      migrated["bundle"]["licenseFile"],
      original["tauri"]["bundle"]["windows"]["nsis"]["license"]
    );

    // bundle appimage and deb
    assert_eq!(
      migrated["bundle"]["linux"]["deb"],
      original["tauri"]["bundle"]["deb"]
    );
    assert_eq!(
      migrated["bundle"]["linux"]["appimage"],
      original["tauri"]["bundle"]["appimage"]
    );

    // app information
    assert_eq!(migrated["productName"], original["package"]["productName"]);
    assert_eq!(
      migrated["mainBinaryName"],
      original["package"]["productName"]
    );
    assert_eq!(migrated["version"], original["package"]["version"]);
    assert_eq!(
      migrated["identifier"],
      original["tauri"]["bundle"]["identifier"]
    );

    // build object
    assert_eq!(
      migrated["build"]["frontendDist"],
      original["build"]["distDir"]
    );
    assert_eq!(migrated["build"]["devUrl"], original["build"]["devPath"]);
    assert_eq!(
      migrated["app"]["withGlobalTauri"],
      original["build"]["withGlobalTauri"]
    );

    assert_eq!(migrated["app"]["windows"][0]["useHttpsScheme"], true);
  }

  #[test]
  fn skips_migrating_updater() {
    let original = serde_json::json!({
      "tauri": {
        "updater": {
          "active": false
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(migrated["plugins"]["updater"], serde_json::Value::Null);
  }

  #[test]
  fn migrate_updater_pubkey() {
    let original = serde_json::json!({
      "tauri": {
        "updater": {
          "active": false,
          "pubkey": "somekey"
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(
      migrated["plugins"]["updater"]["pubkey"],
      original["tauri"]["updater"]["pubkey"]
    );
  }

  #[test]
  fn migrate_dangerous_use_http_scheme() {
    let original = serde_json::json!({
      "tauri": {
        "windows": [{}],
        "security": {
          "dangerousUseHttpScheme": true,
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(
      !migrated["app"]["windows"][0]["useHttpsScheme"]
        .as_bool()
        .unwrap(),
      original["tauri"]["security"]["dangerousUseHttpScheme"]
        .as_bool()
        .unwrap()
    );
  }

  #[test]
  fn can_migrate_default_config() {
    let original = serde_json::to_value(tauri_utils_v1::config::Config::default()).unwrap();
    migrate(&original);
  }

  #[test]
  fn can_migrate_api_example_config() {
    let original =
      serde_json::from_str(include_str!("./fixtures/api-example.tauri.conf.json")).unwrap();
    migrate(&original);
  }

  #[test]
  fn can_migrate_cli_template_config() {
    let original =
      serde_json::from_str(include_str!("./fixtures/cli-template.tauri.conf.json")).unwrap();
    migrate(&original);
  }

  #[test]
  fn migrate_updater_target() {
    let original = serde_json::json!({});

    let migrated = migrate(&original);
    assert_eq!(
      migrated["bundle"]["createUpdaterArtifacts"],
      serde_json::Value::Null
    );

    let original = serde_json::json!({
      "tauri": {
        "updater": {
          "active": true
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(migrated["bundle"]["createUpdaterArtifacts"], "v1Compatible");

    let original = serde_json::json!({
      "tauri": {
        "bundle": {
          "targets": ["nsis", "updater"]
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(migrated["bundle"]["createUpdaterArtifacts"], "v1Compatible");
    assert_eq!(
      migrated["bundle"]["targets"].as_array(),
      Some(&vec!["nsis".into()])
    );

    let original =
      serde_json::from_str(include_str!("./fixtures/cli-template.tauri.conf.json")).unwrap();
    let migrated = migrate(&original);
    assert_eq!(
      migrated["bundle"]["createUpdaterArtifacts"],
      serde_json::Value::Null
    );

    let original = serde_json::json!({
      "tauri": {
        "bundle": {
          "targets": "all"
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(
      migrated["bundle"]["createUpdaterArtifacts"],
      serde_json::Value::Null
    );
    assert_eq!(migrated["bundle"]["targets"], "all");

    let original = serde_json::json!({
      "tauri": {
        "bundle": {
          "targets": "all"
        },
        "updater": {
          "active": true
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(migrated["bundle"]["createUpdaterArtifacts"], "v1Compatible");
    assert_eq!(migrated["bundle"]["targets"], "all");

    let original = serde_json::json!({
      "tauri": {
        "bundle": {
          "targets": "updater"
        }
      }
    });

    let migrated = migrate(&original);
    assert_eq!(migrated["bundle"]["createUpdaterArtifacts"], "v1Compatible");
    assert_eq!(migrated["bundle"].get("targets"), None);
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
      migrated["app"]["security"]["csp"]["default-src"],
      original["tauri"]["security"]["csp"]["default-src"]
    );
    assert!(migrated["app"]["security"]["csp"]["connect-src"]
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
      migrated["app"]["security"]["csp"]["default-src"],
      original["tauri"]["security"]["csp"]["default-src"]
    );
    assert_eq!(
      migrated["app"]["security"]["csp"]["connect-src"]
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
      migrated["app"]["security"]["csp"]["default-src"],
      original["tauri"]["security"]["csp"]["default-src"]
    );

    let migrated_connect_src = migrated["app"]["security"]["csp"]["connect-src"]
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

  #[test]
  fn migrate_invalid_url_dev_path() {
    let original = serde_json::json!({
      "build": {
        "devPath": "../src",
        "distDir": "../src"
      }
    });

    let migrated = migrate(&original);

    assert!(migrated["build"].get("devUrl").is_none());
    assert_eq!(
      migrated["build"]["distDir"],
      original["build"]["frontendDist"]
    );
  }

  #[test]
  fn migrate_webview_fixed_runtime_path() {
    let original = serde_json::json!({
      "tauri": {
        "bundle": {
          "windows": {
            "webviewFixedRuntimePath": "./path/to/runtime"
          }
        }
      }
    });

    let migrated = migrate(&original);

    assert_eq!(
      migrated["bundle"]["windows"]["webviewInstallMode"]["type"],
      "fixedRuntime"
    );

    assert_eq!(
      migrated["bundle"]["windows"]["webviewInstallMode"]["path"],
      original["tauri"]["bundle"]["windows"]["webviewFixedRuntimePath"]
    );
  }
}
