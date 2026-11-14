// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Every kind of Tauri menu: the app menu (a menu bar on each window), the "⋮" menu and the
//! page's right-click menu (native popup menus), the site information menu, and the tray icon's
//! menu. Their events all arrive in `on_menu_event`.

use tauri::{
  AppHandle, LogicalPosition, Manager, Runtime, Url, Webview,
  image::Image,
  menu::{
    AboutMetadataBuilder, CheckMenuItemBuilder, Menu, MenuBuilder, MenuEvent, MenuItemBuilder,
    MenuItemKind, SubmenuBuilder,
  },
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::browser::{
  self, CHROME_HEIGHT, PageContext, active_tab, browser, current_window, open_tab, open_window,
  run_action, tab_id, tab_webview,
};

const TRAY: &str = "main";

fn item<R: Runtime>(
  app: &AppHandle<R>,
  id: &str,
  text: &str,
  accelerator: Option<&str>,
) -> tauri::Result<tauri::menu::MenuItem<R>> {
  let mut b = MenuItemBuilder::with_id(id, text);
  if let Some(a) = accelerator {
    b = b.accelerator(a);
  }
  b.build(app)
}

fn disabled<R: Runtime>(app: &AppHandle<R>, text: &str) -> tauri::Result<tauri::menu::MenuItem<R>> {
  MenuItemBuilder::new(text).enabled(false).build(app)
}

// --- the app menu -----------------------------------------------------------------------------------

/// The app menu, set on every window by `Builder::menu` and shown as a menu bar when enabled
/// (View > Show Menu Bar, or Settings). Its item ids are browser actions (`browser::run_action`).
pub fn app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
  let settings = browser(app).settings.clone();
  let file = SubmenuBuilder::new(app, "&File")
    .item(&item(app, "new-tab", "New &Tab", Some("CmdOrCtrl+T"))?)
    .item(&item(
      app,
      "new-window",
      "&New Window",
      Some("CmdOrCtrl+N"),
    )?)
    .separator()
    .item(&item(app, "open-file", "&Open File…", Some("CmdOrCtrl+O"))?)
    .item(&item(
      app,
      "save-page",
      "&Save Page As…",
      Some("CmdOrCtrl+S"),
    )?)
    .item(&item(app, "print", "&Print…", Some("CmdOrCtrl+P"))?)
    .separator()
    .item(&item(app, "close-tab", "&Close Tab", Some("CmdOrCtrl+W"))?)
    .item(&item(
      app,
      "close-window",
      "Close &Window",
      Some("CmdOrCtrl+Shift+W"),
    )?)
    .separator()
    .item(&item(app, "quit", "&Quit", Some("CmdOrCtrl+Q"))?)
    .build()?;
  // Undo/Redo/Cut/Copy/Paste/Select All are predefined: they act on the focused page or field.
  let edit = SubmenuBuilder::new(app, "&Edit")
    .undo()
    .redo()
    .separator()
    .cut()
    .copy()
    .paste()
    .select_all()
    .separator()
    .item(&item(app, "copy-url", "Copy Page &Address", None)?)
    .separator()
    .item(&item(app, "settings", "Se&ttings", Some("CmdOrCtrl+,"))?)
    .build()?;
  let view = SubmenuBuilder::new(app, "&View")
    .item(
      &CheckMenuItemBuilder::with_id("toggle-menu-bar", "Show &Menu Bar")
        .checked(settings.show_menu_bar)
        .build(app)?,
    )
    .item(
      &CheckMenuItemBuilder::with_id("always-on-top", "Always on &Top")
        .checked(false)
        .build(app)?,
    )
    .item(&item(app, "fullscreen", "&Full Screen", Some("F11"))?)
    .separator()
    .item(&item(app, "zoom-in", "Zoom &In", Some("CmdOrCtrl+="))?)
    .item(&item(app, "zoom-out", "Zoom &Out", Some("CmdOrCtrl+-"))?)
    .item(&item(
      app,
      "zoom-reset",
      "&Actual Size",
      Some("CmdOrCtrl+0"),
    )?)
    .separator()
    .item(&item(app, "reload", "&Reload", Some("CmdOrCtrl+R"))?)
    .item(&item(
      app,
      "view-source",
      "View Page &Source",
      Some("CmdOrCtrl+U"),
    )?)
    .item(&item(app, "devtools", "&Developer Tools", Some("F12"))?)
    .build()?;
  let history = SubmenuBuilder::new(app, "Hi&story")
    .item(&item(app, "back", "&Back", Some("Alt+Left"))?)
    .item(&item(app, "forward", "&Forward", Some("Alt+Right"))?)
    .item(&item(app, "home", "&Home", Some("Alt+Home"))?)
    .separator()
    .item(&item(
      app,
      "reopen-tab",
      "&Reopen Closed Tab",
      Some("CmdOrCtrl+Shift+T"),
    )?)
    .build()?;
  let window = SubmenuBuilder::new(app, "&Window")
    .minimize()
    .maximize()
    .separator()
    .item(&item(
      app,
      "next-tab",
      "&Next Tab",
      Some("CmdOrCtrl+PageDown"),
    )?)
    .item(&item(
      app,
      "prev-tab",
      "&Previous Tab",
      Some("CmdOrCtrl+PageUp"),
    )?)
    .separator()
    .item(&item(app, "downloads", "&Downloads", Some("CmdOrCtrl+J"))?)
    .build()?;
  let about = AboutMetadataBuilder::new()
    .name(Some("Qt Browser"))
    .version(Some(env!("CARGO_PKG_VERSION")))
    .comments(Some(
      "A Chromium-style browser built with Tauri on the Qt runtime (QtWebEngine).",
    ))
    .website(Some(
      "https://github.com/KRTirtho/tauri/tree/qt-support/crates/tauri-runtime-qt",
    ))
    .website_label(Some("tauri-runtime-qt"))
    .license(Some("Apache-2.0 OR MIT"))
    .icon(app.default_window_icon().cloned())
    .build();
  let help = SubmenuBuilder::new(app, "&Help")
    .about(Some(about))
    .item(&item(app, "about", "About This &Browser", None)?)
    .item(&item(app, "restart", "&Restart", None)?)
    .build()?;
  MenuBuilder::new(app)
    .items(&[&file, &edit, &view, &history, &window, &help])
    .build()
}

