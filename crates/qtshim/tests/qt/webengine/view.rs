// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `qtshim::webengine::View`: loading, navigation, events and their replies, scripts, history,
//! IPC, popups, permissions, downloads, devtools, crashes, geometry and teardown.
//!
//! Content comes from `load_html`, `data:` and `about:` URLs (or a custom scheme registered by
//! the case itself): no network.

use std::{
  cell::{Cell, RefCell},
  rc::Rc,
  time::{Duration, Instant},
};

use qtshim::test_harness::Case;
use qtshim::{
  app::{self, Event, EventKind, Reply},
  webengine::{self, Profile, SchemeFlags, View, ViewOptions, Window},
};

use crate::common::{WAIT, clear_events, events, has_event, init, pump, record_events, wait_for};

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass("view: load html and read the title back", load_html_title),
    Case::pass(
      "view: load events report started then finished",
      load_events,
    ),
    Case::pass("view: title events carry the title", title_events),
    Case::pass("view: load_html without base url", load_html_no_base),
    Case::pass("view: load_html resolves against base url", load_html_base),
    Case::pass("view: load_url data and about:blank", load_url_data),
    Case::pass("view: load_url sends extra headers", load_url_headers),
    Case::pass("view: reload reloads the page", reload),
    Case::pass(
      "view: navigate reply 0 blocks the navigation",
      navigate_block,
    ),
    Case::pass(
      "view: navigate events for every main-frame load",
      navigate_events,
    ),
    Case::pass(
      "view: pushState history back and forward",
      history_push_state,
    ),
    Case::pass("view: history across documents", history_documents),
    Case::pass("view: history without javascript", history_no_js),
    Case::pass("view: set_zoom changes the page zoom", zoom),
    Case::pass("view: background color and transparency", background),
    Case::pass(
      "view: init script runs before page scripts",
      init_script_order,
    ),
    Case::pass(
      "view: init script main frame only vs all frames",
      init_script_frames,
    ),
    Case::pass(
      "view: init script applies from the next navigation",
      init_script_next_nav,
    ),
    Case::pass("view: eval result types", eval_types),
    Case::pass("view: eval without callback", eval_no_callback),
    Case::pass(
      "view: eval callback panic is contained",
      eval_callback_panic,
    ),
    Case::pass(
      "view: eval callback dropped with the view",
      eval_callback_view_dropped,
    ),
    Case::pass("view: ipc postMessage before and after connect", ipc),
    Case::pass("view: ipc object is read-only", ipc_frozen),
    Case::pass("view: javascript disabled", javascript_disabled),
    Case::pass("view: new window denied", new_window_deny),
    Case::pass("view: new window default reply denies", new_window_default),
    Case::pass("view: new window as popup", new_window_popup),
    Case::pass("view: new window in another view", new_window_in_view),
    Case::pass("view: new window in unknown view", new_window_unknown_view),
    Case::pass("view: permission granted", permission_grant),
    Case::pass("view: permission denied", permission_deny),
    Case::pass(
      "view: geolocation permission denied",
      permission_geolocation,
    ),
    Case::pass(
      "view: permission default reply shows a prompt",
      permission_prompt,
    ),
    Case::pass("view: download to a chosen path", download_accept),
    Case::pass("view: download cancelled", download_cancel),
    Case::pass("view: devtools open and close", devtools),
    Case::pass("view: devtools disabled cannot open", devtools_disabled),
    Case::pass(
      "view: renderer crash reloads by default",
      crash_default_reload,
    ),
    Case::pass("view: renderer crash handled, manual reload", crash_handled),
    Case::pass("view: set_bounds and bounds", bounds),
    Case::pass("view: fill follows the window", fill_follows_window),
    Case::pass("view: fill option at creation", fill_option),
    Case::pass("view: visibility", visibility),
    Case::pass("view: focus", focus),
    Case::pass(
      "view: focus option and clipboard and drag options",
      other_options,
    ),
    Case::pass("view: qt_view and qt_page", qt_pointers),
    Case::pass("view: multiple views in one window", multiple_views),
    Case::pass("view: reparent to another window", reparent),
    Case::pass(
      "view: inert after its window is dropped",
      inert_after_window_dropped,
    ),
    Case::pass(
      "view: url after its window is dropped",
      url_after_window_dropped,
    ),
    Case::pass(
      "view: eval on inert view drops the callback",
      eval_inert_drops_callback,
    ),
    Case::pass(
      "view: dropped inside its own title handler",
      drop_in_title_handler,
    ),
    Case::pass(
      "view: dropped inside its load handler",
      drop_in_load_handler,
    ),
    Case::pass(
      "view: dropped inside new window handler",
      drop_in_new_window_handler,
    ),
    Case::pass(
      "view: dropped inside permission handler",
      drop_in_permission_handler,
    ),
    Case::pass("view: dropped while loading", drop_while_loading),
    Case::pass(
      "view: survives shutdown and drops quietly",
      survives_shutdown,
    ),
    Case::panics(
      "view: use after shutdown panics",
      "Qt can only be used on the thread",
      use_after_shutdown,
    ),
    // Opens a modal QPrintDialog (`exec`), which blocks forever without a user.
    Case::pass("view: print opens a dialog", print).ignored(),
  ]
}

// ---------------------------------------------------------------------------------------------
// helpers

