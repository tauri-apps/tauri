// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `qtshim::menu`: tracked widgets, menubars, menus, actions, context menus, standard actions and
//! the `debug` helpers.

use std::{
  cell::{Cell, RefCell},
  ffi::c_void,
  rc::Rc,
  thread,
  time::Duration,
};

use qtshim::test_harness::Case;
use qtshim::{
  app::{self, EventKind, Reply},
  menu::{
    ActionOptions, Container, Icon, Menu, MenuBar, PopupMenu, PopupPosition, StandardAction,
    Widget, debug, show_about,
  },
  webengine::Window,
};

use crate::common::{has_event, init, pump, record_events, wait_for};

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass("menu: build a menubar and read it back", build_menubar),
    // Widget
    Case::pass("menu: Widget::from_ptr rejects null", from_ptr_null),
    Case::pass(
      "menu: Widget::from_ptr rejects dangling and misaligned pointers",
      from_ptr_bogus,
    ),
    Case::pass(
      "menu: Widget::from_ptr rejects non-widget objects",
      from_ptr_non_widget,
    ),
    Case::pass(
      "menu: Widget::from_ptr accepts a live window",
      from_ptr_live,
    ),
    Case::pass(
      "menu: Widget::from_ptr accepts a menubar and a menu",
      from_ptr_menu_widgets,
    ),
    Case::pass(
      "menu: Widget::from_ptr on another thread panics",
      from_ptr_other_thread,
    ),
    Case::pass(
      "menu: Widget goes inert when its window is dropped",
      widget_inert_after_drop,
    ),
    Case::pass(
      "menu: Widget goes inert after a deferred delete",
      widget_inert_after_deferred,
    ),
    Case::pass(
      "menu: Widget clone of a dead widget is dead",
      widget_clone_dead,
    ),
    Case::pass("menu: Widget equality", widget_eq),
    Case::pass(
      "menu: Widget dropped after shutdown",
      widget_drop_after_shutdown,
    ),
    Case::panics(
      "menu: Widget::is_alive after shutdown panics",
      "Qt can only be used on the thread",
      widget_use_after_shutdown,
    ),
    Case::panics(
      "menu: MenuBar::get after shutdown panics",
      "Qt can only be used on the thread",
      menubar_after_shutdown,
    ),
    // MenuBar
    Case::pass(
      "menu: MenuBar get / get_or_create / get",
      menubar_get_create,
    ),
    Case::pass(
      "menu: MenuBar remove then recreate in one tick",
      menubar_remove_recreate,
    ),
    Case::pass(
      "menu: MenuBar remove without a menubar",
      menubar_remove_none,
    ),
    Case::pass("menu: MenuBar set_visible / is_visible", menubar_visibility),
    Case::pass(
      "menu: MenuBar on a dead window is inert",
      menubar_dead_window,
    ),
    Case::pass("menu: MenuBar sits above the content", menubar_layout),
    Case::pass(
      "menu: MenuBar on a widget without a layout",
      menubar_no_layout,
    ),
    // Container
    Case::pass("menu: Container nested menus", container_nested),
    Case::pass("menu: Container action flags", container_action_flags),
    Case::pass("menu: Container shortcuts and hints", container_shortcuts),
    Case::pass("menu: Container icons", container_icons),
    Case::pass("menu: Container labels", container_labels),
    Case::pass("menu: Container separators", container_separators),
    Case::pass("menu: Container clear", container_clear),
    Case::pass("menu: Container ops on a dead container", container_dead),
    // Callbacks
    Case::pass(
      "menu: action callback runs on the main thread",
      callback_main_thread,
    ),
    Case::pass("menu: trigger_menu_item paths", trigger_paths),
    Case::pass(
      "menu: disabled item does not run its callback",
      trigger_disabled,
    ),
    Case::pass(
      "menu: checkable item toggles when triggered",
      trigger_checkable,
    ),
    Case::pass(
      "menu: callback dropped when cleared",
      callback_dropped_on_clear,
    ),
    Case::pass(
      "menu: callback dropped when the menubar is removed",
      callback_dropped_on_remove,
    ),
    Case::pass(
      "menu: callback dropped when the window is dropped",
      callback_dropped_on_window,
    ),
    Case::pass(
      "menu: callback dropped at shutdown",
      callback_dropped_at_shutdown,
    ),
    Case::pass("menu: callback panic is contained", callback_panic),
    Case::pass("menu: callback rebuilds the menubar", callback_rebuilds),
    Case::pass("menu: callback removes the menubar", callback_removes_bar),
    Case::pass(
      "menu: callback triggers itself recursively",
      callback_recursive,
    ),
    Case::pass(
      "menu: callback calling shutdown is refused",
      callback_shutdown,
    ),
    Case::pass(
      "menu: callback drops its window inside the event loop",
      callback_drops_window_in_loop,
    ),
    Case::pass(
      "menu: callback drops its window outside the event loop",
      callback_drops_window_outside_loop,
    ),
    // PopupMenu
    Case::pass(
      "menu: PopupMenu on a dead parent is None",
      popup_dead_parent,
    ),
    Case::pass("menu: PopupMenu shows at the cursor", popup_show_cursor),
    Case::pass(
      "menu: PopupMenu shows at logical/physical positions",
      popup_show_positions,
    ),
    Case::pass(
      "menu: PopupMenu shows at non-finite positions",
      popup_show_bad_positions,
    ),
    Case::pass("menu: PopupMenu nested tree", popup_nested),
    Case::pass("menu: PopupMenu parented to a menu", popup_parented_to_menu),
    Case::pass(
      "menu: PopupMenu callbacks dropped when it closes",
      popup_callbacks_dropped,
    ),
    Case::pass("menu: PopupMenu dropped unshown", popup_dropped_unshown),
    Case::pass("menu: PopupMenu show after parent died", popup_parent_died),
    // StandardAction / about
    Case::pass(
      "menu: StandardAction editing actions without focus",
      standard_editing,
    ),
    Case::pass(
      "menu: StandardAction window actions without a window",
      standard_window_none,
    ),
    Case::pass(
      "menu: StandardAction window actions on the active window",
      standard_window_active,
    ),
    Case::pass(
      "menu: StandardAction::Quit makes exec return",
      standard_quit,
    ),
    Case::pass("menu: show_about is non-blocking", about_non_blocking),
    // debug
    Case::pass("menu: debug helpers without a menubar", debug_no_menubar),
    Case::pass(
      "menu: take_popup_menu with nothing shown",
      debug_take_popup_empty,
    ),
  ]
}