/// Find a menu item anywhere in `items` (`Menu::get` only looks at the top level).
fn find<R: Runtime>(items: Vec<MenuItemKind<R>>, id: &str) -> Option<MenuItemKind<R>> {
  items.into_iter().find_map(|i| match i {
    MenuItemKind::Submenu(s) => find(s.items().ok()?, id),
    i if i.id() == id => Some(i),
    _ => None,
  })
}

/// Update the check marks that mirror browser state.
pub fn sync_checks<R: Runtime>(app: &AppHandle<R>) {
  let settings = browser(app).settings.clone();
  let on_top = current_window(app)
    .and_then(|w| app.get_window(&w))
    .and_then(|w| w.is_always_on_top().ok())
    .unwrap_or(false);
  if let Some(menu) = app.menu() {
    let items = menu.items().unwrap_or_default();
    for (id, checked) in [
      ("toggle-menu-bar", settings.show_menu_bar),
      ("always-on-top", on_top),
    ] {
      if let Some(MenuItemKind::Check(c)) = find(items.clone(), id) {
        let _ = c.set_checked(checked);
      }
    }
  }
  // The tray menu is rebuilt with the current state.
  if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY), tray_menu(app)) {
    let _ = tray.set_menu(Some(menu));
  }
}

// --- popup menus ------------------------------------------------------------------------------------

/// The "⋮" menu, at `(x, y)` in the window.
pub fn more_menu<R: Runtime>(app: &AppHandle<R>, zoom: f64) -> tauri::Result<Menu<R>> {
  let zoom_menu = SubmenuBuilder::new(app, format!("Zoom ({:.0}%)", zoom * 100.0))
    .item(&item(app, "zoom-in", "Zoom in", Some("CmdOrCtrl+="))?)
    .item(&item(app, "zoom-out", "Zoom out", Some("CmdOrCtrl+-"))?)
    .item(&item(app, "zoom-reset", "Reset zoom", Some("CmdOrCtrl+0"))?)
    .build()?;
  MenuBuilder::new(app)
    .item(&item(app, "new-tab", "New tab", Some("CmdOrCtrl+T"))?)
    .item(&item(app, "new-window", "New window", Some("CmdOrCtrl+N"))?)
    .item(&item(
      app,
      "reopen-tab",
      "Reopen closed tab",
      Some("CmdOrCtrl+Shift+T"),
    )?)
    .separator()
    .item(&item(app, "downloads", "Downloads", Some("CmdOrCtrl+J"))?)
    .item(&zoom_menu)
    .item(&item(app, "print", "Print…", Some("CmdOrCtrl+P"))?)
    .item(&item(
      app,
      "save-page",
      "Save page as…",
      Some("CmdOrCtrl+S"),
    )?)
    .item(&item(app, "fullscreen", "Full screen", Some("F11"))?)
    .separator()
    .item(&item(app, "settings", "Settings", Some("CmdOrCtrl+,"))?)
    .item(&item(app, "about", "About Qt Browser", None)?)
    .item(&item(app, "devtools", "Developer tools", Some("F12"))?)
    .separator()
    .item(&item(app, "quit", "Exit", None)?)
    .build()
}