const WIN: u64 = 1;
const VIEW: u64 = 7;
/// A secure origin for pages that need one (permissions, pushState). Never fetched.
const BASE: &str = "https://app.invalid/";

/// Init, record events (answered by `reply`), and create a shown window with a filling view.
fn setup(reply: impl Fn(&Event<'_>) -> Reply + 'static) -> (Window, Rc<View>) {
  setup_with(ViewOptions::default(), reply)
}

fn setup_with(
  options: ViewOptions,
  reply: impl Fn(&Event<'_>) -> Reply + 'static,
) -> (Window, Rc<View>) {
  init();
  record_events(reply);
  let window = Window::new(WIN, false);
  let view = Rc::new(View::new(VIEW, &window, Profile::get(None, true), options));
  view.set_fill(true);
  window.set_visible(true);
  (window, view)
}

fn default_reply(_: &Event<'_>) -> Reply {
  Reply::default()
}

fn is_finished(e: &crate::common::Recorded, view: u64) -> bool {
  e.kind == EventKind::ViewLoad && e.view == view && e.a == 1
}

/// Wait for a finished load of `view` (after the last [`clear_events`]).
fn wait_loaded(view: u64) {
  wait_for("the page to load", move || {
    has_event(|e| is_finished(e, view))
  });
}

/// Load `html` (no base URL) and wait until it has loaded.
fn load(view: &View, html: &str) {
  load_base(view, html, None);
}

fn load_base(view: &View, html: &str, base: Option<&str>) {
  clear_events();
  view.load_html(html, base);
  wait_loaded(VIEW);
}

/// Evaluate `js` and wait for its JSON result.
fn eval(view: &View, js: &str) -> String {
  let out: Rc<RefCell<Option<String>>> = Rc::default();
  let o = out.clone();
  view.eval_with_callback(js, move |s| *o.borrow_mut() = Some(s.to_owned()));
  let o = out.clone();
  wait_for(&format!("the result of {js}"), move || o.borrow().is_some());
  out.take().unwrap()
}

/// Evaluate `js` until it returns `expected`.
#[track_caller]
fn eval_until(view: &View, js: &str, expected: &str) {
  let deadline = Instant::now() + WAIT;
  loop {
    let got = eval(view, js);
    if got == expected {
      return;
    }
    assert!(
      Instant::now() < deadline,
      "{js}: expected {expected}, last got {got}"
    );
    pump(Duration::from_millis(20));
  }
}

/// Wait for a title event with exactly `title` for `view`.
fn wait_title(view: u64, title: &'static str) {
  wait_for(&format!("title {title:?}"), move || {
    has_event(|e| e.kind == EventKind::ViewTitle && e.view == view && e.s1 == title)
  });
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
  let dir = std::env::temp_dir().join(format!("qtshim-view-{tag}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&dir);
  std::fs::create_dir_all(&dir).unwrap();
  dir
}

/// Counts drops of the value it's captured in.
struct DropFlag(Rc<Cell<u32>>);
impl Drop for DropFlag {
  fn drop(&mut self) {
    self.0.set(self.0.get() + 1);
  }
}

// ---------------------------------------------------------------------------------------------
// loading

fn load_html_title() {
  let (_window, view) = setup(default_reply);
  view.load_html("<title>hello qt</title><p>hi</p>", None);
  wait_title(VIEW, "hello qt");
  assert_eq!(eval(&view, "document.title"), "\"hello qt\"");
}

fn load_events() {
  let (_window, view) = setup(default_reply);
  let url = "data:text/html,<title>t</title>";
  view.load_url(url, None);
  wait_loaded(VIEW);
  let loads: Vec<_> = events()
    .into_iter()
    .filter(|e| e.kind == EventKind::ViewLoad)
    .collect();
  let started = loads
    .iter()
    .position(|e| e.a == 0)
    .expect("a started event");
  let finished = loads.iter().position(|e| e.a == 1).unwrap();
  assert!(started < finished, "{loads:?}");
  for e in &loads {
    assert_eq!((e.window, e.view), (WIN, VIEW));
    assert!(e.s1.starts_with("data:text/html"), "{e:?}");
  }
}

fn title_events() {
  let (_window, view) = setup(default_reply);
  load(&view, "<title>first</title>");
  wait_title(VIEW, "first");
  view.eval("document.title = 'second'");
  wait_title(VIEW, "second");
  let e = events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewTitle && e.s1 == "second")
    .unwrap();
  assert_eq!((e.window, e.view), (WIN, VIEW));
}

fn load_html_no_base() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p id=p>x</p>");
  assert_eq!(
    eval(&view, "document.getElementById('p').textContent"),
    "\"x\""
  );
  // Without a base URL the document has an opaque origin.
  assert_eq!(eval(&view, "location.origin"), "\"null\"");
}

fn load_html_base() {
  let (_window, view) = setup(default_reply);
  load_base(&view, "<a id=a href='sub/page.html'>x</a>", Some(BASE));
  assert_eq!(
    eval(&view, "document.getElementById('a').href"),
    "\"https://app.invalid/sub/page.html\""
  );
  assert_eq!(eval(&view, "location.origin"), "\"https://app.invalid\"");
  assert_eq!(view.url(), BASE);
}

fn load_url_data() {
  let (_window, view) = setup(default_reply);
  let url = "data:text/html,<title>data</title>";
  view.load_url(url, None);
  wait_title(VIEW, "data");
  assert!(view.url().starts_with("data:text/html"), "{}", view.url());
  clear_events();
  view.load_url("about:blank", None);
  wait_loaded(VIEW);
  assert_eq!(view.url(), "about:blank");
}

fn load_url_headers() {
  init();
  assert!(webengine::register_scheme("viewtest", SchemeFlags::APP));
  /// (view id, url, `x-test` header) of each scheme request.
  type Seen = Rc<RefCell<Vec<(u64, String, Option<String>)>>>;
  let seen: Seen = Rc::default();
  let s = seen.clone();
  webengine::set_scheme_handler(move |req, responder| {
    let h = req
      .request
      .headers()
      .get("x-test")
      .map(|v| v.to_str().unwrap().to_owned());
    s.borrow_mut()
      .push((req.view, req.request.uri().to_string(), h));
    responder.respond(
      http::Response::builder()
        .header("content-type", "text/html")
        .body(b"<title>scheme</title>".to_vec())
        .unwrap(),
    );
  });
  let (_window, view) = setup(default_reply);
  let mut headers = http::HeaderMap::new();
  headers.insert("x-test", "yes".parse().unwrap());
  view.load_url("viewtest://host/index.html", Some(&headers));
  wait_title(VIEW, "scheme");
  let seen = seen.borrow();
  let (vid, url, h) = seen
    .iter()
    .find(|(_, u, _)| u.contains("index.html"))
    .expect("the scheme request");
  assert_eq!(*vid, VIEW);
  assert_eq!(url, "viewtest://host/index.html");
  assert_eq!(h.as_deref(), Some("yes"));
}

fn reload() {
  let (_window, view) = setup(default_reply);
  view.load_url("data:text/html,<title>r</title>", None);
  wait_title(VIEW, "r");
  view.eval("window.marker = 1");
  assert_eq!(eval(&view, "window.marker"), "1");
  clear_events();
  view.reload();
  wait_loaded(VIEW);
  assert_eq!(eval(&view, "typeof window.marker"), "\"undefined\"");
}

fn navigate_block() {
  let (_window, view) = setup(|e| {
    if e.kind == EventKind::ViewNavigate && e.s1.contains("blocked") {
      Reply::code(0)
    } else {
      Reply::default()
    }
  });
  load(&view, "<title>start</title>");
  clear_events();
  view.load_url("data:text/html,<title>blocked</title>", None);
  wait_for("the navigate event", || {
    has_event(|e| e.kind == EventKind::ViewNavigate && e.s1.contains("blocked"))
  });
  // An allowed navigation afterwards goes through; the blocked one never did.
  view.load_url("data:text/html,<title>allowed</title>", None);
  wait_title(VIEW, "allowed");
  assert!(!has_event(
    |e| e.kind == EventKind::ViewTitle && e.s1 == "blocked"
  ));
  assert!(view.url().contains("allowed"));
}

fn navigate_events() {
  let (_window, view) = setup(default_reply);
  let url = "data:text/html,<title>nav</title><iframe srcdoc='<p>sub'></iframe>";
  view.load_url(url, None);
  wait_title(VIEW, "nav");
  wait_loaded(VIEW);
  let navs: Vec<_> = events()
    .into_iter()
    .filter(|e| e.kind == EventKind::ViewNavigate)
    .collect();
  // Only the main frame is reported (the iframe isn't).
  assert_eq!(navs.len(), 1, "{navs:?}");
  assert_eq!((navs[0].window, navs[0].view), (WIN, VIEW));
  assert!(navs[0].s1.starts_with("data:text/html"));
}

// ---------------------------------------------------------------------------------------------
// history

fn history_push_state() {
  // Served from a custom scheme like a real app (a `load_html` page's history entries are all
  // the same `data:` URL). The page pushes its routes itself, without a user gesture, like an
  // SPA router: Chromium's intervention would make a relative Back skip "/a".
  init();
  assert!(webengine::register_scheme("spa", SchemeFlags::APP));
  webengine::set_scheme_handler(|_, responder| {
    responder.respond(
      http::Response::builder()
        .header("content-type", "text/html")
        .body(
          b"<title>h</title><script>setTimeout(() => { history.pushState({}, '', '/a'); \
            history.pushState({}, '', '/b') }, 0)</script>"
            .to_vec(),
        )
        .unwrap(),
    )
  });
  let (_window, view) = setup(default_reply);
  view.load_url("spa://app/", None);
  eval_until(&view, "location.pathname", "\"/b\"");
  assert!(view.can_go_back());
  assert!(!view.can_go_forward());
  view.go_back();
  eval_until(&view, "location.pathname", "\"/a\"");
  let v = view.clone();
  wait_for("a forward entry", move || v.can_go_forward());
  assert!(view.url().ends_with("/a"), "{}", view.url());

  view.go_forward();
  eval_until(&view, "location.pathname", "\"/b\"");
  let v = view.clone();
  wait_for("no forward entry", move || !v.can_go_forward());
}

fn history_documents() {
  let (_window, view) = setup(default_reply);
  view.load_url("data:text/html,<title>one</title>", None);
  wait_title(VIEW, "one");
  view.load_url("data:text/html,<title>two</title>", None);
  wait_title(VIEW, "two");
  let v = view.clone();
  wait_for("a back entry", move || v.can_go_back());
  clear_events();
  view.go_back();
  wait_title(VIEW, "one");
  assert!(view.can_go_forward());
  clear_events();
  view.go_forward();
  wait_title(VIEW, "two");
}

fn history_no_js() {
  let options = ViewOptions {
    javascript_disabled: true,
    ..Default::default()
  };
  let (_window, view) = setup_with(options, default_reply);
  view.load_url("data:text/html,<title>one</title>", None);
  wait_title(VIEW, "one");
  view.load_url("data:text/html,<title>two</title>", None);
  wait_title(VIEW, "two");
  let v = view.clone();
  wait_for("a back entry", move || v.can_go_back());
  // Falls back to triggerAction(Back) / (Forward).
  view.go_back();
  let v = view.clone();
  wait_for("going back", move || v.url().contains("one"));
  view.go_forward();
  let v = view.clone();
  wait_for("going forward", move || v.url().contains("two"));
}

// ---------------------------------------------------------------------------------------------
// appearance

fn zoom() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>z</p>");
  let base: f64 = eval(&view, "window.devicePixelRatio").parse().unwrap();
  view.set_zoom(2.0);
  eval_until(&view, "window.devicePixelRatio", &format!("{}", base * 2.0));
  view.set_zoom(1.0);
  eval_until(&view, "window.devicePixelRatio", &format!("{base}"));
}

fn background() {
  let options = ViewOptions {
    transparent: true,
    ..Default::default()
  };
  let (_window, view) = setup_with(options, default_reply);
  view.set_background_color((255, 0, 0, 255));
  load(&view, "<title>bg</title>");
  view.set_background_color((0, 0, 0, 0));
  assert_eq!(eval(&view, "document.title"), "\"bg\"");
}

// ---------------------------------------------------------------------------------------------
// scripts

fn init_script_order() {
  let (_window, view) = setup(default_reply);
  view.add_init_script("window.__init = 'early'", true);
  view.load_html(
    "<script>document.title = String(window.__init)</script>",
    None,
  );
  wait_title(VIEW, "early");
}

fn init_script_frames() {
  let (_window, view) = setup(default_reply);
  view.add_init_script("self.__main = 1", true);
  view.add_init_script("self.__all = 1", false);
  load_base(&view, "<iframe srcdoc='<p>sub</p>'></iframe>", Some(BASE));
  eval_until(&view, "frames.length", "1");
  eval_until(&view, "frames[0].document.readyState", "\"complete\"");
  assert_eq!(eval(&view, "[self.__main, self.__all]"), "[1,1]");
  assert_eq!(
    eval(&view, "[frames[0].__main === undefined, frames[0].__all]"),
    "[true,1]"
  );
}

fn init_script_next_nav() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>a</p>");
  view.add_init_script("window.__late = 1", true);
  assert_eq!(eval(&view, "typeof window.__late"), "\"undefined\"");
  clear_events();
  view.reload();
  wait_loaded(VIEW);
  assert_eq!(eval(&view, "window.__late"), "1");
}