// ---------------------------------------------------------------------------------------------
// helpers

/// A visible 400x300 window.
fn shown_window(id: u64) -> Window {
  let w = Window::new(id, false);
  w.resize(400, 300);
  w.set_visible(true);
  w
}

/// Sets its flag when dropped (to observe when a callback is dropped).
struct DropFlag(Rc<Cell<bool>>);

impl Drop for DropFlag {
  fn drop(&mut self) {
    self.0.set(true);
  }
}

/// A shared flag or counter.
type Shared<T> = Rc<Cell<T>>;

/// A callback counting its calls, and a flag set once the callback is dropped.
fn counting() -> (Shared<u32>, Shared<bool>, Box<dyn Fn()>) {
  let count = Rc::new(Cell::new(0));
  let dropped = Rc::new(Cell::new(false));
  let guard = DropFlag(dropped.clone());
  let c = count.clone();
  let cb = Box::new(move || {
    let _ = &guard;
    c.set(c.get() + 1);
  });
  (count, dropped, cb)
}

fn action<'a>(text: &'a str) -> ActionOptions<'a> {
  ActionOptions {
    text,
    ..Default::default()
  }
}

/// A window with an empty `File` menu in its menubar.
fn window_with_file_menu() -> (Window, Widget, Menu) {
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let file = bar.add_menu("&File", true, None).unwrap();
  (window, widget, file)
}

/// Run the event loop briefly so deferred deletes run.
fn flush() {
  pump(Duration::from_millis(50));
}

// ---------------------------------------------------------------------------------------------
// Widget

fn build_menubar() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).expect("menubar");
  let file = bar.add_menu("&File", true, None).expect("menu");
  file.add_action(
    &ActionOptions {
      text: "&Open",
      shortcut: Some("Ctrl+O"),
      ..Default::default()
    },
    None,
  );
  let tree = debug::menubar_tree(&widget);
  assert!(
    tree.contains("File") && tree.contains("Open <Ctrl+O>"),
    "{tree}"
  );
}

fn from_ptr_null() {
  init();
  assert!(Widget::from_ptr(std::ptr::null_mut()).is_none());
}

fn from_ptr_bogus() {
  init();
  let window = Window::new(1, false);
  let ptr = window.widget_ptr().as_ptr();
  for bogus in [
    std::ptr::dangling_mut::<c_void>(),
    0xdead_beef_usize as *mut c_void,
    usize::MAX as *mut c_void,
    ptr.wrapping_byte_add(1),
    ptr.wrapping_byte_add(8),
    ptr.wrapping_byte_sub(8),
  ] {
    assert!(Widget::from_ptr(bogus).is_none(), "{bogus:p}");
  }
}

fn from_ptr_non_widget() {
  init();
  let mut local = 0u64;
  assert!(Widget::from_ptr(&mut local as *mut u64 as *mut c_void).is_none());
  let boxed = Box::new([0u8; 256]);
  assert!(Widget::from_ptr(Box::into_raw(boxed) as *mut c_void).is_none());
  // A QsWindow's views live in a child widget; a pointer into the middle of the
  // window object isn't a widget either.
  let window = Window::new(1, false);
  let bar = MenuBar::get_or_create(&window.widget()).unwrap();
  let bar_ptr = bar.widget().as_ptr().unwrap().as_ptr();
  assert!(Widget::from_ptr(bar_ptr.wrapping_byte_add(16)).is_none());
}

fn from_ptr_live() {
  init();
  let window = Window::new(1, false);
  let widget = Widget::from_ptr(window.widget_ptr().as_ptr()).expect("live window");
  assert!(widget.is_alive());
  assert_eq!(widget.as_ptr(), Some(window.widget_ptr()));
  assert_eq!(widget, window.widget());
}

fn from_ptr_menu_widgets() {
  init();
  let window = Window::new(1, false);
  let bar = MenuBar::get_or_create(&window.widget()).unwrap();
  let menu = bar.add_menu("M", true, None).unwrap();
  let bar_w = Widget::from_ptr(bar.widget().as_ptr().unwrap().as_ptr()).expect("menubar");
  let menu_w = Widget::from_ptr(menu.widget().as_ptr().unwrap().as_ptr()).expect("menu");
  assert_eq!(&bar_w, bar.widget());
  assert_eq!(&menu_w, menu.widget());
  assert_ne!(bar_w, menu_w);
}

fn from_ptr_other_thread() {
  init();
  let window = Window::new(1, false);
  let addr = window.widget_ptr().as_ptr() as usize;
  let r = thread::spawn(move || Widget::from_ptr(addr as *mut c_void).is_some()).join();
  assert!(r.is_err(), "from_ptr must panic off the main thread");
}

fn widget_inert_after_drop() {
  init();
  let window = Window::new(1, false);
  let ptr = window.widget_ptr().as_ptr();
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  assert!(widget.is_alive() && bar.widget().is_alive());
  drop(window); // outside the event loop: deleted right away
  assert!(!widget.is_alive());
  assert!(widget.as_ptr().is_none());
  assert!(
    !bar.widget().is_alive(),
    "child menubar dies with its window"
  );
  assert!(Widget::from_ptr(ptr).is_none());
}

fn widget_inert_after_deferred() {
  init();
  let slot = Rc::new(RefCell::new(Some(Window::new(1, false))));
  let widget = slot.borrow().as_ref().unwrap().widget();
  {
    let slot = slot.clone();
    // Inside the event loop the window is deleteLater'd.
    app::post(move || drop(slot.borrow_mut().take()));
  }
  let w = widget.clone();
  wait_for("the widget to be deleted", move || !w.is_alive());
  assert!(widget.as_ptr().is_none());
}

