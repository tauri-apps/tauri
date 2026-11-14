// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Custom URL schemes: registration, the scheme handler, `Responder` (any thread), ranges, and
//! the engine-level functions (`engine_version`, Chromium flags, proxy).
//!
//! Most cases register `app://`, serve a small index page at `app://<any host>/` and drive
//! `fetch()` from that page (see [`start`], [`open`] and [`js`]).

use std::{
  cell::RefCell,
  io::{Read, Write},
  net::TcpListener,
  rc::Rc,
  sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
  },
  thread,
  time::Duration,
};

use http::{Response, header};
use qtshim::test_harness::Case;
use qtshim::{
  app,
  webengine::{
    self, Profile, ProxyKind, Responder, SchemeFlags, SchemeRequest, View, ViewOptions, Window,
  },
};

use crate::common::{init, pump, run_until, wait_for};

pub fn cases() -> Vec<Case> {
  vec![
    // engine
    Case::pass(
      "scheme: register before the engine starts, refused after",
      register_before_and_after_start,
    ),
    Case::pass(
      "scheme: a scheme registered after start is not served",
      late_scheme_not_served,
    ),
    Case::pass(
      "scheme: engine_version names Qt and Chromium",
      engine_version_format,
    ),
    Case::pass(
      "scheme: chromium flags and proxy reach the engine at start",
      chromium_flags_and_proxy,
    ),
    Case::pass(
      "scheme: chromium flags after start are ignored",
      chromium_flags_after_start,
    ),
    Case::panics(
      "scheme: register_scheme before app::init panics",
      "Qt can only be used",
      || {
        webengine::register_scheme("x", SchemeFlags::APP);
      },
    ),
    Case::panics(
      "scheme: engine_started off the main thread panics",
      "Qt can only be used",
      || {
        init();
        let r = thread::spawn(webengine::engine_started).join();
        std::panic::resume_unwind(r.unwrap_err());
      },
    ),
    // flags
    Case::pass(
      "scheme: a scheme without fetch_api can't be fetched",
      flags_no_fetch,
    ),
    Case::pass("scheme: cross-origin fetch needs the cors flag", flags_cors),
    Case::pass(
      "scheme: the secure flag makes a secure context",
      flags_secure,
    ),
    Case::pass(
      "scheme: csp_bypass ignores the page's CSP",
      flags_csp_bypass,
    ),
    // requests
    Case::pass(
      "scheme: a navigation is a GET without Origin",
      navigation_request,
    ),
    Case::pass(
      "scheme: handler sees method, url, headers and body of a POST",
      post_request,
    ),
    Case::pass(
      "scheme: binary request bodies arrive intact",
      binary_request_body,
    ),
    Case::pass(
      "scheme: request bodies of every size arrive intact (non-periodic bytes)",
      request_body_sizes,
    ),
    Case::pass(
      "scheme: each request carries the id of its view",
      view_routing,
    ),
    Case::pass(
      "scheme: X-Qs-View is stripped and can't be spoofed",
      view_header_hidden,
    ),
    Case::pass(
      "scheme: Origin is rebuilt from the initiator",
      origin_rebuilt,
    ),
    Case::pass(
      "scheme: URLs with unusual characters reach the handler",
      unusual_url_characters,
    ),
    Case::pass(
      "scheme: X-Qs-View is not sent to http servers",
      view_header_not_leaked_to_http,
    ),
    Case::pass("scheme: without a handler requests fail", no_handler_fails),
    Case::pass(
      "scheme: the latest set_scheme_handler wins",
      handler_replaced,
    ),
    // responses
    Case::pass("scheme: 200 with body and content type", response_ok),
    Case::pass(
      "scheme: content type defaults to text/plain",
      response_default_content_type,
    ),
    Case::pass(
      "scheme: custom response headers are visible to fetch",
      response_custom_headers,
    ),
    Case::pass(
      "scheme: CSP hashes of HTML responses keep their case",
      csp_hash_case,
    ),
    // QT BUG: QWebEngineUrlRequestJob::setAdditionalResponseHeaders (Qt 6.11) lowercases header
    // values. Only Content-Security-Policy of HTML documents is worked around (as a <meta> tag).
    Case::pass(
      "scheme: response header values keep their case",
      response_header_case,
    )
    .ignored(),
    Case::pass(
      "scheme: repeated response headers keep their order",
      response_repeated_headers_in_order,
    ),
    Case::pass(
      "scheme: fetch rejects when the request fails",
      fetch_rejects_failed_requests,
    ),
    Case::pass(
      "scheme: a wrong Content-Length is ignored",
      response_content_length_ignored,
    ),
    Case::pass(
      "scheme: error statuses with an empty body fail the request",
      response_errors_fail,
    ),
    Case::pass(
      "scheme: error statuses with a body are served",
      response_error_with_body,
    ),
    Case::pass("scheme: 3xx with Location redirects", response_redirect),
    Case::pass(
      "scheme: 3xx without Location is served as a body",
      response_redirect_without_location,
    ),
    Case::pass(
      "scheme: 206 with Content-Range serves the range",
      response_range,
    ),
    Case::pass(
      "scheme: ranges of a full 200 body are cut by Qt",
      range_of_full_body,
    ),
    Case::pass(
      "scheme: 206 without Content-Range is served whole",
      response_range_without_content_range,
    ),
    Case::pass(
      "scheme: <audio> plays through range requests",
      media_range_requests,
    ),
    Case::pass("scheme: an 8 MB body arrives intact", response_large),
    Case::pass("scheme: an empty 200 body", response_empty),
    Case::pass("scheme: every byte value survives", response_binary),
    Case::pass(
      "scheme: several requests in flight answer out of order",
      out_of_order,
    ),
    // Responder
    Case::pass("scheme: respond from another thread", respond_other_thread),
    Case::pass("scheme: respond after a delay", respond_delayed),
    Case::pass(
      "scheme: a dropped Responder fails the request",
      responder_dropped,
    ),
    Case::pass(
      "scheme: a dropped Responder on another thread fails the request",
      responder_dropped_other_thread,
    ),
    Case::pass(
      "scheme: a panicking handler fails the request, the app keeps going",
      handler_panics,
    ),
    Case::pass(
      "scheme: respond after the view was dropped",
      respond_after_view_dropped,
    ),
    Case::pass(
      "scheme: respond after the page navigated away",
      respond_after_navigation,
    ),
    Case::pass(
      "scheme: respond and drop after shutdown",
      respond_after_shutdown,
    ),
    Case::pass(
      "scheme: respond queued from a thread, then shutdown",
      respond_queued_then_shutdown,
    ),
    Case::pass(
      "scheme: respond from a thread racing shutdown",
      respond_racing_shutdown,
    ),
  ]
}

