// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Browser windows and tabs, and every browser action (shortcuts, menus, buttons).
//!
//! Each browser window is a frameless Tauri window holding child webviews: `chrome-<n>` (the
//! title bar and toolbar, `ui/chrome.html`, `CHROME_HEIGHT` tall) and one `tab-<id>` per tab
//! below it, of which only the active one is shown.

use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{
    Mutex, MutexGuard,
    atomic::{AtomicU32, Ordering},
  },
};

use serde_json::json;
use tauri::{
  AppHandle, Emitter, EventTarget, LogicalPosition, LogicalSize, Manager, Runtime, Url, Webview,
  WebviewUrl, WebviewWindowBuilder, Window,
  webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
  window::{Color, WindowBuilder},
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::{downloads, menus, settings::Settings};

/// Height of the tab strip plus the toolbar, in logical pixels (matches ui/chrome.css).
pub const CHROME_HEIGHT: f64 = 84.0;
/// The scheme of the browser's own pages, served by `pages::protocol`.
pub const INTERNAL: &str = "qtbrowser";

/// Runs in every tab's top document. Forwards browser shortcuts, the right-click menu and (for
/// Save page as) the page's HTML to the only commands web pages may call
/// (capabilities/web-pages.json).
const TAB_SCRIPT: &str = r#"
(() => {
  if (window.top !== window) return;
  const invoke = (cmd, args) => window.__TAURI_INTERNALS__?.invoke(cmd, args).catch(() => {});
  const action = (e) => {
    const ctrl = e.ctrlKey || e.metaKey, k = e.key.toLowerCase();
    if (ctrl && e.shiftKey && k === 't') return 'reopen-tab';
    if (ctrl && e.shiftKey && k === 'i') return 'devtools';
    if (ctrl && e.shiftKey && k === 'w') return 'close-window';
    if (ctrl && k === 'tab') return e.shiftKey ? 'prev-tab' : 'next-tab';
    if (ctrl && k === 'pagedown') return 'next-tab';
    if (ctrl && k === 'pageup') return 'prev-tab';
    if (ctrl && !e.shiftKey && !e.altKey) {
      const map = { t: 'new-tab', n: 'new-window', w: 'close-tab', l: 'focus-address', r: 'reload',
        ',': 'settings', '=': 'zoom-in', '+': 'zoom-in', '-': 'zoom-out', '0': 'zoom-reset',
        o: 'open-file', s: 'save-page', p: 'print', j: 'downloads' };
      if (map[k]) return map[k];
      if (k >= '1' && k <= '9') return 'tab-' + k;
    }
    if (e.altKey && !ctrl && k === 'arrowleft') return 'back';
    if (e.altKey && !ctrl && k === 'arrowright') return 'forward';
    if (e.altKey && !ctrl && k === 'home') return 'home';
    if (k === 'f5') return 'reload';
    if (k === 'f11') return 'fullscreen';
    if (k === 'f12') return 'devtools';
    return null;
  };
  addEventListener('keydown', (e) => {
    const a = e.isTrusted && action(e);
    if (!a) return;
    e.preventDefault();
    e.stopPropagation();
    invoke('shortcut', { action: a });
  }, true);
  // The native right-click menu, unless the page has its own.
  addEventListener('contextmenu', (e) => {
    if (!e.isTrusted || e.defaultPrevented || e.shiftKey) return;
    e.preventDefault();
    const link = e.target.closest && e.target.closest('a[href]');
    const img = e.target.closest && e.target.closest('img[src]');
    const editable = e.target.isContentEditable || /^(INPUT|TEXTAREA)$/.test(e.target.tagName);
    invoke('page_context_menu', {
      x: e.clientX, y: e.clientY,
      link: link ? link.href : null,
      image: img ? img.currentSrc || img.src : null,
      selection: String(getSelection()).slice(0, 200) || null,
      editable,
    });
  });
  // Save page as: the browser asks for the HTML after the user picked a file.
  window.__qtBrowserSendHtml = () =>
    invoke('page_html', { html: '<!doctype html>\n' + document.documentElement.outerHTML });
})();
"#;

// --- state ----------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Tab {
  pub window: String,
  pub title: String,
  pub url: String,
  pub loading: bool,
  pub zoom: f64,
}

