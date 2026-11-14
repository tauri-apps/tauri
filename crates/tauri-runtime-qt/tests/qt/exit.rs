// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The run loop and the app lifecycle: `run_return`, `Ready`/`Exit`, `ExitRequested` (`code`
//! `None` when the last window goes, `Some` from `request_exit`, preventing it), close/destroy
//! semantics, and dispatchers after the loop ended.

use std::{
  sync::{Arc, Mutex},
  thread,
  time::{Duration, Instant},
};

use qtshim::test_harness::Case;
use tauri_runtime::{
  Error, EventLoopProxy, Runtime, RuntimeHandle, WebviewDispatch, WindowDispatch,
  window::WindowBuilder,
  window::{PendingWindow, RawWindow},
};

use crate::common::*;

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass(
      "exit: run_return emits Ready first and returns the request_exit code",
      request_exit_code,
    ),
    Case::pass(
      "exit: events queued before run_return are delivered after Ready",
      queued_before_run,
    ),
    Case::pass(
      "exit: closing the last window requests exit with code None",
      last_window_closed,
    ),
    Case::pass(
      "exit: closing one of two windows doesn't request exit",
      one_of_two_closed,
    ),
    Case::pass(
      "exit: a prevented exit request keeps the loop running",
      prevent_exit,
    ),
    Case::pass(
      "exit: a prevented close keeps the window; destroy skips CloseRequested",
      prevent_close_then_destroy,
    ),
    Case::pass(
      "exit: request_exit and user events from another thread",
      request_exit_other_thread,
    ),
    Case::pass(
      "exit: dispatchers fail with EventLoopClosed after run_return",
      dispatchers_after_run_return,
    ),
    Case::pass(
      "exit: a thread blocked on the main loop is released when it exits",
      blocked_thread_released,
    ),
    Case::pass(
      "exit: closing the last window outside run_return requests exit",
      close_with_run_iteration,
    ),
  ]
}

/// Kill the test if `run_return` doesn't return in time (instead of waiting for the harness).
fn watchdog(rt: &Rt) {
  let handle = rt.handle();
  thread::spawn(move || {
    thread::sleep(WAIT);
    let _ = handle.request_exit(99);
  });
}

fn request_exit_code() {
  let rt = runtime();
  watchdog(&rt);
  let handle = rt.handle();
  let code = rt.run_return(move |e| {
    if record(e) == Some(Ev::Ready) {
      handle.request_exit(7).unwrap();
    }
  });
  assert_eq!(code, 7);
  let ev = events();
  assert_eq!(ev.first(), Some(&Ev::Ready));
  assert_eq!(&ev[ev.len() - 2..], [Ev::ExitRequested(Some(7)), Ev::Exit]);
  assert!(!qtshim::app::is_running(), "run_return shuts Qt down");
}

fn queued_before_run() {
  let rt = runtime();
  watchdog(&rt);
  rt.create_proxy().send_event(Msg::Ping(1)).unwrap();
  let handle = rt.handle();
  let code = rt.run_return(move |e| {
    if record(e) == Some(Ev::User(Msg::Ping(1))) {
      handle.request_exit(0).unwrap();
    }
  });
  assert_eq!(code, 0);
  let ev = events();
  assert_eq!(ev[0], Ev::Ready, "{ev:?}");
  assert!(ev.contains(&Ev::User(Msg::Ping(1))));
}

fn last_window_closed() {
  let rt = runtime();
  watchdog(&rt);
  let w = window(&rt, "main", builder());
  let code = rt.run_return(move |e| {
    if record(e) == Some(Ev::Ready) {
      w.dispatcher.close().unwrap();
    }
  });
  assert_eq!(code, 0);
  let ev = events();
  let tail: Vec<_> = ev
    .iter()
    .filter(|e| {
      !matches!(
        e,
        Ev::Window(_, Win::Resized(..) | Win::Moved(..) | Win::Focused(_))
      )
    })
    .cloned()
    .collect();
  assert_eq!(
    tail,
    [
      Ev::Ready,
      Ev::Window("main".into(), Win::CloseRequested),
      Ev::Window("main".into(), Win::Destroyed),
      Ev::ExitRequested(None),
      Ev::Exit,
    ]
  );
}

