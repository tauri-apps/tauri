// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Helpers shared by the runtime test cases. Every case runs in its own process (see
//! `qtshim::test_harness`): it creates the runtime with [`runtime`], drives the loop with
//! [`run_until`] / [`wait_for`] (built on `Runtime::run_iteration`) or with `run_return`, and
//! [`teardown`] shuts Qt down afterwards if the case left it running.

#![allow(dead_code)] // not every helper is used by every module

use std::{
  borrow::Cow,
  cell::{Cell, RefCell},
  collections::HashSet,
  sync::{Arc, Mutex},
  thread,
  time::{Duration, Instant},
};

use tauri_runtime::{
  ExitRequestedEventAction, RunEvent, Runtime, RuntimeInitArgs, WebviewDispatch,
  webview::{PendingWebview, WebviewAttributes},
  window::{DetachedWindow, PendingWindow, WebviewEvent, WindowBuilder, WindowEvent},
};
use tauri_runtime_qt::{Qt, QtRuntime, QtWebviewAttributes, QtWebviewDispatcher, QtWindowBuilder};
use tauri_utils::{Theme, config::WebviewUrl};

/// The user event type of the test runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
  Ping(u32),
}

pub type Rt = QtRuntime<Msg>;
pub type Detached = DetachedWindow<Msg, Rt>;
pub type Webview = QtWebviewDispatcher<Msg>;

/// How long [`wait_for`] waits. Generous: the first page load starts Chromium.
pub const WAIT: Duration = Duration::from_secs(20);

/// The custom scheme served by [`serve`]; registered at init by [`runtime`].
pub const SCHEME: &str = "testapp";

/// Passed to `qtshim::test_harness::main`: shut Qt down if the case left it running.
pub fn teardown() {
  if qtshim::app::is_running() {
    qtshim::app::shutdown();
  }
}

/// A runtime with [`SCHEME`] registered and default [`Qt`] attributes.
pub fn runtime() -> Rt {
  runtime_with(&[SCHEME], Qt::default())
}

pub fn runtime_with(custom_schemes: &[&str], attrs: Qt) -> Rt {
  Rt::new(RuntimeInitArgs {
    app_id: Some("tauri-runtime-qt-test".into()),
    identifier: "com.example.test".into(),
    custom_schemes: custom_schemes.iter().map(|s| s.to_string()).collect(),
    runtime_init_attrs: attrs,
  })
  .expect("Runtime::new")
}

// ---------------------------------------------------------------------------------------------
// recording run events

/// An owned, comparable summary of a [`RunEvent`].
#[derive(Debug, Clone, PartialEq)]
pub enum Ev {
  Ready,
  Exit,
  ExitRequested(Option<i32>),
  User(Msg),
  Window(String, Win),
  Webview(String, String),
  Other(String),
}

/// A [`WindowEvent`] summary.
#[derive(Debug, Clone, PartialEq)]
pub enum Win {
  Resized(u32, u32),
  Moved(i32, i32),
  Focused(bool),
  CloseRequested,
  Destroyed,
  Theme(Theme),
  Scale(f64),
  Other(String),
}

thread_local! {
  static LOG: RefCell<Vec<Ev>> = const { RefCell::new(Vec::new()) };
  static PREVENT_CLOSE: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
  static PREVENT_EXIT: Cell<u32> = const { Cell::new(0) };
}

/// Record `event` (ignoring `MainEventsCleared`), answering `CloseRequested` for the labels
/// passed to [`prevent_close`] and the next [`prevent_exits`] `ExitRequested`s. Returns the
/// recorded summary.
pub fn record(event: RunEvent<Msg>) -> Option<Ev> {
  let ev = match event {
    RunEvent::MainEventsCleared => return None,
    RunEvent::Ready => Ev::Ready,
    RunEvent::Exit => Ev::Exit,
    RunEvent::ExitRequested { code, tx } => {
      if PREVENT_EXIT.with(|p| p.get()) > 0 {
        PREVENT_EXIT.with(|p| p.set(p.get() - 1));
        tx.send(ExitRequestedEventAction::Prevent).unwrap();
      }
      Ev::ExitRequested(code)
    }
    RunEvent::UserEvent(m) => Ev::User(m),
    RunEvent::WindowEvent { label, event } => {
      let win = match event {
        WindowEvent::Resized(s) => Win::Resized(s.width, s.height),
        WindowEvent::Moved(p) => Win::Moved(p.x, p.y),
        WindowEvent::Focused(f) => Win::Focused(f),
        WindowEvent::CloseRequested { signal_tx } => {
          if PREVENT_CLOSE.with(|p| p.borrow().contains(&label)) {
            signal_tx.send(true).unwrap();
          }
          Win::CloseRequested
        }
        WindowEvent::Destroyed => Win::Destroyed,
        WindowEvent::ThemeChanged(t) => Win::Theme(t),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => Win::Scale(scale_factor),
        other => Win::Other(format!("{other:?}")),
      };
      Ev::Window(label, win)
    }
    RunEvent::WebviewEvent { label, event } => {
      let WebviewEvent::DragDrop(d) = event;
      Ev::Webview(label, format!("{d:?}"))
    }
    other => Ev::Other(format!("{other:?}")),
  };
  LOG.with(|l| l.borrow_mut().push(ev.clone()));
  Some(ev)
}