#[derive(Debug, Default)]
pub struct BrowserWindow {
  /// Tab ids in strip order.
  pub tabs: Vec<u32>,
  pub active: Option<u32>,
  /// URLs of closed tabs, for Ctrl+Shift+T.
  pub closed: Vec<String>,
}

/// What the right-click menu was opened on, for its items.
#[derive(Debug, Clone, Default)]
pub struct PageContext {
  pub tab: u32,
  pub link: Option<String>,
  pub image: Option<String>,
  pub selection: Option<String>,
}

#[derive(Default)]
pub struct Inner {
  pub windows: HashMap<String, BrowserWindow>,
  pub tabs: HashMap<u32, Tab>,
  pub settings: Settings,
  /// The browser window that last had the focus: app menu, tray and context menu events carry
  /// no window.
  pub current_window: Option<String>,
  pub context: Option<PageContext>,
  /// Tabs whose HTML is awaited by Save page as, and where it goes.
  pub pending_saves: HashMap<u32, PathBuf>,
  pub downloads: downloads::Downloads,
}

/// Browser state. Locked only briefly, never while calling webview APIs: Qt fires page events
/// synchronously, and their handlers lock it too.
#[derive(Default)]
pub struct Browser(Mutex<Inner>);

/// Lock the browser state of `app`.
pub fn browser<R: Runtime>(app: &AppHandle<R>) -> MutexGuard<'_, Inner> {
  app.state::<Browser>().inner().0.lock().unwrap()
}

static NEXT_TAB: AtomicU32 = AtomicU32::new(1);
static NEXT_WINDOW: AtomicU32 = AtomicU32::new(1);

pub fn tab_label(id: u32) -> String {
  format!("tab-{id}")
}

pub fn chrome_label(window: &str) -> String {
  format!("chrome-{}", window.trim_start_matches("browser-"))
}

pub fn tab_id(label: &str) -> Option<u32> {
  label.strip_prefix("tab-")?.parse().ok()
}

pub fn tab_webview<R: Runtime>(app: &AppHandle<R>, id: u32) -> Option<Webview<R>> {
  app.get_webview(&tab_label(id))
}

/// The browser window to use for an action without one (menu, tray): the last focused one.
pub fn current_window<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
  let inner = browser(app);
  inner
    .current_window
    .clone()
    .filter(|w| inner.windows.contains_key(w))
    .or_else(|| inner.windows.keys().next().cloned())
}

// --- URLs -----------------------------------------------------------------------------------------

/// The URL of one of the browser's own pages, e.g. `qtbrowser://settings/`.
pub fn internal_url(page: &str) -> Url {
  Url::parse(&format!("{INTERNAL}://{page}/")).unwrap()
}

/// What the address bar shows for `url`: nothing for the new tab page.
pub fn display_url(url: &str) -> String {
  match Url::parse(url) {
    Ok(u) if u.scheme() == INTERNAL => match u.host_str() {
      Some("newtab") | None => String::new(),
      Some(page) => format!("{INTERNAL}://{page}"),
    },
    _ => url.to_string(),
  }
}

/// Turn address bar input into a URL: an internal page, a URL, a host, or a search.
pub fn resolve_input(input: &str, settings: &Settings) -> Url {
  let input = input.trim();
  if let Ok(url) = Url::parse(input)
    && matches!(
      url.scheme(),
      "http" | "https" | "file" | "about" | "data" | "view-source" | INTERNAL
    )
  {
    return url;
  }
  let looks_like_host = !input.contains(char::is_whitespace)
    && (input.contains('.') || input.starts_with("localhost"))
    && !input.starts_with('.');
  if looks_like_host {
    let local = input.starts_with("localhost") || input.starts_with("127.");
    let scheme = if local { "http" } else { "https" };
    if let Ok(url) = Url::parse(&format!("{scheme}://{input}")) {
      return url;
    }
  }
  Url::parse(&settings.search_url(input)).unwrap()
}