fn eval_types() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>e</p>");
  assert_eq!(eval(&view, "'abc'"), "\"abc\"");
  assert_eq!(eval(&view, "'quote\"d'"), "\"quote\\\"d\"");
  assert_eq!(eval(&view, "40 + 2"), "42");
  assert_eq!(eval(&view, "1.5"), "1.5");
  assert_eq!(eval(&view, "true"), "true");
  assert_eq!(eval(&view, "null"), "null");
  assert_eq!(eval(&view, "undefined"), "null");
  assert_eq!(eval(&view, "({a: 1, b: 'x'})"), "{\"a\":1,\"b\":\"x\"}");
  assert_eq!(eval(&view, "[1, 'two']"), "[1,\"two\"]");
  // An exception gives no value.
  assert_eq!(eval(&view, "throw new Error('boom')"), "null");
  assert_eq!(eval(&view, "syntax error here"), "null");
  assert_eq!(
    eval(&view, "'unicode \u{e9}\u{1f600}'"),
    "\"unicode \u{e9}\u{1f600}\""
  );
}

fn eval_no_callback() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>e</p>");
  view.eval("document.title = 'from eval'");
  wait_title(VIEW, "from eval");
}

fn eval_callback_panic() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>e</p>");
  let called = Rc::new(Cell::new(false));
  let c = called.clone();
  view.eval_with_callback("1", move |_| {
    c.set(true);
    panic!("callback panic (expected)");
  });
  let c = called.clone();
  wait_for("the panicking callback", move || c.get());
  // Still usable.
  assert_eq!(eval(&view, "2"), "2");
}

