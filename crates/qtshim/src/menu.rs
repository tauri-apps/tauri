// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Menus: `QMenuBar`, `QMenu` and `QAction`.
//!
//! Widgets are owned by Qt, so the types here are *tracked handles*: each one watches its widget
//! (a `QPointer`) and becomes inert once Qt destroys it — operations then do nothing or return
//! `None` instead of touching freed memory.
//!
//! Like the rest of the crate this must be used on the main thread (see [`crate::app`]); the
//! handles are not `Send`.

use std::{
  ffi::{c_int, c_void},
  marker::PhantomData,
  ptr::NonNull,
};

use qtshim_sys as sys;

use crate::{
  lifecycle,
  util::{boxed, cstring, drop_box, take_cstring},
};

/// A tracked handle to a live `QWidget` of this application.
///
/// Get one from a window created by this crate, or validate a pointer received from elsewhere
/// with [`Widget::from_ptr`]. The handle notices when Qt destroys the widget.
#[derive(Debug)]
pub struct Widget {
  /// A heap `QPointer<QWidget>`.
  tracker: NonNull<c_void>,
  _not_send: PhantomData<*const ()>,
}

impl Widget {
  /// Validate `ptr` as a live `QWidget` of this application and track it. Returns `None` for
  /// anything else, including null or dangling pointers: `ptr` is only compared against the
  /// application's widgets, never dereferenced.
  // `qs_widget_find` only compares `ptr` with the addresses of live widgets.
  #[allow(clippy::not_unsafe_ptr_arg_deref)]
  pub fn from_ptr(ptr: *mut c_void) -> Option<Self> {
    lifecycle::ensure();
    let widget = NonNull::new(unsafe { sys::qs_widget_find(ptr) })?;
    // SAFETY: `widget` was just found among the live widgets.
    Some(unsafe { Self::track(widget) })
  }

  /// Track a widget known to be alive.
  ///
  /// # Safety
  ///
  /// `widget` must point to a live `QWidget`, on the main thread.
  pub(crate) unsafe fn track(widget: NonNull<c_void>) -> Self {
    let tracker = unsafe { sys::qs_widget_track(widget.as_ptr()) };
    Self {
      tracker: NonNull::new(tracker).expect("qs_widget_track returned null"),
      _not_send: PhantomData,
    }
  }

  /// Whether the widget still exists.
  pub fn is_alive(&self) -> bool {
    lifecycle::ensure();
    self.get().is_some()
  }

  /// The widget's address while it exists, e.g. to hand it to another library. Using the
  /// pointer is up to the receiver.
  pub fn as_ptr(&self) -> Option<NonNull<c_void>> {
    lifecycle::ensure();
    self.get()
  }

  /// The widget, if alive. Callers must have checked [`lifecycle::ensure`].
  fn get(&self) -> Option<NonNull<c_void>> {
    NonNull::new(unsafe { sys::qs_widget_get(self.tracker.as_ptr()) })
  }
}

impl Clone for Widget {
  /// Another handle to the same widget (inert if the widget is gone).
  fn clone(&self) -> Self {
    lifecycle::ensure();
    match self.get() {
      // SAFETY: the tracker says the widget is alive.
      Some(widget) => unsafe { Self::track(widget) },
      None => Self {
        tracker: NonNull::new(unsafe { sys::qs_widget_track(std::ptr::null_mut()) })
          .expect("qs_widget_track returned null"),
        _not_send: PhantomData,
      },
    }
  }
}

impl PartialEq for Widget {
  /// Two handles are equal if they point to the same live widget.
  fn eq(&self, other: &Self) -> bool {
    lifecycle::ensure();
    matches!((self.get(), other.get()), (Some(a), Some(b)) if a == b)
  }
}

impl Drop for Widget {
  fn drop(&mut self) {
    // After `app::shutdown` the tracker's memory is left alone: Qt is gone.
    if lifecycle::is_running_here() {
      unsafe { sys::qs_widget_untrack(self.tracker.as_ptr()) }
    }
  }
}

/// An RGBA image used as a menu or item icon.
#[derive(Debug, Clone, Copy)]
pub struct Icon<'a> {
  /// `width * height * 4` bytes of RGBA pixels.
  pub rgba: &'a [u8],
  /// Width in pixels.
  pub width: u32,
  /// Height in pixels.
  pub height: u32,
}