pub fn home_url(settings: &Settings) -> Url {
  if settings.homepage.trim().is_empty() {
    internal_url("newtab")
  } else {
    resolve_input(&settings.homepage, settings)
  }
}

/// Schemes the browser loads itself; anything else (mailto:, magnet:, ...) goes to the system.
fn loads_itself(url: &Url) -> bool {
  matches!(
    url.scheme(),
    "http" | "https" | "file" | "about" | "data" | "blob" | "view-source" | "chrome" | INTERNAL
  )
}

fn webview_url(url: Url) -> WebviewUrl {
  if url.scheme() == INTERNAL {
    WebviewUrl::CustomProtocol(url)
  } else {
    WebviewUrl::External(url)
  }
}

// --- tab strip ------------------------------------------------------------------------------------

fn fallback_title(url: &str) -> String {
  match display_url(url).as_str() {
    "" => "New Tab".into(),
    "qtbrowser://settings" => "Settings".into(),
    "qtbrowser://downloads" => "Downloads".into(),
    "qtbrowser://about" => "About".into(),
    other => other.to_string(),
  }
}

fn tab_title(tab: &Tab) -> String {
  if tab.title.is_empty() || tab.title == tab.url {
    fallback_title(&tab.url)
  } else {
    tab.title.clone()
  }
}

/// The tab strip of `window`, for its chrome webview.
pub fn snapshot<R: Runtime>(app: &AppHandle<R>, window: &str) -> serde_json::Value {
  let (active, tabs, settings, can_reopen, downloads) = {
    let inner = browser(app);
    let Some(w) = inner.windows.get(window) else {
      return json!(null);
    };
    let tabs: Vec<(u32, Tab)> = w
      .tabs
      .iter()
      .filter_map(|id| inner.tabs.get(id).map(|t| (*id, t.clone())))
      .collect();
    (
      w.active,
      tabs,
      inner.settings.clone(),
      !w.closed.is_empty(),
      inner.downloads.active(),
    )
  };
  let tabs: Vec<_> = tabs
    .into_iter()
    .map(|(id, t)| {
      let (back, forward) = tab_webview(app, id)
        .map(|w| {
          (
            w.can_go_back().unwrap_or(false),
            w.can_go_forward().unwrap_or(false),
          )
        })
        .unwrap_or_default();
      let parsed = Url::parse(&t.url).ok();
      let internal = parsed.as_ref().is_some_and(|u| u.scheme() == INTERNAL);
      let origin = parsed
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .map(|u| u.origin().ascii_serialization())
        .unwrap_or_default();
      json!({
        "id": id,
        "title": tab_title(&t),
        "url": display_url(&t.url),
        "origin": origin,
        "internal": internal,
        "loading": t.loading,
        "canGoBack": back,
        "canGoForward": forward,
        "zoom": t.zoom,
      })
    })
    .collect();
  json!({
    "window": window,
    "tabs": tabs,
    "active": active,
    "canReopen": can_reopen,
    "downloads": downloads,
    "settings": settings,
  })
}

/// Send `window`'s tab strip to its chrome webview, and title the window after the active tab
/// (the title shows in the taskbar and window switcher).
pub fn refresh<R: Runtime>(app: &AppHandle<R>, window: &str) {
  let payload = snapshot(app, window);
  let title = {
    let inner = browser(app);
    inner
      .windows
      .get(window)
      .and_then(|w| w.active)
      .and_then(|id| inner.tabs.get(&id))
      .map(tab_title)
  };
  if let (Some(title), Some(w)) = (title, app.get_window(window)) {
    let _ = w.set_title(&format!("{title} - Qt Browser"));
  }
  let _ = app.emit_to(
    EventTarget::webview(chrome_label(window)),
    "browser-tabs",
    payload,
  );
}