fn widget_clone_dead() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  drop(window);
  let clone = widget.clone();
  assert!(!clone.is_alive());
  assert!(clone.as_ptr().is_none());
  let clone2 = clone.clone();
  assert!(!clone2.is_alive());
}

fn widget_eq() {
  init();
  let a = Window::new(1, false);
  let b = Window::new(2, false);
  let wa = a.widget();
  let wa2 = a.widget();
  let wb = b.widget();
  assert_eq!(wa, wa2);
  assert_eq!(wa, wa.clone());
  assert_ne!(wa, wb);
  drop(b);
  // Dead handles equal nothing, not even themselves.
  #[allow(clippy::eq_op)]
  let self_eq = wb == wb;
  assert!(!self_eq);
  assert_ne!(wa, wb);
  assert_ne!(wb, wa);
}

fn widget_drop_after_shutdown() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let menu = bar.add_menu("x", true, None).unwrap();
  let popup = PopupMenu::new(&widget).unwrap();
  app::shutdown();
  drop((popup, menu, bar, widget, window));
}

fn widget_use_after_shutdown() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  app::shutdown();
  widget.is_alive();
}

fn menubar_after_shutdown() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  app::shutdown();
  MenuBar::get(&widget);
}

// ---------------------------------------------------------------------------------------------
// MenuBar

fn menubar_get_create() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  assert!(MenuBar::get(&widget).is_none());
  assert!(!MenuBar::is_visible(&widget));
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let again = MenuBar::get(&widget).expect("menubar now exists");
  assert_eq!(bar.widget(), again.widget());
  let again2 = MenuBar::get_or_create(&widget).unwrap();
  assert_eq!(bar.widget(), again2.widget());
  // The bar is a child of the window, not the window itself.
  assert_ne!(bar.widget(), &widget);
}

fn menubar_remove_recreate() {
  init();
  let window = shown_window(1);
  let widget = window.widget();
  let old = MenuBar::get_or_create(&widget).unwrap();
  old.add_menu("Old", true, None);
  assert!(MenuBar::is_visible(&widget));
  MenuBar::remove(&widget);
  // The old bar is only deleteLater'd, but it must not be found again.
  assert!(old.widget().is_alive());
  assert!(MenuBar::get(&widget).is_none());
  assert!(!MenuBar::is_visible(&widget));
  assert_eq!(debug::menubar_tree(&widget), "");
  let new = MenuBar::get_or_create(&widget).unwrap();
  assert_ne!(new.widget(), old.widget());
  assert!(MenuBar::is_visible(&widget), "the new bar is visible");
  new.add_menu("New", true, None);
  assert_eq!(debug::menubar_tree(&widget), "New\n");
  let o = old.widget().clone();
  wait_for("the old menubar to be deleted", move || !o.is_alive());
  assert!(new.widget().is_alive());
  assert_eq!(MenuBar::get(&widget).unwrap().widget(), new.widget());
  assert!(MenuBar::is_visible(&widget));
}

fn menubar_remove_none() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  MenuBar::remove(&widget);
  MenuBar::remove(&widget);
  assert!(MenuBar::get(&widget).is_none());
  MenuBar::set_visible(&widget, true);
  assert!(!MenuBar::is_visible(&widget));
  assert!(
    MenuBar::get(&widget).is_none(),
    "set_visible doesn't create a bar"
  );
}

fn menubar_visibility() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  MenuBar::get_or_create(&widget).unwrap();
  // The bar's own setting, also while the window is hidden (like muda's GTK backend).
  assert!(MenuBar::is_visible(&widget));
  window.set_visible(true);
  assert!(MenuBar::is_visible(&widget));
  MenuBar::set_visible(&widget, false);
  assert!(!MenuBar::is_visible(&widget));
  assert!(MenuBar::get(&widget).is_some(), "hidden, not removed");
  MenuBar::set_visible(&widget, true);
  assert!(MenuBar::is_visible(&widget));
  window.set_visible(false);
  assert!(MenuBar::is_visible(&widget));
  MenuBar::set_visible(&widget, false);
  assert!(!MenuBar::is_visible(&widget));
}

fn menubar_dead_window() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let menu = bar.add_menu("File", true, None).unwrap();
  drop(window);
  assert!(MenuBar::get(&widget).is_none());
  assert!(MenuBar::get_or_create(&widget).is_none());
  MenuBar::remove(&widget);
  MenuBar::set_visible(&widget, true);
  assert!(!MenuBar::is_visible(&widget));
  assert_eq!(debug::menubar_tree(&widget), "");
  assert!(!debug::trigger_menu_item(&widget, "File"));
  // Containers of the dead window are inert too.
  assert!(bar.add_menu("x", true, None).is_none());
  assert!(menu.add_menu("x", true, None).is_none());
  bar.add_separator();
  menu.add_action(&action("y"), None);
  bar.clear();
}

fn menubar_layout() {
  init();
  let window = shown_window(1);
  let widget = window.widget();
  let w = Rc::new(window);
  let (w0, h0) = {
    let w = w.clone();
    let size = Rc::new(Cell::new((0, 0)));
    let s = size.clone();
    wait_for("the content to be laid out", move || {
      s.set(w.content_size());
      s.get().1 > 0
    });
    size.get()
  };
  let bar = MenuBar::get_or_create(&widget).unwrap();
  bar.add_menu("&File", true, None);
  {
    let w = w.clone();
    wait_for("the content to shrink below the menubar", move || {
      let (cw, ch) = w.content_size();
      cw == w0 && ch < h0
    });
  }
  MenuBar::set_visible(&widget, false);
  {
    let w = w.clone();
    wait_for("the content to grow back while hidden", move || {
      w.content_size() == (w0, h0)
    });
  }
  MenuBar::set_visible(&widget, true);
  {
    let w = w.clone();
    wait_for("the content to shrink again", move || {
      w.content_size().1 < h0
    });
  }
  MenuBar::remove(&widget);
  let w2 = w.clone();
  wait_for("the content to grow back after remove", move || {
    w2.content_size() == (w0, h0)
  });
}