// ---------------------------------------------------------------------------------------------
// helpers (also used by `profile.rs`)

/// A request as the handler saw it.
#[derive(Debug, Clone)]
pub struct Seen {
  pub view: u64,
  pub method: String,
  pub uri: String,
  pub headers: Vec<(String, String)>,
  pub body: Vec<u8>,
}

impl Seen {
  /// The first value of header `name` (case-insensitive).
  pub fn header(&self, name: &str) -> Option<&str> {
    self
      .headers
      .iter()
      .find(|(k, _)| k.eq_ignore_ascii_case(name))
      .map(|(_, v)| v.as_str())
  }
  fn path(&self) -> &str {
    let rest = self.uri.split_once("://").map_or("", |(_, r)| r);
    rest.find('/').map_or("/", |i| &rest[i..])
  }
}

thread_local! {
  static SEEN: RefCell<Vec<Seen>> = const { RefCell::new(Vec::new()) };
}

/// Every request the handler installed by [`start`] / [`install`] received.
pub fn seen() -> Vec<Seen> {
  SEEN.with(|s| s.borrow().clone())
}

/// The requests whose path (with query) starts with `path`.
pub fn seen_path(path: &str) -> Vec<Seen> {
  seen()
    .into_iter()
    .filter(|s| s.path().starts_with(path))
    .collect()
}

/// `init`, register `app://` with [`SchemeFlags::APP`] and [`install`] `handler`.
pub fn start(handler: impl Fn(SchemeRequest, Responder) + 'static) {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  install(handler);
}

/// Install a scheme handler that records each request, serves [`index`] for `GET /` and passes
/// everything else to `handler`.
pub fn install(handler: impl Fn(SchemeRequest, Responder) + 'static) {
  webengine::set_scheme_handler(move |req, res| {
    let r = &req.request;
    let seen = Seen {
      view: req.view,
      method: r.method().to_string(),
      uri: r.uri().to_string(),
      headers: r
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_owned()))
        .collect(),
      body: r.body().clone(),
    };
    SEEN.with(|s| s.borrow_mut().push(seen));
    if r.method() == http::Method::GET && r.uri().path() == "/" {
      res.respond(index());
    } else {
      handler(req, res);
    }
  });
}

/// The page served at `/`.
pub fn index() -> Response<Vec<u8>> {
  resp(
    200,
    &[("Content-Type", "text/html")],
    "<!doctype html><title>index</title><p>index</p>",
  )
}

/// A response with `status`, `headers` and `body`. Cross-origin reads are allowed.
pub fn resp(status: u16, headers: &[(&str, &str)], body: impl Into<Vec<u8>>) -> Response<Vec<u8>> {
  let mut b = Response::builder()
    .status(status)
    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
  for (k, v) in headers {
    b = b.header(*k, *v);
  }
  b.body(body.into()).unwrap()
}

/// A plain-text `200`.
pub fn text(body: &str) -> Response<Vec<u8>> {
  resp(200, &[("Content-Type", "text/plain")], body)
}

/// A window with one view per id, all showing `url` (loaded).
pub struct Page {
  // Declared before the window so the views drop first.
  pub views: Vec<Rc<View>>,
  _window: Window,
}

impl Page {
  /// The first view.
  pub fn view(&self) -> &Rc<View> {
    &self.views[0]
  }
}

/// Open a window with a view per id in `ids` on `profile` and load `url` in each.
pub fn open(profile: Profile, ids: &[u64], url: &str) -> Page {
  let window = Window::new(ids[0], false);
  window.resize(400, 300);
  let views: Vec<_> = ids
    .iter()
    .map(|&id| Rc::new(View::new(id, &window, profile, ViewOptions::default())))
    .collect();
  for (i, v) in views.iter().enumerate() {
    v.set_bounds(0, i as i32 * 50, 400, 50);
  }
  window.set_visible(true);
  for v in &views {
    load(v, url);
  }
  Page {
    views,
    _window: window,
  }
}

/// Load `url` in `view` and wait until it finished loading.
pub fn load(view: &Rc<View>, url: &str) {
  view.load_url(url, None);
  wait_loaded(view, url);
}

/// Wait until `view` shows `url` and the document is complete.
pub fn wait_loaded(view: &Rc<View>, url: &str) {
  wait_true(
    view,
    &format!("document.readyState === 'complete' && location.href === {url:?}"),
    &format!("{url} to load"),
  );
}

/// Wait until the JS expression `cond` evaluates to `true` in `view`.
pub fn wait_true(view: &Rc<View>, cond: &str, what: &str) {
  let (v, cond) = (view.clone(), cond.to_owned());
  let ok = Rc::new(RefCell::new(false));
  let o = ok.clone();
  wait_for(what, move || {
    let o2 = o.clone();
    v.eval_with_callback(&cond, move |s| {
      if s == "true" {
        *o2.borrow_mut() = true;
      }
    });
    *o.borrow()
  });
}

/// Evaluate `js` and wait for its (JSON-encoded) result.
pub fn eval(view: &Rc<View>, js: &str) -> String {
  let out = Rc::new(RefCell::new(None::<String>));
  let o = out.clone();
  view.eval_with_callback(js, move |s| *o.borrow_mut() = Some(s.to_owned()));
  let o = out.clone();
  wait_for(&format!("the result of {js:?}"), move || {
    o.borrow().is_some()
  });
  out.take().unwrap()
}

static JS_ID: AtomicU64 = AtomicU64::new(0);

/// Run `body` as the body of an async JS function in `view` and wait for it to settle.
/// Returns `"ok:<value>"` or `"err:<error>"` (values are stringified).
pub fn js(view: &Rc<View>, body: &str) -> String {
  js_timeout(view, body, crate::common::WAIT)
}

