// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `QtWebviewDispatcher` and the per-webview handlers of `PendingWebview`: navigation, scripts,
//! history, zoom, bounds, child webviews, reparenting, cookies, IPC, new windows, downloads,
//! page load / title / navigation handlers and renderer crashes.

use std::{
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
  },
  time::Duration,
};

use qtshim::test_harness::Case;
use tauri_runtime::{
  Cookie, Error, Runtime, RuntimeHandle, WebviewDispatch, WindowDispatch,
  dpi::{LogicalPosition, LogicalSize, Rect},
  webview::{
    DownloadEvent, InitializationScript, NewWindowResponse, PageLoadEvent,
    WebContentProcessTerminationReason,
  },
  window::{PendingWindow, RawWindow, WindowBuilder},
};
use tauri_utils::config::Color;

use crate::common::*;

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass(
      "webview: navigate, url, reload and page load events",
      navigate_and_page_load,
    ),
    Case::pass(
      "webview: eval_script and eval_script_with_callback",
      eval_scripts,
    ),
    Case::pass("webview: document title handler", title_handler),
    Case::pass(
      "webview: the navigation handler blocks navigations",
      navigation_handler,
    ),
    Case::pass("webview: go_back, go_forward and can_go_*", history),
    Case::pass(
      "webview: initialization scripts and user agent",
      init_scripts_and_user_agent,
    ),
    Case::pass("webview: set_zoom changes the zoom", zoom),
    Case::pass(
      "webview: child webview bounds, position, size and auto-resize",
      child_bounds,
    ),
    Case::pass(
      "webview: show, hide, focus, background, devtools and with_webview",
      misc,
    ),
    Case::pass("webview: close removes a child webview", close_child),
    Case::pass(
      "webview: reparent moves a webview to another window",
      reparent,
    ),
    Case::pass(
      "webview: cookies set, get, filter by url and delete",
      cookies,
    ),
    Case::pass(
      "webview: clear_all_browsing_data removes cookies",
      clear_browsing_data,
    ),
    Case::pass(
      "webview: a scheme request from a page carries its origin",
      scheme_origin,
    ),
    Case::pass(
      "webview: window.ipc.postMessage reaches the IPC handler",
      ipc,
    ),
    Case::pass(
      "webview: the new-window handler can deny a popup",
      new_window_deny,
    ),
    Case::pass(
      "webview: the new-window handler can open a popup in a created window",
      new_window_create,
    ),
    Case::pass(
      "webview: the download handler picks the destination and sees the result",
      download,
    ),
    Case::pass(
      "webview: the download handler can cancel a download",
      download_cancel,
    ),
    Case::pass(
      "webview: a killed renderer reaches the terminate handler",
      renderer_killed,
    ),
    Case::pass(
      "webview: a persistent data directory and incognito profiles",
      data_directory,
    ),
    Case::pass(
      "webview: dispatcher calls on a closed webview",
      closed_webview,
    ),
  ]
}

