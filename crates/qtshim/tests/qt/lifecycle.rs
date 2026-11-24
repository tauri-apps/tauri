// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The safety guarantees of `qtshim::lifecycle`: Qt is only reachable from the main thread,
//! between `app::init` and `app::shutdown`; `shutdown` refuses to run inside callbacks; handles
//! that outlive the application are inert; handles may be dropped from inside callbacks.

use std::{
  any::Any,
  cell::{Cell, RefCell},
  rc::Rc,
  sync::{Mutex, mpsc},
  thread,
};

use qtshim::test_harness::Case;
use qtshim::{
  app::{self, EventKind, Reply},
  menu::{self, ActionOptions, Container, MenuBar, PopupMenu, StandardAction, Widget},
  webengine::{self, Profile, Responder, SchemeFlags, View, ViewOptions, Window},
};

use crate::common::{init, wait_for};

const WRONG_THREAD: &str = "Qt can only be used on the thread that called `app::init`";
const ALREADY_INIT: &str = "`app::init` was already called";
const IN_CALLBACK: &str = "can't be called from inside the event loop or a Qt callback";

pub fn cases() -> Vec<Case> {
  vec![
    // Off the main thread (while the application runs on the main thread).
    Case::pass("lifecycle: app::init on a second thread panics", || {
      init();
      off_thread_msg(ALREADY_INIT, || {
        app::init(None);
      });
      assert!(app::is_running());
    }),
    Case::pass("lifecycle: app::exec panics off the main thread", || {
      init();
      off_thread(|| {
        app::exec();
      });
    }),
    Case::pass("lifecycle: app::quit panics off the main thread", || {
      init();
      off_thread(|| app::quit(0));
    }),
    Case::pass(
      "lifecycle: app::process_events panics off the main thread",
      || {
        init();
        off_thread(app::process_events);
      },
    ),
    Case::pass(
      "lifecycle: app::shutdown panics off the main thread",
      || {
        init();
        off_thread(app::shutdown);
        assert!(app::is_running());
      },
    ),
    Case::pass("lifecycle: app::post panics off the main thread", || {
      init();
      off_thread(|| app::post(|| unreachable!()));
    }),
    Case::pass(
      "lifecycle: app::set_event_handler panics off the main thread",
      || {
        init();
        off_thread(|| app::set_event_handler(|_| Reply::default()));
      },
    ),
    Case::pass("lifecycle: app::screens panics off the main thread", || {
      init();
      off_thread(|| {
        app::screens();
      });
    }),
    Case::pass(
      "lifecycle: app::color_scheme panics off the main thread",
      || {
        init();
        off_thread(|| {
          app::color_scheme();
        });
        off_thread(|| app::set_color_scheme(None));
      },
    ),
    Case::pass(
      "lifecycle: app::cursor_position panics off the main thread",
      || {
        init();
        off_thread(|| {
          app::cursor_position();
        });
      },
    ),
    Case::pass(
      "lifecycle: app::native_display panics off the main thread",
      || {
        init();
        off_thread(|| {
          app::native_display();
        });
      },
    ),
    Case::pass(
      "lifecycle: app::launcher_update panics off the main thread",
      || {
        init();
        off_thread(|| app::launcher_update(None, Some(1), None));
      },
    ),
    Case::pass(
      "lifecycle: menu::Widget::from_ptr panics off the main thread",
      || {
        init();
        let window = Window::new(1, false);
        let ptr = window.widget_ptr().as_ptr() as usize;
        off_thread(move || {
          Widget::from_ptr(ptr as *mut _);
        });
        off_thread(|| {
          Widget::from_ptr(std::ptr::null_mut());
        });
      },
    ),
    Case::pass(
      "lifecycle: menu functions panic off the main thread",
      || {
        init();
        off_thread(|| StandardAction::Copy.trigger());
        off_thread(|| menu::show_about("t", "", None));
        off_thread(|| {
          menu::debug::take_popup_menu();
        });
      },
    ),
    Case::pass(
      "lifecycle: webengine::Window::new panics off the main thread",
      || {
        init();
        off_thread(|| {
          Window::new(1, false);
        });
      },
    ),
    Case::pass(
      "lifecycle: webengine engine functions panic off the main thread",
      || {
        init();
        off_thread(|| {
          webengine::register_scheme("offthread", SchemeFlags::APP);
        });
        off_thread(|| webengine::add_chromium_flags("--foo"));
        off_thread(|| {
          webengine::engine_started();
        });
        off_thread(|| {
          webengine::engine_version();
        });
        off_thread(|| webengine::set_scheme_handler(|_, _| {}));
        off_thread(|| {
          Profile::get(None, true);
        });
      },
    ),
    // Before init (on the main thread).
    Case::panics(
      "lifecycle: app::post panics before init",
      WRONG_THREAD,
      || app::post(|| {}),
    ),
    Case::panics(
      "lifecycle: app::exec panics before init",
      WRONG_THREAD,
      || {
        app::exec();
      },
    ),
    Case::panics(
      "lifecycle: app::shutdown panics before init",
      WRONG_THREAD,
      app::shutdown,
    ),
    Case::panics(
      "lifecycle: webengine::Window::new panics before init",
      WRONG_THREAD,
      || {
        Window::new(1, false);
      },
    ),
    // After shutdown.
    Case::panics(
      "lifecycle: Qt calls panic after shutdown",
      WRONG_THREAD,
      || {
        init();
        app::shutdown();
        app::screens();
      },
    ),
    Case::panics(
      "lifecycle: app::init after shutdown panics",
      ALREADY_INIT,
      || {
        init();
        app::shutdown();
        init();
      },
    ),
    Case::pass(
      "lifecycle: app::init on another thread after shutdown panics",
      || {
        init();
        app::shutdown();
        off_thread_msg(ALREADY_INIT, || {
          app::init(None);
        });
      },
    ),
    Case::panics(
      "lifecycle: app::shutdown twice panics",
      WRONG_THREAD,
      || {
        init();
        app::shutdown();
        app::shutdown();
      },
    ),
    Case::panics(
      "lifecycle: app::exec panics after shutdown",
      WRONG_THREAD,
      || {
        init();
        app::shutdown();
        app::exec();
      },
    ),
    Case::panics(
      "lifecycle: app::post panics after shutdown",
      WRONG_THREAD,
      || {
        init();
        app::shutdown();
        app::post(|| {});
      },
    ),
    Case::panics(
      "lifecycle: app::set_event_handler panics after shutdown",
      WRONG_THREAD,
      || {
        init();
        app::shutdown();
        app::set_event_handler(|_| Reply::default());
      },
    ),
    Case::panics(
      "lifecycle: menu::Widget::from_ptr panics after shutdown",
      WRONG_THREAD,
      || {
        init();
        app::shutdown();
        Widget::from_ptr(std::ptr::null_mut());
      },
    ),
    Case::panics(
      "lifecycle: menu::Widget methods panic after shutdown",
      WRONG_THREAD,
      || {
        init();
        let window = Window::new(1, false);
        let widget = window.widget();
        app::shutdown();
        widget.is_alive();
      },
    ),
    Case::panics(
      "lifecycle: menu containers panic after shutdown",
      WRONG_THREAD,
      || {
        init();
        let window = Window::new(1, false);
        let bar = MenuBar::get_or_create(&window.widget()).unwrap();
        let file = bar.add_menu("File", true, None).unwrap();
        app::shutdown();
        file.add_action(&ActionOptions::default(), None);
      },
    ),
    Case::panics(
      "lifecycle: webengine::Window methods panic after shutdown",
      WRONG_THREAD,
      || {
        init();
        let window = Window::new(1, false);
        app::shutdown();
        window.set_title("gone");
      },
    ),
    Case::panics(
      "lifecycle: webengine::Profile methods panic after shutdown",
      WRONG_THREAD,
      || {
        init();
        let profile = Profile::get(None, true);
        app::shutdown();
        profile.cookies();
      },
    ),
    Case::panics(
      "lifecycle: webengine::View methods panic after shutdown",
      WRONG_THREAD,
      || {
        init();
        let window = Window::new(1, false);
        let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
        app::shutdown();
        view.load_html("<p>gone</p>", None);
      },
    ),
    // shutdown inside callbacks.
    Case::pass(
      "lifecycle: app::shutdown inside the event handler panics",
      shutdown_in_event_handler,
    ),
    Case::pass(
      "lifecycle: app::shutdown inside the event handler is caught",
      shutdown_in_event_handler_uncaught,
    ),
    Case::pass(
      "lifecycle: app::shutdown inside a dispatched closure panics",
      shutdown_in_dispatched,
    ),
    Case::pass(
      "lifecycle: app::shutdown inside a posted closure panics",
      shutdown_in_posted,
    ),
    Case::pass(
      "lifecycle: app::shutdown inside a menu action panics",
      shutdown_in_menu_action,
    ),
    Case::pass(
      "lifecycle: app::shutdown inside a JavaScript callback panics",
      shutdown_in_js_callback,
    )
    .timeout(std::time::Duration::from_secs(90)),
    // Handles outliving the application.
    Case::pass(
      "lifecycle: handles dropped after shutdown don't touch Qt",
      handles_dropped_after_shutdown,
    ),
    Case::pass(
      "lifecycle: handles leaked past shutdown in a thread-local are fine",
      handles_in_thread_local,
    ),
    Case::pass(
      "lifecycle: a Responder outliving the application is inert",
      responder_after_shutdown,
    )
    .timeout(std::time::Duration::from_secs(90)),
    // Dropping handles from inside callbacks.
    Case::pass(
      "lifecycle: dropping a window inside a dispatched closure",
      drop_window_in_dispatched,
    ),
    // Window events are delivered synchronously from the call that caused them (offscreen),
    // while the caller still borrows the window: safe code can't drop a window inside its own
    // window event. Other handles can be dropped there.
    Case::pass(
      "lifecycle: dropping other handles inside a window's resize event",
      drop_others_in_resize_event,
    ),
    Case::pass(
      "lifecycle: dropping a window inside its own menu action (event loop)",
      drop_window_in_menu_action,
    ),
    Case::pass(
      "lifecycle: dropping a window inside its own menu action (no event loop)",
      drop_window_in_menu_action_no_loop,
    ),
    Case::pass(
      "lifecycle: clearing a menu inside one of its actions",
      clear_menu_in_action,
    ),
    Case::pass(
      "lifecycle: dropping a view inside its own load event (event loop)",
      || drop_in_load_event(Victim::View, Pump::Exec),
    )
    .timeout(std::time::Duration::from_secs(90)),
    Case::pass(
      "lifecycle: dropping a view inside its own load event (no event loop)",
      || drop_in_load_event(Victim::View, Pump::ProcessEvents),
    )
    .timeout(std::time::Duration::from_secs(90)),
    Case::pass(
      "lifecycle: dropping a window inside its view's load event (event loop)",
      || drop_in_load_event(Victim::Window, Pump::Exec),
    )
    .timeout(std::time::Duration::from_secs(90)),
    Case::pass(
      "lifecycle: dropping a window inside its view's load event (no event loop)",
      || drop_in_load_event(Victim::Window, Pump::ProcessEvents),
    )
    .timeout(std::time::Duration::from_secs(90)),
  ]
}

