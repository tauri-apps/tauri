// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `qtshim::webengine::Profile`: persistent/incognito profiles, user agent, cookies, data.

use std::{path::PathBuf, time::Duration};

use qtshim::test_harness::Case;
use qtshim::{
  app,
  webengine::{Profile, View, ViewOptions, Window},
};

use super::scheme::{eval, js, open, start, text, wait_true};
use crate::common::{init, pump, wait_for};

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass(
      "profile: the first get starts the engine",
      first_get_starts_engine,
    ),
    Case::pass(
      "profile: same directory and kind give the same profile",
      same_profile,
    ),
    Case::pass(
      "profile: a persistent profile writes to its directory",
      persistent_writes_to_dir,
    ),
    Case::pass(
      "profile: an incognito profile leaves its directory alone",
      incognito_leaves_dir_alone,
    ),
    Case::pass(
      "profile: set_user_agent reaches navigator.userAgent",
      user_agent,
    ),
    Case::pass(
      "profile: set_cookie with a Domain is listed",
      set_cookie_with_domain,
    ),
    Case::pass(
      "profile: set_cookie without a Domain uses the origin",
      set_cookie_with_origin,
    ),
    Case::pass(
      "profile: set_cookie replaces a cookie with the same identity",
      set_cookie_replaces,
    ),
    Case::pass("profile: delete_cookie removes a cookie", delete_cookie),
    Case::pass(
      "profile: delete_cookie without a Domain matches by name",
      delete_cookie_by_name,
    ),
    Case::pass(
      "profile: clear_data removes every cookie from the store",
      clear_data,
    ),
    Case::pass(
      "profile: cookies() is empty after clear_data",
      clear_data_empties_cookies,
    ),
    Case::pass(
      "profile: cookies set by a page are listed",
      page_cookie_listed,
    ),
    Case::pass(
      "profile: a set cookie is sent to http and visible to the page",
      cookie_visible_to_http_page,
    ),
    Case::pass("profile: profiles keep separate cookie jars", separate_jars),
    Case::pass(
      "profile: a persistent profile keeps cookies across get",
      persistent_cookies_across_get,
    ),
    Case::panics(
      "profile: get before app::init panics",
      "Qt can only be used",
      || {
        Profile::get(None, false);
      },
    ),
    Case::panics(
      "profile: cookies after shutdown panic",
      "Qt can only be used",
      || after_shutdown(|p| drop(p.cookies())),
    ),
    Case::panics(
      "profile: set_cookie after shutdown panics",
      "Qt can only be used",
      || after_shutdown(|p| p.set_cookie("a=1; Domain=example.com", None)),
    ),
    Case::panics(
      "profile: delete_cookie after shutdown panics",
      "Qt can only be used",
      || after_shutdown(|p| p.delete_cookie("a=1; Domain=example.com", None)),
    ),
    Case::panics(
      "profile: set_user_agent after shutdown panics",
      "Qt can only be used",
      || after_shutdown(|p| p.set_user_agent("x")),
    ),
    Case::panics(
      "profile: clear_data after shutdown panics",
      "Qt can only be used",
      || after_shutdown(|p| p.clear_data()),
    ),
    Case::panics(
      "profile: get after shutdown panics",
      "Qt can only be used",
      || {
        after_shutdown(|_| {
          Profile::get(None, false);
        })
      },
    ),
  ]
}

/// A fresh, unique directory under the system temp dir, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
  fn new(tag: &str) -> Self {
    let p = std::env::temp_dir().join(format!(
      "qtshim-profile-{tag}-{}-{:?}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    Self(p)
  }
  fn path(&self) -> &str {
    self.0.to_str().unwrap()
  }
  fn is_empty(&self) -> bool {
    std::fs::read_dir(&self.0).unwrap().next().is_none()
  }
}

impl Drop for TempDir {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.0);
  }
}

/// Keeps a view on a profile alive. Qt applies cookie-store calls (and reports cookies) only once
/// some page uses the profile.
struct Live {
  _view: View,
  _window: Window,
}

/// `profile` plus a view using it, so its cookie store is live.
fn live(profile: Profile) -> (Profile, Live) {
  let window = Window::new(1000, false);
  let view = View::new(1000, &window, profile, ViewOptions::default());
  view.load_html("<p>live</p>", None);
  (
    profile,
    Live {
      _view: view,
      _window: window,
    },
  )
}

fn after_shutdown(f: impl FnOnce(Profile)) {
  init();
  let p = Profile::get(None, false);
  app::shutdown();
  f(p);
}

/// Wait until `p.cookies()` satisfies `cond`.
fn wait_cookies(p: Profile, what: &str, cond: impl Fn(&[String]) -> bool + 'static) {
  wait_for(what, move || cond(&p.cookies()));
}