fn navigate_and_page_load() {
  let mut rt = runtime();
  let loads = Arc::new(Mutex::new(Vec::new()));
  let l = loads.clone();
  let mut pending = pending_webview("main", &page("one"));
  pending.on_page_load_handler = Some(Box::new(move |url, ev| {
    l.lock().unwrap().push((url.to_string(), ev));
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "one");
  let l = loads.clone();
  wait_for(&mut rt, "the first load to finish", move || {
    l.lock()
      .unwrap()
      .contains(&(page("one"), PageLoadEvent::Finished))
  });
  assert!(
    loads
      .lock()
      .unwrap()
      .contains(&(page("one"), PageLoadEvent::Started))
  );

  wv.navigate(page("two").parse().unwrap()).unwrap();
  wait_title(&mut rt, &wv, "two");
  assert_eq!(wv.url().unwrap(), page("two"));

  // reload runs the page again (a fresh window object).
  eval(&mut rt, &wv, "window.marker = 1");
  wv.reload().unwrap();
  wait_js(&mut rt, &wv, "typeof window.marker", "\"undefined\"");
  wait_title(&mut rt, &wv, "two");
}

fn eval_scripts() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("eval"));
  wait_title(&mut rt, &wv, "eval");
  wv.eval_script("document.body.dataset.x = 'set'").unwrap();
  wait_js(&mut rt, &wv, "document.body.dataset.x", "\"set\"");
  assert_eq!(eval(&mut rt, &wv, "1 + 2"), "3");
  assert_eq!(eval(&mut rt, &wv, "({a: [1, 'b']})"), r#"{"a":[1,"b"]}"#);
  // From another thread.
  let (wv2, out) = (wv.clone(), Arc::new(Mutex::new(None)));
  let o = out.clone();
  std::thread::spawn(move || {
    wv2
      .eval_script_with_callback("'threaded'", move |r| *o.lock().unwrap() = Some(r))
      .unwrap();
  })
  .join()
  .unwrap();
  wait_for(&mut rt, "the threaded eval", || {
    out.lock().unwrap().is_some()
  });
  assert_eq!(out.lock().unwrap().as_deref(), Some("\"threaded\""));
}

fn title_handler() {
  let mut rt = runtime();
  let titles = Arc::new(Mutex::new(Vec::new()));
  let t = titles.clone();
  let mut pending = pending_webview("main", &page("first"));
  pending.document_title_changed_handler = Some(Box::new(move |title| {
    t.lock().unwrap().push(title);
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  let t = titles.clone();
  wait_for(&mut rt, "the page title", move || {
    t.lock().unwrap().iter().any(|t| t == "first")
  });
  wv.eval_script("document.title = 'changed'").unwrap();
  let t = titles.clone();
  wait_for(&mut rt, "the changed title", move || {
    t.lock().unwrap().iter().any(|t| t == "changed")
  });
}

fn navigation_handler() {
  let mut rt = runtime();
  let seen = Arc::new(Mutex::new(Vec::new()));
  let s = seen.clone();
  let mut pending = pending_webview("main", &page("start"));
  pending.navigation_handler = Some(Box::new(move |url| {
    s.lock().unwrap().push(url.to_string());
    !url.path().contains("blocked")
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "start");
  assert!(seen.lock().unwrap().contains(&page("start")));

  // A page-initiated navigation to a blocked URL is refused...
  wv.eval_script(format!("location.href = {:?}", page("blocked")))
    .unwrap();
  let s = seen.clone();
  wait_for(&mut rt, "the blocked navigation", move || {
    s.lock().unwrap().contains(&page("blocked"))
  });
  pump(&mut rt, Duration::from_millis(100));
  assert_eq!(wv.url().unwrap(), page("start"));
  assert_eq!(eval(&mut rt, &wv, "document.title"), "\"start\"");
  // ...an allowed one goes through. The handler sees navigations started by the app too.
  wv.eval_script(format!("location.href = {:?}", page("allowed")))
    .unwrap();
  wait_title(&mut rt, &wv, "allowed");
  wv.navigate(page("blocked2").parse().unwrap()).unwrap();
  let s = seen.clone();
  wait_for(&mut rt, "the blocked app navigation", move || {
    s.lock().unwrap().contains(&page("blocked2"))
  });
  pump(&mut rt, Duration::from_millis(100));
  assert_eq!(eval(&mut rt, &wv, "document.title"), "\"allowed\"");
}

fn history() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("h1"));
  wait_title(&mut rt, &wv, "h1");
  assert!(!wv.can_go_back().unwrap());
  wv.navigate(page("h2").parse().unwrap()).unwrap();
  wait_title(&mut rt, &wv, "h2");
  wait_for(&mut rt, "can_go_back", || wv.can_go_back().unwrap());
  assert!(!wv.can_go_forward().unwrap());
  wv.go_back().unwrap();
  wait_title(&mut rt, &wv, "h1");
  wait_for(&mut rt, "can_go_forward", || wv.can_go_forward().unwrap());
  wv.go_forward().unwrap();
  wait_title(&mut rt, &wv, "h2");
  // pushState entries count too (go_back runs history.go(-1) in the page).
  wv.eval_script("history.pushState({}, '', '#pushed')")
    .unwrap();
  wait_for(&mut rt, "the pushed url", || {
    wv.url().unwrap().ends_with("#pushed")
  });
  wv.go_back().unwrap();
  wait_for(&mut rt, "back from the pushed entry", || {
    wv.url().unwrap() == page("h2")
  });
}

fn init_scripts_and_user_agent() {
  let mut rt = runtime();
  let mut pending = pending_webview("main", &page("init"));
  pending.webview_attributes.initialization_scripts = vec![
    InitializationScript {
      script: "window.__early = document.readyState".into(),
      for_main_frame_only: true,
    },
    InitializationScript {
      script: "window.__all = true".into(),
      for_main_frame_only: false,
    },
  ];
  pending.webview_attributes.user_agent = Some("QtRuntimeTest/1.0".into());
  pending.webview_attributes.background_color = Some(Color(0, 0, 0, 255));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "init");
  assert_eq!(eval(&mut rt, &wv, "window.__early"), "\"loading\"");
  assert_eq!(eval(&mut rt, &wv, "window.__all"), "true");
  assert_eq!(
    eval(&mut rt, &wv, "navigator.userAgent"),
    "\"QtRuntimeTest/1.0\""
  );
}

fn zoom() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("zoom"));
  wait_title(&mut rt, &wv, "zoom");
  let base: f64 = eval(&mut rt, &wv, "window.devicePixelRatio")
    .parse()
    .unwrap();
  wv.set_zoom(2.0).unwrap();
  let want = format!("{}", base * 2.0);
  wait_js(&mut rt, &wv, "window.devicePixelRatio", &want);
  wv.set_zoom(1.0).unwrap();
  wait_js(&mut rt, &wv, "window.devicePixelRatio", &format!("{base}"));
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
  Rect {
    position: LogicalPosition::new(x, y).into(),
    size: LogicalSize::new(w, h).into(),
  }
}

fn logical(r: Rect, scale: f64) -> (f64, f64, f64, f64) {
  let p = r.position.to_logical::<f64>(scale);
  let s = r.size.to_logical::<f64>(scale);
  (p.x, p.y, s.width, s.height)
}

fn child_bounds() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder().inner_size(400.0, 300.0));
  let scale = w.dispatcher.scale_factor().unwrap();
  let mut pending = pending_webview("child", &page("child"));
  pending.webview_attributes.bounds = Some(rect(10.0, 20.0, 200.0, 100.0));
  pending.webview_attributes.auto_resize = true;
  let child = rt.create_webview(w.id, pending).unwrap().dispatcher;
  assert_eq!(
    logical(child.bounds().unwrap(), scale),
    (10.0, 20.0, 200.0, 100.0)
  );
  assert_eq!(
    child.position().unwrap(),
    LogicalPosition::new(10.0, 20.0).to_physical(scale)
  );
  assert_eq!(
    child.size().unwrap(),
    LogicalSize::new(200.0, 100.0).to_physical(scale)
  );
  wait_title(&mut rt, &child, "child");

  // auto_resize: the child keeps its proportions when the window grows.
  w.dispatcher
    .set_size(LogicalSize::new(800.0, 600.0).into())
    .unwrap();
  wait_for(&mut rt, "the child to follow the window", || {
    logical(child.bounds().unwrap(), scale) == (20.0, 40.0, 400.0, 200.0)
  });

  child.set_bounds(rect(0.0, 0.0, 100.0, 50.0)).unwrap();
  assert_eq!(
    logical(child.bounds().unwrap(), scale),
    (0.0, 0.0, 100.0, 50.0)
  );
  child
    .set_position(LogicalPosition::new(5.0, 6.0).into())
    .unwrap();
  assert_eq!(
    logical(child.bounds().unwrap(), scale),
    (5.0, 6.0, 100.0, 50.0)
  );
  child.set_size(LogicalSize::new(70.0, 80.0).into()).unwrap();
  assert_eq!(
    logical(child.bounds().unwrap(), scale),
    (5.0, 6.0, 70.0, 80.0)
  );

  // Without auto_resize it stays put.
  child.set_auto_resize(false).unwrap();
  w.dispatcher
    .set_size(LogicalSize::new(400.0, 300.0).into())
    .unwrap();
  pump(&mut rt, Duration::from_millis(50));
  assert_eq!(
    logical(child.bounds().unwrap(), scale),
    (5.0, 6.0, 70.0, 80.0)
  );
  child.set_auto_resize(true).unwrap();
  w.dispatcher
    .set_size(LogicalSize::new(800.0, 600.0).into())
    .unwrap();
  wait_for(&mut rt, "auto-resize turned back on", || {
    logical(child.bounds().unwrap(), scale) == (10.0, 12.0, 140.0, 160.0)
  });

  // A child without bounds fills the window like the window's own webview.
  let fill = rt
    .create_webview(w.id, pending_webview("fill", "about:blank"))
    .unwrap()
    .dispatcher;
  let (_, _, fw, fh) = logical(fill.bounds().unwrap(), scale);
  assert_eq!((fw, fh), (800.0, 600.0));
}