impl Icon<'_> {
  /// The pixels and size as FFI arguments; null if the buffer is too small.
  fn raw(icon: Option<&Self>) -> (*const u8, c_int, c_int) {
    match icon.and_then(|i| {
      Some((
        i.rgba.as_ptr(),
        crate::util::rgba_dims(i.rgba.len(), i.width, i.height)?,
      ))
    }) {
      Some((rgba, (w, h))) => (rgba, w, h),
      None => (std::ptr::null(), 0, 0),
    }
  }
}

/// How an action looks and behaves. Used with [`Container::add_action`].
#[derive(Debug, Clone, Copy)]
pub struct ActionOptions<'a> {
  /// The label; `&` marks the mnemonic (`&&` for a literal `&`).
  pub text: &'a str,
  /// Whether the action can be triggered.
  pub enabled: bool,
  /// Whether the action has a check mark.
  pub checkable: bool,
  /// The initial check state (with `checkable`).
  pub checked: bool,
  /// A shortcut that triggers the action while its window is active, as a `QKeySequence`
  /// portable string such as `"Ctrl+Shift+N"`.
  pub shortcut: Option<&'a str>,
  /// A shortcut that is only *displayed* next to the label (e.g. `"Ctrl+C"` for a Copy item,
  /// whose key the focused widget handles itself).
  pub shortcut_hint: Option<&'a str>,
  /// An icon shown next to the label.
  pub icon: Option<Icon<'a>>,
}

impl Default for ActionOptions<'_> {
  fn default() -> Self {
    Self {
      text: "",
      enabled: true,
      checkable: false,
      checked: false,
      shortcut: None,
      shortcut_hint: None,
      icon: None,
    }
  }
}

mod sealed {
  pub trait Sealed {}
}

/// A menubar or menu that menus, actions and separators can be added to. Once Qt destroys the
/// container, these methods do nothing.
///
/// This trait is sealed: it's implemented by [`MenuBar`], [`Menu`] and [`PopupMenu`].
pub trait Container: sealed::Sealed {
  /// The tracked `QMenuBar` / `QMenu`.
  fn widget(&self) -> &Widget;

  /// Remove everything from this container (Qt deletes the removed widgets later).
  fn clear(&self) {
    if let Some(container) = self.widget().as_ptr() {
      unsafe { sys::qs_container_clear(container.as_ptr()) }
    }
  }

  /// Add a submenu and return it (`None` if this container is gone).
  fn add_menu(&self, text: &str, enabled: bool, icon: Option<Icon<'_>>) -> Option<Menu> {
    let container = self.widget().as_ptr()?;
    let text = cstring(text);
    let (rgba, w, h) = Icon::raw(icon.as_ref());
    let menu = unsafe {
      sys::qs_container_add_menu(
        container.as_ptr(),
        text.as_ptr(),
        c_int::from(enabled),
        rgba,
        w,
        h,
      )
    };
    // SAFETY: Qt just created the menu.
    NonNull::new(menu).map(|menu| Menu {
      widget: unsafe { Widget::track(menu) },
    })
  }

  /// Add an action. `on_click` runs on the main thread each time it's triggered and is dropped
  /// together with the action.
  fn add_action(&self, options: &ActionOptions<'_>, on_click: Option<Box<dyn Fn()>>) {
    type Callback = Box<dyn Fn()>;
    unsafe extern "C" fn tramp(ud: *mut c_void) {
      let _guard = lifecycle::enter_callback();
      let f = unsafe { &*(ud as *const Callback) };
      crate::util::catch_panic("a menu action", f);
    }
    let Some(container) = self.widget().as_ptr() else {
      return;
    };
    let text = cstring(options.text);
    let shortcut = options.shortcut.map(cstring);
    let hint = options.shortcut_hint.map(cstring);
    let (rgba, w, h) = Icon::raw(options.icon.as_ref());
    let ud = on_click.map(boxed);
    unsafe {
      sys::qs_container_add_action(
        container.as_ptr(),
        text.as_ptr(),
        c_int::from(options.enabled),
        c_int::from(options.checkable),
        c_int::from(options.checked),
        shortcut.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
        hint.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
        rgba,
        w,
        h,
        ud.map(|_| tramp as sys::qs_fn),
        ud.unwrap_or(std::ptr::null_mut()),
        ud.map(|_| drop_box::<Callback> as sys::qs_drop_fn),
      )
    }
  }