fn eval_callback_view_dropped() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>e</p>");
  let drops = Rc::new(Cell::new(0));
  let calls = Rc::new(Cell::new(0));
  let flag = DropFlag(drops.clone());
  let c = calls.clone();
  view.eval_with_callback("1", move |_| {
    let _keep = &flag;
    c.set(c.get() + 1);
  });
  drop(Rc::into_inner(view).unwrap());
  let d = drops.clone();
  wait_for("the callback to be dropped", move || d.get() > 0);
  assert_eq!(drops.get(), 1);
  assert!(calls.get() <= 1);
}

// ---------------------------------------------------------------------------------------------
// ipc

fn ipc() {
  let (_window, view) = setup(default_reply);
  // Posted while the page loads, before the web channel connects: queued.
  load_base(
    &view,
    "<script>window.ipc.postMessage('early'); window.ipc.postMessage(42)</script>",
    Some(BASE),
  );
  wait_for("the queued messages", || {
    has_event(|e| e.kind == EventKind::ViewIpc && e.s1 == "42")
  });
  view.eval("window.ipc.postMessage('late \u{e9}')");
  wait_for("the late message", || {
    has_event(|e| e.kind == EventKind::ViewIpc && e.s1 == "late \u{e9}")
  });
  let msgs: Vec<_> = events()
    .into_iter()
    .filter(|e| e.kind == EventKind::ViewIpc)
    .collect();
  let texts: Vec<_> = msgs.iter().map(|e| e.s1.as_str()).collect();
  assert_eq!(texts, ["early", "42", "late \u{e9}"]);
  for e in &msgs {
    assert_eq!((e.window, e.view), (WIN, VIEW));
    assert_eq!(e.s2, BASE);
  }
}

