// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `QtRuntime` / `QtHandle` / `QtEventProxy`: init arguments, custom schemes, user events,
//! `run_on_main_thread`, monitors, theme and other app-wide queries.

use std::{
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
  },
  thread,
};

use qtshim::test_harness::Case;
use tauri_runtime::{
  EventLoopProxy, Runtime, RuntimeHandle, WebviewDispatch, WindowDispatch,
  webview::PageLoadEvent,
  window::WindowBuilder,
  window::{PendingWindow, RawWindow},
};
use tauri_runtime_qt::Qt;
use tauri_utils::Theme;

use crate::common::*;

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass(
      "runtime: user events from the main thread are delivered by run_iteration",
      user_events_main_thread,
    ),
    Case::pass(
      "runtime: user events from other threads arrive in order",
      user_events_other_threads,
    ),
    Case::pass(
      "runtime: run_on_main_thread runs closures on the main thread",
      run_on_main_thread,
    ),
    Case::pass(
      "runtime: monitors and cursor position from the runtime and the handle",
      monitors,
    ),
    // Offscreen's platform theme ignores `QStyleHints::setColorScheme` (the scheme stays
    // Light/Unknown and `colorSchemeChanged` never fires); run with a real display.
    Case::pass(
      "runtime: set_theme changes the app theme and emits ThemeChanged",
      set_theme,
    )
    .ignored(),
    Case::pass(
      "runtime: set_theme from the runtime, handle and window succeeds",
      set_theme_calls,
    ),
    Case::pass(
      "runtime: custom_scheme_url, webview_version and display_handle",
      misc_queries,
    ),
    Case::pass(
      "runtime: a custom scheme from the init args serves the first page",
      custom_scheme_serves_page,
    ),
    Case::pass(
      "runtime: chromium flags and the remote debugging port are accepted",
      chromium_flags,
    ),
    Case::pass(
      "runtime: a scheme first seen after the engine started is not routed",
      late_scheme_not_routed,
    ),
    Case::pass(
      "runtime: the handle creates windows and webviews from another thread",
      handle_creates_from_other_thread,
    ),
    Case::pass(
      "runtime: new_any_thread creates a working runtime",
      new_any_thread,
    ),
  ]
}

fn user_events_main_thread() {
  let mut rt = runtime();
  let proxy = rt.create_proxy();
  proxy.send_event(Msg::Ping(1)).unwrap();
  rt.handle().create_proxy().send_event(Msg::Ping(2)).unwrap();
  wait_for(&mut rt, "both user events", || {
    has_event(&Ev::User(Msg::Ping(2)))
  });
  let users: Vec<_> = events()
    .into_iter()
    .filter(|e| matches!(e, Ev::User(_)))
    .collect();
  assert_eq!(users, [Ev::User(Msg::Ping(1)), Ev::User(Msg::Ping(2))]);
}

fn user_events_other_threads() {
  let mut rt = runtime();
  let proxy = rt.create_proxy();
  let sender = thread::spawn(move || {
    for i in 0..50 {
      proxy.send_event(Msg::Ping(i)).unwrap();
    }
  });
  wait_for(&mut rt, "50 user events", || {
    has_event(&Ev::User(Msg::Ping(49)))
  });
  sender.join().unwrap();
  let got: Vec<_> = events()
    .into_iter()
    .filter_map(|e| match e {
      Ev::User(Msg::Ping(i)) => Some(i),
      _ => None,
    })
    .collect();
  assert_eq!(got, (0..50).collect::<Vec<_>>());
}