fn has(cookies: &[String], needle: &str) -> bool {
  cookies.iter().any(|c| c.contains(needle))
}

// ---------------------------------------------------------------------------------------------

fn first_get_starts_engine() {
  init();
  assert!(!qtshim::webengine::engine_started());
  let _ = Profile::get(None, true);
  assert!(qtshim::webengine::engine_started());
}

fn same_profile() {
  init();
  let a = TempDir::new("a");
  let b = TempDir::new("b");
  let default = Profile::get(None, false);
  assert_eq!(default, Profile::get(None, false));
  let incognito = Profile::get(None, true);
  assert_eq!(incognito, Profile::get(None, true));
  assert_ne!(default, incognito);
  let pa = Profile::get(Some(a.path()), false);
  assert_eq!(pa, Profile::get(Some(a.path()), false));
  assert_ne!(pa, default);
  assert_ne!(pa, Profile::get(Some(b.path()), false));
  assert_ne!(pa, Profile::get(Some(a.path()), true));
  app::shutdown();
}

fn persistent_writes_to_dir() {
  let dir = TempDir::new("persist");
  init();
  let (p, _live_p) = live(Profile::get(Some(dir.path()), false));
  p.set_cookie("keep=1; Domain=example.com; Path=/; Max-Age=3600", None);
  wait_cookies(p, "the cookie", |c| has(c, "keep=1"));
  app::shutdown();
  assert!(!dir.is_empty(), "nothing written to {}", dir.path());
}

fn incognito_leaves_dir_alone() {
  let dir = TempDir::new("incognito");
  init();
  let (p, _live_p) = live(Profile::get(Some(dir.path()), true));
  p.set_cookie("x=1; Domain=example.com; Path=/; Max-Age=3600", None);
  wait_cookies(p, "the cookie", |c| has(c, "x=1"));
  app::shutdown();
  assert!(
    dir.is_empty(),
    "an incognito profile wrote to its directory"
  );
}

fn user_agent() {
  start(|req, res| {
    let ua = req
      .request
      .headers()
      .get(http::header::USER_AGENT)
      .map_or("none", |v| v.to_str().unwrap())
      .to_owned();
    res.respond(text(&ua))
  });
  let custom = Profile::get(None, false);
  custom.set_user_agent("QtShimTest/1.0");
  let other = Profile::get(None, true);
  let a = open(custom, &[1], "app://host/");
  let b = open(other, &[2], "app://host/");
  assert_eq!(eval(a.view(), "navigator.userAgent"), "\"QtShimTest/1.0\"");
  let ua = eval(b.view(), "navigator.userAgent");
  assert!(ua.contains("QtWebEngine"), "{ua}");
}

fn set_cookie_with_domain() {
  init();
  let (p, _live_p) = live(Profile::get(None, true));
  assert!(p.cookies().is_empty());
  p.set_cookie("a=1; Domain=example.com; Path=/", None);
  wait_cookies(p, "a=1", |c| has(c, "a=1"));
  let c = p.cookies();
  assert_eq!(c.len(), 1, "{c:?}");
  assert!(c[0].contains("example.com"), "{c:?}");
}

fn set_cookie_with_origin() {
  init();
  let (p, _live_p) = live(Profile::get(None, true));
  p.set_cookie("b=2; Path=/", Some("http://origin.example/"));
  wait_cookies(p, "b=2", |c| has(c, "b=2"));
  assert!(
    p.cookies()[0].contains("origin.example"),
    "{:?}",
    p.cookies()
  );
  // Several cookies in one call (newline-separated `Set-Cookie` values).
  p.set_cookie("c=3; Path=/\nd=4; Path=/", Some("http://origin.example/"));
  wait_cookies(p, "c=3 and d=4", |c| has(c, "c=3") && has(c, "d=4"));
}

fn set_cookie_replaces() {
  init();
  let (p, _live_p) = live(Profile::get(None, true));
  p.set_cookie("r=old; Domain=example.com; Path=/", None);
  wait_cookies(p, "r=old", |c| has(c, "r=old"));
  p.set_cookie("r=new; Domain=example.com; Path=/", None);
  wait_cookies(p, "r=new", |c| has(c, "r=new"));
  let c = p.cookies();
  assert_eq!(c.len(), 1, "{c:?}");
}

fn delete_cookie() {
  init();
  let (p, _live_p) = live(Profile::get(None, true));
  p.set_cookie("del=1; Domain=example.com; Path=/", None);
  p.set_cookie("stay=1; Domain=example.com; Path=/", None);
  wait_cookies(p, "both", |c| has(c, "del=1") && has(c, "stay=1"));
  p.delete_cookie("del=1; Domain=example.com; Path=/", None);
  wait_cookies(p, "del gone", |c| !has(c, "del=1"));
  assert!(has(&p.cookies(), "stay=1"));
}