fn ipc_frozen() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>i</p>");
  assert_eq!(
    eval(
      &view,
      "window.ipc = 1; window.ipc.postMessage = null; typeof window.ipc.postMessage"
    ),
    "\"function\""
  );
  view.eval("window.ipc.postMessage('still')");
  wait_for("the message", || {
    has_event(|e| e.kind == EventKind::ViewIpc && e.s1 == "still")
  });
}

fn javascript_disabled() {
  let options = ViewOptions {
    javascript_disabled: true,
    ..Default::default()
  };
  let (_window, view) = setup_with(options, default_reply);
  load(
    &view,
    "<title>nojs</title><script>document.title = 'js'</script>",
  );
  wait_title(VIEW, "nojs");
  pump(Duration::from_millis(100));
  assert!(!has_event(
    |e| e.kind == EventKind::ViewTitle && e.s1 == "js"
  ));
}

// ---------------------------------------------------------------------------------------------
// new windows

/// `window.open` always returns a window proxy; a denied popup is closed right after.
const OPEN: &str = "window.__w = window.open('about:blank#popup', '_blank', \
                    'left=10,top=20,width=300,height=200'); 0";

/// Open a popup and wait until the page sees it closed (`true`, denied) or not.
fn open_popup(view: &View, closed: bool) {
  assert_eq!(eval(view, OPEN), "0");
  wait_for("the new window event", || {
    has_event(|e| e.kind == EventKind::ViewNewWindow)
  });
  if closed {
    eval_until(view, "window.__w.closed", "true");
  } else {
    pump(Duration::from_millis(200));
    assert_eq!(eval(view, "window.__w.closed"), "false");
  }
}

fn new_window_event() -> crate::common::Recorded {
  events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewNewWindow)
    .expect("a new window event")
}

fn new_window_deny() {
  let (_window, view) = setup(|e| match e.kind {
    EventKind::ViewNewWindow => Reply::code(0),
    _ => Reply::default(),
  });
  load(&view, "<p>n</p>");
  open_popup(&view, true);
  let e = new_window_event();
  assert_eq!((e.window, e.view), (WIN, VIEW));
  assert_eq!(e.s1, "about:blank#popup");
  assert_eq!((e.c, e.d), (300, 200));
}

fn new_window_default() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>n</p>");
  open_popup(&view, true);
}

fn new_window_popup() {
  let (_window, view) = setup(|e| match e.kind {
    EventKind::ViewNewWindow => Reply::code(1),
    _ => Reply::default(),
  });
  load(&view, "<p>n</p>");
  open_popup(&view, false);
  view.eval("window.__w = window.open('about:blank#second')");
  pump(Duration::from_millis(100));
  assert_eq!(
    events()
      .iter()
      .filter(|e| e.kind == EventKind::ViewNewWindow)
      .count(),
    2
  );
}

fn new_window_in_view() {
  const TARGET: u64 = 99;
  let (window, view) = setup(|e| match e.kind {
    EventKind::ViewNewWindow => Reply {
      code: 2,
      int: TARGET as i64,
      string: None,
    },
    _ => Reply::default(),
  });
  let target = Rc::new(View::new(
    TARGET,
    &window,
    Profile::get(None, true),
    ViewOptions::default(),
  ));
  load(&view, "<p>n</p>");
  open_popup(&view, false);
  let t = target.clone();
  wait_for("the popup url in the target view", move || {
    t.url().contains("popup")
  });
  assert_eq!(eval(&target, "window.opener !== null"), "true");
  // The opener can script the popup it got back.
  view.eval("window.open('about:blank#x', 'named').document.title = 'scripted'");
  wait_title(TARGET, "scripted");
}

fn new_window_unknown_view() {
  let (_window, view) = setup(|e| match e.kind {
    EventKind::ViewNewWindow => Reply {
      code: 2,
      int: 12345,
      string: None,
    },
    _ => Reply::default(),
  });
  load(&view, "<p>n</p>");
  open_popup(&view, true);
  assert_eq!(eval(&view, "1"), "1");
}

// ---------------------------------------------------------------------------------------------
// permissions

const NOTIFY: &str = "Notification.requestPermission().then(r => document.title = 'perm:' + r); 0";
/// `QWebEnginePermission::PermissionType::Notifications`.
const NOTIFICATIONS: i64 = 7;
/// `QWebEnginePermission::PermissionType::Geolocation`.
const GEOLOCATION: i64 = 8;

fn permission_reply(code: i32) -> impl Fn(&Event<'_>) -> Reply {
  move |e| match e.kind {
    EventKind::ViewPermission => Reply::code(code),
    _ => Reply::default(),
  }
}

