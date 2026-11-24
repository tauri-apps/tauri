// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `qtshim::app`: initialization, the event loop, dispatch/post, the event channel and app-wide
//! queries.

use std::{
  cell::{Cell, RefCell},
  rc::Rc,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
  },
  thread,
  time::Duration,
};

use qtshim::test_harness::Case;
use qtshim::{
  app::{self, ColorScheme, EventKind, NativeHandle, Reply},
  webengine::Window,
};

use crate::common::{clear_events, has_event, init, pump, record_events, wait_for};

pub fn cases() -> Vec<Case> {
  vec![
    // init / exec / quit
    Case::pass("app::init describes the platform", init_describes_platform),
    Case::pass("app::init twice on the main thread is a no-op", init_twice),
    Case::pass("app::init without an app id", init_without_app_id),
    Case::pass("app::init with a NUL in the app id", init_app_id_with_nul),
    Case::pass(
      "app::is_running before init, while running and after shutdown",
      is_running_lifecycle,
    ),
    Case::pass(
      "app::is_running is false on other threads",
      is_running_other_thread,
    ),
    Case::pass(
      "app::exec returns the code passed to quit",
      exec_returns_quit_code,
    ),
    Case::pass("app::exec can run again after returning", exec_runs_again),
    Case::pass(
      "app::quit dispatched from another thread stops exec",
      quit_from_other_thread,
    ),
    Case::pass(
      "app::exec inside the event loop returns at once",
      nested_exec,
    ),
    Case::pass(
      "app::process_events runs queued closures without exec",
      process_events_runs_queue,
    ),
    Case::pass(
      "app::process_events inside a callback runs nested closures",
      process_events_nested,
    ),
    // dispatch
    Case::pass(
      "app::dispatch from many threads runs every closure",
      dispatch_many_threads,
    ),
    Case::pass(
      "app::dispatch keeps the order of each thread",
      dispatch_order_per_thread,
    ),
    Case::pass(
      "app::dispatch runs closures on the main thread",
      dispatch_runs_on_main,
    ),
    Case::pass(
      "app::dispatch before init returns false and drops the closure",
      dispatch_before_init,
    ),
    Case::pass(
      "app::dispatch after shutdown returns false and drops the closure",
      dispatch_after_shutdown,
    ),
    Case::pass(
      "app::dispatch from another thread after shutdown returns false",
      dispatch_after_shutdown_other_thread,
    ),
    Case::pass(
      "app::shutdown drops dispatched closures that never ran",
      shutdown_drops_dispatched,
    ),
    Case::pass(
      "app::shutdown wakes threads waiting on a dropped dispatched closure",
      shutdown_wakes_waiting_thread,
    ),
    Case::pass(
      "app::dispatch racing shutdown never leaks a closure",
      dispatch_racing_shutdown,
    ),
    Case::pass(
      "app::shutdown: a dropped closure may call dispatch in its drop",
      shutdown_drop_calls_dispatch,
    ),
    Case::pass(
      "app::dispatch: a panicking closure doesn't stop the app",
      dispatch_panic_caught,
    ),
    Case::pass(
      "app::dispatch: a panicking closure on another thread's request is caught",
      dispatch_panic_from_other_thread,
    ),
    Case::pass(
      "app::dispatch from inside a dispatched closure",
      dispatch_nested,
    ),
    Case::pass(
      "app::post from inside a dispatched closure and vice versa",
      dispatch_post_mixed,
    ),
    // post
    Case::pass("app::post runs closures in order", post_in_order),
    Case::pass(
      "app::post from inside a posted closure runs after queued ones",
      post_nested_order,
    ),
    Case::pass(
      "app::post: a panicking closure doesn't stop the app",
      post_panic_caught,
    ),
    Case::pass(
      "app::shutdown drops posted closures that never ran",
      shutdown_drops_posted,
    ),
    Case::pass(
      "app::post closures capturing Qt handles are dropped safely at shutdown",
      shutdown_drops_posted_with_window,
    ),
    // events
    Case::pass(
      "app::set_event_handler receives AboutToBlock",
      handler_receives_about_to_block,
    ),
    Case::pass(
      "app::set_event_handler replaces the previous handler",
      handler_replacement,
    ),
    Case::pass(
      "app::set_event_handler from inside the handler",
      handler_replaced_from_inside,
    ),
    Case::pass("app::event handler panics are caught", handler_panic_caught),
    Case::pass(
      "app::event handler is re-entered by nested events",
      handler_reentrant,
    ),
    Case::pass(
      "app::window events reach the handler with the window id",
      window_events_reach_handler,
    ),
    Case::pass(
      "app::the old handler is dropped when replaced and at shutdown",
      handler_dropped,
    ),
    // queries
    Case::pass(
      "app::color_scheme round-trips set_color_scheme",
      color_scheme_round_trip,
    ),
    Case::pass(
      "app::set_color_scheme fires a Theme event",
      color_scheme_theme_event,
    ),
    Case::pass("app::screens offscreen are sane", screens_sane),
    Case::pass("app::cursor_position offscreen", cursor_position),
    Case::pass("app::native_display doesn't crash", native_display_no_crash),
    Case::pass("app::native_display offscreen is None", native_display),
    Case::pass(
      "app::launcher_update without a D-Bus listener",
      launcher_update,
    ),
  ]
}