// ---------------------------------------------------------------------------------------------
// helpers

fn panic_message(err: &(dyn Any + Send)) -> String {
  err
    .downcast_ref::<&str>()
    .map(|s| s.to_string())
    .or_else(|| err.downcast_ref::<String>().cloned())
    .unwrap_or_default()
}

/// Run `f` on another thread; it must panic with a message containing `msg`.
#[track_caller]
fn off_thread_msg(msg: &str, f: impl FnOnce() + Send) {
  let result = thread::scope(|s| s.spawn(f).join());
  let err = result.expect_err("expected a panic on the other thread");
  let got = panic_message(&*err);
  assert!(got.contains(msg), "unexpected panic message: {got:?}");
}

/// Run `f` on another thread; it must panic because Qt is used off the main thread.
#[track_caller]
fn off_thread(f: impl FnOnce() + Send) {
  off_thread_msg(WRONG_THREAD, f);
}

/// Run `f` and return its panic message (`None` if it didn't panic).
fn catch(f: impl FnOnce()) -> Option<String> {
  std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
    .err()
    .map(|e| panic_message(&*e))
}

/// After a refused `shutdown`: the application still works, and a later `shutdown` succeeds.
fn still_usable_then_shutdown() {
  assert!(app::is_running());
  let ran = Rc::new(Cell::new(false));
  let r = ran.clone();
  app::post(move || r.set(true));
  wait_for("a closure after the refused shutdown", move || ran.get());
  app::shutdown();
  assert!(!app::is_running());
}

