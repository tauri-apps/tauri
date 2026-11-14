// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Commands of the title bar (`ui/chrome.js`) and the new tab page: tabs, navigation, windows.
//! Web pages may only call `shortcut` (capabilities/web-pages.json).

use tauri::{AppHandle, Runtime, Url, Webview};

use crate::browser::{self, browser, resolve_input, tab_webview};

/// The window a command came from.
fn window_of<R: Runtime>(webview: &Webview<R>) -> String {
  webview.window().label().to_string()
}

#[tauri::command]
pub fn browser_state<R: Runtime>(app: AppHandle<R>, webview: Webview<R>) -> serde_json::Value {
  browser::snapshot(&app, &window_of(&webview))
}

/// A new tab with `url` (address bar input; the new tab page without one).
#[tauri::command]
pub fn new_tab<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  url: Option<String>,
) -> Result<u32, String> {
  let settings = browser(&app).settings.clone();
  let url = url.map_or_else(
    || browser::internal_url("newtab"),
    |u| resolve_input(&u, &settings),
  );
  browser::open_tab(&app, &window_of(&webview), url, true, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_tab<R: Runtime>(app: AppHandle<R>, id: u32) {
  browser::close_tab(&app, id);
}

#[tauri::command]
pub fn activate_tab<R: Runtime>(app: AppHandle<R>, id: u32) {
  browser::activate_tab(&app, id, true);
}

/// Move tab `id` to position `index` in its strip.
#[tauri::command]
pub fn move_tab<R: Runtime>(app: AppHandle<R>, id: u32, index: usize) {
  let window = {
    let mut inner = browser(&app);
    let Some(window) = inner.tabs.get(&id).map(|t| t.window.clone()) else {
      return;
    };
    if let Some(w) = inner.windows.get_mut(&window) {
      w.tabs.retain(|t| *t != id);
      let index = index.min(w.tabs.len());
      w.tabs.insert(index, id);
    }
    window
  };
  browser::refresh(&app, &window);
}

/// Load address bar input (a URL, a host, an internal page or a search) in tab `id`.
#[tauri::command]
pub fn navigate<R: Runtime>(app: AppHandle<R>, id: u32, input: String) -> Result<(), String> {
  let settings = browser(&app).settings.clone();
  let url: Url = resolve_input(&input, &settings);
  let webview = tab_webview(&app, id).ok_or("no such tab")?;
  webview.navigate(url).map_err(|e| e.to_string())?;
  let _ = webview.set_focus();
  Ok(())
}

#[tauri::command]
pub fn go_back<R: Runtime>(app: AppHandle<R>, id: u32) {
  if let Some(w) = tab_webview(&app, id) {
    let _ = w.go_back();
  }
}

#[tauri::command]
pub fn go_forward<R: Runtime>(app: AppHandle<R>, id: u32) {
  if let Some(w) = tab_webview(&app, id) {
    let _ = w.go_forward();
  }
}

#[tauri::command]
pub fn reload<R: Runtime>(app: AppHandle<R>, id: u32) {
  if let Some(w) = tab_webview(&app, id) {
    let _ = w.reload();
  }
}

#[tauri::command]
pub fn stop<R: Runtime>(app: AppHandle<R>, id: u32) {
  if let Some(w) = tab_webview(&app, id) {
    let _ = w.eval("window.stop()");
  }
}

/// Set tab `id`'s zoom to `zoom`, or change it by `delta`.
#[tauri::command]
pub fn zoom<R: Runtime>(app: AppHandle<R>, id: u32, zoom: Option<f64>, delta: Option<f64>) {
  browser::set_zoom(&app, id, zoom, delta.unwrap_or(0.0));
}

#[tauri::command]
pub fn new_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
  browser::open_window(&app, None)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reopen_closed_tab<R: Runtime>(app: AppHandle<R>, webview: Webview<R>) {
  browser::run_action(&app, &window_of(&webview), "reopen-tab");
}

/// Give the active tab the keyboard focus (e.g. after Escape in the address bar).
#[tauri::command]
pub fn focus_content<R: Runtime>(app: AppHandle<R>, webview: Webview<R>) {
  if let Some(w) =
    browser::active_tab(&app, &window_of(&webview)).and_then(|id| tab_webview(&app, id))
  {
    let _ = w.set_focus();
  }
}

/// A browser action from a keyboard shortcut (in a page or the title bar) or a button.
#[tauri::command]
pub fn shortcut<R: Runtime>(app: AppHandle<R>, webview: Webview<R>, action: String) {
  run(&app, &window_of(&webview), &action);
}

fn run<R: Runtime>(app: &AppHandle<R>, window: &str, action: &str) {
  browser(app).current_window = Some(window.to_string());
  browser::run_action(app, window, action);
}