pub fn refresh_all<R: Runtime>(app: &AppHandle<R>) {
  let windows: Vec<String> = browser(app).windows.keys().cloned().collect();
  for w in windows {
    refresh(app, &w);
  }
}

/// Position the chrome and the tabs of `window` for its current inner size (with the Qt runtime,
/// the area below the menu bar when that's shown).
pub fn layout<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) {
  let Ok(size) = window.inner_size() else {
    return;
  };
  let scale = window.scale_factor().unwrap_or(1.0);
  let size = size.to_logical::<f64>(scale);
  let content_height = (size.height - CHROME_HEIGHT).max(0.0);
  if let Some(chrome) = app.get_webview(&chrome_label(window.label())) {
    let _ = chrome.set_position(LogicalPosition::new(0.0, 0.0));
    let _ = chrome.set_size(LogicalSize::new(size.width, CHROME_HEIGHT));
  }
  let ids = browser(app)
    .windows
    .get(window.label())
    .map(|w| w.tabs.clone())
    .unwrap_or_default();
  for id in ids {
    if let Some(tab) = tab_webview(app, id) {
      let _ = tab.set_position(LogicalPosition::new(0.0, CHROME_HEIGHT));
      let _ = tab.set_size(LogicalSize::new(size.width, content_height));
    }
  }
}

/// A page background matching the theme, so a new tab doesn't flash white in dark mode.
fn page_background(settings: &Settings, window: &Window<impl Runtime>) -> Color {
  let dark = match settings.theme() {
    Some(t) => t == tauri::Theme::Dark,
    None => window.theme().is_ok_and(|t| t == tauri::Theme::Dark),
  };
  if dark {
    Color(0x20, 0x21, 0x24, 255)
  } else {
    Color(255, 255, 255, 255)
  }
}

// --- windows and tabs -----------------------------------------------------------------------------

/// Open a browser window with one tab showing `url` (the home page if `None`).
pub fn open_window<R: Runtime>(app: &AppHandle<R>, url: Option<Url>) -> tauri::Result<String> {
  let n = NEXT_WINDOW.fetch_add(1, Ordering::Relaxed);
  let label = format!("browser-{n}");
  let settings = browser(app).settings.clone();
  // The window-state plugin restores the size and position it last had.
  let window = WindowBuilder::new(app, &label)
    .title("Qt Browser")
    .inner_size(1200.0, 800.0)
    .min_inner_size(480.0, 320.0)
    .decorations(false)
    .theme(settings.theme())
    .build()?;
  // The app menu (src/menus.rs) is a menu bar on every window; hidden unless enabled.
  if !settings.show_menu_bar {
    let _ = window.hide_menu();
  }
  {
    let mut inner = browser(app);
    inner
      .windows
      .insert(label.clone(), BrowserWindow::default());
    inner.current_window = Some(label.clone());
  }
  let size = window
    .inner_size()?
    .to_logical::<f64>(window.scale_factor()?);
  window.add_child(
    WebviewBuilder::new(chrome_label(&label), WebviewUrl::App("chrome.html".into()))
      .position(LogicalPosition::new(0.0, 0.0))
      .size(LogicalSize::new(size.width, CHROME_HEIGHT)),
  )?;
  open_tab(
    app,
    &label,
    url.unwrap_or_else(|| home_url(&settings)),
    true,
    None,
  )?;
  layout(app, &window);
  Ok(label)
}