fn misc() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("misc"));
  wait_title(&mut rt, &wv, "misc");
  wv.hide().unwrap();
  wv.show().unwrap();
  wv.set_focus().unwrap();
  wv.set_background_color(Some(Color(255, 0, 0, 255)))
    .unwrap();
  wv.set_background_color(None).unwrap();

  assert!(!wv.is_devtools_open().unwrap());
  wv.open_devtools();
  wait_for(&mut rt, "devtools to open", || {
    wv.is_devtools_open().unwrap()
  });
  wv.close_devtools();
  wait_for(&mut rt, "devtools to close", || {
    !wv.is_devtools_open().unwrap()
  });

  let got = Arc::new(AtomicBool::new(false));
  let g = got.clone();
  wv.with_webview(move |w| {
    assert!(!w.view.is_null() && !w.page.is_null());
    g.store(true, Ordering::Release);
  })
  .unwrap();
  assert!(
    got.load(Ordering::Acquire),
    "runs inline on the main thread"
  );
  // Still alive.
  assert_eq!(eval(&mut rt, &wv, "1"), "1");
}

fn close_child() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let child = rt
    .create_webview(w.id, pending_webview("child", &page("child")))
    .unwrap()
    .dispatcher;
  wait_title(&mut rt, &child, "child");
  child.close().unwrap();
  wait_for(&mut rt, "the webview to go", || {
    matches!(child.url(), Err(Error::WindowNotFound))
  });
  // The window stays.
  assert!(w.dispatcher.is_visible().unwrap());
  assert!(!events().iter().any(|e| matches!(e, Ev::ExitRequested(_))));
}

