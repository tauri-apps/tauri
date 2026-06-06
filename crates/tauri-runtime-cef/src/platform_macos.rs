//! macOS implementation of the platform window helpers.

#![allow(clippy::missing_safety_doc)]

use std::cell::RefCell;
use std::collections::HashMap;

use cef::ImplWindow;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
  NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication,
  NSApplicationPresentationOptions, NSBackingStoreType, NSCursor, NSScreen, NSView, NSWindow,
  NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSString};
use tauri_runtime::dpi::{PhysicalPosition, Position};
use tauri_runtime::window::CursorIcon;
use tauri_runtime::{ProgressBarState, ResizeDirection, UserAttentionType};

#[repr(C)]
struct CGPoint {
  x: f64,
  y: f64,
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
  fn CGWarpMouseCursorPosition(new_cursor_position: CGPoint) -> i32;
  fn CGAssociateMouseAndMouseCursorPosition(connected: i32) -> i32;
}

fn ns_window(window: &cef::Window) -> Option<Retained<NSWindow>> {
  unsafe {
    let ns_view = Retained::<NSView>::retain(window.window_handle() as _)?;
    ns_view.window()
  }
}

fn main_thread() -> Option<MainThreadMarker> {
  MainThreadMarker::new()
}

/// macOS has no taskbar, so skipping it is a no-op.
pub fn set_skip_taskbar(_window: &cef::Window, _skip: bool) {}

pub fn set_always_on_bottom(window: &cef::Window, on_bottom: bool) {
  if let Some(ns_window) = ns_window(window) {
    // `NSNormalWindowLevel` is 0; one below it keeps the window beneath others.
    let level = if on_bottom { -1 } else { 0 };
    ns_window.setLevel(level);
  }
}

pub fn set_visible_on_all_workspaces(window: &cef::Window, visible: bool) {
  if let Some(ns_window) = ns_window(window) {
    unsafe {
      let mut behavior = ns_window.collectionBehavior();
      if visible {
        behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
      } else {
        behavior &= !NSWindowCollectionBehavior::CanJoinAllSpaces;
      }
      ns_window.setCollectionBehavior(behavior);
    }
  }
}

pub fn set_shadow(window: &cef::Window, enable: bool) {
  if let Some(ns_window) = ns_window(window) {
    ns_window.setHasShadow(enable);
  }
}

pub fn request_user_attention(_window: &cef::Window, request_type: Option<UserAttentionType>) {
  let Some(mtm) = main_thread() else {
    return;
  };
  let app = NSApplication::sharedApplication(mtm);
  // NSRequestUserAttentionType: CriticalRequest = 0, InformationalRequest = 10.
  match request_type {
    Some(UserAttentionType::Critical) => {
      app.requestUserAttention(objc2_app_kit::NSRequestUserAttentionType::CriticalRequest);
    }
    Some(UserAttentionType::Informational) => {
      app.requestUserAttention(objc2_app_kit::NSRequestUserAttentionType::InformationalRequest);
    }
    None => {}
  }
}

pub fn global_cursor_position() -> Option<PhysicalPosition<f64>> {
  let mtm = main_thread()?;
  // `NSEvent::mouseLocation` uses a bottom-left origin; flip to top-left.
  let location: NSPoint = unsafe { objc2_app_kit::NSEvent::mouseLocation() };
  let screen_height = NSScreen::mainScreen(mtm)
    .map(|s| s.frame().size.height)
    .unwrap_or(0.0);
  Some(PhysicalPosition::new(
    location.x,
    screen_height - location.y,
  ))
}

pub fn set_cursor_position(window: &cef::Window, position: Position, scale_factor: f64) {
  // Like `tao`, the position is relative to the window's content area. Translate
  // it to global top-left (point) coordinates that CoreGraphics expects.
  let Some(mtm) = main_thread() else {
    return;
  };
  let Some(ns_window) = ns_window(window) else {
    return;
  };
  let logical = position.to_logical::<f64>(scale_factor);
  let content = ns_window.contentRectForFrameRect(ns_window.frame());
  // `content` uses a bottom-left origin; flip it to top-left using the main
  // screen height.
  let screen_height = NSScreen::mainScreen(mtm)
    .map(|s| s.frame().size.height)
    .unwrap_or(0.0);
  let window_x = content.origin.x;
  let window_y = screen_height - (content.origin.y + content.size.height);
  unsafe {
    CGWarpMouseCursorPosition(CGPoint {
      x: window_x + logical.x,
      y: window_y + logical.y,
    });
    // Re-associate so cursor movement keeps tracking the mouse.
    CGAssociateMouseAndMouseCursorPosition(1);
  }
}