/// Add a tab showing `url` to `window`, after tab `after` (or at the end).
pub fn open_tab<R: Runtime>(
  app: &AppHandle<R>,
  window: &str,
  url: Url,
  activate: bool,
  after: Option<u32>,
) -> tauri::Result<u32> {
  let Some(win) = app.get_window(window) else {
    return Err(tauri::Error::WindowNotFound);
  };
  let id = NEXT_TAB.fetch_add(1, Ordering::Relaxed);
  let settings = {
    let mut inner = browser(app);
    let zoom = inner.settings.default_zoom;
    inner.tabs.insert(
      id,
      Tab {
        window: window.to_string(),
        title: String::new(),
        url: url.to_string(),
        loading: true,
        zoom,
      },
    );
    let w = inner.windows.entry(window.to_string()).or_default();
    let at = after
      .and_then(|a| w.tabs.iter().position(|t| *t == a))
      .map_or(w.tabs.len(), |i| i + 1);
    w.tabs.insert(at, id);
    inner.settings.clone()
  };
  let size = win.inner_size()?.to_logical::<f64>(win.scale_factor()?);

  let page_load = app.clone();
  let titles = app.clone();
  let popups = app.clone();
  let navigation = app.clone();
  let downloads = app.clone();
  let mut builder = WebviewBuilder::new(tab_label(id), webview_url(url))
    .position(LogicalPosition::new(0.0, CHROME_HEIGHT))
    .size(LogicalSize::new(
      size.width,
      (size.height - CHROME_HEIGHT).max(0.0),
    ));
  let user_agent = settings.user_agent();
  if let Some(ua) = user_agent {
    builder = builder.user_agent(ua);
  }
  let builder = builder
    .background_color(page_background(&settings, &win))
    .initialization_script(TAB_SCRIPT)
    // Pages get file drops themselves (uploads); drops on the tab strip open tabs.
    .disable_drag_drop_handler()
    .on_page_load(move |webview, payload| {
      let window = {
        let mut inner = browser(&page_load);
        let Some(tab) = tab_id(webview.label()).and_then(|id| inner.tabs.get_mut(&id)) else {
          return;
        };
        tab.url = payload.url().to_string();
        tab.loading = matches!(payload.event(), PageLoadEvent::Started);
        if tab.loading {
          tab.title.clear();
        }
        tab.window.clone()
      };
      refresh(&page_load, &window);
    })
    .on_document_title_changed(move |webview, title| {
      let window = {
        let mut inner = browser(&titles);
        let Some(tab) = tab_id(webview.label()).and_then(|id| inner.tabs.get_mut(&id)) else {
          return;
        };
        tab.title = title;
        tab.window.clone()
      };
      refresh(&titles, &window);
    })
    .on_navigation(move |url| {
      // mailto:, magnet:, ...: the system's handler (opener plugin).
      if loads_itself(url) {
        return true;
      }
      log::info!("opening {url} with the system");
      let _ = navigation.opener().open_url(url.as_str(), None::<&str>);
      false
    })
    .on_new_window(move |url, features| {
      // window.open() with a size: a real popup window, linked to its opener (sign-in flows).
      if features.size().is_some() {
        let label = format!("popup-{}", NEXT_TAB.fetch_add(1, Ordering::Relaxed));
        let mut popup = WebviewWindowBuilder::new(
          &popups,
          &label,
          WebviewUrl::External("about:blank".parse().unwrap()),
        )
        .title(url.as_str())
        .window_features(features);
        if let Some(ua) = user_agent {
          popup = popup.user_agent(ua);
        }
        return match popup.build() {
          Ok(window) => NewWindowResponse::Create { window },
          Err(_) => NewWindowResponse::Deny,
        };
      }
      // target=_blank and friends: a new tab next to this one (Tauri can only give a new
      // request a whole window, so the tab isn't linked to its opener).
      let app = popups.clone();
      let _ = popups.run_on_main_thread(move || {
        let from = browser(&app).tabs.get(&id).map(|t| t.window.clone());
        if let Some(window) = from
          && let Err(e) = open_tab(&app, &window, url, true, Some(id))
        {
          log::warn!("couldn't open a tab for a new window request: {e}");
        }
      });
      NewWindowResponse::Deny
    })
    .on_download(move |webview, event| downloads::on_download(&downloads, &webview, event));
  let webview = win.add_child(builder)?;
  if (settings.default_zoom - 1.0).abs() > f64::EPSILON {
    let _ = webview.set_zoom(settings.default_zoom);
  }
  if activate {
    activate_tab(app, id, true);
  } else {
    let _ = webview.hide();
    refresh(app, window);
  }
  Ok(id)
}

