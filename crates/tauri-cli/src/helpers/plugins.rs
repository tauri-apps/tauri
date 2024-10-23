// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

#[derive(Default)]
pub struct PluginMetadata {
  pub desktop_only: bool,
  pub mobile_only: bool,
  pub rust_only: bool,
  pub builder: bool,
  pub version_req: Option<String>,
}

// known plugins with particular cases
pub fn known_plugins() -> HashMap<&'static str, PluginMetadata> {
  let mut plugins: HashMap<&'static str, PluginMetadata> = HashMap::new();

  // desktop-only
  for p in [
    "authenticator",
    "autostart",
    "cli",
    "global-shortcut",
    "positioner",
    "single-instance",
    "updater",
    "window-state",
  ] {
    plugins.entry(p).or_default().desktop_only = true;
  }

  // mobile-only
  for p in ["barcode-scanner", "biometric", "nfc", "haptics"] {
    plugins.entry(p).or_default().mobile_only = true;
  }

  // uses builder pattern
  for p in [
    "global-shortcut",
    "localhost",
    "log",
    "sql",
    "store",
    "stronghold",
    "updater",
    "window-state",
  ] {
    plugins.entry(p).or_default().builder = true;
  }

  // rust-only
  #[allow(clippy::single_element_loop)]
  for p in ["localhost", "persisted-scope", "single-instance"] {
    plugins.entry(p).or_default().rust_only = true;
  }

  // known, but no particular config
  for p in [
    "geolocation",
    "deep-link",
    "dialog",
    "fs",
    "http",
    "notification",
    "os",
    "process",
    "shell",
    "upload",
    "websocket",
  ] {
    plugins.entry(p).or_default();
  }

  let version_req = version_req();
  for plugin in plugins.values_mut() {
    plugin.version_req.replace(version_req.clone());
  }

  plugins
}

fn version_req() -> String {
  let pre = env!("CARGO_PKG_VERSION_PRE");
  if pre.is_empty() {
    env!("CARGO_PKG_VERSION_MAJOR").to_string()
  } else {
    format!(
      "{}.0.0-{}",
      env!("CARGO_PKG_VERSION_MAJOR"),
      pre.split('.').next().unwrap()
    )
  }
}
