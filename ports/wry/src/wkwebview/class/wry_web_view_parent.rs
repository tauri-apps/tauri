// Copyright 2020-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "macos")]
use objc2::DefinedClass;
use objc2::{define_class, msg_send, rc::Retained, MainThreadOnly};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSEvent, NSView, NSWindow, NSWindowButton};
use objc2_foundation::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_foundation::NSRect;
#[cfg(target_os = "ios")]
use objc2_ui_kit::UIView as NSView;

pub struct WryWebViewParentIvars {
  #[cfg(target_os = "macos")]
  traffic_light_inset: std::cell::Cell<Option<(f64, f64)>>,
}

define_class!(
  #[unsafe(super(NSView))]
  #[ivars = WryWebViewParentIvars]
  pub struct WryWebViewParent;

  /// Overridden NSView methods.
  impl WryWebViewParent {
    #[cfg(target_os = "macos")]
    #[unsafe(method(keyDown:))]
    fn key_down(&self, event: &NSEvent) {
      let mtm = MainThreadMarker::new().unwrap();
      let app = NSApplication::sharedApplication(mtm);
      unsafe {
        if let Some(menu) = app.mainMenu() {
          menu.performKeyEquivalent(event);
        }
      }
    }

    #[cfg(target_os = "macos")]
    #[unsafe(method(drawRect:))]
    fn draw(&self, _dirty_rect: NSRect) {
      if let Some((x, y)) = self.ivars().traffic_light_inset.get() {
        unsafe { inset_traffic_lights(&self.window().unwrap(), x, y) };
      }
    }
  }
);

impl WryWebViewParent {
  #[allow(dead_code)]
  pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
    let delegate = WryWebViewParent::alloc(mtm).set_ivars(WryWebViewParentIvars {
      #[cfg(target_os = "macos")]
      traffic_light_inset: Default::default(),
    });
    unsafe { msg_send![super(delegate), init] }
  }

  #[cfg(target_os = "macos")]
  pub fn set_traffic_light_inset(&self, ns_window: &NSWindow, position: dpi::Position) {
    let scale_factor = NSWindow::backingScaleFactor(ns_window);
    let position = position.to_logical(scale_factor);
    self
      .ivars()
      .traffic_light_inset
      .replace(Some((position.x, position.y)));

    unsafe {
      inset_traffic_lights(ns_window, position.x, position.y);
    }
  }
}

#[cfg(target_os = "macos")]
pub unsafe fn inset_traffic_lights(window: &NSWindow, x: f64, y: f64) {
  let Some(close) = window.standardWindowButton(NSWindowButton::CloseButton) else {
    #[cfg(feature = "tracing")]
    tracing::warn!("skipping inset_traffic_lights, close button not found");
    return;
  };
  let Some(miniaturize) = window.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
    #[cfg(feature = "tracing")]
    tracing::warn!("skipping inset_traffic_lights, miniaturize button not found");
    return;
  };
  let zoom = window.standardWindowButton(NSWindowButton::ZoomButton);

  let title_bar_container_view = close.superview().unwrap().superview().unwrap();

  let close_rect = NSView::frame(&close);
  let title_bar_frame_height = close_rect.size.height + y;
  let mut title_bar_rect = NSView::frame(&title_bar_container_view);
  title_bar_rect.size.height = title_bar_frame_height;
  title_bar_rect.origin.y = window.frame().size.height - title_bar_frame_height;
  title_bar_container_view.setFrame(title_bar_rect);

  let space_between = NSView::frame(&miniaturize).origin.x - close_rect.origin.x;

  let mut window_buttons = vec![close, miniaturize];
  if let Some(zoom) = zoom {
    window_buttons.push(zoom);
  }

  for (i, button) in window_buttons.into_iter().enumerate() {
    let mut rect = NSView::frame(&button);
    rect.origin.x = x + (i as f64 * space_between);
    button.setFrameOrigin(rect.origin);
  }
}
