// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Browser settings, kept with the store plugin (`settings.json` in the app's data directory).

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const STORE: &str = "settings.json";
const KEY: &str = "settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
  /// `google`, `duckduckgo`, `bing` or `brave`.
  pub search_engine: String,
  /// Opened by the home button and new windows; empty for the new tab page.
  pub homepage: String,
  /// `system`, `light` or `dark`: the browser's pages and menus.
  pub theme: String,
  /// Zoom of new tabs.
  pub default_zoom: f64,
  pub show_home_button: bool,
  /// The app menu as a menu bar (hidden by default, like Chrome on Linux).
  pub show_menu_bar: bool,
  /// Keep running in the tray when the last window closes.
  pub keep_in_tray: bool,
  /// Show notifications (finished downloads). The tray's "Do Not Disturb" turns this off.
  pub notifications: bool,
  /// Ask before closing a window with several tabs.
  pub confirm_close: bool,
  /// Where downloads go; empty for the system's downloads directory.
  pub download_dir: String,
  /// Register Ctrl+Alt+B (show/hide the browser) system-wide.
  pub global_shortcut: bool,
  /// What the browser says it is: `chrome`, `firefox` (Google's sign-in refuses embedded
  /// Chromium engines, but not Firefox) or `engine` (QtWebEngine's own). For new tabs.
  pub user_agent: String,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      search_engine: "duckduckgo".into(),
      homepage: String::new(),
      theme: "system".into(),
      default_zoom: 1.0,
      show_home_button: true,
      show_menu_bar: false,
      keep_in_tray: false,
      notifications: true,
      confirm_close: true,
      download_dir: String::new(),
      global_shortcut: true,
      user_agent: "chrome".into(),
    }
  }
}

impl Settings {
  pub fn search_url(&self, query: &str) -> String {
    let base = match self.search_engine.as_str() {
      "google" => "https://www.google.com/search?q=",
      "bing" => "https://www.bing.com/search?q=",
      "brave" => "https://search.brave.com/search?q=",
      _ => "https://duckduckgo.com/?q=",
    };
    let q: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    format!("{base}{q}")
  }

  /// The user agent of new tabs; `None` for the engine's own.
  pub fn user_agent(&self) -> Option<&'static str> {
    match self.user_agent.as_str() {
      "firefox" => Some("Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0"),
      "engine" => None,
      _ => Some(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
         Chrome/140.0.0.0 Safari/537.36",
      ),
    }
  }

  pub fn theme(&self) -> Option<tauri::Theme> {
    match self.theme.as_str() {
      "light" => Some(tauri::Theme::Light),
      "dark" => Some(tauri::Theme::Dark),
      _ => None,
    }
  }
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> Settings {
  app
    .store(STORE)
    .ok()
    .and_then(|store| store.get(KEY))
    .and_then(|v| serde_json::from_value(v).ok())
    .unwrap_or_default()
}

pub fn save<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
  let store = app.store(STORE).map_err(|e| e.to_string())?;
  store.set(
    KEY,
    serde_json::to_value(settings).map_err(|e| e.to_string())?,
  );
  store.save().map_err(|e| e.to_string())
}