fn permission_grant() {
  let (_window, view) = setup(permission_reply(1));
  load_base(&view, "<p>p</p>", Some(BASE));
  view.eval(NOTIFY);
  wait_title(VIEW, "perm:granted");
  let e = events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewPermission)
    .unwrap();
  assert_eq!((e.window, e.view, e.a), (WIN, VIEW, NOTIFICATIONS));
  assert!(e.s1.starts_with("https://app.invalid"), "{e:?}");
}

fn permission_deny() {
  let (_window, view) = setup(permission_reply(0));
  load_base(&view, "<p>p</p>", Some(BASE));
  view.eval(NOTIFY);
  wait_title(VIEW, "perm:denied");
}

fn permission_geolocation() {
  let (_window, view) = setup(permission_reply(0));
  load_base(&view, "<p>p</p>", Some(BASE));
  view.eval(
    "navigator.geolocation.getCurrentPosition(() => document.title = 'geo:ok', \
     e => document.title = 'geo:' + e.code); 0",
  );
  wait_title(VIEW, "geo:1");
  assert!(has_event(
    |e| e.kind == EventKind::ViewPermission && e.a == GEOLOCATION
  ));
}

fn permission_prompt() {
  // -1 opens the built-in (non-blocking) prompt: the request stays pending, nothing crashes.
  let (_window, view) = setup(permission_reply(-1));
  load_base(&view, "<p>p</p>", Some(BASE));
  view.eval(NOTIFY);
  wait_for("the permission event", || {
    has_event(|e| e.kind == EventKind::ViewPermission)
  });
  pump(Duration::from_millis(200));
  assert!(!has_event(
    |e| e.kind == EventKind::ViewTitle && e.s1.starts_with("perm:")
  ));
  assert_eq!(eval(&view, "1"), "1");
}

// ---------------------------------------------------------------------------------------------
// downloads

const DOWNLOAD: &str = "var a = document.createElement('a'); \
                        a.href = 'data:text/plain,hello%20download'; a.download = 'x.txt'; \
                        document.body.appendChild(a); a.click(); 0";

fn download_accept() {
  let dir = temp_dir("dl");
  let dest = dir.join("saved.txt");
  let d = dest.to_string_lossy().into_owned();
  let (_window, view) = setup(move |e| match e.kind {
    EventKind::ViewDownload => Reply {
      code: 1,
      int: 0,
      string: Some(d.clone()),
    },
    _ => Reply::default(),
  });
  load(&view, "<body></body>");
  view.eval(DOWNLOAD);
  wait_for("the download to finish", || {
    has_event(|e| e.kind == EventKind::ViewDownloadDone)
  });
  let start = events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewDownload)
    .unwrap();
  assert_eq!((start.window, start.view), (WIN, VIEW));
  assert!(start.s1.starts_with("data:text/plain"), "{start:?}");
  assert!(start.s2.ends_with("x.txt"), "{start:?}");
  let done = events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewDownloadDone)
    .unwrap();
  assert_eq!((done.window, done.view, done.a), (WIN, VIEW, 1));
  assert_eq!(done.s2, dest.to_string_lossy());
  assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello download");
  let _ = std::fs::remove_dir_all(&dir);
}

fn download_cancel() {
  let (_window, view) = setup(|e| match e.kind {
    EventKind::ViewDownload => Reply::code(0),
    _ => Reply::default(),
  });
  load(&view, "<body></body>");
  view.eval(DOWNLOAD);
  wait_for("the download request", || {
    has_event(|e| e.kind == EventKind::ViewDownload)
  });
  pump(Duration::from_millis(200));
  assert!(!has_event(|e| e.kind == EventKind::ViewDownloadDone));
}

// ---------------------------------------------------------------------------------------------
// devtools

fn devtools() {
  let options = ViewOptions {
    devtools: true,
    ..Default::default()
  };
  let (_window, view) = setup_with(options, default_reply);
  load(&view, "<p>d</p>");
  assert!(!view.is_devtools_open());
  view.close_devtools(); // nothing to close
  view.open_devtools();
  assert!(view.is_devtools_open());
  view.open_devtools(); // already open: reuses it
  assert!(view.is_devtools_open());
  view.close_devtools();
  assert!(!view.is_devtools_open());
  view.open_devtools();
  assert!(view.is_devtools_open());
  pump(Duration::from_millis(100));
  // Dropped with the view.
}

fn devtools_disabled() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>d</p>");
  view.open_devtools();
  assert!(!view.is_devtools_open());
}

// ---------------------------------------------------------------------------------------------
// renderer crash

fn crash_default_reload() {
  let (_window, view) = setup(default_reply);
  view.load_url("data:text/html,<title>before</title>", None);
  wait_title(VIEW, "before");
  view.eval("window.marker = 1");
  clear_events();
  view.load_url("chrome://kill", None);
  wait_for("the crash event", || {
    has_event(|e| e.kind == EventKind::ViewCrashed)
  });
  let e = events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewCrashed)
    .unwrap();
  assert_eq!((e.window, e.view), (WIN, VIEW));
  assert_ne!(e.a, 0, "abnormal termination: {e:?}");
  // Reloaded by default: a fresh document.
  wait_loaded(VIEW);
  eval_until(&view, "typeof window.marker", "\"undefined\"");
}