fn reparent() {
  let mut rt = runtime();
  let a = window(&rt, "a", builder());
  let b = window(&rt, "b", builder());
  let child = rt
    .create_webview(a.id, pending_webview("child", &page("moved")))
    .unwrap()
    .dispatcher;
  wait_title(&mut rt, &child, "moved");
  child.reparent(b.id).unwrap();
  // Reparenting to the same window is a no-op; an unknown window fails.
  child.reparent(b.id).unwrap();
  assert!(matches!(
    child.reparent(tauri_runtime::window::WindowId::from(999)),
    Err(Error::WindowNotFound)
  ));
  // Destroying the old window no longer affects it.
  a.dispatcher.destroy().unwrap();
  wait_window_event(&mut rt, "a", |e| *e == Win::Destroyed);
  assert_eq!(eval(&mut rt, &child, "document.title"), "\"moved\"");
  // Destroying the new one takes it down.
  b.dispatcher.destroy().unwrap();
  wait_window_event(&mut rt, "b", |e| *e == Win::Destroyed);
  assert!(matches!(child.url(), Err(Error::WindowNotFound)));
}

fn cookie_names(cookies: &[Cookie<'static>]) -> Vec<String> {
  let mut n: Vec<_> = cookies.iter().map(|c| c.name().to_string()).collect();
  n.sort();
  n
}

fn cookies() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("cookies"));
  wait_title(&mut rt, &wv, "cookies");
  wv.set_cookie(
    Cookie::build(("api", "1"))
      .domain("example.com")
      .path("/api")
      .build(),
  )
  .unwrap();
  wv.set_cookie(
    Cookie::build(("secure", "2"))
      .domain("example.com")
      .path("/")
      .secure(true)
      .build(),
  )
  .unwrap();
  wv.set_cookie(
    Cookie::build(("other", "3"))
      .domain("other.org")
      .path("/")
      .build(),
  )
  .unwrap();
  wait_for(&mut rt, "three cookies", || {
    wv.cookies().unwrap().len() >= 3
  });
  assert_eq!(
    cookie_names(&wv.cookies().unwrap()),
    ["api", "other", "secure"]
  );
  let api = wv
    .cookies()
    .unwrap()
    .into_iter()
    .find(|c| c.name() == "api")
    .unwrap();
  assert_eq!(api.value(), "1");

  let for_url = |u: &str| cookie_names(&wv.cookies_for_url(u.parse().unwrap()).unwrap());
  assert_eq!(for_url("https://example.com/api/x"), ["api", "secure"]);
  assert_eq!(for_url("http://sub.example.com/api"), ["api"]);
  assert_eq!(for_url("https://example.com/"), ["secure"]);
  assert_eq!(for_url("https://other.org/"), ["other"]);
  assert!(for_url("https://nothing.net/").is_empty());

  wv.delete_cookie(Cookie::build(("api", "")).domain("example.com").build())
    .unwrap();
  wait_for(&mut rt, "the deleted cookie", || {
    !cookie_names(&wv.cookies().unwrap()).contains(&"api".to_string())
  });
  assert_eq!(cookie_names(&wv.cookies().unwrap()), ["other", "secure"]);
}