fn menubar_no_layout() {
  init();
  let window = shown_window(1);
  let outer = MenuBar::get_or_create(&window.widget()).unwrap();
  let menu = outer.add_menu("M", true, None).unwrap();
  // A QMenu has no layout: one is created to host the bar. Odd, but it must not crash.
  let inner = MenuBar::get_or_create(menu.widget()).expect("bar on a layout-less widget");
  inner.add_menu("Inner", true, None);
  assert_eq!(debug::menubar_tree(menu.widget()), "Inner\n");
  assert_eq!(
    MenuBar::get(menu.widget()).unwrap().widget(),
    inner.widget()
  );
  MenuBar::remove(menu.widget());
  assert!(MenuBar::get(menu.widget()).is_none());
  flush();
}

// ---------------------------------------------------------------------------------------------
// Container

fn container_nested() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let file = bar.add_menu("&File", true, None).unwrap();
  let recent = file.add_menu("&Recent", true, None).unwrap();
  let deeper = recent.add_menu("Deeper", false, None).unwrap();
  deeper.add_action(&action("Leaf"), None);
  recent.add_action(&action("a.txt"), None);
  file.add_action(&action("Quit"), None);
  bar.add_menu("&Edit", true, None);
  assert_eq!(
    debug::menubar_tree(&widget),
    "File\n  Recent\n    Deeper {disabled}\n      Leaf\n    a.txt\n  Quit\nEdit\n"
  );
  assert!(debug::trigger_menu_item(&widget, "File/Recent/Deeper/Leaf"));
  assert!(debug::trigger_menu_item(&widget, "File/Recent/a.txt"));
}

fn container_action_flags() {
  init();
  let (_window, widget, file) = window_with_file_menu();
  file.add_action(
    &ActionOptions {
      text: "Off",
      enabled: false,
      ..Default::default()
    },
    None,
  );
  file.add_action(
    &ActionOptions {
      text: "Unchecked",
      checkable: true,
      ..Default::default()
    },
    None,
  );
  file.add_action(
    &ActionOptions {
      text: "Checked",
      checkable: true,
      checked: true,
      ..Default::default()
    },
    None,
  );
  file.add_action(
    &ActionOptions {
      text: "CheckedNotCheckable",
      checked: true,
      ..Default::default()
    },
    None,
  );
  file.add_action(
    &ActionOptions {
      text: "All",
      enabled: false,
      checkable: true,
      checked: true,
      shortcut: Some("Ctrl+Q"),
      ..Default::default()
    },
    None,
  );
  assert_eq!(
    debug::menubar_tree(&widget),
    "File\n  Off {disabled}\n  [ ] Unchecked\n  [x] Checked\n  CheckedNotCheckable\n  [x] All <Ctrl+Q> {disabled}\n"
  );
}

fn container_shortcuts() {
  init();
  let (_window, widget, file) = window_with_file_menu();
  let add = |text, shortcut, shortcut_hint| {
    file.add_action(
      &ActionOptions {
        text,
        shortcut,
        shortcut_hint,
        ..Default::default()
      },
      None,
    )
  };
  add("New", Some("Ctrl+Shift+N"), None);
  add("Copy", None, Some("Ctrl+C"));
  add("Both", Some("Ctrl+B"), Some("Ctrl+H"));
  add("Empty", Some(""), Some(""));
  add("Garbage", Some("NotAKey+++Nope"), None);
  add("Function", Some("F5"), None);
  let tree = debug::menubar_tree(&widget);
  let lines: Vec<_> = tree.lines().collect();
  assert_eq!(lines[1], "  New <Ctrl+Shift+N>", "{tree}");
  assert_eq!(lines[2], "  Copy (Ctrl+C)", "{tree}");
  // A registered shortcut wins over the hint in the dump; the hint isn't part of the label.
  assert_eq!(lines[3], "  Both <Ctrl+B>", "{tree}");
  assert_eq!(lines[4], "  Empty", "{tree}");
  assert!(lines[5].starts_with("  Garbage"), "{tree}");
  assert_eq!(lines[6], "  Function <F5>", "{tree}");
  // The hint doesn't change the label used for paths.
  assert!(debug::trigger_menu_item(&widget, "File/Copy"));
  assert!(debug::trigger_menu_item(&widget, "File/Both"));
}

fn container_icons() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let px = [255u8; 4 * 4 * 4];
  let ok = Icon {
    rgba: &px,
    width: 4,
    height: 4,
  };
  let too_small = Icon {
    rgba: &px[..10],
    width: 4,
    height: 4,
  };
  let zero = Icon {
    rgba: &[],
    width: 0,
    height: 0,
  };
  let zero_w = Icon {
    rgba: &px,
    width: 0,
    height: 1000,
  };
  let huge = Icon {
    rgba: &px,
    width: 1 << 20,
    height: 1 << 20,
  };
  let negative = Icon {
    rgba: &[],
    width: u32::MAX,
    height: 0,
  };
  let larger_buffer = Icon {
    rgba: &px,
    width: 2,
    height: 2,
  };
  for (i, icon) in [ok, too_small, zero, zero_w, huge, negative, larger_buffer]
    .into_iter()
    .enumerate()
  {
    let text = format!("M{i}");
    let menu = bar.add_menu(&text, true, Some(icon)).expect("menu");
    menu.add_action(
      &ActionOptions {
        text: "item",
        icon: Some(icon),
        ..Default::default()
      },
      None,
    );
  }
  show_about("About", "<b>hi</b>", Some(ok));
  show_about("About", "hi", Some(too_small));
  let tree = debug::menubar_tree(&widget);
  assert_eq!(tree.lines().count(), 14, "{tree}");
  flush();
}