fn crash_handled() {
  let (_window, view) = setup(|e| match e.kind {
    EventKind::ViewCrashed => Reply::code(1),
    _ => Reply::default(),
  });
  view.load_url("data:text/html,<title>before</title>", None);
  wait_title(VIEW, "before");
  clear_events();
  view.load_url("chrome://kill", None);
  wait_for("the crash event", || {
    has_event(|e| e.kind == EventKind::ViewCrashed)
  });
  pump(Duration::from_millis(300));
  assert!(
    !has_event(|e| e.kind == EventKind::ViewLoad && e.a == 0),
    "{:?}",
    events()
  );
  clear_events();
  view.load_url("data:text/html,<title>after</title>", None);
  wait_title(VIEW, "after");
  assert_eq!(eval(&view, "document.title"), "\"after\"");
}

// ---------------------------------------------------------------------------------------------
// geometry

fn bounds() {
  let (window, view) = setup(default_reply);
  view.set_bounds(10, 20, 300, 200);
  assert_eq!(view.bounds(), (10, 20, 300, 200));
  // set_bounds disabled fill: resizing the window keeps the bounds.
  window.resize(700, 500);
  pump(Duration::from_millis(50));
  assert_eq!(view.bounds(), (10, 20, 300, 200));
  load(&view, "<p>b</p>");
  eval_until(&view, "[innerWidth, innerHeight]", "[300,200]");
}

fn fill_follows_window() {
  let (window, view) = setup(default_reply);
  let (w, h) = window.content_size();
  assert_eq!(view.bounds(), (0, 0, w, h));
  window.resize(640, 480);
  let v = view.clone();
  wait_for("the view to follow", move || {
    let b = v.bounds();
    (b.2, b.3) == (640, 480)
  });
  view.set_bounds(0, 0, 10, 10);
  view.set_fill(true);
  let (w, h) = window.content_size();
  assert_eq!(view.bounds(), (0, 0, w, h));
  view.set_fill(false);
  window.resize(500, 400);
  pump(Duration::from_millis(50));
  assert_eq!(view.bounds(), (0, 0, w, h));
}

fn fill_option() {
  init();
  let window = Window::new(WIN, false);
  window.resize(420, 330);
  let options = ViewOptions {
    fill: true,
    ..Default::default()
  };
  let view = View::new(VIEW, &window, Profile::get(None, true), options);
  let (w, h) = window.content_size();
  assert_eq!(view.bounds(), (0, 0, w, h));
  let unfilled = View::new(
    VIEW + 1,
    &window,
    Profile::get(None, true),
    ViewOptions::default(),
  );
  assert_ne!(unfilled.bounds(), (0, 0, w, h));
}

fn visibility() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>v</p>");
  eval_until(&view, "document.visibilityState", "\"visible\"");
  view.set_visible(false);
  eval_until(&view, "document.visibilityState", "\"hidden\"");
  view.set_visible(true);
  eval_until(&view, "document.visibilityState", "\"visible\"");
}

fn focus() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>f</p>");
  view.focus();
  eval_until(&view, "document.hasFocus()", "true");
}

fn other_options() {
  let options = ViewOptions {
    focus: true,
    clipboard: true,
    drag_handler: true,
    ..Default::default()
  };
  let (_window, view) = setup_with(options, default_reply);
  load(&view, "<p>o</p>");
  eval_until(&view, "document.hasFocus()", "true");
}

fn qt_pointers() {
  let (_window, view) = setup(default_reply);
  let (v, p) = (view.qt_view(), view.qt_page());
  assert!(!v.is_null());
  assert!(!p.is_null());
  assert_ne!(v, p);
  assert_eq!(view.qt_view(), v);
}

fn multiple_views() {
  let (window, a) = setup(default_reply);
  let b = View::new(8, &window, Profile::get(None, true), ViewOptions::default());
  a.set_bounds(0, 0, 400, 600);
  b.set_bounds(400, 0, 400, 600);
  a.load_html("<title>left</title>", None);
  b.load_html("<title>right</title>", None);
  wait_title(VIEW, "left");
  wait_title(8, "right");
  assert_eq!(a.bounds(), (0, 0, 400, 600));
  assert_eq!(b.bounds(), (400, 0, 400, 600));
  assert_eq!(eval(&b, "document.title"), "\"right\"");
  assert!(!has_event(|e| e.kind == EventKind::ViewTitle
    && e.view == 8
    && e.s1 == "left"));
  // Dropping one leaves the other working.
  drop(b);
  assert_eq!(eval(&a, "document.title"), "\"left\"");
}

fn reparent() {
  let (first, view) = setup(default_reply);
  load(&view, "<title>moving</title>");
  let second = Window::new(2, false);
  second.resize(500, 300);
  second.set_visible(true);
  view.reparent(&second);
  drop(first);
  let (w, h) = second.content_size();
  assert_eq!(view.bounds(), (0, 0, w, h));
  view.eval("document.title = 'moved'");
  wait_title(VIEW, "moved");
  let e = events()
    .into_iter()
    .find(|e| e.kind == EventKind::ViewTitle && e.s1 == "moved")
    .unwrap();
  assert_eq!(e.window, 2);
  // Not inert: it belongs to the second window now.
  assert_eq!(eval(&view, "1"), "1");
}

// ---------------------------------------------------------------------------------------------
// teardown