fn run_on_main_thread() {
  let mut rt = runtime();
  let main = thread::current().id();
  let (w, wv) = webview_window(&rt, "main", "about:blank");
  let handle = rt.handle();
  let ran = Arc::new(Mutex::new(Vec::new()));

  // On the main thread they run inline.
  let r = ran.clone();
  handle
    .run_on_main_thread(move || {
      r.lock()
        .unwrap()
        .push(("handle-inline", thread::current().id()))
    })
    .unwrap();
  let r = ran.clone();
  w.dispatcher
    .run_on_main_thread(move || {
      r.lock()
        .unwrap()
        .push(("window-inline", thread::current().id()))
    })
    .unwrap();
  let r = ran.clone();
  wv.run_on_main_thread(move || {
    r.lock()
      .unwrap()
      .push(("webview-inline", thread::current().id()))
  })
  .unwrap();
  assert_eq!(ran.lock().unwrap().len(), 3);

  // From another thread they're queued.
  let (r, window, webview) = (ran.clone(), w.dispatcher.clone(), wv.clone());
  thread::spawn(move || {
    let r1 = r.clone();
    handle
      .run_on_main_thread(move || r1.lock().unwrap().push(("handle", thread::current().id())))
      .unwrap();
    let r2 = r.clone();
    window
      .run_on_main_thread(move || r2.lock().unwrap().push(("window", thread::current().id())))
      .unwrap();
    webview
      .run_on_main_thread(move || r.lock().unwrap().push(("webview", thread::current().id())))
      .unwrap();
  })
  .join()
  .unwrap();
  let r = ran.clone();
  wait_for(&mut rt, "the queued closures", move || {
    r.lock().unwrap().len() == 6
  });
  let ran = ran.lock().unwrap();
  assert!(ran.iter().all(|(_, id)| *id == main), "{ran:?}");
  let names: Vec<_> = ran.iter().map(|(n, _)| *n).collect();
  assert_eq!(
    names,
    [
      "handle-inline",
      "window-inline",
      "webview-inline",
      "handle",
      "window",
      "webview"
    ]
  );
}

fn monitors() {
  let mut rt = runtime();
  let primary = rt
    .primary_monitor()
    .expect("offscreen has a primary screen");
  let all = rt.available_monitors();
  assert!(!all.is_empty());
  assert!(all.iter().any(|m| m.name == primary.name));
  assert!(primary.size.width > 0 && primary.size.height > 0);
  assert!(primary.scale_factor > 0.0);
  let p = primary.position;
  let at = rt
    .monitor_from_point(p.x as f64 + 1.0, p.y as f64 + 1.0)
    .expect("a monitor contains its own origin");
  assert_eq!(at.name, primary.name);
  assert!(rt.monitor_from_point(-1.0e7, -1.0e7).is_none());
  // Offscreen may or may not report a cursor; either way it must not fail differently.
  let cursor = rt.cursor_position();

  // The handle answers the same from another thread (through the main loop).
  let handle = rt.handle();
  let done = Arc::new(AtomicBool::new(false));
  let d = done.clone();
  let (name, origin) = (primary.name.clone(), (p.x as f64 + 1.0, p.y as f64 + 1.0));
  let cursor_ok = cursor.is_ok();
  let t = thread::spawn(move || {
    assert_eq!(handle.primary_monitor().unwrap().unwrap().name, name);
    assert!(!handle.available_monitors().unwrap().is_empty());
    assert_eq!(
      handle
        .monitor_from_point(origin.0, origin.1)
        .unwrap()
        .unwrap()
        .name,
      name
    );
    assert!(handle.monitor_from_point(-1.0e7, -1.0e7).unwrap().is_none());
    assert_eq!(handle.cursor_position().is_ok(), cursor_ok);
    d.store(true, Ordering::Release);
  });
  wait_for(&mut rt, "the handle queries", || {
    done.load(Ordering::Acquire)
  });
  t.join().unwrap();

  // On the main thread the handle runs inline.
  let handle = rt.handle();
  assert_eq!(
    handle.primary_monitor().unwrap().unwrap().name,
    primary.name
  );
  assert_eq!(handle.available_monitors().unwrap().len(), all.len());
}

fn set_theme() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  rt.set_theme(Some(Theme::Dark));
  assert_eq!(w.dispatcher.theme().unwrap(), Theme::Dark);
  wait_window_event(&mut rt, "main", |e| *e == Win::Theme(Theme::Dark));

  rt.handle().set_theme(Some(Theme::Light));
  wait_for(&mut rt, "the light theme", || {
    w.dispatcher.theme().unwrap() == Theme::Light
  });
  wait_window_event(&mut rt, "main", |e| *e == Win::Theme(Theme::Light));

  // The window dispatcher's setter is app-wide too.
  w.dispatcher.set_theme(Some(Theme::Dark)).unwrap();
  wait_for(&mut rt, "the dark theme", || {
    w.dispatcher.theme().unwrap() == Theme::Dark
  });
}

fn set_theme_calls() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder().theme(Some(Theme::Dark)));
  rt.set_theme(Some(Theme::Dark));
  rt.set_theme(None);
  rt.handle().set_theme(Some(Theme::Light));
  rt.handle().set_theme(None);
  w.dispatcher.set_theme(Some(Theme::Dark)).unwrap();
  w.dispatcher.set_theme(None).unwrap();
  pump(&mut rt, std::time::Duration::from_millis(20));
  // Whatever the platform reports, it's one of the two.
  let theme = w.dispatcher.theme().unwrap();
  assert!(matches!(theme, Theme::Light | Theme::Dark));
}