fn container_labels() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("&&Tools && &More", true, None).unwrap();
  let count = Rc::new(Cell::new(0));
  for text in [
    "Save && &Quit",
    "",
    "Ünïcødé 日本語 🎉",
    "a\0b",
    "trailing&",
    "tab\tinside",
  ] {
    let c = count.clone();
    m.add_action(&action(text), Some(Box::new(move || c.set(c.get() + 1))));
  }
  assert_eq!(
    debug::menubar_tree(&widget),
    "&Tools & More\n  Save & Quit\n  \n  Ünïcødé 日本語 🎉\n  ab\n  trailing\n  tab (inside)\n"
  );
  assert!(debug::trigger_menu_item(
    &widget,
    "&Tools & More/Save & Quit"
  ));
  assert!(debug::trigger_menu_item(
    &widget,
    "&Tools & More/Ünïcødé 日本語 🎉"
  ));
  assert!(debug::trigger_menu_item(&widget, "&Tools & More/ab"));
  assert!(debug::trigger_menu_item(&widget, "&Tools & More/"));
  assert_eq!(count.get(), 4);
  // Interior NULs in the path are dropped too.
  assert!(debug::trigger_menu_item(&widget, "&Tools & More/a\0b"));
  assert_eq!(count.get(), 5);
}

fn container_separators() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("M", true, None).unwrap();
  m.add_separator();
  m.add_action(&action("A"), None);
  m.add_separator();
  m.add_separator();
  bar.add_separator();
  bar.add_menu("N", true, None);
  assert_eq!(
    debug::menubar_tree(&widget),
    "M\n  ---\n  A\n  ---\n  ---\n---\nN\n"
  );
  // Separators are skipped when resolving paths.
  assert!(debug::trigger_menu_item(&widget, "N"));
  assert!(!debug::trigger_menu_item(&widget, "---"));
  assert!(!debug::trigger_menu_item(&widget, "M/"));
}

fn container_clear() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("M", true, None).unwrap();
  let sub = m.add_menu("Sub", true, None).unwrap();
  sub.add_action(&action("x"), None);
  m.add_separator();
  m.clear();
  assert_eq!(debug::menubar_tree(&widget), "M\n");
  // The cleared submenu goes away with the next event loop pass.
  let s = sub.widget().clone();
  wait_for("the cleared submenu to be deleted", move || !s.is_alive());
  m.add_action(&action("again"), None);
  assert_eq!(debug::menubar_tree(&widget), "M\n  again\n");
  bar.clear();
  assert_eq!(debug::menubar_tree(&widget), "");
  bar.clear();
  let mw = m.widget().clone();
  wait_for("the cleared menu to be deleted", move || !mw.is_alive());
  // The bar itself survives.
  assert!(bar.widget().is_alive());
  assert!(MenuBar::get(&widget).is_some());
}

fn container_dead() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("M", true, None).unwrap();
  bar.clear();
  let mw = m.widget().clone();
  wait_for("the menu to be deleted", move || !mw.is_alive());
  assert!(m.add_menu("x", true, None).is_none());
  let (count, dropped, cb) = counting();
  m.add_action(&action("x"), Some(cb));
  assert!(
    dropped.get(),
    "the callback of an action that was never added is dropped"
  );
  assert_eq!(count.get(), 0);
  m.add_separator();
  m.clear();
  assert_eq!(debug::menubar_tree(&widget), "");
}

// ---------------------------------------------------------------------------------------------
// callbacks

fn callback_main_thread() {
  init();
  let (_window, widget, _file) = window_with_file_menu();
  let bar = MenuBar::get(&widget).unwrap();
  let main = thread::current().id();
  let ran_on = Rc::new(Cell::new(None));
  let r = ran_on.clone();
  bar.add_menu("Edit", true, None).unwrap().add_action(
    &action("Go"),
    Some(Box::new(move || r.set(Some(thread::current().id())))),
  );
  // Triggered from inside the event loop.
  let w = widget.clone();
  app::post(move || assert!(debug::trigger_menu_item(&w, "Edit/Go")));
  let r = ran_on.clone();
  wait_for("the callback", move || r.get().is_some());
  assert_eq!(ran_on.get(), Some(main));
}

fn trigger_paths() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let file = bar.add_menu("&File", true, None).unwrap();
  let (count, _dropped, cb) = counting();
  file.add_action(&action("&Open"), Some(cb));
  file.add_action(&action("Open"), None); // duplicate label: the first one wins
  assert!(debug::trigger_menu_item(&widget, "File/Open"));
  assert_eq!(count.get(), 1);
  assert!(!debug::trigger_menu_item(&widget, "File/Missing"));
  assert!(!debug::trigger_menu_item(&widget, "Missing/Open"));
  assert!(
    !debug::trigger_menu_item(&widget, "&File/Open"),
    "labels without `&`"
  );
  assert!(!debug::trigger_menu_item(&widget, "File/Open/Deeper"));
  assert!(!debug::trigger_menu_item(&widget, ""));
  assert!(!debug::trigger_menu_item(&widget, "/"));
  assert!(!debug::trigger_menu_item(&widget, "File/"));
  assert!(
    !debug::trigger_menu_item(&widget, "file/open"),
    "case-sensitive"
  );
  // A path naming a submenu triggers its menu action.
  assert!(debug::trigger_menu_item(&widget, "File"));
  assert_eq!(count.get(), 1);
}

fn trigger_disabled() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let file = bar.add_menu("File", true, None).unwrap();
  let (count, _dropped, cb) = counting();
  file.add_action(
    &ActionOptions {
      text: "Off",
      enabled: false,
      ..Default::default()
    },
    Some(cb),
  );
  let (count2, _d2, cb2) = counting();
  let disabled_menu = bar.add_menu("Disabled", false, None).unwrap();
  disabled_menu.add_action(&action("Inner"), Some(cb2));
  // The item exists (true) but Qt doesn't trigger disabled actions.
  assert!(debug::trigger_menu_item(&widget, "File/Off"));
  assert_eq!(count.get(), 0);
  // An enabled item inside a disabled menu: a user can't reach it, but `trigger_menu_item`
  // walks the tree and triggers the (enabled) action itself.
  assert!(debug::trigger_menu_item(&widget, "Disabled/Inner"));
  assert_eq!(count2.get(), 1);
}