fn inert_after_window_dropped() {
  let (window, view) = setup(default_reply);
  load(&view, "<title>x</title>");
  drop(window);
  pump(Duration::from_millis(50));
  clear_events();
  // Everything is a no-op now.
  let other = Window::new(2, false);
  view.reparent(&other);
  view.set_fill(true);
  view.set_bounds(1, 2, 3, 4);
  assert_eq!(view.bounds(), (0, 0, 0, 0));
  view.set_visible(true);
  view.focus();
  view.load_url("about:blank", None);
  view.load_html("<p>", None);
  view.reload();
  view.go_back();
  view.go_forward();
  assert!(!view.can_go_back());
  assert!(!view.can_go_forward());
  view.set_zoom(2.0);
  view.set_background_color((1, 2, 3, 4));
  view.add_init_script("1", true);
  view.eval("1");
  let called = Rc::new(Cell::new(false));
  let c = called.clone();
  view.eval_with_callback("1", move |_| c.set(true));
  view.open_devtools();
  assert!(!view.is_devtools_open());
  view.close_devtools();
  view.print();
  pump(Duration::from_millis(100));
  assert!(!called.get());
  assert!(!has_event(|e| e.view == VIEW), "{:?}", events());
  drop(view);
  drop(other);
}

fn url_after_window_dropped() {
  let (window, view) = setup(default_reply);
  load_base(&view, "<p>x</p>", Some(BASE));
  drop(window);
  pump(Duration::from_millis(100));
  // Expected: an empty string (or the last URL), not a read of the deleted page.
  let url = view.url();
  assert!(url.is_empty() || url == BASE, "{url:?}");
}

fn eval_inert_drops_callback() {
  let (window, view) = setup(default_reply);
  drop(window);
  let drops = Rc::new(Cell::new(0));
  let flag = DropFlag(drops.clone());
  view.eval_with_callback("1", move |_| {
    let _keep = &flag;
  });
  assert_eq!(drops.get(), 1, "the callback should be dropped right away");
}

thread_local! {
  static HELD: RefCell<Option<View>> = const { RefCell::new(None) };
}

/// Setup where the view lives in [`HELD`], so an event handler can drop it.
fn setup_held(reply: impl Fn(&Event<'_>) -> Reply + 'static) -> Window {
  let (window, view) = setup(reply);
  HELD.with(|h| *h.borrow_mut() = Some(Rc::into_inner(view).unwrap()));
  window
}

fn held_eval(js: &str) {
  HELD.with(|h| h.borrow().as_ref().unwrap().eval(js));
}

fn drop_held() {
  let v = HELD.with(|h| h.borrow_mut().take());
  drop(v);
}

fn is_held() -> bool {
  HELD.with(|h| h.borrow().is_some())
}

fn drop_in_title_handler() {
  let _window = setup_held(|e| {
    if e.kind == EventKind::ViewTitle && e.s1 == "drop me" {
      drop_held();
    }
    Reply::default()
  });
  HELD.with(|h| {
    h.borrow()
      .as_ref()
      .unwrap()
      .load_html("<title>drop me</title>", None)
  });
  wait_for("the view to be dropped", || !is_held());
  pump(Duration::from_millis(200));
}

fn drop_in_load_handler() {
  let _window = setup_held(|e| {
    if e.kind == EventKind::ViewLoad && e.a == 0 {
      drop_held();
    }
    Reply::default()
  });
  HELD.with(|h| h.borrow().as_ref().unwrap().load_html("<p>x</p>", None));
  wait_for("the view to be dropped", || !is_held());
  pump(Duration::from_millis(200));
}

fn drop_in_new_window_handler() {
  let _window = setup_held(|e| {
    if e.kind == EventKind::ViewNewWindow {
      drop_held();
      return Reply::code(1);
    }
    Reply::default()
  });
  HELD.with(|h| h.borrow().as_ref().unwrap().load_html("<p>n</p>", None));
  wait_loaded(VIEW);
  held_eval("window.open('about:blank')");
  wait_for("the view to be dropped", || !is_held());
  pump(Duration::from_millis(200));
}

fn drop_in_permission_handler() {
  let _window = setup_held(|e| {
    if e.kind == EventKind::ViewPermission {
      drop_held();
      return Reply::code(-1);
    }
    Reply::default()
  });
  HELD.with(|h| {
    h.borrow()
      .as_ref()
      .unwrap()
      .load_html("<p>p</p>", Some(BASE))
  });
  wait_loaded(VIEW);
  held_eval(NOTIFY);
  wait_for("the view to be dropped", || !is_held());
  pump(Duration::from_millis(200));
}

fn drop_while_loading() {
  let (window, view) = setup(default_reply);
  view.load_html("<title>never</title>", None);
  drop(Rc::into_inner(view).unwrap());
  pump(Duration::from_millis(200));
  // The window still works for a new view.
  let view = View::new(8, &window, Profile::get(None, true), ViewOptions::default());
  view.load_html("<title>next</title>", None);
  wait_title(8, "next");
}

fn survives_shutdown() {
  let (window, view) = setup(default_reply);
  load(&view, "<p>s</p>");
  app::shutdown();
  // Both drops are no-ops after shutdown.
  drop(view);
  drop(window);
}

fn use_after_shutdown() {
  let (_window, view) = setup(default_reply);
  app::shutdown();
  view.url();
}

fn print() {
  let (_window, view) = setup(default_reply);
  load(&view, "<p>p</p>");
  view.print();
}