/// Prevent closing the window labelled `label` (answer `CloseRequested` with `true`).
pub fn prevent_close(label: &str, prevent: bool) {
  PREVENT_CLOSE.with(|p| {
    if prevent {
      p.borrow_mut().insert(label.to_string());
    } else {
      p.borrow_mut().remove(label);
    }
  });
}

/// Prevent the next `n` exit requests.
pub fn prevent_exits(n: u32) {
  PREVENT_EXIT.with(|p| p.set(n));
}

/// The events recorded so far.
pub fn events() -> Vec<Ev> {
  LOG.with(|l| l.borrow().clone())
}

pub fn has_event(ev: &Ev) -> bool {
  LOG.with(|l| l.borrow().contains(ev))
}

pub fn clear_events() {
  LOG.with(|l| l.borrow_mut().clear());
}

/// The recorded events of window `label`.
pub fn window_events(label: &str) -> Vec<Win> {
  events()
    .into_iter()
    .filter_map(|e| match e {
      Ev::Window(l, w) if l == label => Some(w),
      _ => None,
    })
    .collect()
}

// ---------------------------------------------------------------------------------------------
// driving the loop

/// Run loop iterations (`Runtime::run_iteration` with [`record`]) until `cond` returns `true` or
/// `timeout` passes. Returns whether `cond` became true.
pub fn run_until(rt: &mut Rt, timeout: Duration, mut cond: impl FnMut() -> bool) -> bool {
  let deadline = Instant::now() + timeout;
  loop {
    rt.run_iteration(|e| {
      record(e);
    });
    if cond() {
      return true;
    }
    if Instant::now() >= deadline {
      return false;
    }
    // `process_events` doesn't block; don't spin.
    thread::sleep(Duration::from_millis(1));
  }
}

/// [`run_until`] with [`WAIT`], panicking with `what` on timeout.
#[track_caller]
pub fn wait_for(rt: &mut Rt, what: &str, cond: impl FnMut() -> bool) {
  assert!(run_until(rt, WAIT, cond), "timed out waiting for {what}");
}

/// Run the loop for `duration`.
pub fn pump(rt: &mut Rt, duration: Duration) {
  run_until(rt, duration, || false);
}

/// Wait until window `label` got an event matching `pred`.
#[track_caller]
pub fn wait_window_event(rt: &mut Rt, label: &str, pred: impl Fn(&Win) -> bool) {
  let what = format!("a matching event of window {label:?}");
  let label = label.to_string();
  wait_for(rt, &what, move || window_events(&label).iter().any(&pred));
}

/// Evaluate `js` in `webview` and wait for its (JSON) result.
#[track_caller]
pub fn eval(rt: &mut Rt, webview: &Webview, js: &str) -> String {
  let out = Arc::new(Mutex::new(None));
  let o = out.clone();
  webview
    .eval_script_with_callback(js, move |r| *o.lock().unwrap() = Some(r))
    .unwrap();
  wait_for(rt, &format!("the result of {js:?}"), || {
    out.lock().unwrap().is_some()
  });
  out.lock().unwrap().take().unwrap()
}

/// Evaluate `js` until it returns `expected` (JSON).
#[track_caller]
pub fn wait_js(rt: &mut Rt, webview: &Webview, js: &str, expected: &str) {
  let deadline = Instant::now() + WAIT;
  let mut last = String::new();
  while Instant::now() < deadline {
    last = eval(rt, webview, js);
    if last == expected {
      return;
    }
    pump(rt, Duration::from_millis(20));
  }
  panic!("{js:?} returned {last:?}, expected {expected:?}");
}