// ---------------------------------------------------------------------------------------------
// helpers

/// Counts its drops.
struct DropCount(Arc<AtomicUsize>);

impl Drop for DropCount {
  fn drop(&mut self) {
    self.0.fetch_add(1, Ordering::SeqCst);
  }
}

/// Sets a flag on drop (main thread only).
struct DropFlag(Rc<Cell<bool>>);

impl Drop for DropFlag {
  fn drop(&mut self) {
    self.0.set(true);
  }
}

/// Like [`init`], but with the GTK 3 platform theme, the one installed theme that honours
/// [`app::set_color_scheme`] offscreen (the offscreen platform's own theme ignores it).
fn init_with_theme() {
  // SAFETY: the case's process is still single-threaded.
  unsafe { std::env::set_var("QT_QPA_PLATFORMTHEME", "gtk3") };
  init();
}

// ---------------------------------------------------------------------------------------------
// init / exec / quit

fn init_describes_platform() {
  let description = app::init(Some("qtshim-test"));
  assert!(app::is_running());
  assert!(description.contains("offscreen"), "{description}");
}

fn init_twice() {
  let first = app::init(Some("qtshim-test"));
  let second = app::init(Some("ignored"));
  assert!(app::is_running());
  assert_eq!(first, second);
}

fn init_without_app_id() {
  let description = app::init(None);
  assert!(description.contains(" on "), "{description}");
  assert!(app::is_running());
}

fn init_app_id_with_nul() {
  app::init(Some("qtshim\0test"));
  assert!(app::is_running());
}

fn is_running_lifecycle() {
  assert!(!app::is_running());
  init();
  assert!(app::is_running());
  app::shutdown();
  assert!(!app::is_running());
}

fn is_running_other_thread() {
  init();
  let other = thread::spawn(app::is_running).join().unwrap();
  assert!(!other);
  assert!(app::is_running());
}

fn exec_returns_quit_code() {
  init();
  app::post(|| app::quit(7));
  assert_eq!(app::exec(), 7);
}

fn exec_runs_again() {
  init();
  app::post(|| app::quit(1));
  assert_eq!(app::exec(), 1);
  app::post(|| app::quit(2));
  assert_eq!(app::exec(), 2);
}

fn quit_from_other_thread() {
  init();
  let t = thread::spawn(|| assert!(app::dispatch(|| app::quit(42))));
  assert_eq!(app::exec(), 42);
  t.join().unwrap();
}

fn nested_exec() {
  init();
  let inner = Rc::new(Cell::new(None));
  let i = inner.clone();
  app::post(move || {
    // Qt refuses to run `exec` twice: it warns and returns -1.
    i.set(Some(app::exec()));
    app::quit(3);
  });
  assert_eq!(app::exec(), 3);
  assert_eq!(inner.get(), Some(-1));
}

