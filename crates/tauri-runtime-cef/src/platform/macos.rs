// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::cell::Cell;

use crate::{platform::EventLoopExt, webview::AppWebview, window::AppWindow};
use cef::{
  ImplBrowserHost,
  application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol},
};
use objc2::{
  ClassType, DefinedClass, MainThreadMarker, define_class, extern_methods, msg_send, rc::Retained,
  runtime::Bool,
};
use objc2_app_kit::{
  NSApp, NSApplication, NSApplicationActivationPolicy, NSEvent, NSScreen, NSView, NSWindowButton,
  NSWindowStyleMask,
};
use objc2_foundation::{NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use tauri_runtime::{
  Error, Result,
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, Position, Rect},
};
use tauri_utils::TitleBarStyle;
use winit::event_loop::ActiveEventLoop;

#[derive(Default)]
struct CefWinitApplicationIvars {
  handling_send_event: Cell<Bool>,
}

define_class!(
  #[unsafe(super(NSApplication))]
  #[ivars = CefWinitApplicationIvars]
  struct CefWinitApplication;

  impl CefWinitApplication {
    #[unsafe(method(sendEvent:))]
    unsafe fn send_event(&self, event: &NSEvent) {
      let was_handling = self.ivars().handling_send_event.get();
      self.ivars().handling_send_event.set(Bool::YES);
      let _: () = unsafe { msg_send![super(self), sendEvent: event] };
      self.ivars().handling_send_event.set(was_handling);
    }
  }

  unsafe impl CrAppControlProtocol for CefWinitApplication {
    #[unsafe(method(setHandlingSendEvent:))]
    unsafe fn set_handling_send_event(&self, handling_send_event: Bool) {
      self.ivars().handling_send_event.set(handling_send_event);
    }
  }

  unsafe impl CrAppProtocol for CefWinitApplication {
    #[unsafe(method(isHandlingSendEvent))]
    unsafe fn is_handling_send_event(&self) -> Bool {
      self.ivars().handling_send_event.get()
    }
  }

  unsafe impl CefAppProtocol for CefWinitApplication {}
);

impl CefWinitApplication {
  extern_methods! {
    #[unsafe(method(sharedApplication))]
    fn shared_application() -> Retained<Self>;
  }
}

pub fn setup_application() {
  let _ = CefWinitApplication::shared_application();
  let mtm = MainThreadMarker::new().expect("macOS application must start on the main thread");
  assert!(NSApp(mtm).isKindOfClass(CefWinitApplication::class()));
}

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_activation_policy(&self, policy: tauri_runtime::ActivationPolicy) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let policy = match policy {
      tauri_runtime::ActivationPolicy::Regular => NSApplicationActivationPolicy::Regular,
      tauri_runtime::ActivationPolicy::Accessory => NSApplicationActivationPolicy::Accessory,
      tauri_runtime::ActivationPolicy::Prohibited => NSApplicationActivationPolicy::Prohibited,
      _ => NSApplicationActivationPolicy::Regular,
    };
    app.setActivationPolicy(policy);
  }

  fn set_dock_visibility(&self, visible: bool) {
    self.set_activation_policy(if visible {
      tauri_runtime::ActivationPolicy::Regular
    } else {
      tauri_runtime::ActivationPolicy::Accessory
    });
  }

  fn show_application(&self) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    NSApp(mtm).unhide(None);
  }

  fn hide_application(&self) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    NSApp(mtm).hide(None);
  }

  fn set_badge_count(&self, count: Option<i64>, _desktop_filename: Option<String>) {
    self.set_badge_label(count.map(|count| count.to_string()));
  }

  fn set_badge_label(&self, label: Option<String>) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let dock_tile = app.dockTile();
    let ns_label = label.map(|label| NSString::from_str(&label));
    dock_tile.setBadgeLabel(ns_label.as_deref());
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    let Some(mtm) = MainThreadMarker::new() else {
      return Err(Error::FailedToGetCursorPosition);
    };
    // `NSEvent::mouseLocation` uses a bottom-left origin; flip to top-left.
    let location: NSPoint = NSEvent::mouseLocation();
    let screen_height = NSScreen::mainScreen(mtm)
      .map(|screen| screen.frame().size.height)
      .ok_or(Error::FailedToGetCursorPosition)?;
    Ok(PhysicalPosition::new(
      location.x,
      screen_height - location.y,
    ))
  }
}