fn one_of_two_closed() {
  let mut rt = runtime();
  let a = window(&rt, "a", builder());
  let _b = window(&rt, "b", builder());
  a.dispatcher.close().unwrap();
  wait_window_event(&mut rt, "a", |e| *e == Win::Destroyed);
  pump(&mut rt, Duration::from_millis(50));
  assert!(
    !events().iter().any(|e| matches!(e, Ev::ExitRequested(_))),
    "{:?}",
    events()
  );
  assert!(matches!(a.dispatcher.title(), Err(Error::WindowNotFound)));
}

fn prevent_exit() {
  let rt = runtime();
  watchdog(&rt);
  prevent_exits(1);
  let handle = rt.handle();
  let w = window(&rt, "main", builder());
  let code = rt.run_return(move |e| match record(e) {
    // The last window goes, the exit is prevented, and the loop keeps running...
    Some(Ev::Ready) => w.dispatcher.close().unwrap(),
    // ...until the app asks to exit itself.
    Some(Ev::ExitRequested(None)) => handle.create_proxy().send_event(Msg::Ping(1)).unwrap(),
    Some(Ev::User(Msg::Ping(1))) => handle.request_exit(3).unwrap(),
    _ => {}
  });
  assert_eq!(code, 3);
  let exits: Vec<_> = events()
    .into_iter()
    .filter(|e| matches!(e, Ev::ExitRequested(_) | Ev::Exit | Ev::User(_)))
    .collect();
  assert_eq!(
    exits,
    [
      Ev::ExitRequested(None),
      Ev::User(Msg::Ping(1)),
      Ev::ExitRequested(Some(3)),
      Ev::Exit
    ]
  );
}

fn prevent_close_then_destroy() {
  let mut rt = runtime();
  prevent_close("main", true);
  let w = window(&rt, "main", builder().title("kept"));
  w.dispatcher.close().unwrap();
  wait_window_event(&mut rt, "main", |e| *e == Win::CloseRequested);
  pump(&mut rt, Duration::from_millis(50));
  assert_eq!(w.dispatcher.title().unwrap(), "kept");
  assert!(!window_events("main").contains(&Win::Destroyed));

  clear_events();
  w.dispatcher.destroy().unwrap();
  wait_for(&mut rt, "the exit request", || {
    has_event(&Ev::ExitRequested(None))
  });
  assert_eq!(window_events("main"), [Win::Destroyed]);
  assert!(matches!(w.dispatcher.title(), Err(Error::WindowNotFound)));
  // Closing or destroying it again is harmless.
  w.dispatcher.close().unwrap();
  w.dispatcher.destroy().unwrap();
  pump(&mut rt, Duration::from_millis(50));
  assert_eq!(window_events("main"), [Win::Destroyed]);
}

fn request_exit_other_thread() {
  let rt = runtime();
  watchdog(&rt);
  let handle = rt.handle();
  let proxy = rt.create_proxy();
  let code = rt.run_return(move |e| {
    if record(e) == Some(Ev::Ready) {
      let (handle, proxy) = (handle.clone(), proxy.clone());
      thread::spawn(move || {
        proxy.send_event(Msg::Ping(1)).unwrap();
        handle.request_exit(42).unwrap();
      });
    }
  });
  assert_eq!(code, 42);
  let ev = events();
  let i = ev
    .iter()
    .position(|e| *e == Ev::User(Msg::Ping(1)))
    .unwrap();
  let j = ev
    .iter()
    .position(|e| *e == Ev::ExitRequested(Some(42)))
    .unwrap();
  assert!(i < j, "{ev:?}");
}

