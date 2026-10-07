// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};
use tauri_runtime::dpi::{PhysicalSize, Rect};
use winit::cursor::{Cursor, CursorIcon, CustomCursorSource};

type PaintHandler = dyn Fn(&OffscreenView, OffscreenEvent<'_>) + Send + Sync;

/// Browser output delivered synchronously on the CEF UI thread.
///
/// Tauri retains no pixels. Copy or upload software pixels during the callback if
/// they are needed afterwards. Shared GPU resources must be opened, copied and
/// synchronized before returning, as required by CEF; their handles cannot be cached.
#[non_exhaustive]
pub enum OffscreenEvent<'a> {
  /// A complete premultiplied BGRA8 image, with damage in physical pixels.
  Paint {
    /// Main view or popup widget.
    element: cef::PaintElementType,
    /// Regions changed since the previous paint.
    dirty_rects: &'a [cef::Rect],
    /// Tightly packed image data with an upper-left origin.
    pixels: &'a [u8],
    /// Image dimensions in physical pixels.
    size: PhysicalSize<u32>,
  },
  /// A platform resource borrowed from CEF's frame pool for this callback only.
  AcceleratedPaint {
    /// Main view or popup widget.
    element: cef::PaintElementType,
    /// Regions changed since the previous paint, in physical pixels.
    dirty_rects: &'a [cef::Rect],
    /// Platform resource and image metadata.
    info: &'a cef::AcceleratedPaintInfo,
  },
  /// Popup bounds in logical pixels relative to the view, or `None` when hidden.
  Popup(Option<cef::Rect>),
  /// Requested webview visibility changed.
  Visibility(bool),
  /// The browser has closed. No further output will be delivered for this view.
  Closed,
}

/// Windowless rendering with Tauri-managed geometry, input, focus and IME.
///
/// Enable `settings.windowless_rendering_enabled` through [`crate::Cef::with_settings`].
/// The application owns GPU resources and final composition. Present on native
/// redraw events from [`crate::Cef::on_window_event`]; no graphics API is required
/// by the runtime. Offscreen webviews cannot be reparented.
///
/// IME composition is supported on Windows and macOS. It is currently unsupported
/// on Linux because the GTK4 window backend does not provide IME events.
///
/// Native screen-reader bridging and browser-originated native drag sources
/// are not implemented yet. Native file drops use Tauri events by default;
/// disabling the drag-drop handler forwards incoming files to HTML instead.
#[derive(Clone)]
pub struct Offscreen {
  /// Deliver `AcceleratedPaint` instead of software `Paint` events.
  pub shared_texture_enabled: bool,
  /// The application advances Chromium with [`crate::WebviewCefExt::send_external_begin_frame`].
  pub external_begin_frame_enabled: bool,
  pub(crate) handler: Arc<PaintHandler>,
}

impl Offscreen {
  /// Receives browser output without an intermediate frame cache or mandatory copy.
  ///
  /// The callback runs on the CEF UI thread and must not wait for a queued window
  /// operation. A native redraw is requested after output changes.
  pub fn new<F: Fn(&OffscreenView, OffscreenEvent<'_>) + Send + Sync + 'static>(
    handler: F,
  ) -> Self {
    Self {
      shared_texture_enabled: false,
      external_begin_frame_enabled: false,
      handler: Arc::new(handler),
    }
  }

  pub(crate) fn window_info(&self, parent: cef::sys::cef_window_handle_t) -> cef::WindowInfo {
    let mut info = cef::WindowInfo::default().set_as_windowless(parent);
    info.shared_texture_enabled = i32::from(self.shared_texture_enabled);
    info.external_begin_frame_enabled = i32::from(self.external_begin_frame_enabled);
    info
  }
}

/// Live layout and native host of an offscreen webview.
#[derive(Clone)]
pub struct OffscreenView {
  pub(crate) window: Arc<dyn winit::window::Window>,
  pub(crate) state: Arc<Mutex<OffscreenState>>,
  label: Arc<str>,
  pub(crate) external_begin_frame_enabled: bool,
  handler: Arc<PaintHandler>,
}

pub(crate) struct OffscreenState {
  pub(crate) bounds: Rect,
  pub(crate) visible: bool,
  pub(crate) occluded: bool,
  pub(crate) hidden: bool,
  closed: bool,
  popup_visible: bool,
  popup_rect: cef::Rect,
  ime_bounds: Vec<cef::Rect>,
  pub(crate) cursor: CursorUpdate,
  pub(crate) cursor_serial: u64,
}

#[derive(Clone)]
pub(crate) enum CursorUpdate {
  Hidden,
  Ready(Cursor),
  Image(CustomCursorSource),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OffscreenBounds {
  pub(crate) x: i32,
  pub(crate) y: i32,
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) scale_factor: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OffscreenImeCursorArea {
  pub(crate) x: i32,
  pub(crate) y: i32,
  pub(crate) width: u32,
  pub(crate) height: u32,
}

impl OffscreenView {
  pub(crate) fn new(
    window: Arc<dyn winit::window::Window>,
    label: &str,
    bounds: Rect,
    options: &Offscreen,
  ) -> Self {
    Self {
      state: Arc::new(Mutex::new(OffscreenState {
        bounds,
        visible: true,
        occluded: false,
        hidden: false,
        closed: false,
        popup_visible: false,
        popup_rect: Default::default(),
        ime_bounds: Vec::new(),
        cursor: CursorUpdate::Ready(CursorIcon::Default.into()),
        cursor_serial: 0,
      })),
      window,
      external_begin_frame_enabled: options.external_begin_frame_enabled,
      label: label.into(),
      handler: options.handler.clone(),
    }
  }