fn clear_browsing_data() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("clear"));
  wait_title(&mut rt, &wv, "clear");
  wv.set_cookie(
    Cookie::build(("c", "1"))
      .domain("example.com")
      .path("/")
      .build(),
  )
  .unwrap();
  wait_for(&mut rt, "the cookie", || !wv.cookies().unwrap().is_empty());
  wv.clear_all_browsing_data().unwrap();
  wait_for(&mut rt, "no cookies", || wv.cookies().unwrap().is_empty());
}

fn scheme_origin() {
  let mut rt = runtime();
  let (_w, wv) = webview_window(&rt, "main", &page("origin"));
  wait_title(&mut rt, &wv, "origin");
  wv.eval_script(format!(
    "fetch({:?}, {{method: 'POST', body: 'hello'}}).then(r => r.text()).then(t => document.title = 'got:' + t)",
    format!("{SCHEME}://localhost/body/echo")
  ))
  .unwrap();
  wait_title(&mut rt, &wv, "got:hello");
  let reqs = REQUESTS.lock().unwrap().clone();
  let (label, method, _, origin) = reqs
    .iter()
    .find(|r| r.2.ends_with("/body/echo"))
    .expect("the fetch");
  assert_eq!(label, "main");
  assert_eq!(method, "POST");
  assert_eq!(origin.as_deref(), Some(&*format!("{SCHEME}://localhost")));
}