fn dispatchers_after_run_return() {
  let rt = runtime();
  watchdog(&rt);
  let (w, wv) = webview_window(&rt, "main", "about:blank");
  let handle = rt.handle();
  let proxy = rt.create_proxy();
  let h = handle.clone();
  let code = rt.run_return(move |e| {
    if record(e) == Some(Ev::Ready) {
      h.request_exit(0).unwrap();
    }
  });
  assert_eq!(code, 0);

  // From another thread (must not hang)...
  let (wd, wv2, p2, h2) = (
    w.dispatcher.clone(),
    wv.clone(),
    proxy.clone(),
    handle.clone(),
  );
  let t = thread::spawn(move || check_closed(&wd, &wv2, &p2, &h2));
  let deadline = Instant::now() + Duration::from_secs(10);
  while !t.is_finished() {
    assert!(Instant::now() < deadline, "a dispatcher call hung");
    thread::sleep(Duration::from_millis(5));
  }
  t.join().unwrap();
  // ...and from the main thread.
  check_closed(&w.dispatcher, &wv, &proxy, &handle);
}

fn check_closed(
  w: &<Rt as Runtime<Msg>>::WindowDispatcher,
  wv: &Webview,
  proxy: &<Rt as Runtime<Msg>>::EventLoopProxy,
  handle: &<Rt as Runtime<Msg>>::Handle,
) {
  let closed = |r: tauri_runtime::Result<()>| matches!(r, Err(Error::EventLoopClosed));
  assert!(matches!(w.title(), Err(Error::EventLoopClosed)));
  assert!(closed(w.set_title("x")));
  assert!(closed(w.close()));
  assert!(closed(w.destroy()));
  assert!(closed(wv.eval_script("1")));
  assert!(matches!(wv.url(), Err(Error::EventLoopClosed)));
  assert!(closed(proxy.send_event(Msg::Ping(0))));
  assert!(closed(handle.request_exit(1)));
  assert!(matches!(
    handle.primary_monitor(),
    Err(Error::EventLoopClosed)
  ));
  assert!(matches!(
    handle.create_window(
      PendingWindow::new(builder(), "late").unwrap(),
      None::<fn(RawWindow)>
    ),
    Err(Error::EventLoopClosed)
  ));
}

fn blocked_thread_released() {
  let rt = runtime();
  watchdog(&rt);
  let w = window(&rt, "main", builder());
  let result = Arc::new(Mutex::new(None));
  let r = result.clone();
  let handle = rt.handle();
  let code = rt.run_return(move |e| {
    if record(e) == Some(Ev::Ready) {
      // A thread asks the main loop for something right as the app exits: it must get an
      // answer or an error, not block forever.
      let (dispatcher, r, handle) = (w.dispatcher.clone(), r.clone(), handle.clone());
      thread::spawn(move || {
        handle.request_exit(5).unwrap();
        let mut last = Ok(String::new());
        for _ in 0..200 {
          last = dispatcher.title();
          if last.is_err() {
            break;
          }
        }
        *r.lock().unwrap() = Some(last.is_err());
      });
    }
  });
  assert_eq!(code, 5);
  let deadline = Instant::now() + Duration::from_secs(10);
  while result.lock().unwrap().is_none() {
    assert!(Instant::now() < deadline, "the thread is still blocked");
    thread::sleep(Duration::from_millis(5));
  }
}

fn close_with_run_iteration() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  w.dispatcher.close().unwrap();
  wait_for(&mut rt, "the exit request", || {
    has_event(&Ev::ExitRequested(None))
  });
  let main: Vec<_> = window_events("main")
    .into_iter()
    .filter(|e| matches!(e, Win::CloseRequested | Win::Destroyed))
    .collect();
  assert_eq!(main, [Win::CloseRequested, Win::Destroyed]);
}
