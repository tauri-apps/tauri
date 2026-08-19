// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::Cell, ffi::c_void};

use objc2::{
  DefinedClass, MainThreadOnly, define_class,
  rc::Retained,
  runtime::{NSObject, NSObjectProtocol},
  sel,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSView, NSWindow, NSWindowButton, NSWindowCollectionBehavior,
  NSWindowDidBecomeKeyNotification, NSWindowDidChangeBackingPropertiesNotification,
  NSWindowDidChangeScreenNotification, NSWindowDidDeminiaturizeNotification,
  NSWindowDidEndLiveResizeNotification, NSWindowDidExitFullScreenNotification,
  NSWindowDidResizeNotification, NSWindowStyleMask,
};
use objc2_foundation::{
  MainThreadMarker, NSNotification, NSNotificationCenter, NSObjectNSDelayedPerforming, NSPoint,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri_runtime::dpi::Position;
use tauri_utils::{TitleBarStyle, config::Color};

use crate::window::AppWindow;

use super::utils;

impl AppWindow {
  pub(crate) fn raw_cef_handle(&self) -> cef::sys::cef_window_handle_t {
    let nsview = self.nsview();
    Retained::as_ptr(&nsview).cast_mut().cast()
  }

  pub(crate) fn nsview(&self) -> Retained<NSView> {
    let handle = self
      .window
      .window_handle()
      .expect("failed to get window handle");
    match handle.as_raw() {
      RawWindowHandle::AppKit(handle) => unsafe {
        Retained::<NSView>::retain(handle.ns_view.as_ptr().cast::<NSView>())
          .expect("failed to retain NSView")
      },
      other => panic!("expected AppKit window handle, got {other:?}"),
    }
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    if enabled {
      if let Some(attached) = nswindow.attachedSheet() {
        nswindow.endSheet(&attached);
      }
    } else {
      if nswindow.attachedSheet().is_some() {
        return;
      }

      let Some(mtm) = MainThreadMarker::new() else {
        return;
      };
      let frame = nswindow.frame();
      let sheet = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          mtm.alloc(),
          frame,
          NSWindowStyleMask::Titled,
          NSBackingStoreType::Buffered,
          false,
        )
      };
      sheet.setAlphaValue(0.5);
      nswindow.beginSheet_completionHandler(&sheet, None);
    }
  }

  pub(crate) fn is_enabled(&self) -> bool {
    self
      .nsview()
      .window()
      .map(|nswindow| nswindow.attachedSheet().is_none())
      .unwrap_or(true)
  }

  pub(crate) fn set_traffic_light_position(&self, position: &Position) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let pos = position.to_logical::<f64>(nswindow.backingScaleFactor());
    let pos = NSPoint::new(pos.x, pos.y);

    inset_traffic_lights(&nswindow, pos.x, pos.y);
    observe_traffic_light_resets(&nswindow, pos);
  }

  /// Restore the traffic light position after a window appearance change.
  ///
  /// Changing the appearance rebuilds the titlebar like a geometry change does,
  /// but posts no window notification, so the theme paths have to ask for it.
  /// AppKit rebuilds *after* the new appearance is observable, so the inset is
  /// restored on the next run loop turn — restoring it inline is overwritten.
  pub(crate) fn reapply_traffic_light_position_after_appearance_change(&self) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };
    let Some(observer) = traffic_light_observer(&nswindow) else {
      // No traffic light position configured for this window.
      return;
    };

    // SAFETY: `observer` implements the selector and takes the window as its
    // argument. The run loop keeps both alive until it fires.
    unsafe {
      observer.performSelector_withObject_afterDelay(
        sel!(reapplyTrafficLightPosition:),
        Some(&nswindow),
        0.0,
      );
    }
  }

  pub(crate) fn set_title_bar_style(&self, style: TitleBarStyle) {
    let nsview = self.nsview();
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

  pub(crate) fn set_visible_on_all_workspaces(&self, visible: bool) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let mut collection_behavior = nswindow.collectionBehavior();
    collection_behavior.set(NSWindowCollectionBehavior::CanJoinAllSpaces, visible);
    nswindow.setCollectionBehavior(collection_behavior);
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let nscolor = color
      .map(utils::ns_color_from_tauri_color)
      .unwrap_or_else(NSColor::windowBackgroundColor);
    nswindow.setOpaque(color.map(|color| color.3 == u8::MAX).unwrap_or(true));
    nswindow.setBackgroundColor(Some(&nscolor));
  }
}

fn inset_traffic_lights(nswindow: &NSWindow, x: f64, y: f64) {
  let Some(close) = nswindow.standardWindowButton(NSWindowButton::CloseButton) else {
    return;
  };
  let Some(miniaturize) = nswindow.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
    return;
  };
  let Some(zoom) = nswindow.standardWindowButton(NSWindowButton::ZoomButton) else {
    return;
  };

  let title_bar_container_view = unsafe { close.superview().and_then(|view| view.superview()) };
  let Some(title_bar_container_view) = title_bar_container_view else {
    return;
  };

  let close_rect = close.frame();
  let title_bar_frame_height = close_rect.size.height + y;
  let mut title_bar_rect = title_bar_container_view.frame();
  title_bar_rect.size.height = title_bar_frame_height;
  title_bar_rect.origin.y = nswindow.frame().size.height - title_bar_frame_height;
  title_bar_container_view.setFrame(title_bar_rect);

  let space_between = miniaturize.frame().origin.x - close_rect.origin.x;
  for (index, button) in [close, miniaturize, zoom].into_iter().enumerate() {
    let mut origin = button.frame().origin;
    origin.x = x + (index as f64 * space_between);
    button.setFrameOrigin(origin);
  }
}

