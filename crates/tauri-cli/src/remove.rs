// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Parser;

use crate::{
  Result, acl,
  helpers::{app_paths::resolve_frontend_dir, cargo, npm::PackageManager},
};

#[derive(Debug, Parser)]
#[clap(about = "Remove a tauri plugin from the project")]
pub struct Options {
  /// The plugin to remove.
  pub plugin: String,
}

pub fn command(options: Options) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();
  let plugin = options.plugin;

  let crate_name = format!("tauri-plugin-{plugin}");

  let mut plugins = crate::helpers::plugins::known_plugins();
  let (metadata, is_known) = plugins
    .remove(plugin.as_str())
    .map(|metadata| (metadata, true))
    .unwrap_or_default();

  let frontend_dir = resolve_frontend_dir();

  let target_str = metadata
    .desktop_only
    .then_some(r#"cfg(not(any(target_os = "android", target_os = "ios")))"#)
    .or_else(|| {
      metadata
        .mobile_only
        .then_some(r#"cfg(any(target_os = "android", target_os = "ios"))"#)
    });

  cargo::uninstall_one(cargo::CargoUninstallOptions {
    name: &crate_name,
    cwd: Some(dirs.tauri),
    target: target_str,
  })?;

  if !metadata.rust_only {
    // `tauri add` only installs the JS bindings of known plugins; for community plugins, only
    // remove the package older CLI versions installed, and only if the project actually uses it
    let npm_name = if is_known {
      format!("@tauri-apps/plugin-{plugin}")
    } else {
      format!("tauri-plugin-{plugin}-api")
    };
    if let Some(frontend_dir) = frontend_dir {
      let package_json = std::fs::read(dirs.frontend.join("package.json"))
        .ok()
        .and_then(|content| serde_json::from_slice::<serde_json::Value>(&content).ok());
      if package_json
        .as_ref()
        .is_some_and(|package_json| lists_dependency(package_json, &npm_name))
      {
        let manager = PackageManager::from_project(frontend_dir);
        // not fatal: the ACL cleanup below must still run
        if let Err(e) = manager.remove(std::slice::from_ref(&npm_name), dirs.frontend) {
          log::warn!("Failed to remove `{npm_name}`, remove it manually: {e}");
        }
      }
    }

    acl::permission::rm::command(acl::permission::rm::Options {
      identifier: format!("{plugin}:*"),
    })?;
  }

  log::info!("Now, you must manually remove the plugin from your Rust code.",);

  Ok(())
}

/// Whether the given `package.json` lists `name` as any kind of dependency.
fn lists_dependency(package_json: &serde_json::Value, name: &str) -> bool {
  [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
  ]
  .iter()
  .any(|key| {
    package_json
      .get(key)
      .and_then(|deps| deps.get(name))
      .is_some()
  })
}

#[cfg(test)]
mod tests {
  use super::lists_dependency;
  use serde_json::json;

  #[test]
  fn finds_dependency_in_any_section() {
    let package_json = json!({
      "dependencies": { "@tauri-apps/plugin-fs": "~2" },
      "devDependencies": { "tauri-plugin-foo-api": "^1" },
    });
    assert!(lists_dependency(&package_json, "@tauri-apps/plugin-fs"));
    assert!(lists_dependency(&package_json, "tauri-plugin-foo-api"));
    assert!(!lists_dependency(&package_json, "@tauri-apps/plugin-os"));
    assert!(!lists_dependency(&json!({}), "@tauri-apps/plugin-fs"));
  }
}