impl AppWindow {
  pub(crate) fn nsview(&self) -> Option<Retained<NSView>> {
    let handle = self.raw_handle_as_cef_handle();
    let view = handle.cast::<NSView>();
    unsafe { Retained::<NSView>::retain(view) }
  }

  pub(crate) fn apply_traffic_light_position(&self, position: &Position) {
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let Some(close) = nswindow.standardWindowButton(NSWindowButton::CloseButton) else {
      return;
    };
    let Some(miniaturize) = nswindow.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
      return;
    };
    let Some(zoom) = nswindow.standardWindowButton(NSWindowButton::ZoomButton) else {
      return;
    };

    let pos = position.to_logical::<f64>(nswindow.backingScaleFactor());
    let title_bar_container_view = unsafe { close.superview().and_then(|view| view.superview()) };
    let Some(title_bar_container_view) = title_bar_container_view else {
      return;
    };

    let close_rect = close.frame();
    let title_bar_frame_height = close_rect.size.height + pos.y;
    let mut title_bar_rect = title_bar_container_view.frame();
    title_bar_rect.size.height = title_bar_frame_height;
    title_bar_rect.origin.y = nswindow.frame().size.height - title_bar_frame_height;
    title_bar_container_view.setFrame(title_bar_rect);

    let space_between = miniaturize.frame().origin.x - close_rect.origin.x;
    for (index, button) in [close, miniaturize, zoom].into_iter().enumerate() {
      let mut origin = button.frame().origin;
      origin.x = pos.x + (index as f64 * space_between);
      button.setFrameOrigin(origin);
    }
  }

  pub(crate) fn set_title_bar_style(&self, style: TitleBarStyle) {
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(nswindow) = nsview.window() else {
      return;
    };

    match style {
      TitleBarStyle::Visible => {
        nswindow.setTitlebarAppearsTransparent(false);
        let mut mask = nswindow.styleMask();
        mask.remove(NSWindowStyleMask::FullSizeContentView);
        nswindow.setStyleMask(mask);
      }
      TitleBarStyle::Transparent => {
        nswindow.setTitlebarAppearsTransparent(true);
        let mut mask = nswindow.styleMask();
        mask.remove(NSWindowStyleMask::FullSizeContentView);
        nswindow.setStyleMask(mask);
      }
      TitleBarStyle::Overlay => {
        nswindow.setTitlebarAppearsTransparent(true);
        let mut mask = nswindow.styleMask();
        mask.insert(NSWindowStyleMask::FullSizeContentView);
        nswindow.setStyleMask(mask);
      }
      _ => {}
    }
  }
}

impl AppWebview {
  pub(crate) fn nsview(&self) -> Option<Retained<NSView>> {
    let handle = self.host.window_handle();
    let view = handle.cast::<NSView>();
    unsafe { Retained::<NSView>::retain(view) }
  }

  pub(crate) fn bounds(&self) -> Option<Rect> {
    let Some(nsview) = self.nsview() else {
      return None;
    };

    let parent = unsafe { nsview.superview()? };
    let parent_frame = parent.frame();
    let frame = nsview.frame();

    let y = if parent.isFlipped() {
      frame.origin.y
    } else {
      parent_frame.size.height - frame.origin.y - frame.size.height
    };

    let position = LogicalPosition::new(frame.origin.x, y);
    let size = LogicalSize::new(frame.size.width, frame.size.height);

    Some(Rect {
      position: position.into(),
      size: size.into(),
    })
  }

  pub(crate) fn reparent(&self, parent: &AppWindow) {
    let Some(view) = self.nsview() else {
      return;
    };
    let Some(parent) = parent.nsview() else {
      return;
    };

    parent.addSubview(&view);
  }

  pub(crate) fn apply_visible(&self, visible: bool) {
    let Some(nsview) = self.nsview() else {
      return;
    };

    nsview.setHidden(!visible);
  }

  pub(crate) fn apply_physical_bounds(&self, scale: f64, x: i32, y: i32, width: i32, height: i32) {
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(parent) = (unsafe { nsview.superview() }) else {
      return;
    };

    // CEF provides child bounds as physical pixels, but NSView frames are logical pixels.
    let x = x as f64 / scale;
    let y = y as f64 / scale;
    let width = width as f64 / scale;
    let height = height as f64 / scale;

    let parent_frame = parent.frame();
    let y = if parent.isFlipped() {
      y
    } else {
      parent_frame.size.height - (y + height)
    };

    let frame = NSRect::new(NSPoint::new(x, y), NSSize::new(width, height));
    nsview.setFrame(frame);
  }
}