/// Call `process_events` until `cond` holds (posted events may need a few rounds).
fn process_until(mut cond: impl FnMut() -> bool) -> bool {
  for _ in 0..100 {
    if cond() {
      return true;
    }
    app::process_events();
    thread::sleep(Duration::from_millis(1));
  }
  cond()
}

fn process_events_runs_queue() {
  init();
  let ran = Rc::new(Cell::new(0));
  for _ in 0..3 {
    let r = ran.clone();
    app::post(move || r.set(r.get() + 1));
  }
  let flag = Arc::new(AtomicBool::new(false));
  let f = flag.clone();
  assert!(app::dispatch(move || f.store(true, Ordering::SeqCst)));
  assert!(process_until(
    || ran.get() == 3 && flag.load(Ordering::SeqCst)
  ));
}

fn process_events_nested() {
  init();
  let log = Rc::new(RefCell::new(Vec::new()));
  let l = log.clone();
  app::post(move || {
    l.borrow_mut().push("outer start");
    let l2 = l.clone();
    app::post(move || l2.borrow_mut().push("inner"));
    // Runs the closure just posted, from inside this one.
    let l3 = l.clone();
    assert!(process_until(|| l3.borrow().contains(&"inner")));
    l.borrow_mut().push("outer end");
  });
  let l = log.clone();
  assert!(process_until(move || l.borrow().len() == 3));
  assert_eq!(*log.borrow(), ["outer start", "inner", "outer end"]);
}

// ---------------------------------------------------------------------------------------------
// dispatch

fn dispatch_many_threads() {
  init();
  let count = Arc::new(AtomicUsize::new(0));
  let main = thread::current().id();
  let handles: Vec<_> = (0..8)
    .map(|_| {
      let count = count.clone();
      thread::spawn(move || {
        for _ in 0..100 {
          let count = count.clone();
          assert!(app::dispatch(move || {
            // A panic here would only be logged: count main-thread runs instead.
            if thread::current().id() == main {
              count.fetch_add(1, Ordering::SeqCst);
            }
          }));
        }
      })
    })
    .collect();
  for h in handles {
    h.join().unwrap();
  }
  let c = count.clone();
  wait_for("800 closures run on the main thread", move || {
    c.load(Ordering::SeqCst) == 800
  });
}

fn dispatch_order_per_thread() {
  init();
  const THREADS: usize = 4;
  const PER: usize = 200;
  let log = Arc::new(Mutex::new(Vec::new()));
  let handles: Vec<_> = (0..THREADS)
    .map(|t| {
      let log = log.clone();
      thread::spawn(move || {
        for i in 0..PER {
          let log = log.clone();
          assert!(app::dispatch(move || log.lock().unwrap().push((t, i))));
        }
      })
    })
    .collect();
  for h in handles {
    h.join().unwrap();
  }
  let l = log.clone();
  wait_for("every dispatched closure", move || {
    l.lock().unwrap().len() == THREADS * PER
  });
  let log = log.lock().unwrap();
  for t in 0..THREADS {
    let seen: Vec<_> = log
      .iter()
      .filter(|(tt, _)| *tt == t)
      .map(|(_, i)| *i)
      .collect();
    assert_eq!(seen, (0..PER).collect::<Vec<_>>(), "thread {t}");
  }
}

fn dispatch_runs_on_main() {
  init();
  let main = thread::current().id();
  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    assert!(app::dispatch(move || {
      tx.send((thread::current().id(), app::is_running()))
        .unwrap();
    }));
  })
  .join()
  .unwrap();
  let got = Rc::new(RefCell::new(None));
  let g = got.clone();
  wait_for("the dispatched closure", move || {
    if let Ok(v) = rx.try_recv() {
      *g.borrow_mut() = Some(v);
    }
    g.borrow().is_some()
  });
  assert_eq!(*got.borrow(), Some((main, true)));
}