fn trigger_checkable() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("M", true, None).unwrap();
  let (count, _dropped, cb) = counting();
  m.add_action(
    &ActionOptions {
      text: "Toggle",
      checkable: true,
      ..Default::default()
    },
    Some(cb),
  );
  assert_eq!(debug::menubar_tree(&widget), "M\n  [ ] Toggle\n");
  assert!(debug::trigger_menu_item(&widget, "M/Toggle"));
  assert_eq!(debug::menubar_tree(&widget), "M\n  [x] Toggle\n");
  assert!(debug::trigger_menu_item(&widget, "M/Toggle"));
  assert_eq!(debug::menubar_tree(&widget), "M\n  [ ] Toggle\n");
  assert_eq!(count.get(), 2);
}

fn callback_dropped_on_clear() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("M", true, None).unwrap();
  let (_c1, dropped_top, cb1) = counting();
  let (_c2, dropped_nested, cb2) = counting();
  m.add_action(&action("A"), Some(cb1));
  m.add_menu("Sub", true, None)
    .unwrap()
    .add_action(&action("B"), Some(cb2));
  m.clear();
  assert!(!dropped_top.get(), "deleted later, not now");
  let (a, b) = (dropped_top.clone(), dropped_nested.clone());
  wait_for("the callbacks to be dropped", move || a.get() && b.get());
}

fn callback_dropped_on_remove() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let (_c, dropped, cb) = counting();
  bar
    .add_menu("M", true, None)
    .unwrap()
    .add_action(&action("A"), Some(cb));
  MenuBar::remove(&widget);
  let d = dropped.clone();
  wait_for("the callback to be dropped", move || d.get());
}

fn callback_dropped_on_window() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let (_c, dropped, cb) = counting();
  bar
    .add_menu("M", true, None)
    .unwrap()
    .add_action(&action("A"), Some(cb));
  drop(window);
  assert!(
    dropped.get(),
    "outside the event loop the window is deleted right away"
  );
}

fn callback_dropped_at_shutdown() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let (_c, dropped, cb) = counting();
  bar
    .add_menu("M", true, None)
    .unwrap()
    .add_action(&action("A"), Some(cb));
  let (_c, popup_dropped, cb) = counting();
  let popup = PopupMenu::new(&widget).unwrap();
  popup.add_action(&action("P"), Some(cb));
  app::shutdown();
  assert!(dropped.get(), "menubar callback dropped at shutdown");
  assert!(popup_dropped.get(), "popup callback dropped at shutdown");
  drop((popup, bar, widget, window));
}

fn callback_panic() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let m = bar.add_menu("M", true, None).unwrap();
  let calls = Rc::new(Cell::new(0));
  let c = calls.clone();
  m.add_action(
    &action("Boom"),
    Some(Box::new(move || {
      c.set(c.get() + 1);
      panic!("menu callback panic (expected)");
    })),
  );
  assert!(debug::trigger_menu_item(&widget, "M/Boom"));
  assert!(debug::trigger_menu_item(&widget, "M/Boom"));
  assert_eq!(calls.get(), 2);
  // Also from inside the event loop.
  let w = widget.clone();
  app::post(move || {
    debug::trigger_menu_item(&w, "M/Boom");
  });
  let c = calls.clone();
  wait_for("the third call", move || c.get() == 3);
  assert!(app::is_running());
}

fn callback_rebuilds() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let generation = Rc::new(Cell::new(0u32));
  fn build(widget: &Widget, generation: Rc<Cell<u32>>) {
    let bar = MenuBar::get_or_create(widget).unwrap();
    bar.clear();
    let g = generation.get();
    let m = bar.add_menu(&format!("Gen{g}"), true, None).unwrap();
    let w = widget.clone();
    let generation2 = generation.clone();
    m.add_action(
      &action("Rebuild"),
      Some(Box::new(move || {
        generation2.set(generation2.get() + 1);
        build(&w, generation2.clone());
        // Touch our own captures after the rebuild.
        assert!(w.is_alive());
      })),
    );
  }
  build(&widget, generation.clone());
  drop(bar);
  assert!(debug::trigger_menu_item(&widget, "Gen0/Rebuild"));
  assert_eq!(debug::menubar_tree(&widget), "Gen1\n  Rebuild\n");
  assert!(debug::trigger_menu_item(&widget, "Gen1/Rebuild"));
  assert!(!debug::trigger_menu_item(&widget, "Gen1/Rebuild"));
  // Inside the event loop, with deferred deletes running in between.
  let w = widget.clone();
  app::post(move || {
    debug::trigger_menu_item(&w, "Gen2/Rebuild");
  });
  let g = generation.clone();
  wait_for("the third rebuild", move || g.get() == 3);
  flush();
  assert_eq!(debug::menubar_tree(&widget), "Gen3\n  Rebuild\n");
  assert!(debug::trigger_menu_item(&widget, "Gen3/Rebuild"));
  assert_eq!(generation.get(), 4);
}

fn callback_removes_bar() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let w = widget.clone();
  let dropped = Rc::new(Cell::new(false));
  let flag = DropFlag(dropped.clone());
  bar.add_menu("M", true, None).unwrap().add_action(
    &action("Remove"),
    Some(Box::new(move || {
      let _ = &flag;
      MenuBar::remove(&w);
      let fresh = MenuBar::get_or_create(&w).unwrap();
      fresh.add_menu("Fresh", true, None);
    })),
  );
  assert!(debug::trigger_menu_item(&widget, "M/Remove"));
  assert!(!dropped.get());
  assert_eq!(debug::menubar_tree(&widget), "Fresh\n");
  let d = dropped.clone();
  wait_for("the old callback to be dropped", move || d.get());
}

fn callback_recursive() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let depth = Rc::new(Cell::new(0u32));
  let d = depth.clone();
  let w = widget.clone();
  bar.add_menu("M", true, None).unwrap().add_action(
    &action("Again"),
    Some(Box::new(move || {
      d.set(d.get() + 1);
      if d.get() < 5 {
        assert!(debug::trigger_menu_item(&w, "M/Again"));
      }
    })),
  );
  assert!(debug::trigger_menu_item(&widget, "M/Again"));
  assert_eq!(depth.get(), 5);
}