/// Show `menu` in `window`: at `(x, y)` (logical, in the window) when given and the menu bar is
/// hidden (the chrome then starts at the top of the window), else at the cursor.
pub fn popup<R: Runtime>(
  window: &tauri::Window<R>,
  menu: &Menu<R>,
  at: Option<(f64, f64)>,
) -> tauri::Result<()> {
  match at {
    Some((x, y)) if !window.is_menu_visible().unwrap_or(false) => {
      window.popup_menu_at(menu, LogicalPosition::new(x, y))
    }
    _ => window.popup_menu(menu),
  }
}

/// The right-click menu of a page.
fn page_menu<R: Runtime>(app: &AppHandle<R>, ctx: &PageContext) -> tauri::Result<Menu<R>> {
  let settings = browser(app).settings.clone();
  let webview = tab_webview(app, ctx.tab);
  let mut menu = MenuBuilder::new(app);
  if ctx.link.is_some() {
    menu = menu
      .item(&item(
        app,
        "ctx:open-link-tab",
        "Open link in new tab",
        None,
      )?)
      .item(&item(
        app,
        "ctx:open-link-window",
        "Open link in new window",
        None,
      )?)
      .item(&item(app, "ctx:copy-link", "Copy link address", None)?)
      .separator();
  }
  if ctx.image.is_some() {
    menu = menu
      .item(&item(app, "ctx:open-image", "Open image in new tab", None)?)
      .item(&item(
        app,
        "ctx:copy-image-url",
        "Copy image address",
        None,
      )?)
      .separator();
  }
  if let Some(sel) = &ctx.selection {
    let short: String = sel.chars().take(24).collect();
    let engine = match settings.search_engine.as_str() {
      "google" => "Google",
      "bing" => "Bing",
      "brave" => "Brave",
      _ => "DuckDuckGo",
    };
    menu = menu
      .copy()
      .item(&item(
        app,
        "ctx:search",
        &format!(
          "Search {engine} for “{short}{}”",
          if sel.chars().count() > 24 { "…" } else { "" }
        ),
        None,
      )?)
      .separator();
  }
  let (back, forward) = webview
    .as_ref()
    .map(|w| {
      (
        w.can_go_back().unwrap_or(false),
        w.can_go_forward().unwrap_or(false),
      )
    })
    .unwrap_or_default();
  menu = menu
    .item(
      &MenuItemBuilder::with_id("back", "Back")
        .accelerator("Alt+Left")
        .enabled(back)
        .build(app)?,
    )
    .item(
      &MenuItemBuilder::with_id("forward", "Forward")
        .accelerator("Alt+Right")
        .enabled(forward)
        .build(app)?,
    )
    .item(&item(app, "reload", "Reload", Some("CmdOrCtrl+R"))?)
    .separator()
    .item(&item(app, "save-page", "Save as…", Some("CmdOrCtrl+S"))?)
    .item(&item(app, "print", "Print…", Some("CmdOrCtrl+P"))?)
    .separator()
    .item(&item(
      app,
      "view-source",
      "View page source",
      Some("CmdOrCtrl+U"),
    )?)
    .item(&item(app, "devtools", "Inspect", Some("F12"))?);
  menu.build()
}

/// The site information menu (the lock in the address bar): the connection and the site's
/// cookies (webview cookie API).
fn site_menu<R: Runtime>(app: &AppHandle<R>, webview: &Webview<R>) -> tauri::Result<Menu<R>> {
  let url = webview.url()?;
  let secure = url.scheme() == "https";
  let cookies = webview
    .cookies_for_url(url.clone())
    .map(|c| c.len())
    .unwrap_or(0);
  let host = url.host_str().unwrap_or("this page").to_string();
  MenuBuilder::new(app)
    .item(&disabled(
      app,
      if url.scheme() == browser::INTERNAL {
        "This is a Qt Browser page"
      } else if secure {
        "Connection is secure"
      } else {
        "Connection is not secure"
      },
    )?)
    .item(&disabled(app, &host)?)
    .separator()
    .item(&disabled(
      app,
      &format!(
        "{cookies} cookie{} in use",
        if cookies == 1 { "" } else { "s" }
      ),
    )?)
    .item(
      &MenuItemBuilder::with_id("ctx:clear-cookies", "Delete cookies for this site")
        .enabled(cookies > 0)
        .build(app)?,
    )
    .separator()
    .item(&item(app, "copy-url", "Copy page address", None)?)
    .build()
}