fn dispatch_before_init() {
  let drops = Arc::new(AtomicUsize::new(0));
  let guard = DropCount(drops.clone());
  assert!(!app::dispatch(move || {
    let _g = guard;
    unreachable!()
  }));
  assert_eq!(drops.load(Ordering::SeqCst), 1);
  let d = drops.clone();
  thread::spawn(move || {
    let guard = DropCount(d);
    assert!(!app::dispatch(move || {
      let _g = guard;
      unreachable!()
    }));
  })
  .join()
  .unwrap();
  assert_eq!(drops.load(Ordering::SeqCst), 2);
  // The application can still be created afterwards.
  init();
  assert!(app::is_running());
}

fn dispatch_after_shutdown() {
  init();
  app::shutdown();
  let drops = Arc::new(AtomicUsize::new(0));
  let guard = DropCount(drops.clone());
  assert!(!app::dispatch(move || {
    let _g = guard;
    unreachable!()
  }));
  assert_eq!(drops.load(Ordering::SeqCst), 1);
}

fn dispatch_after_shutdown_other_thread() {
  init();
  // A thread that outlives the application.
  let (go_tx, go_rx) = mpsc::channel::<()>();
  let t = thread::spawn(move || {
    go_rx.recv().unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let guard = DropCount(drops.clone());
    let queued = app::dispatch(move || {
      let _g = guard;
      unreachable!()
    });
    (queued, drops.load(Ordering::SeqCst))
  });
  app::shutdown();
  go_tx.send(()).unwrap();
  assert_eq!(t.join().unwrap(), (false, 1));
}

fn shutdown_drops_dispatched() {
  init();
  let drops = Arc::new(AtomicUsize::new(0));
  let ran = Arc::new(AtomicUsize::new(0));
  for _ in 0..10 {
    let guard = DropCount(drops.clone());
    let ran = ran.clone();
    assert!(app::dispatch(move || {
      let _g = guard;
      ran.fetch_add(1, Ordering::SeqCst);
    }));
  }
  assert_eq!(drops.load(Ordering::SeqCst), 0);
  app::shutdown();
  assert_eq!(drops.load(Ordering::SeqCst), 10);
  assert_eq!(ran.load(Ordering::SeqCst), 0);
}

fn shutdown_wakes_waiting_thread() {
  init();
  let (queued_tx, queued_rx) = mpsc::channel();
  let waiter = thread::spawn(move || {
    let (tx, rx) = mpsc::channel::<u32>();
    assert!(app::dispatch(move || tx.send(1).unwrap()));
    queued_tx.send(()).unwrap();
    // Never answered: the closure must be dropped (closing the channel), not leaked.
    rx.recv()
  });
  queued_rx.recv().unwrap();
  app::shutdown();
  assert!(waiter.join().unwrap().is_err());
}

fn dispatch_racing_shutdown() {
  init();
  let attempts = Arc::new(AtomicUsize::new(0));
  let drops = Arc::new(AtomicUsize::new(0));
  let ran = Arc::new(AtomicUsize::new(0));
  let handles: Vec<_> = (0..6)
    .map(|_| {
      let (attempts, drops, ran) = (attempts.clone(), drops.clone(), ran.clone());
      thread::spawn(move || {
        loop {
          attempts.fetch_add(1, Ordering::SeqCst);
          let guard = DropCount(drops.clone());
          let ran = ran.clone();
          if !app::dispatch(move || {
            let _g = guard;
            ran.fetch_add(1, Ordering::SeqCst);
          }) {
            break;
          }
        }
      })
    })
    .collect();
  // Let some of them run, then shut down while the threads keep dispatching.
  while ran.load(Ordering::SeqCst) < 100 {
    app::process_events();
  }
  app::shutdown();
  for h in handles {
    h.join().unwrap();
  }
  // Every closure was either run or dropped without running, exactly once.
  assert_eq!(
    drops.load(Ordering::SeqCst),
    attempts.load(Ordering::SeqCst)
  );
}

