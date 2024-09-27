// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
use objc2_foundation::MainThreadMarker;
use tao::platform::macos::WindowExtMacOS;

impl super::WindowExt for tao::window::Window {
  // based on electron implementation
  // https://github.com/electron/electron/blob/15db63e26df3e3d59ce6281f030624f746518511/shell/browser/native_window_mac.mm#L474
  fn set_enabled(&self, enabled: bool) {
    let ns_window: &NSWindow = unsafe { &*self.ns_window().cast() };
    if !enabled {
      let frame = ns_window.frame();
      let mtm = MainThreadMarker::new()
        .expect("`Window::set_enabled` can only be called on the main thread");
      let sheet = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          mtm.alloc(),
          frame,
          NSWindowStyleMask::Titled,
          NSBackingStoreType::NSBackingStoreBuffered,
          false,
        )
      };
      unsafe { sheet.setAlphaValue(0.5) };
      unsafe { ns_window.beginSheet_completionHandler(&sheet, None) };
    } else if let Some(attached) = unsafe { ns_window.attachedSheet() } {
      unsafe { ns_window.endSheet(&attached) };
    }
  }

  fn is_enabled(&self) -> bool {
    let ns_window: &NSWindow = unsafe { &*self.ns_window().cast() };
    unsafe { ns_window.attachedSheet() }.is_none()
  }

  fn center(&self) {
    let ns_window: &NSWindow = unsafe { &*self.ns_window().cast() };
    ns_window.center();
  }
}