// --- the tray icon --------------------------------------------------------------------------------

fn tray_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
  let settings = browser(app).settings.clone();
  MenuBuilder::new(app)
    .item(&item(app, "tray:new-window", "New window", None)?)
    .item(&item(app, "tray:new-tab", "New tab", None)?)
    .separator()
    .item(&item(app, "tray:show", "Show all windows", None)?)
    .item(&item(app, "tray:hide", "Hide all windows", None)?)
    .separator()
    .item(
      &CheckMenuItemBuilder::with_id("tray:dnd", "Do not disturb")
        .checked(!settings.notifications)
        .build(app)?,
    )
    .item(
      &CheckMenuItemBuilder::with_id("tray:keep", "Keep running in the background")
        .checked(settings.keep_in_tray)
        .build(app)?,
    )
    .separator()
    .item(&item(app, "quit", "Quit Qt Browser", None)?)
    .build()
}

/// The tray icon: left click shows the browser (or opens a window), right click its menu.
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
  let icon = app
    .default_window_icon()
    .cloned()
    .unwrap_or_else(|| Image::new_owned(vec![0; 4 * 16 * 16], 16, 16));
  TrayIconBuilder::with_id(TRAY)
    .icon(icon)
    .tooltip("Qt Browser")
    .menu(&tray_menu(app)?)
    .show_menu_on_left_click(false)
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        show_browser(tray.app_handle());
      }
    })
    .build(app)?;
  Ok(())
}

/// Show and focus the browser, or open a window if none is left (e.g. kept running in the tray).
pub fn show_browser<R: Runtime>(app: &AppHandle<R>) {
  match current_window(app).and_then(|w| app.get_window(&w)) {
    Some(w) => {
      let _ = w.show();
      let _ = w.unminimize();
      let _ = w.set_focus();
    }
    None => {
      let _ = open_window(app, None);
    }
  }
}

fn set_windows_visible<R: Runtime>(app: &AppHandle<R>, visible: bool) {
  for (label, w) in app.windows() {
    if label.starts_with("browser-") {
      let _ = if visible { w.show() } else { w.hide() };
    }
  }
}

/// Show or hide every browser window (tray, global shortcut).
pub fn toggle_browser<R: Runtime>(app: &AppHandle<R>) {
  let any_visible = app
    .windows()
    .iter()
    .any(|(l, w)| l.starts_with("browser-") && w.is_visible().unwrap_or(false));
  if any_visible {
    set_windows_visible(app, false);
  } else if browser(app).windows.is_empty() {
    let _ = open_window(app, None);
  } else {
    set_windows_visible(app, true);
    show_browser(app);
  }
}

// --- events ---------------------------------------------------------------------------------------

/// Every menu's events: the app menu, popups and the tray.
pub fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
  let id = event.id().as_ref().to_string();
  let window = current_window(app);
  if let Some(tray_action) = id.strip_prefix("tray:") {
    on_tray_action(app, tray_action);
    return;
  }
  if let Some(ctx_action) = id.strip_prefix("ctx:") {
    on_context_action(app, ctx_action);
    return;
  }
  if let Some(window) = window {
    run_action(app, &window, &id);
  } else if id == "quit" {
    app.exit(0);
  }
}

fn on_tray_action<R: Runtime>(app: &AppHandle<R>, action: &str) {
  match action {
    "new-window" => {
      let _ = open_window(app, None);
    }
    "new-tab" => match current_window(app) {
      Some(w) => {
        show_browser(app);
        run_action(app, &w, "new-tab");
      }
      None => {
        let _ = open_window(app, None);
      }
    },
    "show" => {
      set_windows_visible(app, true);
      show_browser(app);
    }
    "hide" => set_windows_visible(app, false),
    "dnd" | "keep" => {
      let settings = {
        let mut inner = browser(app);
        if action == "dnd" {
          inner.settings.notifications = !inner.settings.notifications;
        } else {
          inner.settings.keep_in_tray = !inner.settings.keep_in_tray;
        }
        inner.settings.clone()
      };
      let _ = crate::settings::save(app, &settings);
      sync_checks(app);
      browser::refresh_all(app);
    }
    _ => {}
  }
}