fn shutdown_drop_calls_dispatch() {
  struct DispatchOnDrop(Arc<AtomicUsize>);
  impl Drop for DispatchOnDrop {
    fn drop(&mut self) {
      // The application is going away: this must neither deadlock nor queue anything.
      let refused = !app::dispatch(|| unreachable!());
      if refused {
        self.0.fetch_add(1, Ordering::SeqCst);
      }
    }
  }
  init();
  let refused = Arc::new(AtomicUsize::new(0));
  let guard = DispatchOnDrop(refused.clone());
  assert!(app::dispatch(move || drop(guard)));
  let guard = DispatchOnDrop(refused.clone());
  app::post(move || drop(guard));
  app::shutdown();
  assert_eq!(refused.load(Ordering::SeqCst), 2);
}

fn dispatch_panic_caught() {
  init();
  assert!(app::dispatch(|| panic!("boom in a dispatched closure")));
  let flag = Arc::new(AtomicBool::new(false));
  let f = flag.clone();
  assert!(app::dispatch(move || f.store(true, Ordering::SeqCst)));
  wait_for("the closure after the panicking one", move || {
    flag.load(Ordering::SeqCst)
  });
  assert!(app::is_running());
}

fn dispatch_panic_from_other_thread() {
  init();
  let flag = Arc::new(AtomicBool::new(false));
  let f = flag.clone();
  thread::spawn(move || {
    for i in 0..10 {
      assert!(app::dispatch(move || panic!("boom {i}")));
    }
    assert!(app::dispatch(move || f.store(true, Ordering::SeqCst)));
  })
  .join()
  .unwrap();
  wait_for("the closure after the panicking ones", move || {
    flag.load(Ordering::SeqCst)
  });
}

fn dispatch_nested() {
  init();
  let depth = Arc::new(AtomicUsize::new(0));
  fn again(depth: Arc<AtomicUsize>) {
    if depth.fetch_add(1, Ordering::SeqCst) < 50 {
      assert!(app::dispatch(move || again(depth)));
    }
  }
  let d = depth.clone();
  assert!(app::dispatch(move || again(d)));
  wait_for("50 nested dispatches", move || {
    depth.load(Ordering::SeqCst) > 50
  });
}

fn dispatch_post_mixed() {
  init();
  let log = Arc::new(Mutex::new(Vec::new()));
  let l = log.clone();
  assert!(app::dispatch(move || {
    l.lock().unwrap().push("dispatched");
    let l = l.clone();
    app::post(move || {
      l.lock().unwrap().push("posted");
      let l = l.clone();
      assert!(app::dispatch(move || l
        .lock()
        .unwrap()
        .push("dispatched again")));
    });
  }));
  let l = log.clone();
  wait_for("all three closures", move || l.lock().unwrap().len() == 3);
  assert_eq!(
    *log.lock().unwrap(),
    ["dispatched", "posted", "dispatched again"]
  );
}

// ---------------------------------------------------------------------------------------------
// post

fn post_in_order() {
  init();
  let log = Rc::new(RefCell::new(Vec::new()));
  for i in 0..10 {
    let log = log.clone();
    app::post(move || log.borrow_mut().push(i));
  }
  let log2 = log.clone();
  wait_for("posted closures", move || log2.borrow().len() == 10);
  assert_eq!(*log.borrow(), (0..10).collect::<Vec<_>>());
}

fn post_nested_order() {
  init();
  let log = Rc::new(RefCell::new(Vec::new()));
  let l = log.clone();
  app::post(move || {
    l.borrow_mut().push("a");
    let l = l.clone();
    app::post(move || l.borrow_mut().push("c"));
  });
  let l = log.clone();
  app::post(move || l.borrow_mut().push("b"));
  let l = log.clone();
  wait_for("three posted closures", move || l.borrow().len() == 3);
  assert_eq!(*log.borrow(), ["a", "b", "c"]);
}

