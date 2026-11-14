// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A Tauri v3 showcase running on QtWebEngine instead of WebKitGTK.
//!
//!   cargo run -p qt-app                      # interactive showcase
//!   QT_APP_AUTO=1 cargo run -p qt-app        # run the self-test in the page and exit
//!   QT_APP_AUTO=close cargo run -p qt-app    # same, but exit by closing the last window
//!
//! Everything goes through the normal Tauri APIs: the app menu and context menus are Qt menus
//! (the runtime enables `tauri`'s `qt` feature), webview hooks, window events, child webviews,
//! popups, a tray icon, DevTools, cookies, downloads, permissions and custom protocols.

use std::{
  borrow::Cow,
  sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
  },
};

use serde_json::json;
use tauri::{
  AppHandle, Emitter, Listener, LogicalPosition, LogicalSize, Manager, Theme, Webview,
  WebviewBuilder, WebviewUrl, WebviewWindowBuilder, WindowEvent,
  menu::{
    AboutMetadata, CheckMenuItem, CheckMenuItemBuilder, Menu, MenuItemBuilder, MenuItemKind,
    PredefinedMenuItem, SubmenuBuilder,
  },
  webview::{DownloadEvent, NewWindowResponse, PageLoadEvent, PermissionResponse},
};
use tauri_runtime_qt::debug::Widget;

type R = tauri::DynRuntime;

/// Showcase state shared between commands and hooks.
#[derive(Default)]
struct Showcase {
  prevent_close: AtomicBool,
  /// Theme check items in the View menu: (System, Light, Dark).
  theme_items: Mutex<Vec<CheckMenuItem<R>>>,
}

/// Send a line to the page's event log (and to stdout).
fn log(app: &AppHandle, source: &str, message: impl Into<String>) {
  let message = message.into();
  println!("[{source}] {message}");
  let _ = app.emit(
    "showcase-log",
    json!({ "source": source, "message": message }),
  );
}

// --- self-test commands ---------------------------------------------------------------------

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {name}! (from Rust)")
}

#[tauri::command]
async fn slow_add(a: i64, b: i64) -> i64 {
  std::thread::sleep(std::time::Duration::from_millis(100));
  a + b
}

#[tauri::command]
fn fail() -> Result<(), String> {
  Err("expected failure".into())
}

#[tauri::command]
fn whoami(webview: Webview) -> String {
  webview.label().to_string()
}

#[tauri::command]
fn engine(app: AppHandle) -> Result<String, String> {
  app.webview_version().map_err(|e| e.to_string())
}

#[tauri::command]
fn can_go_back(webview: Webview) -> Result<bool, String> {
  webview.can_go_back().map_err(|e| e.to_string())
}

#[tauri::command]
fn go_back(webview: Webview) -> Result<(), String> {
  webview.go_back().map_err(|e| e.to_string())
}

#[tauri::command]
fn ping_event(app: AppHandle) {
  app.emit("from-rust", "pong").unwrap();
}

/// Print a check result. `skipped` checks can't run on this platform (e.g. WebGL offscreen);
/// `bug` marks a check for a known bug that isn't run (see `known_bugs`).
#[tauri::command]
fn report(name: String, ok: bool, detail: String, skipped: Option<bool>, bug: Option<bool>) {
  let status = match (bug.unwrap_or(false), skipped.unwrap_or(false), ok) {
    (true, ..) => "KNOWN-BUG",
    (_, true, _) => "SKIP",
    (_, _, true) => "PASS",
    _ => "FAIL",
  };
  println!("[{status}] {name}: {detail}");
}

/// `QT_APP_BUGS=1` also runs the checks for known bugs (some of them hang the app).
#[tauri::command]
fn known_bugs() -> bool {
  std::env::var_os("QT_APP_BUGS").is_some_and(|v| v != "0")
}

/// Every webview with the label of the window it is in. (`getAllWebviews()` in the JS API
/// can't be used for this: tauri's `WebviewRef` serializes `window_label`, but the JS reads
/// `windowLabel`.)
#[tauri::command]
fn webview_hosts(app: AppHandle) -> std::collections::BTreeMap<String, String> {
  app
    .webviews()
    .into_iter()
    .map(|(label, webview)| (label, webview.window().label().to_string()))
    .collect()
}