/// [`js`] with a custom timeout.
pub fn js_timeout(view: &Rc<View>, body: &str, timeout: Duration) -> String {
  let n = JS_ID.fetch_add(1, Ordering::Relaxed);
  view.eval(&format!(
    "window.__qs = window.__qs || {{}}; (async () => {{ {body} }})().then(\
     v => window.__qs[{n}] = 'ok:' + v, e => window.__qs[{n}] = 'err:' + e);"
  ));
  let out = Rc::new(RefCell::new(None::<String>));
  let (o, v) = (out.clone(), view.clone());
  let poll = format!("window.__qs && window.__qs[{n}] !== undefined ? window.__qs[{n}] : null");
  let done = run_until(timeout, move || {
    let o2 = o.clone();
    v.eval_with_callback(&poll, move |s| {
      if s != "null" {
        *o2.borrow_mut() = Some(json_string(s));
      }
    });
    o.borrow().is_some()
  });
  assert!(done, "timed out running {body:?}");
  out.take().unwrap()
}

/// Run `body` as in [`js`] without waiting.
pub fn js_nowait(view: &Rc<View>, body: &str) {
  view.eval(&format!("(async () => {{ {body} }})()"));
}

/// Decode a JSON string literal (as returned by `eval_with_callback`).
pub fn json_string(s: &str) -> String {
  let Some(inner) = s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) else {
    return s.to_owned();
  };
  let mut out = String::new();
  let mut chars = inner.chars();
  while let Some(c) = chars.next() {
    if c != '\\' {
      out.push(c);
      continue;
    }
    match chars.next() {
      Some('n') => out.push('\n'),
      Some('t') => out.push('\t'),
      Some('r') => out.push('\r'),
      Some('b') => out.push('\u{8}'),
      Some('f') => out.push('\u{c}'),
      Some('u') => {
        let hex: String = chars.by_ref().take(4).collect();
        out.push(char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap_or('\u{fffd}'));
      }
      Some(c) => out.push(c),
      None => {}
    }
  }
  out
}

/// The default profile with `app://host/` open in one view (id 1).
fn app_page() -> Page {
  open(Profile::get(None, false), &[1], "app://host/")
}

/// `fetch(url)` from `view`; returns `"ok:<status> <text>"` or `"err:..."`.
fn fetch(view: &Rc<View>, url: &str) -> String {
  js(
    view,
    &format!("const r = await fetch({url:?}); return r.status + ' ' + await r.text();"),
  )
}

/// An `XMLHttpRequest` GET of `url` from `view`; returns `"ok:load <status> <text>"` or
/// `"ok:error"`. (Unlike `fetch`, XHR reports failed scheme requests: see
/// `fetch_rejects_failed_requests`.)
fn xhr(view: &Rc<View>, url: &str) -> String {
  js(
    view,
    &format!(
      "return await new Promise(ok => {{ const x = new XMLHttpRequest(); x.open('GET', {url:?}); \
       x.onload = () => ok('load ' + x.status + ' ' + x.responseText); \
       x.onerror = () => ok('error'); x.send(); }});"
    ),
  )
}

thread_local! {
  static STASH: RefCell<Vec<Responder>> = const { RefCell::new(Vec::new()) };
}

/// Keep `res` unanswered for later ([`stashed`]).
fn stash(res: Responder) {
  STASH.with(|s| s.borrow_mut().push(res));
}

/// Wait for `n` stashed responders and take them.
fn stashed(n: usize) -> Vec<Responder> {
  wait_for(&format!("{n} stashed requests"), move || {
    STASH.with(|s| s.borrow().len() >= n)
  });
  STASH.with(|s| s.borrow_mut().drain(..).collect())
}

// ---------------------------------------------------------------------------------------------
// engine

fn register_before_and_after_start() {
  init();
  assert!(!webengine::engine_started());
  assert!(webengine::register_scheme("early", SchemeFlags::APP));
  assert!(
    webengine::register_scheme("early", SchemeFlags::default()),
    "registering twice is a no-op success"
  );
  assert!(webengine::register_scheme("other", SchemeFlags::default()));
  assert!(!webengine::engine_started());
  let _ = Profile::get(None, false);
  assert!(webengine::engine_started());
  assert!(!webengine::register_scheme("late", SchemeFlags::APP));
  assert!(
    webengine::register_scheme("early", SchemeFlags::APP),
    "an already registered scheme still reports success"
  );
}

fn late_scheme_not_served() {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  install(|_, res| res.respond(text("late")));
  let page = app_page();
  assert!(!webengine::register_scheme("late", SchemeFlags::APP));
  let r = fetch(page.view(), "late://host/x");
  assert!(r.starts_with("err:"), "{r}");
  assert!(seen_path("/x").is_empty());
}

fn engine_version_format() {
  init();
  let v = webengine::engine_version();
  assert!(v.starts_with("6."), "{v}");
  assert!(v.contains(" (Chromium "), "{v}");
  assert!(v.ends_with(')'), "{v}");
  // Doesn't start the engine, and is stable.
  assert!(!webengine::engine_started());
  assert_eq!(v, webengine::engine_version());
}

fn chromium_flags_and_proxy() {
  // SAFETY: single-threaded at this point (the child process runs only this case).
  unsafe { std::env::remove_var("QTWEBENGINE_CHROMIUM_FLAGS") };
  start(|_, res| res.respond(text("x")));
  webengine::add_chromium_flags("--js-flags=--expose-gc");
  webengine::add_chromium_flags("");
  webengine::set_proxy(ProxyKind::Socks5, "127.0.0.1", 9);
  webengine::set_proxy(ProxyKind::Http, "127.0.0.2", 8080);
  let page = app_page();
  let flags = std::env::var("QTWEBENGINE_CHROMIUM_FLAGS").unwrap();
  assert!(flags.contains("--js-flags=--expose-gc"), "{flags}");
  assert!(
    flags.contains("--proxy-server=socks5://127.0.0.1:9"),
    "{flags}"
  );
  assert!(
    flags.contains("--proxy-server=http://127.0.0.2:8080"),
    "{flags}"
  );
  // The flag took effect in the renderer.
  assert_eq!(eval(page.view(), "typeof gc"), "\"function\"");
}

fn chromium_flags_after_start() {
  unsafe { std::env::remove_var("QTWEBENGINE_CHROMIUM_FLAGS") };
  init();
  let _ = Profile::get(None, false);
  webengine::add_chromium_flags("--js-flags=--expose-gc");
  webengine::set_proxy(ProxyKind::Http, "127.0.0.1", 9);
  let flags = std::env::var("QTWEBENGINE_CHROMIUM_FLAGS").unwrap_or_default();
  assert!(!flags.contains("expose-gc"), "{flags}");
  assert!(!flags.contains("proxy-server"), "{flags}");
}