fn on_context_action<R: Runtime>(app: &AppHandle<R>, action: &str) {
  let Some(ctx) = browser(app).context.clone() else {
    return;
  };
  let Some(window) = browser(app).tabs.get(&ctx.tab).map(|t| t.window.clone()) else {
    return;
  };
  let settings = browser(app).settings.clone();
  let parse = |s: &Option<String>| s.as_deref().and_then(|u| Url::parse(u).ok());
  match action {
    "open-link-tab" => {
      if let Some(url) = parse(&ctx.link) {
        let _ = open_tab(app, &window, url, false, Some(ctx.tab));
      }
    }
    "open-link-window" => {
      if let Some(url) = parse(&ctx.link) {
        let _ = open_window(app, Some(url));
      }
    }
    "copy-link" => {
      if let Some(link) = ctx.link {
        let _ = app.clipboard().write_text(link);
      }
    }
    "open-image" => {
      if let Some(url) = parse(&ctx.image) {
        let _ = open_tab(app, &window, url, true, Some(ctx.tab));
      }
    }
    "copy-image-url" => {
      if let Some(image) = ctx.image {
        let _ = app.clipboard().write_text(image);
      }
    }
    "search" => {
      if let Some(sel) = ctx.selection
        && let Ok(url) = Url::parse(&settings.search_url(&sel))
      {
        let _ = open_tab(app, &window, url, true, Some(ctx.tab));
      }
    }
    "clear-cookies" => {
      if let Some(webview) = tab_webview(app, ctx.tab)
        && let Ok(url) = webview.url()
      {
        for cookie in webview.cookies_for_url(url).unwrap_or_default() {
          let _ = webview.delete_cookie(cookie);
        }
      }
    }
    _ => {}
  }
}

// --- commands ---------------------------------------------------------------------------------------

/// Show the "⋮" menu at `(x, y)` (logical, in the window).
#[tauri::command]
pub fn open_menu<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  x: f64,
  y: f64,
) -> Result<(), String> {
  let window = webview.window();
  browser(&app).current_window = Some(window.label().to_string());
  let zoom = active_tab(&app, window.label())
    .and_then(|id| browser(&app).tabs.get(&id).map(|t| t.zoom))
    .unwrap_or(1.0);
  let menu = more_menu(&app, zoom).map_err(|e| e.to_string())?;
  popup(&window, &menu, Some((x, y))).map_err(|e| e.to_string())
}

/// The page's right-click menu (from the tab's init script). `x`/`y` are CSS pixels in the page.
#[tauri::command]
pub fn page_context_menu<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  x: f64,
  y: f64,
  link: Option<String>,
  image: Option<String>,
  selection: Option<String>,
) -> Result<(), String> {
  let id = tab_id(webview.label()).ok_or("not a tab")?;
  let zoom = browser(&app).tabs.get(&id).map_or(1.0, |t| t.zoom);
  let ctx = PageContext {
    tab: id,
    link,
    image,
    selection,
  };
  let menu = page_menu(&app, &ctx).map_err(|e| e.to_string())?;
  let window = webview.window();
  {
    let mut inner = browser(&app);
    inner.context = Some(ctx);
    inner.current_window = Some(window.label().to_string());
  }
  popup(&window, &menu, Some((x * zoom, CHROME_HEIGHT + y * zoom))).map_err(|e| e.to_string())
}

/// The site information menu of the active tab, under the lock icon at `(x, y)`.
#[tauri::command]
pub fn site_info_menu<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  x: f64,
  y: f64,
) -> Result<(), String> {
  let window = webview.window();
  let id = active_tab(&app, window.label()).ok_or("no tab")?;
  let tab = tab_webview(&app, id).ok_or("no tab")?;
  let menu = site_menu(&app, &tab).map_err(|e| e.to_string())?;
  {
    let mut inner = browser(&app);
    inner.context = Some(PageContext {
      tab: id,
      ..Default::default()
    });
    inner.current_window = Some(window.label().to_string());
  }
  popup(&window, &menu, Some((x, y))).map_err(|e| e.to_string())
}

/// The page's HTML, sent by the tab's init script after Save Page As picked a file.
#[tauri::command]
pub fn page_html<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  html: String,
) -> Result<(), String> {
  let id = tab_id(webview.label()).ok_or("not a tab")?;
  // Only written when the user asked for it (a pending save for this tab).
  let path = browser(&app)
    .pending_saves
    .remove(&id)
    .ok_or("no save pending")?;
  std::fs::write(&path, html).map_err(|e| e.to_string())?;
  let window = webview.window().label().to_string();
  crate::downloads::toast(&app, &window, &format!("Saved {}", path.display()));
  Ok(())
}
