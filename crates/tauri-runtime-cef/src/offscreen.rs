// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{
  Arc, Mutex, Weak,
  atomic::{AtomicBool, Ordering},
};

use tauri_runtime::dpi::Rect;

/// A cursor notification on the CEF UI thread. Any custom cursor buffer is only
/// valid during the callback.
pub type CursorHandler = dyn Fn(cef::CursorType, Option<&cef::CursorInfo>) + Send + Sync;

/// Windowless browser configuration. The application owns painting, input/IME
/// forwarding and composition; Tauri keeps its IPC, protocols and browser lifecycle.
/// Offscreen webviews use Alloy style and cannot be reparented.
///
/// Enable `settings.windowless_rendering_enabled` through [`crate::Cef::with_settings`]
/// before creating an offscreen webview. Use [`crate::Cef::on_window_event`] for
/// native input and redraws, and [`crate::Webview::browser`] to reach CEF's host API.
///
/// The native window and event types are `tauri-winit`'s: depend on the same
/// `tauri-winit` version this crate uses to name them.
#[derive(Clone)]
pub struct Offscreen {
  /// Delivers shared resources through `RenderHandler::on_accelerated_paint`.
  /// Open and copy them during the callback, respecting CEF's resource lifetime.
  pub shared_texture_enabled: bool,
  /// Requires the application to call `BrowserHost::send_external_begin_frame`.
  pub external_begin_frame_enabled: bool,
  /// Handles cursor changes; other display notifications remain managed by Tauri.
  pub cursor_handler: Option<Arc<CursorHandler>>,
  pub(crate) render_handler: Arc<dyn Fn(OffscreenView) -> cef::RenderHandler + Send + Sync>,
}

impl Offscreen {
  /// Creates a render handler for each webview on the CEF UI thread.
  ///
  /// The handler must implement `view_rect` and a paint callback. Read the current
  /// bounds and scale from `view`; call [`OffscreenView::request_redraw`] after painting.
  /// CEF view rectangles use logical pixels, while paint buffers use physical pixels.
  /// CEF callbacks must not block on queued Tauri window operations.
  pub fn new<F: Fn(OffscreenView) -> cef::RenderHandler + Send + Sync + 'static>(
    handler: F,
  ) -> Self {
    Self {
      shared_texture_enabled: false,
      external_begin_frame_enabled: false,
      cursor_handler: None,
      render_handler: Arc::new(handler),
    }
  }

  pub(crate) fn window_info(&self, parent: cef::sys::cef_window_handle_t) -> cef::WindowInfo {
    let mut info = cef::WindowInfo::default().set_as_windowless(parent);
    info.shared_texture_enabled = i32::from(self.shared_texture_enabled);
    info.external_begin_frame_enabled = i32::from(self.external_begin_frame_enabled);
    info
  }
}

/// Live layout of one offscreen webview, shared with its render handler.
#[derive(Clone)]
pub struct OffscreenView {
  /// Weak, so a render handler CEF still holds does not keep a closed window alive.
  pub(crate) window: Weak<dyn winit::window::Window>,
  pub(crate) bounds: Arc<Mutex<Rect>>,
  pub(crate) visible: Arc<AtomicBool>,
}

impl OffscreenView {
  /// Native host used for scale, screen coordinates, IME and redraw requests.
  /// `None` once the runtime released the window; do not keep the returned
  /// reference past `Destroyed`.
  pub fn window(&self) -> Option<Arc<dyn winit::window::Window>> {
    self.window.upgrade()
  }

  /// Asks the host window to redraw, if it still exists.
  pub fn request_redraw(&self) {
    if let Some(window) = self.window.upgrade() {
      window.request_redraw();
    }
  }

  /// Bounds relative to the host, in logical pixels.
  pub fn bounds(&self) -> Rect {
    *self.bounds.lock().unwrap_or_else(|e| e.into_inner())
  }

  /// Requested webview visibility, independent of native window occlusion.
  pub fn is_visible(&self) -> bool {
    self.visible.load(Ordering::Relaxed)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn offscreen_options_select_alloy_and_the_requested_paint_mode() {
    let mut options = Offscreen::new(|_| unreachable!());
    let software = options.window_info(Default::default());
    assert_eq!(software.windowless_rendering_enabled, 1);
    assert_eq!(software.runtime_style, cef::RuntimeStyle::ALLOY);
    assert_eq!(software.shared_texture_enabled, 0);
    assert_eq!(software.external_begin_frame_enabled, 0);

    options.shared_texture_enabled = true;
    options.external_begin_frame_enabled = true;
    let accelerated = options.window_info(Default::default());
    assert_eq!(accelerated.shared_texture_enabled, 1);
    assert_eq!(accelerated.external_begin_frame_enabled, 1);
  }
}