  /// Tauri label of this webview.
  pub fn label(&self) -> &str {
    &self.label
  }

  /// Native host for presentation and redraw requests. Release GPU resources and
  /// retained native window references on the native `Destroyed` event.
  pub fn window(&self) -> &Arc<dyn winit::window::Window> {
    &self.window
  }

  /// Bounds relative to the host, in logical pixels.
  pub fn bounds(&self) -> Rect {
    self.state.lock().unwrap().bounds
  }

  /// Requested visibility, independent of native window occlusion.
  pub fn is_visible(&self) -> bool {
    self.state.lock().unwrap().visible
  }

  pub(crate) fn physical_bounds(&self) -> OffscreenBounds {
    let scale_factor = self.window.scale_factor();
    let bounds = self.bounds().to_physical::<i32, u32>(scale_factor);
    OffscreenBounds {
      x: bounds.position.x,
      y: bounds.position.y,
      width: bounds.size.width,
      height: bounds.size.height,
      scale_factor,
    }
  }

  pub(crate) fn emit(&self, event: OffscreenEvent<'_>) {
    let open = !self.state.lock().unwrap().closed;
    if open {
      (self.handler)(self, event);
      self.window.request_redraw();
    }
  }

  pub(crate) fn close(&self) {
    {
      let mut state = self.state.lock().unwrap();
      if state.closed {
        return;
      }
      state.closed = true;
      state.visible = false;
    }
    (self.handler)(self, OffscreenEvent::Closed);
    self.window.request_redraw();
  }

  pub(crate) fn popup_bounds(&self) -> Option<cef::Rect> {
    self.state.lock().unwrap().popup_bounds()
  }

  pub(crate) fn popup_show(&self, show: bool) {
    let bounds = {
      let mut state = self.state.lock().unwrap();
      state.popup_visible = show;
      state.popup_bounds()
    };
    self.emit(OffscreenEvent::Popup(bounds));
  }

  pub(crate) fn popup_size(&self, rect: &cef::Rect) {
    let bounds = {
      let mut state = self.state.lock().unwrap();
      state.popup_rect = rect.clone();
      state.popup_bounds()
    };
    if bounds.is_some() {
      self.emit(OffscreenEvent::Popup(bounds));
    }
  }

  pub(crate) fn set_ime_bounds(&self, bounds: &[cef::Rect]) {
    self.state.lock().unwrap().ime_bounds = bounds.to_vec();
    self.window.request_redraw();
  }

  pub(crate) fn ime_cursor_area(&self, cursor: usize) -> Option<OffscreenImeCursorArea> {
    let state = self.state.lock().unwrap();
    ime_cursor_area(
      state.bounds,
      self.window.scale_factor(),
      &state.ime_bounds,
      cursor,
    )
  }

  pub(crate) fn set_cursor(&self, cursor: CursorUpdate) {
    let mut state = self.state.lock().unwrap();
    state.cursor = cursor;
    state.cursor_serial = state.cursor_serial.wrapping_add(1);
    drop(state);
    self.window.request_redraw();
  }
}

impl OffscreenState {
  fn popup_bounds(&self) -> Option<cef::Rect> {
    (self.visible && self.popup_visible && self.popup_rect.width > 0 && self.popup_rect.height > 0)
      .then(|| self.popup_rect.clone())
  }
}

fn ime_cursor_area(
  bounds: Rect,
  scale: f64,
  characters: &[cef::Rect],
  cursor: usize,
) -> Option<OffscreenImeCursorArea> {
  let index = cursor
    .saturating_sub(1)
    .min(characters.len().checked_sub(1)?);
  let character = &characters[index];
  let origin = bounds.position.to_logical::<f64>(scale);
  Some(OffscreenImeCursorArea {
    x: ((origin.x + character.x as f64) * scale).round() as i32,
    y: ((origin.y + character.y as f64) * scale).round() as i32,
    width: (character.width as f64 * scale).round().max(1.0) as u32,
    height: (character.height as f64 * scale).round().max(1.0) as u32,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn offscreen_options_select_alloy_and_the_requested_paint_mode() {
    let mut options = Offscreen::new(|_, _| {});
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

  #[test]
  fn offscreen_ime_position_includes_view_offset_and_dpi() {
    let bounds = Rect {
      position: tauri_runtime::dpi::LogicalPosition::new(20., 30.).into(),
      size: PhysicalSize::new(800, 600).into(),
    };
    let characters = [cef::Rect {
      x: 18,
      y: 20,
      width: 9,
      height: 12,
    }];
    assert_eq!(
      ime_cursor_area(bounds, 1.5, &characters, 100),
      Some(OffscreenImeCursorArea {
        x: 57,
        y: 75,
        width: 14,
        height: 18
      })
    );
    assert_eq!(ime_cursor_area(bounds, 1.5, &[], 0), None);
  }
}