fn post_panic_caught() {
  init();
  let ran = Rc::new(Cell::new(false));
  app::post(|| panic!("boom in a posted closure"));
  let r = ran.clone();
  app::post(move || r.set(true));
  wait_for("the closure after the panicking one", move || ran.get());
}

fn shutdown_drops_posted() {
  init();
  let dropped = Rc::new(Cell::new(false));
  let ran = Rc::new(Cell::new(false));
  let guard = DropFlag(dropped.clone());
  let r = ran.clone();
  app::post(move || {
    let _g = guard;
    r.set(true);
  });
  assert!(!dropped.get());
  app::shutdown();
  assert!(dropped.get());
  assert!(!ran.get());
}

fn shutdown_drops_posted_with_window() {
  init();
  let window = Window::new(1, false);
  window.set_visible(true);
  let widget = window.widget();
  app::post(move || {
    let _keep = (&window, &widget);
    unreachable!();
  });
  // Dropping the closure drops `window` and `widget` after Qt stopped being usable: both must
  // leave Qt alone (shutdown destroys the window itself).
  app::shutdown();
}

// ---------------------------------------------------------------------------------------------
// events

fn handler_receives_about_to_block() {
  init();
  let n = Rc::new(Cell::new(0));
  let c = n.clone();
  app::set_event_handler(move |e| {
    if e.kind == EventKind::AboutToBlock {
      assert_eq!((e.window, e.view), (0, 0));
      c.set(c.get() + 1);
    }
    Reply::default()
  });
  wait_for("AboutToBlock", move || n.get() > 0);
}

fn handler_replacement() {
  init();
  let first = Rc::new(Cell::new(0));
  let second = Rc::new(Cell::new(0));
  let f = first.clone();
  app::set_event_handler(move |_| {
    f.set(f.get() + 1);
    Reply::default()
  });
  let f = first.clone();
  wait_for("events to the first handler", move || f.get() > 0);
  let s = second.clone();
  app::set_event_handler(move |_| {
    s.set(s.get() + 1);
    Reply::default()
  });
  let before = first.get();
  let s = second.clone();
  wait_for("events to the second handler", move || s.get() > 2);
  assert_eq!(first.get(), before);
}

fn handler_replaced_from_inside() {
  init();
  let first = Rc::new(Cell::new(0));
  let second = Rc::new(Cell::new(0));
  let (f, s) = (first.clone(), second.clone());
  app::set_event_handler(move |_| {
    f.set(f.get() + 1);
    let s = s.clone();
    // Replacing the running handler: the running one stays alive until it returns.
    app::set_event_handler(move |_| {
      s.set(s.get() + 1);
      Reply::default()
    });
    Reply::default()
  });
  let s = second.clone();
  wait_for("events to the replacement", move || s.get() > 2);
  assert_eq!(first.get(), 1);
}

fn handler_panic_caught() {
  init();
  let calls = Rc::new(Cell::new(0));
  let c = calls.clone();
  app::set_event_handler(move |_| {
    c.set(c.get() + 1);
    panic!("boom in the event handler");
  });
  let c = calls.clone();
  wait_for("several events despite panics", move || c.get() > 3);
  assert!(app::is_running());
}

fn handler_reentrant() {
  init_with_theme();
  let max_depth = Rc::new(Cell::new(0));
  let depth = Rc::new(Cell::new(0));
  let (m, d) = (max_depth.clone(), depth.clone());
  app::set_event_handler(move |e| {
    d.set(d.get() + 1);
    m.set(m.get().max(d.get()));
    if e.kind == EventKind::AboutToBlock && d.get() == 1 {
      // Emits colorSchemeChanged (-> Theme) synchronously, from inside this handler.
      let next = match app::color_scheme() {
        Some(ColorScheme::Dark) => ColorScheme::Light,
        _ => ColorScheme::Dark,
      };
      app::set_color_scheme(Some(next));
    }
    d.set(d.get() - 1);
    Reply::default()
  });
  let m = max_depth.clone();
  wait_for("a nested event", move || m.get() >= 2);
}