fn ipc() {
  let mut rt = runtime();
  let got = Arc::new(Mutex::new(Vec::new()));
  let g = got.clone();
  let mut pending = pending_webview("main", &page("ipc"));
  pending.ipc_handler = Some(Box::new(move |webview, request| {
    g.lock().unwrap().push((
      webview.label.clone(),
      request.uri().to_string(),
      request.into_body(),
    ));
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "ipc");
  wv.eval_script("window.ipc.postMessage('one'); window.ipc.postMessage('two')")
    .unwrap();
  let g = got.clone();
  wait_for(&mut rt, "two IPC messages", move || {
    g.lock().unwrap().len() == 2
  });
  let got = got.lock().unwrap();
  assert_eq!(got[0], ("main".into(), page("ipc"), "one".into()));
  assert_eq!(got[1].2, "two");
}

fn new_window_deny() {
  let mut rt = runtime();
  let asked = Arc::new(Mutex::new(Vec::new()));
  let a = asked.clone();
  let mut pending = pending_webview("main", &page("opener"));
  pending.new_window_handler = Some(Box::new(move |url, features| {
    a.lock().unwrap().push((
      url.to_string(),
      features.size(),
      features.opener().url.clone(),
    ));
    NewWindowResponse::Deny
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "opener");
  wv.eval_script(format!(
    "window.__popup = window.open({:?}, '_blank', 'width=320,height=200')",
    page("popup")
  ))
  .unwrap();
  let a = asked.clone();
  wait_for(&mut rt, "the new-window request", move || {
    !a.lock().unwrap().is_empty()
  });
  let (url, size, opener_url) = asked.lock().unwrap()[0].clone();
  assert_eq!(url, page("popup"));
  assert_eq!(opener_url, page("popup"));
  assert_eq!(size, Some(LogicalSize::new(320.0, 200.0)));
  pump(&mut rt, Duration::from_millis(50));
  assert!(
    !events()
      .iter()
      .any(|e| matches!(e, Ev::Window(l, _) if l != "main")),
    "no window was created"
  );
}

fn new_window_create() {
  let mut rt = runtime();
  let handle = rt.handle();
  let created = Arc::new(Mutex::new(None));
  let c = created.clone();
  let mut pending = pending_webview("main", &page("opener"));
  pending.new_window_handler = Some(Box::new(move |_url, features| {
    // Like tauri: build the target window synchronously, connected to its opener.
    let mut webview = pending_webview("popup", "about:blank");
    webview.opener = Some(features.into_opener());
    let mut window = PendingWindow::new(builder(), "popup").unwrap();
    window.set_webview(webview);
    let w = handle.create_window(window, None::<fn(RawWindow)>).unwrap();
    let id = w.id;
    *c.lock().unwrap() = Some(w);
    NewWindowResponse::Create { window_id: id }
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "opener");
  wv.eval_script(format!(
    "window.__popup = window.open({:?}); window.__popup.name = 'child'",
    page("popup")
  ))
  .unwrap();
  wait_for(&mut rt, "the popup window", || {
    created.lock().unwrap().is_some()
  });
  let popup = created.lock().unwrap().take().unwrap();
  let pwv = popup.webview.unwrap().webview.dispatcher;
  wait_title(&mut rt, &pwv, "popup");
  assert_eq!(pwv.url().unwrap(), page("popup"));
  // The popup is scriptable by (connected to) its opener.
  assert_eq!(eval(&mut rt, &pwv, "window.opener !== null"), "true");
  wait_js(&mut rt, &wv, "window.__popup.document.title", "\"popup\"");
}

fn download() {
  let mut rt = runtime();
  let dir = TempDir::new("download");
  let dest = dir.0.join("saved.bin");
  let events_seen = Arc::new(Mutex::new(Vec::new()));
  let (e, d) = (events_seen.clone(), dest.clone());
  let mut pending = pending_webview("main", &page("dl"));
  pending.download_handler = Some(Arc::new(move |event| {
    match event {
      DownloadEvent::Requested { url, destination } => {
        e.lock().unwrap().push(format!("requested {url}"));
        *destination = d.clone();
      }
      DownloadEvent::Finished { url, path, success } => {
        e.lock()
          .unwrap()
          .push(format!("finished {url} {path:?} {success}"));
      }
    }
    true
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "dl");
  let url = format!("{SCHEME}://localhost/download/file.bin");
  wv.navigate(url.parse().unwrap()).unwrap();
  let e = events_seen.clone();
  wait_for(&mut rt, "the finished download", move || {
    e.lock().unwrap().iter().any(|s| s.starts_with("finished"))
  });
  let seen = events_seen.lock().unwrap().clone();
  assert_eq!(seen[0], format!("requested {url}"));
  assert_eq!(seen[1], format!("finished {url} Some({dest:?}) true"));
  assert_eq!(std::fs::read(&dest).unwrap(), b"downloaded bytes");
}

fn download_cancel() {
  let mut rt = runtime();
  let dir = TempDir::new("download-cancel");
  let asked = Arc::new(AtomicUsize::new(0));
  let a = asked.clone();
  let d = dir.0.join("never.bin");
  let mut pending = pending_webview("main", &page("dl"));
  pending.download_handler = Some(Arc::new(move |event| {
    if let DownloadEvent::Requested { destination, .. } = event {
      *destination = d.clone();
      a.fetch_add(1, Ordering::AcqRel);
    }
    false
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "dl");
  wv.navigate(
    format!("{SCHEME}://localhost/download/x.bin")
      .parse()
      .unwrap(),
  )
  .unwrap();
  wait_for(&mut rt, "the download request", || {
    asked.load(Ordering::Acquire) == 1
  });
  pump(&mut rt, Duration::from_millis(200));
  assert!(!dir.0.join("never.bin").exists());
}

fn renderer_killed() {
  let mut rt = runtime();
  let reasons = Arc::new(Mutex::new(Vec::new()));
  let r = reasons.clone();
  let mut pending = pending_webview("main", &page("alive"));
  pending.on_web_content_process_terminate_handler = Some(Box::new(move |t| {
    r.lock().unwrap().push(t.reason);
  }));
  let (_w, wv) = webview_window_with(&rt, "main", builder(), pending);
  wait_title(&mut rt, &wv, "alive");
  wv.navigate("chrome://kill".parse().unwrap()).unwrap();
  let r = reasons.clone();
  wait_for(&mut rt, "the terminate handler", move || {
    !r.lock().unwrap().is_empty()
  });
  let reason = reasons.lock().unwrap()[0];
  assert!(
    matches!(
      reason,
      WebContentProcessTerminationReason::Killed
        | WebContentProcessTerminationReason::Crashed
        | WebContentProcessTerminationReason::Abnormal
    ),
    "{reason:?}"
  );
  // The handler took over (no automatic reload); the app can load a page again.
  wv.navigate(page("revived").parse().unwrap()).unwrap();
  wait_title(&mut rt, &wv, "revived");
}

fn data_directory() {
  let mut rt = runtime();
  let dir = TempDir::new("profile");
  let mut pending = pending_webview("persistent", &page("persistent"));
  pending.webview_attributes.data_directory = Some(dir.0.clone());
  let (_a, a) = webview_window_with(&rt, "persistent", builder(), pending);
  let mut pending = pending_webview("incognito", &page("incognito"));
  pending.webview_attributes.incognito = true;
  let (_b, b) = webview_window_with(&rt, "incognito", builder(), pending);
  wait_title(&mut rt, &a, "persistent");
  wait_title(&mut rt, &b, "incognito");
  // Separate profiles: a cookie in one isn't in the other.
  a.set_cookie(
    Cookie::build(("only_a", "1"))
      .domain("example.com")
      .path("/")
      .build(),
  )
  .unwrap();
  wait_for(&mut rt, "the cookie", || {
    cookie_names(&a.cookies().unwrap()).contains(&"only_a".to_string())
  });
  assert!(!cookie_names(&b.cookies().unwrap()).contains(&"only_a".to_string()));
}

fn closed_webview() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let child = rt
    .create_webview(w.id, pending_webview("child", "about:blank"))
    .unwrap()
    .dispatcher;
  child.close().unwrap();
  wait_for(&mut rt, "the webview to go", || {
    matches!(child.url(), Err(Error::WindowNotFound))
  });
  assert!(matches!(child.bounds(), Err(Error::WindowNotFound)));
  assert!(matches!(child.cookies(), Err(Error::WindowNotFound)));
  assert!(matches!(
    child.is_devtools_open(),
    Err(Error::WindowNotFound)
  ));
  // Fire-and-forget calls are no-ops.
  child.navigate("about:blank".parse().unwrap()).unwrap();
  child.eval_script("1").unwrap();
  child.set_zoom(2.0).unwrap();
  child.open_devtools();
  child.close().unwrap();
  pump(&mut rt, Duration::from_millis(20));
  assert!(w.dispatcher.is_visible().unwrap());
}