fn callback_shutdown() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let refused = Rc::new(Cell::new(false));
  let r = refused.clone();
  bar.add_menu("M", true, None).unwrap().add_action(
    &action("Shutdown"),
    Some(Box::new(move || {
      let res = std::panic::catch_unwind(app::shutdown);
      r.set(res.is_err());
    })),
  );
  assert!(debug::trigger_menu_item(&widget, "M/Shutdown"));
  assert!(refused.get(), "shutdown inside a callback must panic");
  assert!(app::is_running());
  assert!(widget.is_alive());
}

thread_local! {
  static CB_DROPPED: Cell<bool> = const { Cell::new(false) };
  static FREED_WHILE_RUNNING: Cell<Option<bool>> = const { Cell::new(None) };
}

struct TlDropFlag;
impl Drop for TlDropFlag {
  fn drop(&mut self) {
    CB_DROPPED.with(|d| d.set(true));
  }
}

/// A window whose menubar item `M/Close` drops the window from its callback, and records
/// whether its own closure was freed while running.
fn window_dropping_callback() -> Widget {
  let slot = Rc::new(RefCell::new(Some(Window::new(1, false))));
  let widget = slot.borrow().as_ref().unwrap().widget();
  let bar = MenuBar::get_or_create(&widget).unwrap();
  let guard = TlDropFlag;
  let s = slot.clone();
  bar.add_menu("M", true, None).unwrap().add_action(
    &action("Close"),
    Some(Box::new(move || {
      let _ = &guard;
      let window = s.borrow_mut().take();
      drop(window);
      // Only thread-locals from here: our captures may be gone.
      FREED_WHILE_RUNNING.with(|f| f.set(Some(CB_DROPPED.with(Cell::get))));
    })),
  );
  widget
}

fn callback_drops_window_in_loop() {
  init();
  let widget = window_dropping_callback();
  let w = widget.clone();
  app::post(move || {
    debug::trigger_menu_item(&w, "M/Close");
  });
  wait_for("the callback", || {
    FREED_WHILE_RUNNING.with(Cell::get).is_some()
  });
  assert_eq!(FREED_WHILE_RUNNING.with(Cell::get), Some(false));
  wait_for("the window to be deleted", move || !widget.is_alive());
  assert!(CB_DROPPED.with(Cell::get));
}

fn callback_drops_window_outside_loop() {
  init();
  let widget = window_dropping_callback();
  assert!(debug::trigger_menu_item(&widget, "M/Close"));
  assert_eq!(
    FREED_WHILE_RUNNING.with(Cell::get),
    Some(false),
    "the running callback was freed by its own window drop"
  );
}

// ---------------------------------------------------------------------------------------------
// PopupMenu

fn popup_dead_parent() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  drop(window);
  assert!(PopupMenu::new(&widget).is_none());
}

fn popup_show_cursor() {
  init();
  assert_eq!(debug::take_popup_menu(), ""); // starts recording
  let window = shown_window(1);
  let widget = window.widget();
  let popup = PopupMenu::new(&widget).unwrap();
  popup.add_action(&action("&Cut"), None);
  popup.add_separator();
  popup.add_action(
    &ActionOptions {
      text: "Check",
      checkable: true,
      checked: true,
      shortcut_hint: Some("Ctrl+K"),
      ..Default::default()
    },
    None,
  );
  let menu = popup.widget().clone();
  popup.show(None);
  let tree = Rc::new(RefCell::new(String::new()));
  let t = tree.clone();
  wait_for("the popup to be recorded", move || {
    *t.borrow_mut() = debug::take_popup_menu();
    !t.borrow().is_empty()
  });
  assert_eq!(*tree.borrow(), "Cut\n---\n[x] Check (Ctrl+K)\n");
  // Taken: nothing more to report, and the menu was closed (and deleted on close).
  assert_eq!(debug::take_popup_menu(), "");
  wait_for("the popup to be deleted", move || !menu.is_alive());
}

fn popup_show_positions() {
  init();
  debug::take_popup_menu();
  let window = shown_window(1);
  let widget = window.widget();
  for (i, pos) in [
    PopupPosition::Logical { x: 10.0, y: 20.0 },
    PopupPosition::Physical { x: 10.0, y: 20.0 },
    PopupPosition::Logical { x: -50.0, y: 1e9 },
    PopupPosition::Physical { x: 0.0, y: -0.0 },
  ]
  .into_iter()
  .enumerate()
  {
    let popup = PopupMenu::new(&widget).unwrap();
    let label = format!("Item{i}");
    popup.add_action(&action(&label), None);
    popup.show(Some(pos));
    let expected = format!("{label}\n");
    wait_for("the popup to be recorded", move || {
      debug::take_popup_menu() == expected
    });
  }
}

/// Positions that don't fit an `int` once scaled.
fn popup_show_bad_positions() {
  init();
  debug::take_popup_menu();
  let window = shown_window(1);
  let widget = window.widget();
  for pos in [
    PopupPosition::Logical {
      x: f64::NAN,
      y: 0.0,
    },
    PopupPosition::Physical {
      x: 0.0,
      y: f64::INFINITY,
    },
    PopupPosition::Logical { x: 1e10, y: -1e10 },
  ] {
    let popup = PopupMenu::new(&widget).unwrap();
    popup.add_action(&action("Item"), None);
    popup.show(Some(pos));
    wait_for("the popup to be recorded", || {
      debug::take_popup_menu() == "Item\n"
    });
  }
}

/// A popup whose parent is a menu isn't recorded on show (it looks like a submenu), but
/// `take_popup_menu` still reports it while it's the open popup.
fn popup_parented_to_menu() {
  init();
  debug::take_popup_menu();
  let window = shown_window(1);
  let bar = MenuBar::get_or_create(&window.widget()).unwrap();
  let menu = bar.add_menu("M", true, None).unwrap();
  let popup = PopupMenu::new(menu.widget()).unwrap();
  popup.add_action(&action("Child"), None);
  let p = popup.widget().clone();
  popup.show(Some(PopupPosition::Logical { x: 1.0, y: 1.0 }));
  wait_for("the popup to show", || {
    debug::take_popup_menu() == "Child\n"
  });
  wait_for("the popup to close", move || !p.is_alive());
}