  /// Add a separator.
  fn add_separator(&self) {
    if let Some(container) = self.widget().as_ptr() {
      unsafe { sys::qs_container_add_separator(container.as_ptr()) }
    }
  }
}

/// The `QMenuBar` installed on a window.
#[derive(Debug)]
pub struct MenuBar {
  widget: Widget,
}

impl MenuBar {
  /// The menubar installed on `window`, if any.
  pub fn get(window: &Widget) -> Option<Self> {
    Self::find(window, false)
  }

  /// The menubar of `window`, created if missing. It's installed with
  /// `QMainWindow::setMenuBar`, or at the top of the widget's layout (`QLayout::setMenuBar`; a
  /// `QVBoxLayout` is created if the widget has none). `None` if the window is gone.
  pub fn get_or_create(window: &Widget) -> Option<Self> {
    Self::find(window, true)
  }

  /// Remove (and delete) the menubar of `window`, if any.
  pub fn remove(window: &Widget) {
    if let Some(window) = window.as_ptr() {
      unsafe { sys::qs_menubar_remove(window.as_ptr()) }
    }
  }

  /// Show or hide the menubar of `window`.
  pub fn set_visible(window: &Widget, visible: bool) {
    if let Some(window) = window.as_ptr() {
      unsafe { sys::qs_menubar_set_visible(window.as_ptr(), c_int::from(visible)) }
    }
  }

  /// Whether `window` has a visible menubar.
  pub fn is_visible(window: &Widget) -> bool {
    window
      .as_ptr()
      .is_some_and(|window| unsafe { sys::qs_menubar_is_visible(window.as_ptr()) } != 0)
  }

  fn find(window: &Widget, create: bool) -> Option<Self> {
    let window = window.as_ptr()?;
    let bar = NonNull::new(unsafe { sys::qs_menubar_for(window.as_ptr(), c_int::from(create)) })?;
    // SAFETY: Qt just returned the live menubar.
    Some(Self {
      widget: unsafe { Widget::track(bar) },
    })
  }
}

impl sealed::Sealed for MenuBar {}
impl Container for MenuBar {
  fn widget(&self) -> &Widget {
    &self.widget
  }
}

/// A `QMenu` added to a menubar or another menu.
#[derive(Debug)]
pub struct Menu {
  widget: Widget,
}

impl sealed::Sealed for Menu {}
impl Container for Menu {
  fn widget(&self) -> &Widget {
    &self.widget
  }
}

/// Where to show a [`PopupMenu`], relative to the top-left corner of the parent's window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupPosition {
  /// Logical (device-independent) pixels.
  Logical {
    /// Horizontal offset.
    x: f64,
    /// Vertical offset.
    y: f64,
  },
  /// Physical pixels.
  Physical {
    /// Horizontal offset.
    x: f64,
    /// Vertical offset.
    y: f64,
  },
}

/// A context menu. Fill it through [`Container`], then [`show`](PopupMenu::show) it; Qt deletes
/// it once it closes.
#[derive(Debug)]
pub struct PopupMenu {
  menu: Widget,
  parent: Widget,
}

impl PopupMenu {
  /// Create a context menu for `parent` (being parented to it places it correctly on Wayland).
  /// `None` if `parent` is gone.
  pub fn new(parent: &Widget) -> Option<Self> {
    let parent_ptr = parent.as_ptr()?;
    let menu = NonNull::new(unsafe { sys::qs_popup_menu_new(parent_ptr.as_ptr()) })?;
    Some(Self {
      // SAFETY: Qt just created the menu.
      menu: unsafe { Widget::track(menu) },
      parent: parent.clone(),
    })
  }