pub fn set_cursor_visible(_window: &cef::Window, visible: bool) {
  unsafe {
    if visible {
      NSCursor::unhide();
    } else {
      NSCursor::hide();
    }
  }
}

pub fn set_cursor_grab(_window: &cef::Window, grab: bool) {
  unsafe {
    // Disassociating locks the cursor in place (a "grab").
    CGAssociateMouseAndMouseCursorPosition(if grab { 0 } else { 1 });
  }
}

pub fn set_ignore_cursor_events(window: &cef::Window, ignore: bool) {
  if let Some(ns_window) = ns_window(window) {
    ns_window.setIgnoresMouseEvents(ignore);
  }
}

pub fn set_cursor_icon(_window: &cef::Window, icon: CursorIcon) {
  use CursorIcon::*;
  unsafe {
    let cursor = match icon {
      Crosshair => NSCursor::crosshairCursor(),
      Hand | Grab => NSCursor::openHandCursor(),
      Grabbing => NSCursor::closedHandCursor(),
      Text => NSCursor::IBeamCursor(),
      VerticalText => NSCursor::IBeamCursorForVerticalLayout(),
      NotAllowed | NoDrop => NSCursor::operationNotAllowedCursor(),
      ContextMenu => NSCursor::contextualMenuCursor(),
      Alias => NSCursor::dragLinkCursor(),
      Copy => NSCursor::dragCopyCursor(),
      EResize | EwResize | ColResize => NSCursor::resizeRightCursor(),
      WResize => NSCursor::resizeLeftCursor(),
      NResize | NsResize | RowResize => NSCursor::resizeUpCursor(),
      SResize => NSCursor::resizeDownCursor(),
      _ => NSCursor::arrowCursor(),
    };
    cursor.set();
  }
}

/// macOS has no taskbar progress indicator; left unimplemented.
pub fn set_progress_bar(_window: &cef::Window, _state: ProgressBarState) {}

pub fn set_badge_count(
  _window: &cef::Window,
  count: Option<i64>,
  _desktop_filename: Option<String>,
) {
  let label = count.map(|c| c.to_string());
  set_badge_label(label);
}

pub fn set_badge_label(label: Option<String>) {
  let Some(mtm) = main_thread() else {
    return;
  };
  let app = NSApplication::sharedApplication(mtm);
  let ns_label = label.map(|l| NSString::from_str(&l));
  unsafe {
    let dock_tile = app.dockTile();
    dock_tile.setBadgeLabel(ns_label.as_deref());
  }
}

/// macOS has no public window resize-drag API (matching `tao`).
pub fn start_resize_dragging(_window: &cef::Window, _direction: ResizeDirection) {}

struct SimpleFullscreenState {
  /// Content rect of the window before entering simple fullscreen.
  standard_frame: NSRect,
  saved_style: NSWindowStyleMask,
  saved_presentation_options: NSApplicationPresentationOptions,
}

thread_local! {
  // Per-window saved state, keyed by the `NSWindow` pointer. Presence in the
  // map means the window is currently in simple fullscreen.
  static SIMPLE_FULLSCREEN: RefCell<HashMap<usize, SimpleFullscreenState>> =
    RefCell::new(HashMap::new());
}

/// Pre-Lion style fullscreen, mirroring `tao`'s `set_simple_fullscreen`: hide
/// the dock and menu bar, drop the title bar, resize to the screen frame, and
/// lock the window down — restoring everything on exit.
pub fn set_simple_fullscreen(window: &cef::Window, fullscreen: bool) {
  let Some(mtm) = main_thread() else {
    return;
  };
  let Some(ns_window) = ns_window(window) else {
    return;
  };
  let app = NSApplication::sharedApplication(mtm);
  let key = (&*ns_window as *const NSWindow) as usize;

  SIMPLE_FULLSCREEN.with(|cell| {
    let mut map = cell.borrow_mut();
    let is_simple_fullscreen = map.contains_key(&key);
    if fullscreen == is_simple_fullscreen {
      return;
    }

    if fullscreen {
      // Remember the original window settings (content rect excludes the title bar).
      map.insert(
        key,
        SimpleFullscreenState {
          standard_frame: ns_window.contentRectForFrameRect(ns_window.frame()),
          saved_style: ns_window.styleMask(),
          saved_presentation_options: app.presentationOptions(),
        },
      );

      // Simulate pre-Lion fullscreen by hiding the dock and menu bar.
      app.setPresentationOptions(
        NSApplicationPresentationOptions::AutoHideDock
          | NSApplicationPresentationOptions::AutoHideMenuBar,
      );

      // Hide the title bar.
      let mut mask = ns_window.styleMask();
      mask &= !NSWindowStyleMask::Titled;
      ns_window.setStyleMask(mask);

      // Resize to the full screen frame.
      if let Some(screen) = ns_window.screen() {
        ns_window.setFrame_display(screen.frame(), true);
      }

      // Fullscreen windows can't be resized, minimized, or moved.
      let mut mask = ns_window.styleMask();
      mask &= !(NSWindowStyleMask::Miniaturizable | NSWindowStyleMask::Resizable);
      ns_window.setStyleMask(mask);
      ns_window.setMovable(false);
    } else if let Some(state) = map.remove(&key) {
      ns_window.setStyleMask(state.saved_style);
      app.setPresentationOptions(state.saved_presentation_options);
      ns_window.setFrame_display(state.standard_frame, true);
      ns_window.setMovable(true);
    }
  });
}