// ---------------------------------------------------------------------------------------------
// shutdown inside callbacks

fn shutdown_in_event_handler() {
  init();
  let msg = Rc::new(RefCell::new(None));
  let m = msg.clone();
  app::set_event_handler(move |e| {
    if e.kind == EventKind::AboutToBlock && m.borrow().is_none() {
      *m.borrow_mut() = Some(catch(app::shutdown));
    }
    Reply::default()
  });
  let m = msg.clone();
  wait_for("an event", move || m.borrow().is_some());
  let got = msg.borrow_mut().take().unwrap();
  assert!(got.is_some_and(|m| m.contains(IN_CALLBACK)));
  still_usable_then_shutdown();
}

fn shutdown_in_event_handler_uncaught() {
  init();
  let tried = Rc::new(Cell::new(false));
  let t = tried.clone();
  app::set_event_handler(move |_| {
    if !t.replace(true) {
      // Panics; the trampoline catches it.
      app::shutdown();
    }
    Reply::default()
  });
  let t = tried.clone();
  wait_for("an event", move || t.get());
  still_usable_then_shutdown();
}

fn shutdown_in_dispatched() {
  init();
  let (tx, rx) = mpsc::channel();
  assert!(app::dispatch(move || tx
    .send(catch(app::shutdown))
    .unwrap()));
  // And one without catching: the trampoline must catch it.
  assert!(app::dispatch(app::shutdown));
  let got = Rc::new(RefCell::new(None));
  let g = got.clone();
  wait_for("the dispatched closure", move || {
    if let Ok(v) = rx.try_recv() {
      *g.borrow_mut() = Some(v);
    }
    g.borrow().is_some()
  });
  let msg = got.borrow_mut().take().unwrap();
  assert!(msg.is_some_and(|m| m.contains(IN_CALLBACK)));
  still_usable_then_shutdown();
}