// ---------------------------------------------------------------------------------------------
// flags

fn flags_no_fetch() {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  assert!(webengine::register_scheme(
    "nofetch",
    SchemeFlags::default()
  ));
  install(|_, res| res.respond(text("served")));
  let page = app_page();
  let r = fetch(page.view(), "nofetch://host/x");
  assert!(r.starts_with("err:"), "{r}");
  assert!(seen_path("/x").is_empty());
}

fn flags_cors() {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  let fetch_only = SchemeFlags {
    fetch_api: true,
    ..Default::default()
  };
  assert!(webengine::register_scheme("nocors", fetch_only));
  install(|_, res| res.respond(text("served")));
  let page = app_page();
  // Cross-origin (another host) on a CORS-enabled scheme works...
  assert_eq!(fetch(page.view(), "app://other/x"), "ok:200 served");
  // ...but not on one without the flag.
  let r = fetch(page.view(), "nocors://host/x");
  assert!(r.starts_with("err:"), "{r}");
}

fn flags_secure() {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  let insecure = SchemeFlags {
    secure: false,
    ..SchemeFlags::APP
  };
  assert!(webengine::register_scheme("insecure", insecure));
  install(|_, res| res.respond(index()));
  let page = app_page();
  assert_eq!(eval(page.view(), "isSecureContext"), "true");
  load(page.view(), "insecure://host/");
  assert_eq!(eval(page.view(), "isSecureContext"), "false");
}

fn flags_csp_bypass() {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  let bypass = SchemeFlags {
    csp_bypass: true,
    ..SchemeFlags::APP
  };
  assert!(webengine::register_scheme("bypass", bypass));
  assert!(webengine::register_scheme("strict", SchemeFlags::APP));
  install(|req, res| {
    if req.request.uri().path() == "/csp" {
      res.respond(resp(
        200,
        &[("Content-Type", "text/html")],
        "<meta http-equiv=\"Content-Security-Policy\" content=\"connect-src 'none'\"><p>csp",
      ));
    } else {
      res.respond(text("served"));
    }
  });
  let page = app_page();
  load(page.view(), "app://host/csp");
  assert_eq!(fetch(page.view(), "bypass://host/x"), "ok:200 served");
  let r = fetch(page.view(), "strict://host/x");
  assert!(r.starts_with("err:"), "{r}");
}

// ---------------------------------------------------------------------------------------------
// requests

fn navigation_request() {
  start(|_, res| drop(res));
  let _page = app_page();
  let nav = &seen_path("/")[0];
  assert_eq!(nav.method, "GET");
  assert_eq!(nav.uri, "app://host/");
  assert_eq!(nav.view, 1);
  assert_eq!(nav.header("origin"), None);
  assert!(nav.body.is_empty());
  // Chromium adds User-Agent below the scheme handler layer.
  assert!(nav.header("accept").is_some(), "{nav:?}");
}