fn popup_nested() {
  init();
  debug::take_popup_menu();
  let window = shown_window(1);
  let widget = window.widget();
  let popup = PopupMenu::new(&widget).unwrap();
  let sub = popup.add_menu("Sub", true, None).unwrap();
  sub.add_action(&action("Leaf"), None);
  popup.add_menu("Off", false, None);
  popup.show(Some(PopupPosition::Logical { x: 5.0, y: 5.0 }));
  wait_for("the popup to be recorded", || {
    debug::take_popup_menu() == "Sub\n  Leaf\nOff {disabled}\n"
  });
}

fn popup_callbacks_dropped() {
  init();
  debug::take_popup_menu();
  let window = shown_window(1);
  let widget = window.widget();
  let popup = PopupMenu::new(&widget).unwrap();
  let (count, dropped, cb) = counting();
  popup.add_action(&action("A"), Some(cb));
  popup.show(None);
  wait_for("the popup to be recorded", || {
    !debug::take_popup_menu().is_empty()
  });
  let d = dropped.clone();
  wait_for("the popup callback to be dropped", move || d.get());
  assert_eq!(count.get(), 0);
}

fn popup_dropped_unshown() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let (_c, dropped, cb) = counting();
  {
    let popup = PopupMenu::new(&widget).unwrap();
    popup.add_action(&action("A"), Some(cb));
  }
  flush();
  // The menu stays a child of the window until the window goes.
  assert!(!dropped.get());
  drop(window);
  assert!(dropped.get());
}

fn popup_parent_died() {
  init();
  debug::take_popup_menu();
  let window = shown_window(1);
  let widget = window.widget();
  let popup = PopupMenu::new(&widget).unwrap();
  popup.add_action(&action("A"), None);
  let menu = popup.widget().clone();
  drop(window);
  assert!(!menu.is_alive(), "the popup is a child of the window");
  popup.add_action(&action("B"), None);
  assert!(popup.add_menu("C", true, None).is_none());
  popup.show(None);
  flush();
  assert_eq!(debug::take_popup_menu(), "");
}

// ---------------------------------------------------------------------------------------------
// StandardAction

const ALL_ACTIONS: [StandardAction; 12] = [
  StandardAction::Copy,
  StandardAction::Cut,
  StandardAction::Paste,
  StandardAction::SelectAll,
  StandardAction::Undo,
  StandardAction::Redo,
  StandardAction::Delete,
  StandardAction::Minimize,
  StandardAction::Maximize,
  StandardAction::Fullscreen,
  StandardAction::Hide,
  StandardAction::CloseWindow,
];

fn standard_editing() {
  init();
  for a in ALL_ACTIONS.into_iter().filter(|a| a.is_editing()) {
    a.trigger();
  }
  // With a window but no focus widget.
  let _window = shown_window(1);
  for a in ALL_ACTIONS.into_iter().filter(|a| a.is_editing()) {
    a.trigger();
  }
  flush();
}

fn standard_window_none() {
  init();
  for a in ALL_ACTIONS {
    a.trigger();
  }
  // A hidden window is never the active one.
  let window = Window::new(1, false);
  record_events(|_| Reply::default());
  for a in ALL_ACTIONS {
    a.trigger();
  }
  flush();
  assert!(!has_event(|e| e.kind == EventKind::WindowCloseRequested));
  drop(window);
}

fn standard_window_active() {
  init();
  record_events(|_| Reply::default());
  let window = Rc::new(shown_window(1));
  window.focus();
  let w = window.clone();
  wait_for("the window to be active", move || w.state() & 8 != 0);
  StandardAction::CloseWindow.trigger();
  wait_for("a close request", || {
    has_event(|e| e.kind == EventKind::WindowCloseRequested && e.window == 1)
  });
  StandardAction::Maximize.trigger();
  let w = window.clone();
  wait_for("maximized", move || w.state() & 2 != 0);
  StandardAction::Maximize.trigger();
  let w = window.clone();
  wait_for("unmaximized", move || w.state() & 2 == 0);
  StandardAction::Fullscreen.trigger();
  let w = window.clone();
  wait_for("fullscreen", move || w.state() & 4 != 0);
  StandardAction::Fullscreen.trigger();
  let w = window.clone();
  wait_for("not fullscreen", move || w.state() & 4 == 0);
  StandardAction::Minimize.trigger();
  let w = window.clone();
  wait_for("minimized", move || w.state() & 1 != 0);
  window.focus();
  let w = window.clone();
  wait_for("active again", move || w.state() & 9 == 8);
  StandardAction::Hide.trigger();
  let w = window.clone();
  wait_for("hidden", move || w.state() & 16 == 0);
}

fn standard_quit() {
  init();
  app::post(|| StandardAction::Quit.trigger());
  assert_eq!(app::exec(), 0);
  // Quit outside the loop is harmless, and the app keeps working.
  StandardAction::Quit.trigger();
  assert!(!StandardAction::Quit.is_editing());
  let ran = Rc::new(Cell::new(false));
  let r = ran.clone();
  app::post(move || r.set(true));
  let r = ran.clone();
  wait_for("the loop to run again", move || r.get());
}

fn about_non_blocking() {
  init();
  show_about("About", "<h1>Title</h1><p>body</p>", None);
  show_about("", "", None);
  show_about("nul\0title", "a\0b", None);
  let window = shown_window(1);
  window.focus();
  show_about("About with a window", "x", None);
  flush();
  // Several boxes can be open; shutdown cleans them up.
}

// ---------------------------------------------------------------------------------------------
// debug

fn debug_no_menubar() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  assert_eq!(debug::menubar_tree(&widget), "");
  assert!(!debug::trigger_menu_item(&widget, "File"));
  assert!(!debug::trigger_menu_item(&widget, ""));
  assert_eq!(debug::take_popup_menu(), "");
  // An empty menubar.
  MenuBar::get_or_create(&widget).unwrap();
  assert_eq!(debug::menubar_tree(&widget), "");
  assert!(!debug::trigger_menu_item(&widget, "File"));
}

fn debug_take_popup_empty() {
  init();
  assert_eq!(debug::take_popup_menu(), "");
  assert_eq!(debug::take_popup_menu(), "");
}