/// The Qt platform plugin the app runs on (`offscreen`, `wayland`, `xcb`), from the environment
/// Qt reads it from.
#[tauri::command]
fn qpa_platform() -> String {
  match std::env::var("QT_QPA_PLATFORM") {
    Ok(p) if !p.is_empty() => p.split([';', ':']).next().unwrap_or_default().to_string(),
    _ if std::env::var_os("WAYLAND_DISPLAY").is_some() => "wayland".into(),
    _ => "xcb".into(),
  }
}

/// How many times `on_web_content_process_terminate` fired.
static TERMINATIONS: AtomicU32 = AtomicU32::new(0);

#[tauri::command]
fn terminations() -> u32 {
  TERMINATIONS.load(Ordering::Relaxed)
}

/// Raw IPC: echo a binary request body back reversed, as a raw response.
#[tauri::command]
fn reverse_bytes(request: tauri::ipc::Request<'_>) -> Result<tauri::ipc::Response, String> {
  match request.body() {
    tauri::ipc::InvokeBody::Raw(bytes) => Ok(tauri::ipc::Response::new(
      bytes.iter().rev().copied().collect::<Vec<u8>>(),
    )),
    tauri::ipc::InvokeBody::Json(v) => Err(format!("expected a raw body, got JSON {v}")),
  }
}

/// The request headers the IPC request carried (to check custom `invoke` headers arrive).
#[tauri::command]
fn ipc_header(request: tauri::ipc::Request<'_>, name: String) -> Option<String> {
  request
    .headers()
    .get(&name)
    .and_then(|v| v.to_str().ok())
    .map(str::to_string)
}

/// A large JSON response: `len` characters.
#[tauri::command]
fn big_string(len: usize) -> String {
  "0123456789".chars().cycle().take(len).collect()
}

/// Sum of the bytes of a (JSON-encoded) array argument, and the length of a string argument.
#[tauri::command]
fn measure(bytes: Vec<u8>, text: String) -> (u64, usize) {
  (bytes.iter().map(|&b| b as u64).sum(), text.chars().count())
}

/// Stream `count` numbers through a channel from another thread, then `chunks` raw chunks.
#[tauri::command]
fn stream_numbers(
  count: u32,
  numbers: tauri::ipc::Channel<u32>,
  raw: tauri::ipc::Channel<tauri::ipc::Response>,
) {
  std::thread::spawn(move || {
    for i in 0..count {
      numbers.send(i).unwrap();
    }
    for i in 0..4u8 {
      raw
        .send(tauri::ipc::Response::new(vec![i; 64 * 1024]))
        .unwrap();
    }
  });
}

/// `Emitter::emit_to` from Rust to a specific webview label.
#[tauri::command]
fn emit_to(
  app: AppHandle,
  target: String,
  event: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  app
    .emit_to(target.as_str(), &event, payload)
    .map_err(|e| e.to_string())
}

/// `QT_APP_ONLY`: a regex selecting which self-test checks run (a development aid).
#[tauri::command]
fn selftest_only() -> Option<String> {
  std::env::var("QT_APP_ONLY").ok()
}

#[tauri::command]
fn auto_mode() -> Option<String> {
  std::env::var("QT_APP_AUTO").ok()
}

#[tauri::command]
fn done(app: AppHandle, failures: u32) {
  println!("self-test finished with {failures} failure(s)");
  match std::env::var("QT_APP_AUTO").as_deref() {
    // Close the last window: the runtime must emit ExitRequested and quit on its own.
    Ok("close") if failures == 0 => app.get_webview_window("main").unwrap().close().unwrap(),
    Ok(_) => app.exit(if failures == 0 { 0 } else { 1 }),
    Err(_) => {}
  }
}