// ---------------------------------------------------------------------------------------------
// windows, webviews and content

/// Requests seen by the [`serve`] handler: `(webview label, method, uri, origin header)`.
pub static REQUESTS: Mutex<Vec<Request>> = Mutex::new(Vec::new());

/// `(webview label, method, uri, origin header)` of a scheme request.
pub type Request = (String, String, String, Option<String>);

/// The URL of the test page titled `title` (see [`serve`]).
pub fn page(title: &str) -> String {
  format!("{SCHEME}://localhost/t/{title}")
}

/// Serve [`SCHEME`] on `pending`:
/// - `/t/<title>`: an HTML page with that title,
/// - `/download/<name>`: a file sent as an attachment,
/// - `/body/...`: the request body echoed back as text,
/// - anything else: 404.
pub fn serve(pending: &mut PendingWebview<Msg, Rt>) {
  pending.register_uri_scheme_protocol(SCHEME, |label, request, respond| {
    let origin = request
      .headers()
      .get("origin")
      .and_then(|v| v.to_str().ok())
      .map(str::to_string);
    REQUESTS.lock().unwrap().push((
      label.to_string(),
      request.method().to_string(),
      request.uri().to_string(),
      origin,
    ));
    let path = request.uri().path().to_string();
    let builder = http::Response::builder().header("Access-Control-Allow-Origin", "*");
    let response = if let Some(title) = path.strip_prefix("/t/") {
      builder.header("Content-Type", "text/html").body(Cow::Owned(
        format!("<!doctype html><title>{title}</title><p id=p>{title}</p>").into_bytes(),
      ))
    } else if let Some(name) = path.strip_prefix("/download/") {
      builder
        .header("Content-Type", "application/octet-stream")
        .header(
          "Content-Disposition",
          format!("attachment; filename=\"{name}\""),
        )
        .body(Cow::Borrowed(&b"downloaded bytes"[..]))
    } else if path.starts_with("/body/") {
      builder
        .header("Content-Type", "text/plain")
        .body(Cow::Owned(request.body().clone()))
    } else {
      builder.status(404).body(Cow::Borrowed(&b"not found"[..]))
    };
    respond(response.unwrap());
  });
}

/// A pending webview labelled `label` loading `url`, serving [`SCHEME`].
pub fn pending_webview(label: &str, url: &str) -> PendingWebview<Msg, Rt> {
  let mut p = PendingWebview::new(
    WebviewAttributes::new(WebviewUrl::App("index.html".into())),
    QtWebviewAttributes::default(),
    label,
  )
  .unwrap();
  p.url = url.to_string();
  serve(&mut p);
  p
}

/// The default window builder used by tests (small, visible).
pub fn builder() -> QtWindowBuilder {
  QtWindowBuilder::new().inner_size(400.0, 300.0)
}

/// A window without a webview.
pub fn window(rt: &Rt, label: &str, builder: QtWindowBuilder) -> Detached {
  rt.create_window(
    PendingWindow::new(builder, label).unwrap(),
    None::<fn(tauri_runtime::window::RawWindow)>,
  )
  .expect("create_window")
}

/// A window with a webview (labelled like the window) loading `url`.
pub fn webview_window(rt: &Rt, label: &str, url: &str) -> (Detached, Webview) {
  webview_window_with(rt, label, builder(), pending_webview(label, url))
}

pub fn webview_window_with(
  rt: &Rt,
  label: &str,
  builder: QtWindowBuilder,
  webview: PendingWebview<Msg, Rt>,
) -> (Detached, Webview) {
  let mut pending = PendingWindow::new(builder, label).unwrap();
  pending.set_webview(webview);
  let w = rt
    .create_window(pending, None::<fn(tauri_runtime::window::RawWindow)>)
    .expect("create_window");
  let wv = w.webview.as_ref().unwrap().webview.dispatcher.clone();
  (w, wv)
}

/// Wait until `webview`'s document title is `title`.
#[track_caller]
pub fn wait_title(rt: &mut Rt, webview: &Webview, title: &str) {
  wait_js(rt, webview, "document.title", &format!("{title:?}"));
}

/// A unique temporary directory, removed on drop.
pub struct TempDir(pub std::path::PathBuf);

impl TempDir {
  pub fn new(tag: &str) -> Self {
    let dir = std::env::temp_dir().join(format!(
      "tauri-runtime-qt-test-{tag}-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    Self(dir)
  }
}

impl Drop for TempDir {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.0);
  }
}