fn post_request() {
  start(|req, res| {
    let body = req.request.body().clone();
    res.respond(resp(200, &[], body));
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const r = await fetch('app://host/echo?x=1&y=%20z', { method: 'POST', \
     headers: { 'X-Test': 'yes', 'Content-Type': 'application/json' }, body: '{\"a\":1}' }); \
     return await r.text();",
  );
  assert_eq!(r, "ok:{\"a\":1}");
  let s = &seen_path("/echo")[0];
  assert_eq!(s.method, "POST");
  assert_eq!(s.uri, "app://host/echo?x=1&y=%20z");
  assert_eq!(s.header("x-test"), Some("yes"));
  assert_eq!(s.header("content-type"), Some("application/json"));
  assert_eq!(s.body, b"{\"a\":1}");
  assert_eq!(s.view, 1);

  // Other methods.
  for m in ["PUT", "DELETE", "PATCH"] {
    let r = js(
      page.view(),
      &format!("const r = await fetch('app://host/m', {{ method: '{m}' }}); return r.status;"),
    );
    assert_eq!(r, "ok:200");
    assert_eq!(seen_path("/m").last().unwrap().method, m);
  }
}

fn binary_request_body() {
  start(|req, res| {
    let n = req.request.body().len().to_string();
    res.respond(text(&n));
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const b = new Uint8Array(256 * 1024); for (let i = 0; i < b.length; i++) b[i] = i % 256; \
     const r = await fetch('app://host/bin', { method: 'POST', body: b }); return await r.text();",
  );
  assert_eq!(r, format!("ok:{}", 256 * 1024));
  let body = &seen_path("/bin")[0].body;
  assert!(body.iter().enumerate().all(|(i, &b)| b == (i % 256) as u8));
}

/// The byte at `i` of a non-periodic test pattern (so shifted or repeated chunks show up).
fn pattern_byte(i: usize) -> u8 {
  (i.wrapping_mul(2_654_435_761) >> 24) as u8
}

fn request_body_sizes() {
  start(|req, res| {
    let n = req.request.body().len().to_string();
    res.respond(text(&n));
  });
  let page = app_page();
  for n in [
    16 * 1024 - 1,
    16 * 1024,
    16 * 1024 + 1,
    20_000,
    64 * 1024,
    200_000,
    1 << 20,
    4 << 20,
  ] {
    let r = js(
      page.view(),
      &format!(
        "const b = new Uint8Array({n}); \
         for (let i = 0; i < b.length; i++) b[i] = (Math.imul(i, 2654435761) >>> 24) & 255; \
         const r = await fetch('app://host/sizes/{n}', {{ method: 'POST', body: b }}); \
         return await r.text();"
      ),
    );
    assert_eq!(r, format!("ok:{n}"), "{n}-byte body");
    let body = &seen_path(&format!("/sizes/{n}"))[0].body;
    let bad = body
      .iter()
      .enumerate()
      .position(|(i, &b)| b != pattern_byte(i));
    assert_eq!(bad, None, "{n}-byte body differs from byte {bad:?}");
  }
}

fn csp_hash_case() {
  // sha256("document.title='hashed'"); Qt would lowercase it in a header.
  const CSP: &str =
    "default-src 'self'; script-src 'sha256-8Q56ftgtlXxkQr5RiA6Pw+WlboQEeqNCt3JbdRNi/+I='";
  start(|_, res| {
    res.respond(resp(
      200,
      &[
        ("Content-Type", "text/html; charset=utf-8"),
        ("Content-Security-Policy", CSP),
      ],
      "<!doctype html><html><head><title>t</title></head><body>\
       <script>document.title='hashed'</script>\
       <script>document.title='not allowed'</script></body></html>",
    ))
  });
  let page = open(Profile::get(None, false), &[1], "app://host/csp");
  wait_true(
    page.view(),
    "document.title === 'hashed'",
    "the hashed script to run",
  );
  // The policy is enforced: the second (unhashed) script didn't run.
  assert_eq!(eval(page.view(), "document.title"), "\"hashed\"");
  assert_eq!(
    eval(
      page.view(),
      "document.querySelector('meta[http-equiv=\"Content-Security-Policy\"]').content"
    ),
    format!("{CSP:?}")
  );
}

fn response_header_case() {
  // Case-sensitive values, like CSP hashes ('sha256-<base64>').
  const CSP: &str =
    "default-src 'self'; script-src 'sha256-EOd6N98xxmK5s7VvxV7W2w7YG+dmP52MqNiZUq1NLeE='";
  start(|_, res| {
    res.respond(resp(
      200,
      &[
        ("Content-Type", "text/plain"),
        ("X-Mixed", "AbC-xYz"),
        (
          "Access-Control-Expose-Headers",
          "X-Mixed, Content-Security-Policy",
        ),
        ("Content-Security-Policy", CSP),
      ],
      "ok",
    ))
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const r = await fetch('app://host/case'); \
     return r.headers.get('x-mixed') + '|' + r.headers.get('content-security-policy');",
  );
  assert_eq!(r, format!("ok:AbC-xYz|{CSP}"));
}

fn view_routing() {
  start(|req, res| res.respond(text(&req.view.to_string())));
  let page = open(Profile::get(None, false), &[7, 9], "app://host/");
  let mut navs: Vec<_> = seen_path("/").iter().map(|s| s.view).collect();
  navs.sort_unstable();
  assert_eq!(navs, [7, 9]);
  for _ in 0..3 {
    assert_eq!(fetch(&page.views[0], "app://host/who"), "ok:200 7");
    assert_eq!(fetch(&page.views[1], "app://host/who"), "ok:200 9");
  }
  // A view in another window, on an incognito profile.
  let other = open(Profile::get(None, true), &[42], "app://host/");
  assert_eq!(fetch(other.view(), "app://host/who"), "ok:200 42");
}

fn view_header_hidden() {
  start(|req, res| res.respond(text(&req.view.to_string())));
  let page = app_page();
  let r = js(
    page.view(),
    "const r = await fetch('app://host/who', { headers: { 'X-Qs-View': '999' } }); \
     return await r.text();",
  );
  assert_eq!(r, "ok:1", "a page can't pose as another view");
  for s in seen() {
    assert_eq!(s.header("x-qs-view"), None, "{s:?}");
  }
}

fn origin_rebuilt() {
  start(|req, res| {
    let o = req
      .request
      .headers()
      .get(header::ORIGIN)
      .map_or("none", |v| v.to_str().unwrap())
      .to_owned();
    res.respond(text(&o));
  });
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/o"), "ok:200 app://host");
  // POST (where Chromium itself would send Origin) carries it once.
  let r = js(
    page.view(),
    "const r = await fetch('app://host/o', { method: 'POST', body: 'x' }); return await r.text();",
  );
  assert_eq!(r, "ok:app://host");
  let posted = seen_path("/o")
    .into_iter()
    .find(|s| s.method == "POST")
    .unwrap();
  assert_eq!(
    posted
      .headers
      .iter()
      .filter(|(k, _)| k.eq_ignore_ascii_case("origin"))
      .count(),
    1
  );
  // Cross-origin.
  load(page.view(), "app://other.example/");
  assert_eq!(
    fetch(page.view(), "app://host/o"),
    "ok:200 app://other.example"
  );
}

fn unusual_url_characters() {
  start(|req, res| res.respond(text(&req.request.uri().to_string())));
  let page = app_page();
  for (url, path) in [
    ("app://host/a%20b", "/a%20b"),
    ("app://host/caf%C3%A9?q=%E2%9C%93", "/caf%C3%A9?q=%E2%9C%93"),
    ("app://host/x?a={b}", "/x?a="),
    ("app://host/p|q^r", "/p"),
  ] {
    let r = fetch(page.view(), url);
    assert!(r.starts_with("ok:200 app://host"), "{url}: {r}");
    assert!(
      seen().iter().any(|s| s.path().starts_with(path)),
      "{url} not seen"
    );
  }
}

/// Serve `http://127.0.0.1:<port>/` once per connection, recording the raw requests.
pub fn http_server() -> (u16, Arc<Mutex<Vec<String>>>) {
  let listener = TcpListener::bind("127.0.0.1:0").unwrap();
  let port = listener.local_addr().unwrap().port();
  let log = Arc::new(Mutex::new(Vec::new()));
  let l = log.clone();
  thread::spawn(move || {
    for stream in listener.incoming() {
      let Ok(mut stream) = stream else { break };
      let mut buf = [0u8; 8192];
      let n = stream.read(&mut buf).unwrap_or(0);
      l.lock()
        .unwrap()
        .push(String::from_utf8_lossy(&buf[..n]).into_owned());
      let body = "<!doctype html><title>http</title>";
      let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
      );
    }
  });
  (port, log)
}

fn view_header_not_leaked_to_http() {
  start(|_, res| drop(res));
  let (port, log) = http_server();
  let url = format!("http://127.0.0.1:{port}/");
  let _page = open(Profile::get(None, false), &[1], &url);
  let requests = log.lock().unwrap().clone();
  assert!(!requests.is_empty());
  for r in requests {
    assert!(!r.to_ascii_lowercase().contains("x-qs-view"), "{r}");
  }
}

fn no_handler_fails() {
  init();
  assert!(webengine::register_scheme("app", SchemeFlags::APP));
  let window = Window::new(1, false);
  let view = Rc::new(View::new(
    1,
    &window,
    Profile::get(None, false),
    ViewOptions::default(),
  ));
  window.set_visible(true);
  view.load_html("<title>blank</title>", Some("app://host/"));
  wait_true(&view, "document.title === 'blank'", "the html page");
  assert_eq!(xhr(&view, "app://host/x"), "ok:error");
}

fn handler_replaced() {
  start(|_, res| res.respond(text("first")));
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 first");
  install(|_, res| res.respond(text("second")));
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 second");
}

// ---------------------------------------------------------------------------------------------
// responses

fn response_ok() {
  start(|_, res| {
    res.respond(resp(
      200,
      &[("Content-Type", "application/json")],
      "{\"k\":1}",
    ))
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const r = await fetch('app://host/data.json'); \
     return [r.status, r.ok, r.headers.get('content-type'), JSON.stringify(await r.json())].join('|');",
  );
  assert_eq!(r, "ok:200|true|application/json|{\"k\":1}");
}

fn response_default_content_type() {
  start(|_, res| {
    res.respond(
      Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .body(b"plain".to_vec())
        .unwrap(),
    )
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const r = await fetch('app://host/x'); return r.headers.get('content-type') + '|' + await r.text();",
  );
  assert!(r.starts_with("ok:text/plain"), "{r}");
  assert!(r.ends_with("|plain"), "{r}");
}

fn response_custom_headers() {
  start(|_, res| {
    res.respond(resp(
      200,
      &[
        ("Content-Type", "text/plain"),
        ("X-Custom", "abc"),
        ("X-Other", "x"),
        ("Access-Control-Expose-Headers", "X-Custom, X-Other"),
      ],
      "body",
    ))
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const r = await fetch('app://host/h'); \
     return r.headers.get('x-custom') + '|' + r.headers.get('x-other') + '|' + await r.text();",
  );
  assert_eq!(r, "ok:abc|x|body");
}

fn response_repeated_headers_in_order() {
  start(|_, res| {
    res.respond(resp(
      200,
      &[
        ("X-Multi", "one"),
        ("X-Multi", "two"),
        ("X-Multi", "three"),
        ("Access-Control-Expose-Headers", "X-Multi"),
      ],
      "body",
    ))
  });
  let page = app_page();
  let r = js(
    page.view(),
    "return (await fetch('app://host/h')).headers.get('x-multi');",
  );
  assert_eq!(r, "ok:one, two, three");
}

fn response_content_length_ignored() {
  start(|_, res| {
    res.respond(resp(
      200,
      &[("Content-Type", "text/plain"), ("Content-Length", "3")],
      "0123456789",
    ))
  });
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 0123456789");
}

fn response_errors_fail() {
  start(|req, res| {
    let status: u16 = req.request.uri().path()[1..].parse().unwrap();
    res.respond(resp(status, &[], Vec::new()));
  });
  let page = app_page();
  for status in [400, 403, 404, 410, 500, 503] {
    let r = xhr(page.view(), &format!("app://host/{status}"));
    assert_eq!(r, "ok:error", "{status}");
  }
  // A failed navigation shows Chromium's error page.
  page.view().load_url("app://host/404", None);
  wait_true(
    page.view(),
    "location.href.startsWith('chrome-error:')",
    "the error page",
  );
  assert_eq!(page.view().url(), "app://host/404");
}

fn fetch_rejects_failed_requests() {
  start(|req, res| match req.request.uri().path() {
    "/drop" => drop(res),
    "/panic" => panic!("boom"),
    _ => res.respond(resp(404, &[], Vec::new())),
  });
  let page = app_page();
  for url in ["app://host/404", "app://host/drop", "app://host/panic"] {
    // Expected: fetch rejects (or at least the body read does).
    let r = js_timeout(
      page.view(),
      &format!(
        "const r = await fetch({url:?}, {{ signal: AbortSignal.timeout(5000) }}); \
         return r.status + ' ' + await r.text();"
      ),
      Duration::from_secs(15),
    );
    assert!(r.starts_with("err:TypeError"), "{url}: {r}");
  }
}

fn response_error_with_body() {
  start(|_, res| res.respond(resp(404, &[("Content-Type", "text/plain")], "not here")));
  let page = app_page();
  // Qt can't send status codes: the body arrives as a 200.
  assert_eq!(fetch(page.view(), "app://host/missing"), "ok:200 not here");
}

fn response_redirect() {
  start(|req, res| match req.request.uri().path() {
    "/old" => res.respond(resp(302, &[("Location", "/new?from=old")], Vec::new())),
    "/abs" => res.respond(resp(301, &[("Location", "app://other/new")], Vec::new())),
    "/chain" => res.respond(resp(307, &[("Location", "old")], Vec::new())),
    _ => res.respond(text("new body")),
  });
  let page = app_page();
  let follow = |url: &str| {
    js(
      page.view(),
      &format!(
        "const r = await fetch({url:?}); return [r.redirected, r.url, await r.text()].join('|');"
      ),
    )
  };
  assert_eq!(
    follow("app://host/old"),
    "ok:true|app://host/new?from=old|new body"
  );
  assert_eq!(follow("app://host/abs"), "ok:true|app://other/new|new body");
  assert_eq!(
    follow("app://host/chain"),
    "ok:true|app://host/new?from=old|new body"
  );
  // A redirected navigation ends up at the target.
  page.view().load_url("app://host/old", None);
  wait_loaded(page.view(), "app://host/new?from=old");
  assert_eq!(page.view().url(), "app://host/new?from=old");
}

fn response_redirect_without_location() {
  start(|_, res| res.respond(resp(304, &[], "kept")));
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 kept");
}

/// The test resource for range requests: byte `i` is `i % 251`.
fn media_bytes(len: usize) -> Vec<u8> {
  (0..len).map(|i| (i % 251) as u8).collect()
}

/// Answer `req` for `data` honouring a `Range: bytes=a-b` / `bytes=a-` header with a 206.
fn range_response(req: &SchemeRequest, data: &[u8], content_type: &str) -> Response<Vec<u8>> {
  let total = data.len();
  let range = req
    .request
    .headers()
    .get(header::RANGE)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("bytes="))
    .and_then(|v| v.split_once('-'));
  match range {
    Some(("", n)) => {
      let start = total - n.parse::<usize>().unwrap().min(total);
      resp(
        206,
        &[
          ("Content-Type", content_type),
          ("Accept-Ranges", "bytes"),
          (
            "Content-Range",
            &format!("bytes {start}-{}/{total}", total - 1),
          ),
        ],
        &data[start..],
      )
    }
    Some((a, b)) if !a.is_empty() => {
      let start: usize = a.parse().unwrap();
      let end = if b.is_empty() {
        total - 1
      } else {
        b.parse::<usize>().unwrap().min(total - 1)
      };
      resp(
        206,
        &[
          ("Content-Type", content_type),
          ("Accept-Ranges", "bytes"),
          ("Content-Range", &format!("bytes {start}-{end}/{total}")),
        ],
        &data[start..=end],
      )
    }
    _ => resp(
      200,
      &[("Content-Type", content_type), ("Accept-Ranges", "bytes")],
      data,
    ),
  }
}

fn response_range() {
  start(|req, res| {
    let r = range_response(&req, &media_bytes(1000), "application/octet-stream");
    res.respond(r)
  });
  let page = app_page();
  let get = |range: &str| {
    js(
      page.view(),
      &format!(
        "const r = await fetch('app://host/media', {{ headers: {{ Range: '{range}' }} }}); \
         return r.status + '|' + Array.from(new Uint8Array(await r.arrayBuffer())).join(',');"
      ),
    )
  };
  let expect = |status: u16, range: std::ops::Range<usize>| {
    let bytes: Vec<_> = range.map(|i| (i % 251).to_string()).collect();
    format!("ok:{status}|{}", bytes.join(","))
  };
  assert_eq!(get("bytes=10-19"), expect(206, 10..20));
  assert_eq!(get("bytes=990-"), expect(206, 990..1000));
  assert_eq!(get("bytes=0-0"), expect(206, 0..1));
  assert_eq!(get("bytes=250-260"), expect(206, 250..261));
  // Suffix ranges, answered with a 206 by the handler and as a full 200 that Qt cuts.
  assert_eq!(get("bytes=-5"), expect(206, 995..1000));
  let ranges: Vec<_> = seen_path("/media")
    .iter()
    .map(|s| s.header("range").unwrap_or("").to_owned())
    .collect();
  assert_eq!(
    ranges,
    [
      "bytes=10-19",
      "bytes=990-",
      "bytes=0-0",
      "bytes=250-260",
      "bytes=-5"
    ]
  );
}

fn range_of_full_body() {
  start(|_, res| {
    res.respond(resp(
      200,
      &[("Content-Type", "application/octet-stream")],
      media_bytes(1000),
    ))
  });
  let page = app_page();
  for (range, want) in [
    ("bytes=10-14", 10..15),
    ("bytes=-5", 995..1000),
    ("bytes=998-", 998..1000),
  ] {
    let r = js(
      page.view(),
      &format!(
        "const r = await fetch('app://host/full', {{ headers: {{ Range: '{range}' }} }}); \
         return Array.from(new Uint8Array(await r.arrayBuffer())).join(',');"
      ),
    );
    let bytes: Vec<_> = want.map(|i| (i % 251).to_string()).collect();
    assert_eq!(r, format!("ok:{}", bytes.join(",")), "{range}");
  }
}

fn response_range_without_content_range() {
  start(|_, res| res.respond(resp(206, &[("Content-Type", "text/plain")], "whole body")));
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 whole body");
}

/// A mono 8 kHz 8-bit PCM WAV file of `secs` seconds.
fn wav(secs: u32) -> Vec<u8> {
  let rate = 8000u32;
  let n = rate * secs;
  let mut v = Vec::with_capacity(44 + n as usize);
  v.extend_from_slice(b"RIFF");
  v.extend_from_slice(&(36 + n).to_le_bytes());
  v.extend_from_slice(b"WAVEfmt ");
  v.extend_from_slice(&16u32.to_le_bytes());
  v.extend_from_slice(&1u16.to_le_bytes()); // PCM
  v.extend_from_slice(&1u16.to_le_bytes()); // mono
  v.extend_from_slice(&rate.to_le_bytes());
  v.extend_from_slice(&rate.to_le_bytes()); // byte rate
  v.extend_from_slice(&1u16.to_le_bytes()); // block align
  v.extend_from_slice(&8u16.to_le_bytes()); // bits
  v.extend_from_slice(b"data");
  v.extend_from_slice(&n.to_le_bytes());
  v.extend((0..n).map(|i| 128u8.wrapping_add((i % 32) as u8)));
  v
}

fn media_range_requests() {
  start(|req, res| {
    thread_local!(static WAV: Vec<u8> = wav(600));
    let r = WAV.with(|w| range_response(&req, w, "audio/wav"));
    res.respond(r)
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const a = new Audio(); a.preload = 'auto'; a.src = 'app://host/sound.wav'; \
     await new Promise((ok, err) => { a.onloadedmetadata = ok; a.onerror = () => err(a.error.message); }); \
     const d = Math.round(a.duration); \
     a.currentTime = 590; \
     await new Promise((ok, err) => { a.onseeked = ok; a.onerror = () => err(a.error.message); }); \
     return d + '|' + Math.round(a.currentTime);",
  );
  assert_eq!(r, "ok:600|590");
  let ranges: Vec<_> = seen_path("/sound.wav")
    .iter()
    .filter_map(|s| s.header("range").map(str::to_owned))
    .collect();
  assert!(!ranges.is_empty(), "media requests carry a Range");
  assert!(
    ranges.iter().any(|r| !r.starts_with("bytes=0-")),
    "seeking near the end fetched a later range: {ranges:?}"
  );
}

fn response_large() {
  const LEN: usize = 8 * 1024 * 1024;
  let data: Vec<u8> = (0..LEN).map(|i| (i * 7 % 256) as u8).collect();
  let sum: u64 = data.iter().map(|&b| b as u64).sum();
  let data = Arc::new(data);
  let d = data.clone();
  start(move |_, res| {
    res.respond(resp(
      200,
      &[("Content-Type", "application/octet-stream")],
      d.to_vec(),
    ))
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const b = new Uint8Array(await (await fetch('app://host/big')).arrayBuffer()); \
     let s = 0; let ok = true; \
     for (let i = 0; i < b.length; i++) { s += b[i]; if (b[i] !== (i * 7) % 256) ok = false; } \
     return b.length + '|' + s + '|' + ok;",
  );
  assert_eq!(r, format!("ok:{LEN}|{sum}|true"));
}

fn response_empty() {
  start(|_, res| res.respond(resp(200, &[("Content-Type", "text/plain")], Vec::new())));
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/empty"), "ok:200 ");
}

fn response_binary() {
  start(|_, res| {
    res.respond(resp(
      200,
      &[("Content-Type", "application/octet-stream")],
      (0..=255u8).collect::<Vec<_>>(),
    ))
  });
  let page = app_page();
  let r = js(
    page.view(),
    "const b = new Uint8Array(await (await fetch('app://host/bin')).arrayBuffer()); \
     return b.length + '|' + b.every((x, i) => x === i);",
  );
  assert_eq!(r, "ok:256|true");
}

fn out_of_order() {
  start(|_, res| stash(res));
  let page = app_page();
  js_nowait(
    page.view(),
    "window.__order = []; \
     window.__all = Promise.all([1, 2, 3].map(async i => { \
       const t = await (await fetch('app://host/n' + i)).text(); window.__order.push(t); }));",
  );
  let mut responders = stashed(3);
  // Answer in reverse arrival order; each request gets its own answer.
  let paths: Vec<_> = seen_path("/n")
    .iter()
    .map(|s| s.path().to_owned())
    .collect();
  while let Some(r) = responders.pop() {
    let p = paths[responders.len()].clone();
    r.respond(text(&p));
  }
  let order = js(
    page.view(),
    "await window.__all; return window.__order.join(',');",
  );
  let mut expected = paths.clone();
  expected.reverse();
  assert_eq!(order, format!("ok:{}", expected.join(",")));
}

// ---------------------------------------------------------------------------------------------
// Responder

fn respond_other_thread() {
  start(|req, res| {
    let main = thread::current().id();
    let path = req.request.uri().path().to_owned();
    thread::spawn(move || {
      assert_ne!(thread::current().id(), main);
      res.respond(text(&format!("from thread {path}")));
    });
  });
  let page = app_page();
  assert_eq!(fetch(page.view(), "app://host/t"), "ok:200 from thread /t");
}

fn respond_delayed() {
  start(|_, res| {
    thread::spawn(move || {
      thread::sleep(Duration::from_millis(300));
      res.respond(text("late but fine"));
    });
  });
  let page = app_page();
  assert_eq!(
    fetch(page.view(), "app://host/slow"),
    "ok:200 late but fine"
  );
}

fn responder_dropped() {
  start(|req, res| {
    if req.request.uri().path() == "/drop" {
      drop(res);
    } else {
      res.respond(text("fine"));
    }
  });
  let page = app_page();
  assert_eq!(xhr(page.view(), "app://host/drop"), "ok:error");
  assert_eq!(fetch(page.view(), "app://host/after"), "ok:200 fine");
}

fn responder_dropped_other_thread() {
  start(|_, res| {
    thread::spawn(move || drop(res));
  });
  let page = app_page();
  assert_eq!(xhr(page.view(), "app://host/drop"), "ok:error");
}

fn handler_panics() {
  start(|req, res| {
    if req.request.uri().path() == "/panic" {
      panic!("boom from the scheme handler");
    }
    res.respond(text("still alive"));
  });
  let page = app_page();
  assert_eq!(xhr(page.view(), "app://host/panic"), "ok:error");
  assert_eq!(fetch(page.view(), "app://host/ok"), "ok:200 still alive");
  assert!(app::is_running());
}

fn respond_after_view_dropped() {
  start(|req, res| {
    if req.request.uri().path() == "/hang" {
      stash(res);
    } else {
      res.respond(text("fresh"));
    }
  });
  let page = app_page();
  js_nowait(
    page.view(),
    "fetch('app://host/hang'); fetch('app://host/hang');",
  );
  let mut responders = stashed(2);
  drop(page);
  pump(Duration::from_millis(100));
  // One answered on the main thread, one from another thread.
  responders.pop().unwrap().respond(text("too late"));
  let r = responders.pop().unwrap();
  thread::spawn(move || r.respond(text("too late")))
    .join()
    .unwrap();
  pump(Duration::from_millis(100));
  // Schemes still work for new views.
  let page = open(Profile::get(None, false), &[2], "app://host/");
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 fresh");
}

fn respond_after_navigation() {
  start(|req, res| {
    if req.request.uri().path() == "/hang" {
      stash(res);
    } else {
      res.respond(text("fresh"));
    }
  });
  let page = app_page();
  js_nowait(page.view(), "fetch('app://host/hang')");
  let mut responders = stashed(1);
  load(page.view(), "app://host/?second");
  responders.pop().unwrap().respond(text("too late"));
  assert_eq!(fetch(page.view(), "app://host/x"), "ok:200 fresh");
}

fn respond_after_shutdown() {
  start(|_, res| stash(res));
  let page = app_page();
  js_nowait(
    page.view(),
    "for (let i = 0; i < 4; i++) fetch('app://host/hang' + i);",
  );
  let mut responders = stashed(4);
  drop(page);
  app::shutdown();
  assert!(!app::is_running());
  // Answer / drop on the main thread and on other threads: all no-ops now.
  responders.pop().unwrap().respond(text("after shutdown"));
  drop(responders.pop().unwrap());
  let a = responders.pop().unwrap();
  let b = responders.pop().unwrap();
  thread::spawn(move || a.respond(text("after shutdown")))
    .join()
    .unwrap();
  thread::spawn(move || drop(b)).join().unwrap();
}

fn respond_queued_then_shutdown() {
  start(|_, res| stash(res));
  let page = app_page();
  js_nowait(page.view(), "fetch('app://host/a'); fetch('app://host/b');");
  let mut responders = stashed(2);
  // Queued for the main thread, never run: the loop doesn't spin again before shutdown.
  let a = responders.pop().unwrap();
  let b = responders.pop().unwrap();
  thread::spawn(move || {
    a.respond(text("queued"));
    drop(b);
  })
  .join()
  .unwrap();
  drop(page);
  app::shutdown();
}

fn respond_racing_shutdown() {
  start(|_, res| stash(res));
  let page = app_page();
  js_nowait(
    page.view(),
    "for (let i = 0; i < 20; i++) fetch('app://host/r' + i);",
  );
  let responders = stashed(20);
  let handles: Vec<_> = responders
    .into_iter()
    .enumerate()
    .map(|(i, r)| {
      thread::spawn(move || {
        thread::sleep(Duration::from_micros(i as u64 * 200));
        if i % 2 == 0 {
          r.respond(text("racing"));
        } else {
          drop(r);
        }
      })
    })
    .collect();
  drop(page);
  app::shutdown();
  for h in handles {
    h.join().unwrap();
  }
}