/// Show tab `id` (and hide the others of its window). `focus`: give the page the keyboard focus
/// (not for new tab pages, where the address bar should get it).
pub fn activate_tab<R: Runtime>(app: &AppHandle<R>, id: u32, focus: bool) {
  let (window, others, newtab) = {
    let mut inner = browser(app);
    let Some(tab) = inner.tabs.get(&id).cloned() else {
      return;
    };
    let Some(w) = inner.windows.get_mut(&tab.window) else {
      return;
    };
    w.active = Some(id);
    let others: Vec<u32> = w.tabs.iter().copied().filter(|t| *t != id).collect();
    (tab.window, others, display_url(&tab.url).is_empty())
  };
  if let Some(webview) = tab_webview(app, id) {
    let _ = webview.show();
    if focus && !newtab {
      let _ = webview.set_focus();
    }
  }
  for other in others {
    if let Some(webview) = tab_webview(app, other) {
      let _ = webview.hide();
    }
  }
  refresh(app, &window);
  if newtab {
    focus_address(app, &window);
  }
}

/// Close tab `id`; closing the last tab closes its window.
pub fn close_tab<R: Runtime>(app: &AppHandle<R>, id: u32) {
  let (window, next, last) = {
    let mut inner = browser(app);
    let Some(tab) = inner.tabs.remove(&id) else {
      return;
    };
    let Some(w) = inner.windows.get_mut(&tab.window) else {
      return;
    };
    let index = w.tabs.iter().position(|t| *t == id).unwrap_or(0);
    w.tabs.retain(|t| *t != id);
    if !display_url(&tab.url).is_empty() {
      w.closed.push(tab.url.clone());
    }
    let next = if w.active == Some(id) {
      w.tabs.get(index).or_else(|| w.tabs.last()).copied()
    } else {
      None
    };
    (tab.window, next, w.tabs.is_empty())
  };
  if let Some(webview) = tab_webview(app, id) {
    let _ = webview.close();
  }
  if last {
    if let Some(w) = app.get_window(&window) {
      let _ = w.destroy();
    }
    return;
  }
  match next {
    Some(next) => activate_tab(app, next, true),
    None => refresh(app, &window),
  }
}

pub fn active_tab<R: Runtime>(app: &AppHandle<R>, window: &str) -> Option<u32> {
  browser(app).windows.get(window)?.active
}

pub fn focus_address<R: Runtime>(app: &AppHandle<R>, window: &str) {
  if let Some(chrome) = app.get_webview(&chrome_label(window)) {
    let _ = chrome.set_focus();
  }
  let _ = app.emit_to(
    EventTarget::webview(chrome_label(window)),
    "browser-focus-address",
    (),
  );
}

/// Set tab `id`'s zoom to `zoom`, or change it by `delta` (25% to 500%, in whole percents).
pub fn set_zoom<R: Runtime>(app: &AppHandle<R>, id: u32, zoom: Option<f64>, delta: f64) {
  let (window, zoom) = {
    let mut inner = browser(app);
    let Some(tab) = inner.tabs.get_mut(&id) else {
      return;
    };
    let next = zoom.unwrap_or(tab.zoom + delta);
    tab.zoom = ((next * 100.0).round() / 100.0).clamp(0.25, 5.0);
    (tab.window.clone(), tab.zoom)
  };
  if let Some(webview) = tab_webview(app, id) {
    let _ = webview.set_zoom(zoom);
  }
  refresh(app, &window);
}