/// AppKit rebuilds the titlebar whenever the window geometry changes, which
/// restores the stock window-button layout and drops the inset applied by
/// [`inset_traffic_lights`]. The reset has to be undone after AppKit finished
/// laying the titlebar out again, so the inset is reapplied from the window
/// notifications that follow the relayout.
///
/// Reapplying from a view frame observer instead does not work: that fires in
/// the middle of AppKit's own layout pass, and AppKit overwrites the geometry
/// afterwards.
fn observe_traffic_light_resets(nswindow: &NSWindow, position: NSPoint) {
  // The observer owns the position it applies, so it stays valid for exactly as
  // long as it can receive notifications, and a window only ever registers one.
  if let Some(observer) = traffic_light_observer(nswindow) {
    observer.ivars().position.set(position);
    return;
  }

  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };

  let observer = TrafficLightObserver::new(mtm, position);
  let center = NSNotificationCenter::defaultCenter();
  for name in unsafe {
    [
      NSWindowDidResizeNotification,
      NSWindowDidEndLiveResizeNotification,
      NSWindowDidExitFullScreenNotification,
      NSWindowDidDeminiaturizeNotification,
      NSWindowDidChangeScreenNotification,
      NSWindowDidChangeBackingPropertiesNotification,
      NSWindowDidBecomeKeyNotification,
    ]
  } {
    // SAFETY: `observer` implements the selector, and the notifications are
    // observed for a single `NSWindow`, which is what the handler expects as
    // the notification object.
    unsafe {
      center.addObserver_selector_name_object(
        &observer,
        sel!(handleWindowNotification:),
        Some(name),
        Some(nswindow),
      );
    }
  }

  set_traffic_light_observer(nswindow, &observer);
}

fn reapply_traffic_light_position(nswindow: &NSWindow, position: NSPoint) {
  // In fullscreen the window buttons are owned by the auto-hiding titlebar
  // overlay rather than by the window's own titlebar, so insetting them there
  // would move them out of the overlay. The relayout that follows leaving
  // fullscreen posts a resize notification, which restores the inset.
  if nswindow.styleMask().contains(NSWindowStyleMask::FullScreen) {
    return;
  }

  inset_traffic_lights(nswindow, position.x, position.y);
}

#[derive(Default)]
struct TrafficLightObserverIvars {
  position: Cell<NSPoint>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[name = "TauriCefTrafficLightObserver"]
  #[ivars = TrafficLightObserverIvars]
  #[thread_kind = MainThreadOnly]
  struct TrafficLightObserver;

  unsafe impl NSObjectProtocol for TrafficLightObserver {}

  impl TrafficLightObserver {
    #[unsafe(method(handleWindowNotification:))]
    fn handle_window_notification(&self, notification: &NSNotification) {
      let Some(object) = notification.object() else {
        return;
      };
      // SAFETY: the observer is only registered for `NSWindow` notifications
      // with a window as the notification object.
      let nswindow = unsafe { Retained::cast_unchecked::<NSWindow>(object) };
      reapply_traffic_light_position(&nswindow, self.ivars().position.get());
    }

    #[unsafe(method(reapplyTrafficLightPosition:))]
    fn reapply_traffic_light_position_deferred(&self, nswindow: &NSWindow) {
      reapply_traffic_light_position(nswindow, self.ivars().position.get());
    }
  }
);

impl TrafficLightObserver {
  fn new(mtm: MainThreadMarker, position: NSPoint) -> Retained<Self> {
    let observer = Self::alloc(mtm).set_ivars(TrafficLightObserverIvars {
      position: Cell::new(position),
    });
    unsafe { objc2::msg_send![super(observer), init] }
  }
}

fn traffic_light_observer_key() -> *const c_void {
  static TRAFFIC_LIGHT_OBSERVER_KEY: u8 = 0;
  &TRAFFIC_LIGHT_OBSERVER_KEY as *const u8 as *const c_void
}

fn traffic_light_observer(nswindow: &NSWindow) -> Option<Retained<TrafficLightObserver>> {
  let observer = utils::associated_object(nswindow, traffic_light_observer_key())?;
  // SAFETY: the key is private to this module, and `set_traffic_light_observer`
  // is the only writer.
  Some(unsafe { Retained::cast_unchecked::<TrafficLightObserver>(observer) })
}

fn set_traffic_light_observer(nswindow: &NSWindow, observer: &TrafficLightObserver) {
  utils::set_associated_object(nswindow, traffic_light_observer_key(), observer);
}