/// Run `f` on the main thread with the window's `QWidget` and return its result.
fn with_qt_window<T: Send + 'static>(
  window: &tauri::Window,
  f: impl FnOnce(&Widget) -> T + Send + 'static,
) -> Result<T, String> {
  let (tx, rx) = std::sync::mpsc::channel();
  let window_ = window.clone();
  window
    .run_on_main_thread(move || {
      let widget = window_
        .qt_window()
        .map_err(|e| e.to_string())
        .and_then(|w| Widget::from_ptr(w).ok_or_else(|| "not a live QWidget".to_string()));
      let _ = tx.send(widget.map(|w| f(&w)));
    })
    .map_err(|e| e.to_string())?;
  rx.recv().map_err(|e| e.to_string())?
}

#[tauri::command]
fn menu_tree(window: tauri::Window) -> Result<String, String> {
  with_qt_window(&window, tauri_runtime_qt::debug::menubar_tree)
}

#[tauri::command]
fn menu_trigger(window: tauri::Window, path: String) -> Result<bool, String> {
  with_qt_window(&window, move |w| {
    tauri_runtime_qt::debug::trigger_menu_item(w, &path)
  })
}

#[tauri::command]
fn take_popup(window: tauri::Window) -> Result<String, String> {
  with_qt_window(&window, |_| tauri_runtime_qt::debug::take_popup_menu())
}

/// Find an item anywhere in the app menu (`Menu::get` only looks at top-level items).
fn find_menu_item(app: &AppHandle, id: &str) -> Option<MenuItemKind<R>> {
  fn search(items: Vec<MenuItemKind<R>>, id: &str) -> Option<MenuItemKind<R>> {
    items.into_iter().find_map(|i| match i {
      MenuItemKind::Submenu(s) => search(s.items().ok()?, id),
      other if other.id() == id => Some(other),
      _ => None,
    })
  }
  search(app.menu()?.items().ok()?, id)
}

#[tauri::command]
fn rename_menu_item(app: AppHandle, id: String, text: String) -> Result<(), String> {
  find_menu_item(&app, &id)
    .ok_or("no such item")?
    .as_menuitem()
    .ok_or("not a plain item")?
    .set_text(text)
    .map_err(|e| e.to_string())
}

// --- showcase commands ------------------------------------------------------------------------

#[tauri::command]
fn devtools(webview: Webview, action: String) -> bool {
  match action.as_str() {
    "open" => webview.open_devtools(),
    "close" => webview.close_devtools(),
    "toggle" if webview.is_devtools_open() => webview.close_devtools(),
    "toggle" => webview.open_devtools(),
    _ => {}
  }
  webview.is_devtools_open()
}

#[tauri::command]
fn navigate(webview: Webview, url: String) -> Result<(), String> {
  let url = url.parse().map_err(|e| format!("{e}"))?;
  webview.navigate(url).map_err(|e| e.to_string())
}

#[tauri::command]
fn history(webview: Webview, action: String) -> Result<serde_json::Value, String> {
  let r = match action.as_str() {
    "back" => webview.go_back(),
    "forward" => webview.go_forward(),
    "reload" => webview.reload(),
    _ => Ok(()),
  };
  r.map_err(|e| e.to_string())?;
  Ok(json!({
    "url": webview.url().map(|u| u.to_string()).unwrap_or_default(),
    "canGoBack": webview.can_go_back().unwrap_or(false),
    "canGoForward": webview.can_go_forward().unwrap_or(false),
  }))
}

#[tauri::command]
fn zoom(webview: Webview, factor: f64) -> Result<(), String> {
  webview.set_zoom(factor).map_err(|e| e.to_string())
}