fn shutdown_in_posted() {
  init();
  let msg = Rc::new(RefCell::new(None));
  let m = msg.clone();
  app::post(move || *m.borrow_mut() = Some(catch(app::shutdown)));
  app::post(app::shutdown);
  let m = msg.clone();
  wait_for("the posted closure", move || m.borrow().is_some());
  let got = msg.borrow_mut().take().unwrap();
  assert!(got.is_some_and(|m| m.contains(IN_CALLBACK)));
  still_usable_then_shutdown();
}

fn shutdown_in_menu_action() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let file = bar.add_menu("File", true, None).unwrap();
  let msg = Rc::new(RefCell::new(None));
  let m = msg.clone();
  file.add_action(
    &ActionOptions {
      text: "Quit",
      ..Default::default()
    },
    Some(Box::new(move || {
      *m.borrow_mut() = Some(catch(app::shutdown))
    })),
  );
  file.add_action(
    &ActionOptions {
      text: "Quit uncaught",
      ..Default::default()
    },
    Some(Box::new(app::shutdown)),
  );
  // Outside the event loop, but still inside a Qt callback.
  assert!(menu::debug::trigger_menu_item(&widget, "File/Quit"));
  let got = msg.borrow_mut().take().expect("the action ran");
  assert!(got.is_some_and(|m| m.contains(IN_CALLBACK)));
  assert!(menu::debug::trigger_menu_item(
    &widget,
    "File/Quit uncaught"
  ));
  assert!(app::is_running());
  // And from inside the event loop.
  let w = widget.clone();
  app::post(move || {
    assert!(menu::debug::trigger_menu_item(&w, "File/Quit"));
  });
  let m = msg.clone();
  wait_for("the action inside the loop", move || m.borrow().is_some());
  drop((file, bar, widget, window));
  still_usable_then_shutdown();
}

fn shutdown_in_js_callback() {
  init();
  let window = Window::new(1, false);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  view.load_html("<p>hi</p>", None);
  let msg = Rc::new(RefCell::new(None));
  let m = msg.clone();
  view.eval_with_callback("1 + 1", move |_| {
    if m.borrow().is_none() {
      *m.borrow_mut() = Some(catch(app::shutdown));
    }
  });
  let m = msg.clone();
  wait_for("the JavaScript callback", move || m.borrow().is_some());
  let got = msg.borrow_mut().take().unwrap();
  assert!(got.is_some_and(|m| m.contains(IN_CALLBACK)));
  drop((view, window));
  still_usable_then_shutdown();
}

// ---------------------------------------------------------------------------------------------
// handles outliving the application

fn handles_dropped_after_shutdown() {
  init();
  let window = Window::new(1, false);
  window.set_visible(true);
  let other = Window::new(2, false);
  let profile = Profile::get(None, true);
  let view = View::new(1, &window, profile, ViewOptions::default());
  let widget = window.widget();
  let widget2 = widget.clone();
  let foreign = Widget::from_ptr(other.widget_ptr().as_ptr()).expect("a live widget");
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let menu = bar.add_menu("File", true, None).unwrap();
  let popup = PopupMenu::new(&widget).unwrap();
  popup.add_action(&ActionOptions::default(), Some(Box::new(|| {})));
  app::shutdown();
  // Drop order chosen to be awkward: windows before the views and widgets inside them.
  drop(window);
  drop(other);
  drop(view);
  drop((widget, widget2, foreign));
  drop((bar, menu, popup));
  let _ = profile; // Copy, nothing to drop.
}