pub fn set_activation_policy(policy: tauri_runtime::ActivationPolicy) {
  let Some(mtm) = main_thread() else {
    return;
  };
  let app = NSApplication::sharedApplication(mtm);
  let ns_policy = match policy {
    tauri_runtime::ActivationPolicy::Regular => {
      objc2_app_kit::NSApplicationActivationPolicy::Regular
    }
    tauri_runtime::ActivationPolicy::Accessory => {
      objc2_app_kit::NSApplicationActivationPolicy::Accessory
    }
    tauri_runtime::ActivationPolicy::Prohibited => {
      objc2_app_kit::NSApplicationActivationPolicy::Prohibited
    }
    _ => objc2_app_kit::NSApplicationActivationPolicy::Regular,
  };
  app.setActivationPolicy(ns_policy);
}

/// Sets the application-wide appearance (like tao's `set_ns_theme`), which the
/// runtime-level `set_theme` uses so every window — current and future —
/// follows the theme unless it overrides it explicitly. `None` follows the
/// system theme.
pub fn set_app_theme(theme: Option<tauri_utils::Theme>) {
  let Some(mtm) = main_thread() else {
    return;
  };
  let app = NSApplication::sharedApplication(mtm);
  let appearance = match theme {
    Some(tauri_utils::Theme::Dark) => unsafe {
      NSAppearance::appearanceNamed(NSAppearanceNameDarkAqua)
    },
    Some(tauri_utils::Theme::Light) => unsafe {
      NSAppearance::appearanceNamed(NSAppearanceNameAqua)
    },
    _ => None,
  };
  app.setAppearance(appearance.as_deref());
}

/// Enables or disables all user interaction with the window.
///
/// CEF's `View::set_enabled` only disables the window's root view, not the
/// child browser views layered on top of it, so the web contents stay
/// focusable. Mirroring `tao`/Electron, a disabled window instead gets a
/// translucent modal sheet attached over it, which swallows input for the
/// entire window — browser included; removing the sheet re-enables it.
pub fn set_enabled(window: &cef::Window, enabled: bool) {
  let Some(ns_window) = ns_window(window) else {
    return;
  };
  if !enabled {
    // Avoid stacking multiple sheets if called twice while already disabled.
    if unsafe { ns_window.attachedSheet() }.is_some() {
      return;
    }
    let Some(mtm) = main_thread() else {
      return;
    };
    let frame = ns_window.frame();
    let sheet = unsafe {
      NSWindow::initWithContentRect_styleMask_backing_defer(
        mtm.alloc(),
        frame,
        NSWindowStyleMask::Titled,
        NSBackingStoreType::Buffered,
        false,
      )
    };
    unsafe {
      sheet.setAlphaValue(0.5);
      ns_window.beginSheet_completionHandler(&sheet, None);
    }
  } else if let Some(attached) = unsafe { ns_window.attachedSheet() } {
    unsafe { ns_window.endSheet(&attached) };
  }
}

/// Reports whether the window is enabled, i.e. has no modal sheet attached by
/// [`set_enabled`].
pub fn is_enabled(window: &cef::Window) -> bool {
  ns_window(window)
    .map(|ns_window| unsafe { ns_window.attachedSheet() }.is_none())
    .unwrap_or(true)
}

pub fn set_dock_visibility(visible: bool) {
  set_activation_policy(if visible {
    tauri_runtime::ActivationPolicy::Regular
  } else {
    tauri_runtime::ActivationPolicy::Accessory
  });
}
