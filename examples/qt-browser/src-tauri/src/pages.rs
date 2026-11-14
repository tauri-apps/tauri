// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The browser's own pages, served by a custom URI scheme as `qtbrowser://newtab`,
//! `qtbrowser://settings`, `qtbrowser://downloads` and `qtbrowser://about`, and the commands
//! behind the settings and About pages.

use serde_json::json;
use tauri::{
  AppHandle, Manager, Runtime, UriSchemeContext, UriSchemeResponder, Webview,
  http::{Request, Response, StatusCode, header},
};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
  browser::{self, browser},
  downloads, menus,
  settings::{self, Settings},
};

/// The pages, by host (`qtbrowser://<host>/`).
const PAGES: &[&str] = &["newtab", "settings", "downloads", "about"];

/// The `qtbrowser://` scheme: `qtbrowser://settings/` is `ui/settings.html`, and other paths
/// are the app's assets (`qtbrowser://settings/theme.css` is `ui/theme.css`). Tauri treats pages
/// on the app's own schemes as local, so they get the browser-ui capability.
pub fn protocol<R: Runtime>(
  ctx: UriSchemeContext<'_, R>,
  request: Request<Vec<u8>>,
  responder: UriSchemeResponder,
) {
  let page = request.uri().host().unwrap_or("newtab").to_string();
  let path = request.uri().path().trim_start_matches('/').to_string();
  let file = if path.is_empty() {
    PAGES
      .contains(&page.as_str())
      .then(|| format!("{page}.html"))
  } else {
    Some(path)
  };
  let asset = file.and_then(|f| ctx.app_handle().asset_resolver().get(f));
  let response = match asset {
    Some(asset) => {
      let mut b = Response::builder().header(header::CONTENT_TYPE, asset.mime_type);
      if let Some(csp) = asset.csp_header {
        b = b.header(header::CONTENT_SECURITY_POLICY, csp);
      }
      b.body(asset.bytes)
    }
    None => Response::builder()
      .status(StatusCode::NOT_FOUND)
      .header(header::CONTENT_TYPE, "text/html")
      .body(format!("<title>Not found</title><h1>{page}: no such page</h1>").into_bytes()),
  };
  responder.respond(response.unwrap());
}

// --- settings -------------------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> serde_json::Value {
  let settings = browser(&app).settings.clone();
  let autostart = app.autostart().is_enabled().unwrap_or(false);
  json!({ "settings": settings, "autostart": autostart })
}

/// Save the settings (store plugin) and apply the changes.
#[tauri::command]
pub fn save_settings<R: Runtime>(app: AppHandle<R>, settings: Settings) -> Result<(), String> {
  settings::save(&app, &settings)?;
  let previous = std::mem::replace(&mut browser(&app).settings, settings.clone());
  apply(&app, Some(&previous), &settings);
  Ok(())
}

/// Apply `settings` (at startup, `previous` is `None`).
pub fn apply<R: Runtime>(app: &AppHandle<R>, previous: Option<&Settings>, settings: &Settings) {
  if previous.is_none_or(|p| p.theme != settings.theme) {
    // The app theme also styles the native menus.
    app.set_theme(settings.theme());
  }
  if previous.is_some_and(|p| p.show_menu_bar != settings.show_menu_bar) {
    browser::set_menu_bar(app, settings.show_menu_bar);
  }
  if previous.is_none_or(|p| p.global_shortcut != settings.global_shortcut) {
    set_global_shortcut(app, settings.global_shortcut);
  }
  downloads::allow_download_dir(app);
  menus::sync_checks(app);
  browser::refresh_all(app);
}

/// The system-wide shortcut that shows/hides the browser (global-shortcut plugin). Wayland
/// compositors don't let apps grab keys: registering fails there, which the About page reports.
pub const GLOBAL_SHORTCUT: &str = "Ctrl+Alt+B";

pub fn set_global_shortcut<R: Runtime>(app: &AppHandle<R>, on: bool) {
  let gs = app.global_shortcut();
  let result = if on {
    gs.on_shortcut(GLOBAL_SHORTCUT, |app, _, event| {
      if event.state == ShortcutState::Pressed {
        menus::toggle_browser(app);
      }
    })
  } else {
    gs.unregister(GLOBAL_SHORTCUT)
  };
  if let Err(e) = result {
    log::warn!("global shortcut {GLOBAL_SHORTCUT}: {e}");
  }
}

/// Cookies, cache and storage of every site (the tabs share one profile).
#[tauri::command]
pub fn clear_browsing_data<R: Runtime>(webview: Webview<R>) -> Result<(), String> {
  webview.clear_all_browsing_data().map_err(|e| e.to_string())
}

/// Pick the download folder (dialog plugin). Returns the folder, or `None` if cancelled.
#[tauri::command]
pub async fn pick_download_dir<R: Runtime>(app: AppHandle<R>) -> Option<String> {
  app
    .dialog()
    .file()
    .set_title("Download folder")
    .blocking_pick_folder()
    .and_then(|f| f.into_path().ok())
    .map(|p| p.to_string_lossy().into_owned())
}

/// Start the browser at login (autostart plugin).
#[tauri::command]
pub fn set_autostart<R: Runtime>(app: AppHandle<R>, enabled: bool) -> Result<bool, String> {
  let auto = app.autostart();
  if enabled {
    auto.enable()
  } else {
    auto.disable()
  }
  .map_err(|e| e.to_string())?;
  auto.is_enabled().map_err(|e| e.to_string())
}

// --- about ----------------------------------------------------------------------------------------

/// What the About page shows besides what it gets from the os, app and process JS APIs.
#[tauri::command]
pub fn about_info<R: Runtime>(app: AppHandle<R>) -> serde_json::Value {
  let path = app.path();
  let dir =
    |r: tauri::Result<std::path::PathBuf>| r.map(|p| p.display().to_string()).unwrap_or_default();
  json!({
    "tauri": tauri::VERSION,
    "webview": app.webview_version().unwrap_or_default(),
    "globalShortcut": {
      "keys": GLOBAL_SHORTCUT,
      "registered": app.global_shortcut().is_registered(GLOBAL_SHORTCUT),
    },
    "paths": {
      "config": dir(path.app_config_dir()),
      "data": dir(path.app_data_dir()),
      "logs": dir(path.app_log_dir()),
      "downloads": dir(path.download_dir()),
    },
  })
}