/// Open an internal page, or switch to its tab if `window` already has one.
pub fn open_internal<R: Runtime>(app: &AppHandle<R>, window: &str, page: &str) {
  let url = internal_url(page);
  let existing = {
    let inner = browser(app);
    inner.windows.get(window).and_then(|w| {
      w.tabs.iter().copied().find(|id| {
        inner.tabs.get(id).is_some_and(|t| {
          Url::parse(&t.url).is_ok_and(|u| u.host_str() == url.host_str() && u.scheme() == INTERNAL)
        })
      })
    })
  };
  match existing {
    Some(id) => activate_tab(app, id, true),
    None => {
      let _ = open_tab(app, window, url, true, None);
    }
  }
}

/// Show or hide the app menu bar on every window.
pub fn set_menu_bar<R: Runtime>(app: &AppHandle<R>, visible: bool) {
  for (label, window) in app.windows() {
    if label.starts_with("browser-") {
      let _ = if visible {
        window.show_menu()
      } else {
        window.hide_menu()
      };
    }
  }
}

/// Run a browser action (a shortcut, a menu item, a button) in `window`.
pub fn run_action<R: Runtime>(app: &AppHandle<R>, window: &str, action: &str) {
  let active = active_tab(app, window);
  let webview = active.and_then(|id| tab_webview(app, id));
  let win = app.get_window(window);
  let settings = browser(app).settings.clone();
  match action {
    "new-tab" => {
      let _ = open_tab(app, window, internal_url("newtab"), true, None);
    }
    "new-window" => {
      let _ = open_window(app, None);
    }
    "close-tab" => {
      if let Some(id) = active {
        close_tab(app, id);
      }
    }
    "close-window" => {
      if let Some(w) = win {
        let _ = w.close();
      }
    }
    "reopen-tab" => {
      let url = browser(app)
        .windows
        .get_mut(window)
        .and_then(|w| w.closed.pop());
      if let Some(url) = url.and_then(|u| Url::parse(&u).ok()) {
        let _ = open_tab(app, window, url, true, None);
      }
    }
    "next-tab" | "prev-tab" => {
      let tabs = browser(app)
        .windows
        .get(window)
        .map(|w| w.tabs.clone())
        .unwrap_or_default();
      if let (Some(active), false) = (active, tabs.is_empty()) {
        let i = tabs.iter().position(|t| *t == active).unwrap_or(0);
        let n = tabs.len();
        let next = if action == "next-tab" {
          (i + 1) % n
        } else {
          (i + n - 1) % n
        };
        activate_tab(app, tabs[next], true);
      }
    }
    a if a.starts_with("tab-") => {
      let tabs = browser(app)
        .windows
        .get(window)
        .map(|w| w.tabs.clone())
        .unwrap_or_default();
      // Like Chrome: Ctrl+1..8 pick that tab, Ctrl+9 the last one.
      let pick = match a[4..].parse::<usize>() {
        Ok(9) => tabs.last().copied(),
        Ok(n) if n >= 1 => tabs.get(n - 1).copied(),
        _ => None,
      };
      if let Some(id) = pick {
        activate_tab(app, id, true);
      }
    }
    "focus-address" => focus_address(app, window),
    "reload" => {
      if let Some(w) = webview {
        let _ = w.reload();
      }
    }
    "stop" => {
      if let Some(w) = webview {
        let _ = w.eval("window.stop()");
      }
    }
    "back" => {
      if let Some(w) = webview {
        let _ = w.go_back();
      }
    }
    "forward" => {
      if let Some(w) = webview {
        let _ = w.go_forward();
      }
    }
    "home" => {
      if let Some(w) = webview {
        let _ = w.navigate(home_url(&settings));
      }
    }
    "zoom-in" | "zoom-out" | "zoom-reset" => {
      if let Some(id) = active {
        match action {
          "zoom-in" => set_zoom(app, id, None, 0.1),
          "zoom-out" => set_zoom(app, id, None, -0.1),
          _ => set_zoom(app, id, Some(settings.default_zoom), 0.0),
        }
      }
    }
    "devtools" => {
      if let Some(w) = webview {
        w.open_devtools();
      }
    }
    "print" => {
      if let Some(w) = webview {
        let _ = w.print();
      }
    }
    "view-source" => {
      if let Some(url) = webview.and_then(|w| w.url().ok())
        && let Ok(src) = Url::parse(&format!("view-source:{url}"))
      {
        let _ = open_tab(app, window, src, true, active);
      }
    }
    "copy-url" => {
      if let Some(url) = webview.and_then(|w| w.url().ok()) {
        let _ = app.clipboard().write_text(url.to_string());
      }
    }
    "fullscreen" => {
      if let Some(w) = win {
        let on = w.is_fullscreen().unwrap_or(false);
        let _ = w.set_fullscreen(!on);
      }
    }
    "always-on-top" => {
      if let Some(w) = win {
        let on = w.is_always_on_top().unwrap_or(false);
        let _ = w.set_always_on_top(!on);
        menus::sync_checks(app);
      }
    }
    "toggle-menu-bar" => {
      let visible = {
        let mut inner = browser(app);
        inner.settings.show_menu_bar = !inner.settings.show_menu_bar;
        inner.settings.show_menu_bar
      };
      set_menu_bar(app, visible);
      let s = browser(app).settings.clone();
      let _ = crate::settings::save(app, &s);
      menus::sync_checks(app);
    }
    "open-file" => open_file(app, window),
    "save-page" => {
      if let Some(id) = active {
        save_page(app, id);
      }
    }
    "settings" => open_internal(app, window, "settings"),
    "downloads" => open_internal(app, window, "downloads"),
    "about" => open_internal(app, window, "about"),
    "restart" => app.restart(),
    "quit" | "exit" => app.exit(0),
    other => log::debug!("unknown action {other}"),
  }
}