fn handles_in_thread_local() {
  thread_local! {
    static KEEP: RefCell<Vec<Box<dyn Any>>> = const { RefCell::new(Vec::new()) };
  }
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  KEEP.with(|k| {
    let mut k = k.borrow_mut();
    k.push(Box::new(view));
    k.push(Box::new(widget));
    k.push(Box::new(window));
  });
  app::shutdown();
  // Dropped at thread exit (after the harness teardown), or here: both must be harmless.
  KEEP.with(|k| k.borrow_mut().truncate(1));
}

static RESPONDERS: Mutex<Vec<Responder>> = Mutex::new(Vec::new());

fn responder_after_shutdown() {
  init();
  assert!(webengine::register_scheme("qstest", SchemeFlags::APP));
  webengine::set_scheme_handler(|_, responder| RESPONDERS.lock().unwrap().push(responder));
  let window = Window::new(1, false);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  view.load_html(
    "<img src='qstest://x/a.png'><img src='qstest://x/b.png'><img src='qstest://x/c.png'>",
    Some("qstest://x/"),
  );
  wait_for("three scheme requests", || {
    RESPONDERS.lock().unwrap().len() >= 3
  });
  drop((view, window));
  app::shutdown();
  let mut responders = std::mem::take(&mut *RESPONDERS.lock().unwrap());
  // Answered after shutdown, on the main thread and on another one.
  responders
    .pop()
    .unwrap()
    .respond(http::Response::new(b"late".to_vec()));
  let r = responders.pop().unwrap();
  thread::spawn(move || r.respond(http::Response::new(b"late".to_vec())))
    .join()
    .unwrap();
  // Dropped unanswered, on another thread and on this one.
  let r = responders.pop().unwrap();
  thread::spawn(move || drop(r)).join().unwrap();
  drop(responders);
}

// ---------------------------------------------------------------------------------------------
// dropping handles inside callbacks

thread_local! {
  static WINDOW: RefCell<Option<Window>> = const { RefCell::new(None) };
}

fn take_window() -> Option<Window> {
  WINDOW.with(|w| w.borrow_mut().take())
}

fn drop_window_in_dispatched() {
  init();
  let window = Window::new(1, false);
  window.set_visible(true);
  let widget = window.widget();
  WINDOW.with(|w| *w.borrow_mut() = Some(window));
  assert!(app::dispatch(|| drop(take_window())));
  let w = widget.clone();
  wait_for("the window to be destroyed", move || !w.is_alive());
}

fn drop_others_in_resize_event() {
  thread_local! {
    static OTHERS: RefCell<Option<(Window, View, Widget)>> = const { RefCell::new(None) };
  }
  init();
  let window = Window::new(1, false);
  let other = Window::new(2, false);
  other.set_visible(true);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  view.set_fill(true);
  let other_widget = other.widget();
  OTHERS.with(|o| *o.borrow_mut() = Some((other, view, window.widget())));
  let dropped = Rc::new(Cell::new(false));
  let d = dropped.clone();
  app::set_event_handler(move |e| {
    if e.kind == EventKind::WindowResized && e.window == 1 {
      if let Some(others) = OTHERS.with(|o| o.borrow_mut().take()) {
        drop(others);
        d.set(true);
      }
    }
    Reply::default()
  });
  // Outside the event loop: the resize event fires from inside `set_visible`.
  window.set_visible(true);
  assert!(dropped.get());
  // Destroyed inside a callback: deleted as soon as control is back in Qt's event processing.
  app::process_events();
  assert!(!other_widget.is_alive());
  window.resize(300, 300);
  let w = window.widget();
  wait_for("the window to settle", move || w.is_alive());
}