fn delete_cookie_by_name() {
  init();
  let (p, _live_p) = live(Profile::get(None, true));
  p.set_cookie("n=1; Path=/", Some("http://by-name.example/"));
  wait_cookies(p, "n=1", |c| has(c, "n=1"));
  p.delete_cookie("n=", Some("http://by-name.example/"));
  wait_cookies(p, "n gone", |c| !has(c, "n=1"));
}

fn clear_data() {
  let (port, log) = http_server();
  init();
  let p = Profile::get(None, true);
  let origin = format!("http://127.0.0.1:{port}/");
  let page = open(p, &[1], &origin);
  for i in 0..5 {
    p.set_cookie(&format!("gone{i}=1; Path=/"), Some(&origin));
  }
  wait_cookies(p, "5 cookies", |c| c.len() == 5);
  assert_eq!(
    js(page.view(), "return document.cookie.split('; ').length"),
    "ok:5"
  );
  p.clear_data();
  // The store is empty: the page sees no cookies and none are sent.
  let v = page.view().clone();
  wait_true(&v, "document.cookie === ''", "the page's cookies to go");
  let before = log.lock().unwrap().len();
  page.view().eval("window.__old = true");
  page.view().reload();
  wait_true(
    &v,
    "!window.__old && document.readyState === 'complete'",
    "the reload",
  );
  let after = log.lock().unwrap()[before..].to_vec();
  assert!(!after.is_empty());
  for r in after {
    assert!(!r.contains("Cookie:"), "{r}");
  }
}

fn clear_data_empties_cookies() {
  init();
  let (p, _live_p) = live(Profile::get(None, true));
  for i in 0..5 {
    p.set_cookie(&format!("k{i}=v; Domain=example.com; Path=/"), None);
  }
  wait_cookies(p, "5 cookies", |c| c.len() == 5);
  for _ in 0..3 {
    p.clear_data();
    wait_cookies(p, "no cookies", |c| c.is_empty());
    for i in 0..5 {
      p.set_cookie(&format!("k{i}=v; Domain=example.com; Path=/"), None);
    }
    wait_cookies(p, "5 cookies", |c| c.len() == 5);
  }
}

fn page_cookie_listed() {
  let (port, _log) = http_server();
  init();
  let p = Profile::get(None, true);
  let url = format!("http://127.0.0.1:{port}/");
  let page = open(p, &[1], &url);
  let r = js(
    page.view(),
    "document.cookie = 'fromjs=yes; path=/'; return document.cookie;",
  );
  assert_eq!(r, "ok:fromjs=yes");
  wait_cookies(p, "the page's cookie", |c| has(c, "fromjs=yes"));
}

fn cookie_visible_to_http_page() {
  let (port, log) = http_server();
  init();
  let p = Profile::get(None, true);
  let origin = format!("http://127.0.0.1:{port}/");
  p.set_cookie("seen=1; Path=/", Some(&origin));
  // No view used the profile yet: the cookie is applied once one does (Qt queues cookie-store
  // calls until the profile's network context exists).
  let page = open(p, &[1], &origin);
  assert_eq!(eval(page.view(), "document.cookie"), "\"seen=1\"");
  assert!(has(&p.cookies(), "seen=1"));
  let req = log.lock().unwrap()[0].clone();
  assert!(req.contains("Cookie: seen=1"), "{req}");
}

fn separate_jars() {
  init();
  let (a, _live_a) = live(Profile::get(None, true));
  let dir = TempDir::new("jar");
  let (b, _live_b) = live(Profile::get(Some(dir.path()), true));
  a.set_cookie("only=a; Domain=example.com; Path=/", None);
  wait_cookies(a, "a's cookie", |c| has(c, "only=a"));
  pump(Duration::from_millis(200));
  assert!(!has(&b.cookies(), "only=a"));
}

fn persistent_cookies_across_get() {
  let dir = TempDir::new("across");
  init();
  let (p, _live_p) = live(Profile::get(Some(dir.path()), false));
  p.set_cookie("p=1; Domain=example.com; Path=/; Max-Age=3600", None);
  wait_cookies(p, "p=1", |c| has(c, "p=1"));
  let again = Profile::get(Some(dir.path()), false);
  assert_eq!(p, again);
  assert!(has(&again.cookies(), "p=1"));
  app::shutdown();
}

/// Serve a tiny page on `http://127.0.0.1:<port>/`, recording the raw requests.
fn http_server() -> (u16, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
  super::scheme::http_server()
}