  /// Show the menu without blocking, at `position` or at the cursor. Does nothing if the menu or
  /// its parent is gone.
  pub fn show(self, position: Option<PopupPosition>) {
    let (Some(menu), Some(parent)) = (self.menu.as_ptr(), self.parent.as_ptr()) else {
      return;
    };
    let (has_pos, x, y, physical) = match position {
      Some(PopupPosition::Logical { x, y }) if usable(x, y) => (1, x, y, 0),
      Some(PopupPosition::Physical { x, y }) if usable(x, y) => (1, x, y, 1),
      // Non-finite or absurd coordinates (Qt converts them to `int`) show it at the cursor.
      _ => (0, 0.0, 0.0, 0),
    };
    unsafe { sys::qs_popup_menu_show(menu.as_ptr(), parent.as_ptr(), has_pos, x, y, physical) }
  }
}

/// Whether a popup position can be converted to Qt's `int` coordinates (also after scaling).
fn usable(x: f64, y: f64) -> bool {
  const LIMIT: f64 = 1e7;
  x.is_finite() && y.is_finite() && x.abs() <= LIMIT && y.abs() <= LIMIT
}

impl sealed::Sealed for PopupMenu {}
impl Container for PopupMenu {
  fn widget(&self) -> &Widget {
    &self.menu
  }
}

/// Standard actions for predefined menu items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StandardAction {
  /// Copy the selection of the focused widget.
  Copy,
  /// Cut the selection of the focused widget.
  Cut,
  /// Paste into the focused widget.
  Paste,
  /// Select everything in the focused widget.
  SelectAll,
  /// Undo in the focused widget.
  Undo,
  /// Redo in the focused widget.
  Redo,
  /// Delete the selection of the focused widget.
  Delete,
  /// Minimize the active window.
  Minimize,
  /// Toggle maximizing the active window.
  Maximize,
  /// Toggle fullscreen for the active window.
  Fullscreen,
  /// Hide the active window.
  Hide,
  /// Close the active window (through its normal close handling).
  CloseWindow,
  /// Quit the application's event loop.
  Quit,
}

impl StandardAction {
  /// Whether the focused widget handles this action's shortcut itself (editing actions), so a
  /// menu should only *display* the shortcut.
  pub fn is_editing(self) -> bool {
    matches!(
      self,
      Self::Copy
        | Self::Cut
        | Self::Paste
        | Self::SelectAll
        | Self::Undo
        | Self::Redo
        | Self::Delete
    )
  }

  /// Perform the action.
  pub fn trigger(self) {
    lifecycle::ensure();
    let kind = match self {
      Self::Copy => sys::QS_ACTION_COPY,
      Self::Cut => sys::QS_ACTION_CUT,
      Self::Paste => sys::QS_ACTION_PASTE,
      Self::SelectAll => sys::QS_ACTION_SELECT_ALL,
      Self::Undo => sys::QS_ACTION_UNDO,
      Self::Redo => sys::QS_ACTION_REDO,
      Self::Delete => sys::QS_ACTION_DELETE,
      Self::Minimize => sys::QS_ACTION_MINIMIZE,
      Self::Maximize => sys::QS_ACTION_MAXIMIZE,
      Self::Fullscreen => sys::QS_ACTION_FULLSCREEN,
      Self::Hide => sys::QS_ACTION_HIDE,
      Self::CloseWindow => sys::QS_ACTION_CLOSE_WINDOW,
      Self::Quit => sys::QS_ACTION_QUIT,
    };
    unsafe { sys::qs_standard_action(kind) }
  }
}

/// Show a non-blocking About box. `html` is Qt rich text.
pub fn show_about(title: &str, html: &str, icon: Option<Icon<'_>>) {
  lifecycle::ensure();
  let title = cstring(title);
  let html = cstring(html);
  let (rgba, w, h) = Icon::raw(icon.as_ref());
  unsafe { sys::qs_show_about(title.as_ptr(), html.as_ptr(), rgba, w, h) }
}

/// Inspect and drive menus, for tests and debugging.
pub mod debug {
  use super::*;

  /// The menubar of `window` as an indented tree: `[x]` marks checked items, `<Ctrl+N>`
  /// registered shortcuts, `(Ctrl+C)` displayed-only ones. Empty without a menubar.
  pub fn menubar_tree(window: &Widget) -> String {
    match window.as_ptr() {
      Some(window) => unsafe { take_cstring(sys::qs_debug_menubar(window.as_ptr())) },
      None => String::new(),
    }
  }