/// A window with a "File/Close" action that drops the window; `freed_while_running` is set by a
/// guard owned by the closure when the closure is dropped *while still running* (a use-after-free).
fn window_with_closing_action(freed_while_running: Rc<Cell<bool>>) -> Widget {
  struct Guard {
    running: Rc<Cell<bool>>,
    freed_while_running: Rc<Cell<bool>>,
  }
  impl Drop for Guard {
    fn drop(&mut self) {
      if self.running.get() {
        self.freed_while_running.set(true);
      }
    }
  }
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let file = bar.add_menu("File", true, None).unwrap();
  let running = Rc::new(Cell::new(false));
  let guard = Guard {
    running: running.clone(),
    freed_while_running,
  };
  file.add_action(
    &ActionOptions {
      text: "Close",
      ..Default::default()
    },
    Some(Box::new(move || {
      let _keep = &guard;
      running.set(true);
      drop(take_window());
      running.set(false);
    })),
  );
  WINDOW.with(|w| *w.borrow_mut() = Some(window));
  widget
}

fn drop_window_in_menu_action() {
  init();
  let freed = Rc::new(Cell::new(false));
  let widget = window_with_closing_action(freed.clone());
  let w = widget.clone();
  app::post(move || assert!(menu::debug::trigger_menu_item(&w, "File/Close")));
  let w = widget.clone();
  wait_for("the window to be destroyed", move || !w.is_alive());
  assert!(!freed.get(), "the action's closure was freed while running");
}

fn drop_window_in_menu_action_no_loop() {
  init();
  let freed = Rc::new(Cell::new(false));
  let widget = window_with_closing_action(freed.clone());
  assert!(menu::debug::trigger_menu_item(&widget, "File/Close"));
  assert!(!freed.get(), "the action's closure was freed while running");
  for _ in 0..20 {
    app::process_events();
  }
  assert!(!widget.is_alive());
}

fn clear_menu_in_action() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = Rc::new(MenuBar::get_or_create(&widget).unwrap());
  let file = bar.add_menu("File", true, None).unwrap();
  let b = bar.clone();
  let cleared = Rc::new(Cell::new(false));
  let c = cleared.clone();
  file.add_action(
    &ActionOptions {
      text: "Clear",
      ..Default::default()
    },
    Some(Box::new(move || {
      b.clear();
      MenuBar::remove(b.widget());
      c.set(true);
    })),
  );
  // Outside the loop (deleteLater) and then let the deferred deletes run.
  assert!(menu::debug::trigger_menu_item(&widget, "File/Clear"));
  assert!(cleared.get());
  let w = widget.clone();
  wait_for("the menubar to go", move || {
    menu::debug::menubar_tree(&w).is_empty()
  });
  assert!(!file.widget().is_alive());
}

#[derive(Clone, Copy, PartialEq)]
enum Victim {
  View,
  Window,
}

#[derive(Clone, Copy)]
enum Pump {
  /// Run the event loop (`app::exec`).
  Exec,
  /// Call `app::process_events` without ever entering `exec`.
  ProcessEvents,
}

/// Drop a view, or its window, from the event handler when the view's page finishes loading.
fn drop_in_load_event(what: Victim, pump: Pump) {
  thread_local! {
    static HANDLES: RefCell<Option<(Window, View)>> = const { RefCell::new(None) };
  }
  init();
  let window = Window::new(1, false);
  window.set_visible(true);
  let view = View::new(5, &window, Profile::get(None, true), ViewOptions::default());
  view.set_fill(true);
  let widget = window.widget();
  let dropped = Rc::new(Cell::new(false));
  let d = dropped.clone();
  app::set_event_handler(move |e| {
    if e.kind == EventKind::ViewLoad && e.view == 5 && e.a == 1 {
      if let Some((window, view)) = HANDLES.with(|h| h.borrow_mut().take()) {
        match what {
          Victim::View => {
            drop(view);
            // Kept until the end of the case.
            std::mem::forget(window);
          }
          Victim::Window => {
            drop(window);
            std::mem::forget(view);
          }
        }
        d.set(true);
      }
    }
    Reply::default()
  });
  view.load_html("<p>hi</p>", None);
  HANDLES.with(|h| *h.borrow_mut() = Some((window, view)));
  match pump {
    Pump::Exec => {
      let d = dropped.clone();
      wait_for("the drop in the load event", move || d.get());
      crate::common::pump(std::time::Duration::from_millis(200));
    }
    Pump::ProcessEvents => {
      let deadline = std::time::Instant::now() + crate::common::WAIT;
      while !dropped.get() && std::time::Instant::now() < deadline {
        app::process_events();
        thread::sleep(std::time::Duration::from_millis(1));
      }
      assert!(dropped.get(), "timed out waiting for the load event");
      for _ in 0..50 {
        app::process_events();
        thread::sleep(std::time::Duration::from_millis(2));
      }
    }
  }
  assert!(app::is_running());
  assert_eq!(widget.is_alive(), what == Victim::View);
}