#[tauri::command]
fn print_page(webview: Webview) -> Result<(), String> {
  webview.print().map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_browsing_data(webview: Webview) -> Result<(), String> {
  webview.clear_all_browsing_data().map_err(|e| e.to_string())
}

#[tauri::command]
fn cookies(webview: Webview) -> Result<Vec<String>, String> {
  webview
    .cookies()
    .map(|c| c.iter().map(|c| c.to_string()).collect())
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_cookie(webview: Webview, name: String, value: String) -> Result<(), String> {
  let cookie = tauri::webview::Cookie::build((name, value))
    .domain("localhost")
    .path("/")
    .build();
  webview.set_cookie(cookie).map_err(|e| e.to_string())
}

/// Evaluate JS from Rust and get the JSON result back.
#[tauri::command]
async fn eval_js(webview: Webview, code: String) -> Result<String, String> {
  let (tx, rx) = std::sync::mpsc::channel();
  webview
    .eval_with_callback(code, move |result| {
      let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;
  rx.recv_timeout(std::time::Duration::from_secs(5))
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_prevent_close(state: tauri::State<'_, Showcase>, prevent: bool) {
  state.prevent_close.store(prevent, Ordering::Relaxed);
}

fn apply_theme(app: &AppHandle, theme: Option<Theme>) {
  app.set_theme(theme);
  let index = match theme {
    None => 0,
    Some(Theme::Light) => 1,
    Some(_) => 2,
  };
  for (i, item) in app
    .state::<Showcase>()
    .theme_items
    .lock()
    .unwrap()
    .iter()
    .enumerate()
  {
    let _ = item.set_checked(i == index);
  }
}

#[tauri::command]
fn set_theme(app: AppHandle, theme: String) {
  apply_theme(
    &app,
    match theme.as_str() {
      "light" => Some(Theme::Light),
      "dark" => Some(Theme::Dark),
      _ => None,
    },
  );
}

/// Add a child webview to the main window, bottom-right, following the window size.
#[tauri::command]
fn add_child_webview(app: AppHandle, url: String) -> Result<String, String> {
  static CHILDREN: AtomicU32 = AtomicU32::new(0);
  let window = app.get_window("main").ok_or("no main window")?;
  let label = format!("child-{}", CHILDREN.fetch_add(1, Ordering::Relaxed));
  let url = if url.starts_with("http") {
    WebviewUrl::External(url.parse().map_err(|e| format!("{e}"))?)
  } else {
    WebviewUrl::App(url.into())
  };
  let size = window.inner_size().map_err(|e| e.to_string())?;
  let scale = window.scale_factor().unwrap_or(1.0);
  let (w, h) = (size.width as f64 / scale, size.height as f64 / scale);
  window
    .add_child(
      WebviewBuilder::new(&label, url)
        .position(LogicalPosition::new(w * 0.55, h * 0.55))
        .size(LogicalSize::new(w * 0.42, h * 0.40))
        .auto_resize(),
    )
    .map_err(|e| e.to_string())?;
  Ok(label)
}

#[tauri::command]
fn close_child_webviews(app: AppHandle) -> usize {
  let children: Vec<_> = app
    .webviews()
    .into_iter()
    .filter(|(label, _)| label.starts_with("child-"))
    .collect();
  for (_, webview) in &children {
    let _ = webview.close();
  }
  children.len()
}

/// Open another top-level window (optionally transient for the main window).
#[tauri::command]
fn open_window(app: AppHandle, with_parent: bool) -> Result<String, String> {
  static WINDOWS: AtomicU32 = AtomicU32::new(0);
  let label = format!("window-{}", WINDOWS.fetch_add(1, Ordering::Relaxed));
  let mut builder = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
    .title(format!("Showcase – {label}"))
    .inner_size(820.0, 640.0);
  if with_parent && let Some(main) = app.get_webview_window("main") {
    builder = builder.parent(&main).map_err(|e| e.to_string())?;
  }
  builder.build().map_err(|e| e.to_string())?;
  Ok(label)
}

// --- custom protocol --------------------------------------------------------------------------

/// A custom protocol that supports byte ranges, like Tauri's asset protocol does for video.
/// `/tone.wav` is a 3 s WAV file (for `<audio>` seeking); anything else is 1000 bytes of text.
fn stream_protocol(
  _ctx: tauri::UriSchemeContext<'_, R>,
  request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Cow<'static, [u8]>> {
  if request.uri().path() == "/tone.wav" {
    return ranged(&request, "audio/wav", wav(3));
  }
  let body: Vec<u8> = (0..1000u32)
    .map(|i| b"0123456789abcdefghijklmnopqrstuvwxyz"[(i % 36) as usize])
    .collect();
  ranged(&request, "text/plain", body)
}

/// Parse a single `Range: bytes=` header against a body of `len` bytes (inclusive bounds).
fn parse_range(header: Option<&str>, len: usize) -> Option<(usize, usize)> {
  let (a, b) = header?.strip_prefix("bytes=")?.split_once('-')?;
  let (start, end) = match (a.parse::<usize>().ok(), b.parse::<usize>().ok()) {
    (Some(a), Some(b)) => (a, b.min(len - 1)),
    (Some(a), None) => (a, len - 1),
    (None, Some(n)) => (len.saturating_sub(n), len - 1),
    (None, None) => return None,
  };
  (start <= end && end < len).then_some((start, end))
}

/// Serve `body` honouring a byte range, like Tauri's asset protocol.
fn ranged(
  request: &tauri::http::Request<Vec<u8>>,
  content_type: &str,
  body: Vec<u8>,
) -> tauri::http::Response<Cow<'static, [u8]>> {
  let builder = tauri::http::Response::builder()
    .header("Access-Control-Allow-Origin", "*")
    .header("Content-Type", content_type)
    .header("Accept-Ranges", "bytes");
  let range = request.headers().get("range").and_then(|v| v.to_str().ok());
  match parse_range(range, body.len()) {
    Some((start, end)) => builder
      .status(206)
      .header(
        "Content-Range",
        format!("bytes {start}-{end}/{}", body.len()),
      )
      .body(Cow::Owned(body[start..=end].to_vec()))
      .unwrap(),
    None => builder.body(Cow::Owned(body)).unwrap(),
  }
}

/// A mono 16-bit PCM WAV file of `seconds` seconds (a 440 Hz tone), to test media seeking.
fn wav(seconds: u32) -> Vec<u8> {
  const RATE: u32 = 8000;
  let samples = RATE * seconds;
  let mut out = Vec::with_capacity(44 + samples as usize * 2);
  let data_len = samples * 2;
  out.extend_from_slice(b"RIFF");
  out.extend_from_slice(&(36 + data_len).to_le_bytes());
  out.extend_from_slice(b"WAVEfmt ");
  out.extend_from_slice(&16u32.to_le_bytes());
  out.extend_from_slice(&1u16.to_le_bytes()); // PCM
  out.extend_from_slice(&1u16.to_le_bytes()); // mono
  out.extend_from_slice(&RATE.to_le_bytes());
  out.extend_from_slice(&(RATE * 2).to_le_bytes());
  out.extend_from_slice(&2u16.to_le_bytes());
  out.extend_from_slice(&16u16.to_le_bytes());
  out.extend_from_slice(b"data");
  out.extend_from_slice(&data_len.to_le_bytes());
  for i in 0..samples {
    let v = ((i as f32 * 440.0 * std::f32::consts::TAU / RATE as f32).sin() * 8000.0) as i16;
    out.extend_from_slice(&v.to_le_bytes());
  }
  out
}

/// A custom protocol answering from another thread (`register_asynchronous_uri_scheme_protocol`):
/// - `/echo`: JSON with the method, body length/checksum and the `x-test` header
/// - `/bytes?n=N`: N bytes, byte i = i % 251
/// - `/redirect`: a 302 to `/echo`
/// - `/missing`: an empty 404
fn async_protocol(
  _ctx: tauri::UriSchemeContext<'_, R>,
  request: tauri::http::Request<Vec<u8>>,
  responder: tauri::UriSchemeResponder,
) {
  std::thread::spawn(move || {
    let uri = request.uri().clone();
    let builder = tauri::http::Response::builder().header("Access-Control-Allow-Origin", "*");
    let response = match uri.path() {
      "/echo" => {
        let body = request.body();
        let sum: u64 = body.iter().map(|&b| b as u64).sum();
        let header = request
          .headers()
          .get("x-test")
          .and_then(|v| v.to_str().ok())
          .unwrap_or_default();
        let json = json!({
          "method": request.method().as_str(),
          "len": body.len(),
          "sum": sum,
          "header": header,
          "query": uri.query().unwrap_or_default(),
        });
        builder
          .header("Content-Type", "application/json")
          .header("X-Reply", "from-rust")
          .body(json.to_string().into_bytes())
          .unwrap()
      }
      "/bytes" => {
        let n = uri
          .query()
          .and_then(|q| q.strip_prefix("n="))
          .and_then(|n| n.parse::<usize>().ok())
          .unwrap_or(0);
        builder
          .header("Content-Type", "application/octet-stream")
          .body((0..n).map(|i| (i % 251) as u8).collect())
          .unwrap()
      }
      "/redirect" => builder
        .status(302)
        .header("Location", "async://localhost/echo?redirected=1")
        .body(Vec::new())
        .unwrap(),
      _ => builder.status(404).body(Vec::new()).unwrap(),
    };
    responder.respond(response);
  });
}

// --- menus ------------------------------------------------------------------------------------

fn app_menu(app: &AppHandle) -> tauri::Result<Menu<R>> {
  let file = SubmenuBuilder::new(app, "&File")
    .item(
      &MenuItemBuilder::with_id("new", "&New")
        .accelerator("CmdOrCtrl+N")
        .build(app)?,
    )
    .item(
      &CheckMenuItemBuilder::with_id("autosave", "Auto &save")
        .checked(false)
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?,
    )
    .separator()
    .item(
      &MenuItemBuilder::with_id("new-window", "New &Window")
        .accelerator("CmdOrCtrl+Shift+N")
        .build(app)?,
    )
    .close_window()
    .quit()
    .build()?;
  let edit = SubmenuBuilder::new(app, "&Edit")
    .undo()
    .redo()
    .separator()
    .cut()
    .copy()
    .paste()
    .select_all()
    .build()?;

  let theme_items = vec![
    CheckMenuItemBuilder::with_id("theme-system", "&System")
      .checked(true)
      .build(app)?,
    CheckMenuItemBuilder::with_id("theme-light", "&Light")
      .checked(false)
      .build(app)?,
    CheckMenuItemBuilder::with_id("theme-dark", "&Dark")
      .checked(false)
      .build(app)?,
  ];
  let theme = SubmenuBuilder::new(app, "&Theme")
    .items(&[&theme_items[0], &theme_items[1], &theme_items[2]])
    .build()?;
  *app.state::<Showcase>().theme_items.lock().unwrap() = theme_items;

  let view = SubmenuBuilder::new(app, "&View")
    .item(
      &MenuItemBuilder::with_id("devtools", "Toggle &DevTools")
        .accelerator("F12")
        .build(app)?,
    )
    .item(
      &MenuItemBuilder::with_id("reload", "&Reload")
        .accelerator("CmdOrCtrl+R")
        .build(app)?,
    )
    .separator()
    .item(
      &MenuItemBuilder::with_id("zoom-in", "Zoom &In")
        .accelerator("CmdOrCtrl+=")
        .build(app)?,
    )
    .item(
      &MenuItemBuilder::with_id("zoom-out", "Zoom &Out")
        .accelerator("CmdOrCtrl+-")
        .build(app)?,
    )
    .item(
      &MenuItemBuilder::with_id("zoom-reset", "&Actual Size")
        .accelerator("CmdOrCtrl+0")
        .build(app)?,
    )
    .separator()
    .fullscreen()
    .item(&theme)
    .build()?;
  let window = SubmenuBuilder::new(app, "&Window")
    .minimize()
    .maximize()
    .separator()
    .item(
      &CheckMenuItemBuilder::with_id("on-top", "Always on &Top")
        .checked(false)
        .build(app)?,
    )
    .build()?;
  let help = SubmenuBuilder::new(app, "&Help")
    .about(Some(AboutMetadata {
      name: Some("qt-app".into()),
      version: Some(env!("CARGO_PKG_VERSION").into()),
      comments: Some("A Tauri showcase running on QtWebEngine (Chromium).".into()),
      website: Some("https://tauri.app".into()),
      ..Default::default()
    }))
    .build()?;
  Menu::with_items(app, &[&file, &edit, &view, &window, &help])
}

static ZOOM: Mutex<f64> = Mutex::new(1.0);

fn on_menu_event(app: &AppHandle, id: &str) {
  log(app, "menu", format!("clicked \"{id}\""));
  let _ = app.emit("menu", id.to_string());
  let main = app.get_webview_window("main");
  let zoom = |f: &dyn Fn(f64) -> f64| {
    let mut z = ZOOM.lock().unwrap();
    *z = f(*z).clamp(0.25, 5.0);
    if let Some(w) = &main {
      let _ = w.set_zoom(*z);
    }
  };
  match id {
    "devtools" => {
      if let Some(w) = &main {
        if w.is_devtools_open() {
          w.close_devtools()
        } else {
          w.open_devtools()
        }
      }
    }
    "reload" => {
      if let Some(w) = &main {
        let _ = w.reload();
      }
    }
    "zoom-in" => zoom(&|z| z + 0.1),
    "zoom-out" => zoom(&|z| z - 0.1),
    "zoom-reset" => zoom(&|_| 1.0),
    "new-window" => {
      let _ = open_window(app.clone(), false);
    }
    "on-top" => {
      if let (Some(w), Some(MenuItemKind::Check(item))) = (&main, find_menu_item(app, "on-top")) {
        let _ = w.set_always_on_top(item.is_checked().unwrap_or(false));
      }
    }
    "theme-system" => apply_theme(app, None),
    "theme-light" => apply_theme(app, Some(Theme::Light)),
    "theme-dark" => apply_theme(app, Some(Theme::Dark)),
    "tray-show" => {
      if let Some(w) = &main {
        let _ = w.show();
        let _ = w.set_focus();
      }
    }
    "tray-hide" => {
      if let Some(w) = &main {
        let _ = w.hide();
      }
    }
    "tray-quit" => app.exit(0),
    _ => {}
  }
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
  let menu = Menu::with_items(
    app,
    &[
      &MenuItemBuilder::with_id("tray-show", "Show window").build(app)?,
      &MenuItemBuilder::with_id("tray-hide", "Hide window").build(app)?,
      &PredefinedMenuItem::separator(app)?,
      &MenuItemBuilder::with_id("tray-quit", "Quit").build(app)?,
    ],
  )?;
  let mut tray = tauri::tray::TrayIconBuilder::new()
    .tooltip("Tauri on QtWebEngine")
    .menu(&menu);
  if let Some(icon) = app.default_window_icon() {
    tray = tray.icon(icon.clone());
  }
  tray.build(app)?;
  Ok(())
}

// --- main window ------------------------------------------------------------------------------

/// The main window, with every webview hook wired to the page's event log. `window.open()` from
/// its page opens a real popup window connected to its opener.
fn create_main_window(app: &AppHandle) -> tauri::Result<()> {
  static POPUPS: AtomicU32 = AtomicU32::new(0);
  let handle = app.clone();
  let (nav, load, title, download, perm) = (
    app.clone(),
    app.clone(),
    app.clone(),
    app.clone(),
    app.clone(),
  );
  WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
    .title("Tauri on QtWebEngine")
    .inner_size(1100.0, 820.0)
    .on_new_window(move |url, features| {
      let label = format!("popup-{}", POPUPS.fetch_add(1, Ordering::Relaxed));
      log(&handle, "new-window", format!("{url} -> {label}"));
      let window = WebviewWindowBuilder::new(
        &handle,
        &label,
        WebviewUrl::External("about:blank".parse().unwrap()),
      )
      .title(url.as_str())
      .window_features(features)
      .build()
      .unwrap();
      NewWindowResponse::Create { window }
    })
    .on_navigation(move |url| {
      let blocked = url.host_str() == Some("blocked.example.com");
      if blocked {
        log(&nav, "navigation", format!("blocked {url}"));
      }
      !blocked
    })
    .on_page_load(move |_, payload| {
      let event = match payload.event() {
        PageLoadEvent::Started => "started",
        PageLoadEvent::Finished => "finished",
      };
      log(&load, "page-load", format!("{event}: {}", payload.url()));
    })
    .on_document_title_changed(move |_, t| log(&title, "title", t))
    .on_download(move |_, event| {
      match event {
        DownloadEvent::Requested { url, destination } => log(
          &download,
          "download",
          format!("requested {url} -> {}", destination.display()),
        ),
        DownloadEvent::Finished { url, path, success } => log(
          &download,
          "download",
          format!(
            "finished {url} -> {} (success: {success})",
            path.map(|p| p.display().to_string()).unwrap_or_default()
          ),
        ),
        _ => {}
      }
      true
    })
    .on_permission_request(move |_, kind| {
      log(
        &perm,
        "permission",
        format!("{kind:?} requested -> asking the user"),
      );
      // `Default` lets the runtime show its Allow/Deny prompt.
      PermissionResponse::Default
    })
    .build()?;
  Ok(())
}

fn main() {
  env_logger::init();
  tauri::Builder::default()
    .runtime(tauri_runtime_qt::Qt::default())
    .manage(Showcase::default())
    .register_uri_scheme_protocol("stream", stream_protocol)
    .register_asynchronous_uri_scheme_protocol("async", async_protocol)
    .invoke_handler(tauri::generate_handler![
      greet,
      slow_add,
      fail,
      whoami,
      engine,
      can_go_back,
      go_back,
      ping_event,
      report,
      qpa_platform,
      terminations,
      reverse_bytes,
      ipc_header,
      big_string,
      measure,
      stream_numbers,
      emit_to,
      auto_mode,
      selftest_only,
      known_bugs,
      webview_hosts,
      done,
      menu_tree,
      menu_trigger,
      take_popup,
      rename_menu_item,
      devtools,
      navigate,
      history,
      zoom,
      print_page,
      clear_browsing_data,
      cookies,
      set_cookie,
      eval_js,
      set_prevent_close,
      set_theme,
      add_child_webview,
      close_child_webviews,
      open_window
    ])
    .menu(app_menu)
    .on_menu_event(|app, event| on_menu_event(app, &event.id().0))
    .on_window_event(|window, event| {
      let app = window.app_handle();
      let label = window.label();
      match event {
        WindowEvent::CloseRequested { api, .. } => {
          let prevent = label == "main"
            && app
              .state::<Showcase>()
              .prevent_close
              .load(Ordering::Relaxed);
          if prevent {
            api.prevent_close();
          }
          log(
            app,
            "window",
            format!(
              "{label}: close requested{}",
              if prevent { " (prevented)" } else { "" }
            ),
          );
        }
        WindowEvent::DragDrop(e) => log(app, "drag-drop", format!("{label}: {e:?}")),
        other => log(app, "window", format!("{label}: {other:?}")),
      }
    })
    .on_web_content_process_terminate(|webview, termination| {
      log(
        webview.app_handle(),
        "renderer",
        format!(
          "{} terminated: {termination:?} - reloading",
          webview.label()
        ),
      );
      TERMINATIONS.fetch_add(1, Ordering::Relaxed);
      let _ = webview.reload();
    })
    .setup(|app| {
      app.listen("from-js", |e| {
        println!("[event] from-js payload={}", e.payload())
      });
      create_main_window(app.handle())?;
      if let Err(e) = create_tray(app.handle()) {
        log(app.handle(), "tray", format!("not available: {e}"));
      }
      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|app, event| match event {
      tauri::RunEvent::ExitRequested { code, api, .. } => {
        println!("[run] exit requested (code {code:?})");
        // QT_APP_AUTO=close: the last window closed. Prevent that exit once, then exit
        // explicitly from another thread, which must still reach the (running) event loop.
        static PREVENTED: AtomicBool = AtomicBool::new(false);
        let close_mode = std::env::var("QT_APP_AUTO").as_deref() == Ok("close");
        if close_mode && code.is_none() && !PREVENTED.swap(true, Ordering::Relaxed) {
          api.prevent_exit();
          let windows = app.webview_windows().len();
          println!(
            "[{}] exit: ExitRequested(None) after the last window closed, prevented ({windows} windows left)",
            if windows == 0 { "PASS" } else { "FAIL" }
          );
          let app = app.clone();
          std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            app.exit(0);
          });
        } else if close_mode {
          let ok = code == Some(0) && PREVENTED.load(Ordering::Relaxed);
          println!(
            "[{}] exit: app.exit(0) from another thread after a prevented exit -> ExitRequested({code:?})",
            if ok { "PASS" } else { "FAIL" }
          );
        }
      }
      tauri::RunEvent::Exit => println!("[run] exit"),
      _ => {}
    });
}