  /// Trigger the menubar item at `path` (e.g. `"File/Save"`, labels without `&`). Returns
  /// `false` if there's no such item.
  pub fn trigger_menu_item(window: &Widget, path: &str) -> bool {
    window.as_ptr().is_some_and(|window| unsafe {
      sys::qs_debug_trigger(window.as_ptr(), cstring(path).as_ptr()) != 0
    })
  }

  /// The last context menu Qt showed (since the previous call) as a tree, or an empty string;
  /// closes it if it's still open. Recording starts with the first call to a function here.
  pub fn take_popup_menu() -> String {
    lifecycle::ensure();
    unsafe { take_cstring(sys::qs_debug_take_popup()) }
  }
}

#[cfg(test)]
mod tests {
  //! Pure logic only: no `QApplication` here (see `tests/qt/menu.rs` for that).

  use super::*;

  fn icon(rgba: &[u8], width: u32, height: u32) -> (*const u8, c_int, c_int) {
    Icon::raw(Some(&Icon {
      rgba,
      width,
      height,
    }))
  }

  #[test]
  fn icon_raw_none_is_null() {
    assert_eq!(Icon::raw(None), (std::ptr::null(), 0, 0));
  }

  #[test]
  fn icon_raw_exact_buffer() {
    let px = [0u8; 2 * 3 * 4];
    assert_eq!(icon(&px, 2, 3), (px.as_ptr(), 2, 3));
  }

  #[test]
  fn icon_raw_larger_buffer() {
    let px = [0u8; 100];
    assert_eq!(icon(&px, 2, 2), (px.as_ptr(), 2, 2));
  }

  #[test]
  fn icon_raw_too_small_buffer_is_null() {
    let px = [0u8; 2 * 3 * 4 - 1];
    assert_eq!(icon(&px, 2, 3), (std::ptr::null(), 0, 0));
    assert_eq!(icon(&[], 1, 1), (std::ptr::null(), 0, 0));
  }

  #[test]
  fn icon_raw_zero_size_is_no_icon() {
    assert_eq!(icon(&[], 0, 0), (std::ptr::null(), 0, 0));
    assert_eq!(icon(&[], 0, 1 << 20), (std::ptr::null(), 0, 0));
  }

  #[test]
  fn icon_raw_size_beyond_c_int() {
    // `width as c_int` wraps to a negative number, which the C++ side ignores.
    let (_, w, h) = icon(&[], u32::MAX, 0);
    assert!(w <= 0, "{w}");
    assert_eq!(h, 0);
  }

  #[test]
  fn icon_raw_huge_size_does_not_panic() {
    assert_eq!(icon(&[], u32::MAX, u32::MAX), (std::ptr::null(), 0, 0));
  }

  #[test]
  fn action_options_default() {
    let o = ActionOptions::default();
    assert_eq!(o.text, "");
    assert!(o.enabled);
    assert!(!o.checkable);
    assert!(!o.checked);
    assert!(o.shortcut.is_none());
    assert!(o.shortcut_hint.is_none());
    assert!(o.icon.is_none());
  }

  #[test]
  fn standard_action_is_editing() {
    use StandardAction::*;
    for a in [Copy, Cut, Paste, SelectAll, Undo, Redo, Delete] {
      assert!(a.is_editing(), "{a:?}");
    }
    for a in [Minimize, Maximize, Fullscreen, Hide, CloseWindow, Quit] {
      assert!(!a.is_editing(), "{a:?}");
    }
  }

  #[test]
  fn popup_position_eq() {
    let l = PopupPosition::Logical { x: 1.0, y: 2.0 };
    let p = PopupPosition::Physical { x: 1.0, y: 2.0 };
    assert_eq!(l, l);
    assert_ne!(l, p);
  }

  // Every entry point checks that Qt is running on this thread.

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread")]
  fn standard_action_without_app_panics() {
    StandardAction::Copy.trigger();
  }

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread")]
  fn show_about_without_app_panics() {
    show_about("t", "h", None);
  }

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread")]
  fn take_popup_menu_without_app_panics() {
    debug::take_popup_menu();
  }

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread")]
  fn from_ptr_null_without_app_panics() {
    Widget::from_ptr(std::ptr::null_mut());
  }
}