/// File > Open File: a local file in a new tab (dialog plugin).
fn open_file<R: Runtime>(app: &AppHandle<R>, window: &str) {
  let app_ = app.clone();
  let window = window.to_string();
  app
    .dialog()
    .file()
    .set_title("Open File")
    .add_filter(
      "Web pages and documents",
      &[
        "html", "htm", "xhtml", "svg", "pdf", "txt", "png", "jpg", "jpeg", "gif", "webp",
      ],
    )
    .pick_file(move |file| {
      let Some(path) = file.and_then(|f| f.into_path().ok()) else {
        return;
      };
      if let Ok(url) = Url::from_file_path(&path) {
        let _ = app_.run_on_main_thread({
          let app = app_.clone();
          move || {
            let _ = open_tab(&app, &window, url, true, None);
          }
        });
      }
    });
}

/// File > Save Page As: pick a file (dialog plugin), then ask the page for its HTML
/// (`page_html` in src/menus.rs writes it).
pub fn save_page<R: Runtime>(app: &AppHandle<R>, id: u32) {
  let title = browser(app)
    .tabs
    .get(&id)
    .map(tab_title)
    .unwrap_or_else(|| "page".into());
  let name: String = title
    .chars()
    .map(|c| {
      if c.is_alphanumeric() || c == ' ' || c == '-' {
        c
      } else {
        '_'
      }
    })
    .collect();
  let app_ = app.clone();
  app
    .dialog()
    .file()
    .set_title("Save Page As")
    .set_file_name(format!("{}.html", name.trim()))
    .add_filter("Web page, HTML only", &["html", "htm"])
    .save_file(move |file| {
      let Some(path) = file.and_then(|f| f.into_path().ok()) else {
        return;
      };
      browser(&app_).pending_saves.insert(id, path);
      if let Some(webview) = tab_webview(&app_, id) {
        let _ = webview.eval("window.__qtBrowserSendHtml && window.__qtBrowserSendHtml()");
      }
    });
}