fn misc_queries() {
  let rt = runtime();
  let handle = rt.handle();
  assert_eq!(
    handle.custom_scheme_url("tauri", false),
    "tauri://localhost"
  );
  assert_eq!(handle.custom_scheme_url("ipc", true), "ipc://localhost");
  let version = handle.webview_version().unwrap();
  assert!(!version.is_empty(), "{version:?}");
  // Offscreen has no Wayland/X11 display: an error, not a crash.
  let _ = handle.display_handle();
}

fn custom_scheme_serves_page() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("first"));
  wait_title(&mut rt, &wv, "first");
  assert_eq!(wv.url().unwrap(), page("first"));
  let reqs = REQUESTS.lock().unwrap().clone();
  let (label, method, uri, _) = reqs
    .iter()
    .find(|r| r.2.ends_with("/t/first"))
    .expect("the page request");
  assert_eq!(label, "main");
  assert_eq!(method, "GET");
  assert_eq!(uri, &page("first"));
}

fn chromium_flags() {
  let mut rt = runtime_with(
    &[SCHEME],
    Qt::default()
      .chromium_flags("--disable-gpu")
      .remote_debugging_port(0),
  );
  let (_w, wv) = webview_window(&rt, "main", &page("flags"));
  wait_title(&mut rt, &wv, "flags");
}

fn late_scheme_not_routed() {
  let mut rt = runtime_with(&[], Qt::default());
  // The first webview registers `testapp` before the engine starts: routed.
  let (w, wv) = webview_window(&rt, "main", &page("one"));
  wait_title(&mut rt, &wv, "one");

  // `late` only appears on a second webview, after the engine started: never routed.
  let finished = Arc::new(Mutex::new(Vec::new()));
  let f = finished.clone();
  let mut pending = pending_webview("late", "late://localhost/t/late");
  let hit = Arc::new(AtomicBool::new(false));
  let h = hit.clone();
  pending.register_uri_scheme_protocol("late", move |_, _, respond| {
    h.store(true, Ordering::Release);
    respond(http::Response::new(b"<title>late</title>".to_vec().into()));
  });
  pending.on_page_load_handler = Some(Box::new(move |url, ev| {
    if ev == PageLoadEvent::Finished {
      f.lock().unwrap().push(url.to_string());
    }
  }));
  let mut dispatcher = w.dispatcher.clone();
  let late = dispatcher.create_webview(pending).unwrap();
  let f = finished.clone();
  wait_for(&mut rt, "the late page load to end", move || {
    !f.lock().unwrap().is_empty()
  });
  assert!(!hit.load(Ordering::Acquire));
  assert_ne!(
    eval(&mut rt, &late.dispatcher, "document.title"),
    "\"late\""
  );
}

fn handle_creates_from_other_thread() {
  let mut rt = runtime();
  let handle = rt.handle();
  let done = Arc::new(Mutex::new(None));
  let d = done.clone();
  thread::spawn(move || {
    let mut pending = PendingWindow::new(builder().title("threaded"), "threaded").unwrap();
    pending.set_webview(pending_webview("threaded", &page("threaded")));
    let w = handle
      .create_window(pending, None::<fn(RawWindow)>)
      .unwrap();
    let child = handle
      .create_webview(w.id, pending_webview("child", &page("child")))
      .unwrap();
    assert_eq!(w.dispatcher.title().unwrap(), "threaded");
    *d.lock().unwrap() = Some((w, child));
  });
  wait_for(&mut rt, "the threaded window", || {
    done.lock().unwrap().is_some()
  });
  let (w, child) = done.lock().unwrap().take().unwrap();
  assert_eq!(w.label, "threaded");
  assert_eq!(child.label, "child");
  let wv = w.webview.unwrap().webview.dispatcher;
  wait_title(&mut rt, &wv, "threaded");
  wait_title(&mut rt, &child.dispatcher, "child");
}

fn new_any_thread() {
  let mut rt = Rt::new_any_thread(Default::default()).unwrap();
  rt.create_proxy().send_event(Msg::Ping(7)).unwrap();
  wait_for(&mut rt, "the user event", || {
    has_event(&Ev::User(Msg::Ping(7)))
  });
}