fn window_events_reach_handler() {
  init();
  record_events(|_| Reply::default());
  let window = Window::new(7, false);
  window.resize(320, 240);
  window.set_visible(true);
  wait_for("WindowResized for window 7", || {
    has_event(|e| e.kind == EventKind::WindowResized && e.window == 7 && e.view == 0 && e.a > 0)
  });
  clear_events();
  window.resize(400, 300);
  wait_for("WindowResized to 400x300", || {
    has_event(|e| e.kind == EventKind::WindowResized && e.window == 7 && (e.a, e.b) == (400, 300))
  });
}

fn handler_dropped() {
  init();
  let first = Rc::new(Cell::new(false));
  let guard = DropFlag(first.clone());
  app::set_event_handler(move |_| {
    let _keep = &guard;
    Reply::default()
  });
  let second = Rc::new(Cell::new(false));
  let guard = DropFlag(second.clone());
  app::set_event_handler(move |_| {
    let _keep = &guard;
    Reply::default()
  });
  assert!(first.get());
  assert!(!second.get());
  pump(Duration::from_millis(20));
  app::shutdown();
  assert!(second.get());
}

// ---------------------------------------------------------------------------------------------
// queries

fn color_scheme_round_trip() {
  init_with_theme();
  app::set_color_scheme(Some(ColorScheme::Dark));
  assert_eq!(app::color_scheme(), Some(ColorScheme::Dark));
  app::set_color_scheme(Some(ColorScheme::Light));
  assert_eq!(app::color_scheme(), Some(ColorScheme::Light));
  // Following the system again: offscreen has no opinion, so anything is fine; it must not crash.
  app::set_color_scheme(None);
  let _ = app::color_scheme();
}

fn color_scheme_theme_event() {
  init_with_theme();
  record_events(|_| Reply::default());
  let next = match app::color_scheme() {
    Some(ColorScheme::Dark) => ColorScheme::Light,
    _ => ColorScheme::Dark,
  };
  app::set_color_scheme(Some(next));
  let want = if next == ColorScheme::Dark { 2 } else { 1 };
  wait_for("a Theme event", move || {
    has_event(|e| e.kind == EventKind::Theme && e.a == want && e.window == 0)
  });
}

fn screens_sane() {
  init();
  let screens = app::screens();
  assert!(!screens.is_empty());
  assert_eq!(
    screens.iter().filter(|s| s.primary).count(),
    1,
    "{screens:?}"
  );
  for s in &screens {
    assert!(s.width > 0 && s.height > 0, "{s:?}");
    assert!(s.available_width > 0 && s.available_height > 0, "{s:?}");
    assert!(
      s.available_width <= s.width && s.available_height <= s.height,
      "{s:?}"
    );
    assert!(s.scale > 0.0, "{s:?}");
  }
  // The same answer twice.
  assert_eq!(app::screens(), screens);
}

fn cursor_position() {
  init();
  let (_, _, scale) = app::cursor_position().expect("a cursor position");
  assert!(scale > 0.0);
}

fn native_display_no_crash() {
  init();
  let _ = app::native_display();
  let _ = app::native_display();
}

fn native_display() {
  let description = app::init(Some("qtshim-test"));
  let display = app::native_display();
  if description.ends_with(" on offscreen") {
    assert_eq!(display, None);
  } else if let Some(NativeHandle::X11(d)) = display {
    assert_ne!(d, 0);
  }
}

fn launcher_update() {
  init();
  // Without a session bus (or a listener) the signal goes nowhere; it must not crash or block.
  app::launcher_update(None, None, None);
  app::launcher_update(None, Some(3), Some(0.5));
  app::launcher_update(Some("qtshim-test.desktop"), Some(0), None);
  app::launcher_update(Some("qtshim-test"), None, Some(1.0));
  app::launcher_update(Some(""), Some(-5), Some(-1.0));
  app::launcher_update(Some("with\0nul"), Some(i64::MAX), Some(f64::NAN));
  pump(Duration::from_millis(10));
}
