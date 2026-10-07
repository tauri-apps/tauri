// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};

type AcceleratedPaintHandler =
  dyn Fn(cef::PaintElementType, &[cef::Rect], &cef::AcceleratedPaintInfo) + Send + Sync;

/// A rectangle in physical pixels within an off-screen browser surface.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OffscreenRect {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
}

/// One premultiplied BGRA8 frame rendered by CEF.
#[derive(Clone, Debug)]
pub struct OffscreenFrame {
  pub width: u32,
  pub height: u32,
  /// Damage relative to the preceding CEF paint, in physical pixels. Snapshots may skip
  /// paints; upload the full buffer unless every intervening paint was consumed.
  pub dirty_rects: Arc<[cef::Rect]>,
  pub pixels: Arc<[u8]>,
  pub serial: u64,
}

/// The latest view and popup frames produced by an off-screen CEF browser.
#[derive(Clone, Debug, Default)]
pub struct OffscreenSnapshot {
  /// Requested webview visibility, independent of native window occlusion.
  pub visible: bool,
  pub view: Option<OffscreenFrame>,
  pub popup: Option<OffscreenFrame>,
  /// Whether CEF is showing a popup widget, including in accelerated mode.
  pub popup_visible: bool,
  pub popup_rect: OffscreenRect,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OffscreenBounds {
  pub(crate) x: i32,
  pub(crate) y: i32,
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) scale_factor: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OffscreenImeCursorArea {
  pub(crate) x: i32,
  pub(crate) y: i32,
  pub(crate) width: u32,
  pub(crate) height: u32,
}

impl Default for OffscreenBounds {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      width: 1,
      height: 1,
      scale_factor: 1.0,
    }
  }
}

struct OffscreenState {
  bounds: OffscreenBounds,
  screen_origin: (i32, i32),
  screen_rect: cef::Rect,
  visible: bool,
  popup_visible: bool,
  popup_rect: OffscreenRect,
  view: Option<OffscreenFrame>,
  popup: Option<OffscreenFrame>,
  ime_character_bounds: Vec<OffscreenRect>,
  next_serial: u64,
  cursor: (bool, winit::cursor::CursorIcon),
}

impl Default for OffscreenState {
  fn default() -> Self {
    Self {
      bounds: OffscreenBounds::default(),
      screen_origin: (0, 0),
      screen_rect: cef::Rect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
      },
      visible: true,
      popup_visible: false,
      popup_rect: OffscreenRect::default(),
      view: None,
      popup: None,
      ime_character_bounds: Vec::new(),
      next_serial: 1,
      cursor: (true, winit::cursor::CursorIcon::Default),
    }
  }
}

struct OffscreenSurfaceInner {
  attached: AtomicBool,
  state: Mutex<OffscreenState>,
  request_redraw: Arc<dyn Fn() + Send + Sync>,
}

/// Shared output surface for a windowless CEF webview.
///
/// CEF owns the paint buffers only for the duration of its render callback, so
/// this surface copies each completed frame into application-owned memory. The
/// consumer can then upload it to its own GPU texture and compose it with other
/// native content without retaining CEF's transient pointers or handles.
#[derive(Clone)]
pub struct OffscreenSurface {
  inner: Arc<OffscreenSurfaceInner>,
  accelerated_paint: Option<Arc<AcceleratedPaintHandler>>,
  pub(crate) external_begin_frame: bool,
}

impl std::fmt::Debug for OffscreenSurface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OffscreenSurface").finish_non_exhaustive()
  }
}

impl OffscreenSurface {
  /// Creates a surface for one windowless webview. Clones observe the same browser.
  ///
  /// `request_redraw` runs on the CEF UI thread after paint, popup, or IME updates.
  /// It should schedule presentation, not block waiting for an event-loop operation.
  pub fn new(request_redraw: impl Fn() + Send + Sync + 'static) -> Self {
    Self {
      inner: Arc::new(OffscreenSurfaceInner {
        attached: AtomicBool::new(false),
        state: Mutex::new(OffscreenState::default()),
        request_redraw: Arc::new(request_redraw),
      }),
      accelerated_paint: None,
      external_begin_frame: false,
    }
  }

  pub(crate) fn attach(&self) -> bool {
    !self.inner.attached.swap(true, Ordering::Relaxed)
  }

  /// Uses CEF shared textures instead of copying software paint buffers.
  ///
  /// Configure this before cloning the surface or passing it to a webview builder.
  /// The callback runs synchronously on CEF's UI thread. Each shared resource must
  /// be reopened and copied into an application-owned texture before returning;
  /// GPU work accessing it must be synchronized before CEF can recycle it. Never
  /// retain `info` or its handles for a later frame. No wgpu version is imposed.
  ///
  /// In this mode `snapshot` reports popup geometry but contains no pixel frames.
  #[must_use]
  pub fn with_accelerated_paint<F>(mut self, handler: F) -> Self
  where
    F: Fn(cef::PaintElementType, &[cef::Rect], &cef::AcceleratedPaintInfo) + Send + Sync + 'static,
  {
    self.accelerated_paint = Some(Arc::new(handler));
    self
  }

  /// Lets the application schedule Chromium frames instead of CEF's timer.
  ///
  /// When enabled, call `BrowserHost::send_external_begin_frame` on the CEF UI
  /// thread through `WebviewCefExt::with_cef_webview`. Native redraws alone do not
  /// advance Chromium. Configure this before cloning the surface.
  #[must_use]
  pub fn with_external_begin_frame(mut self, enabled: bool) -> Self {
    self.external_begin_frame = enabled;
    self
  }

  pub(crate) fn is_accelerated(&self) -> bool {
    self.accelerated_paint.is_some()
  }

  pub(crate) fn accelerated_paint(
    &self,
    element: cef::PaintElementType,
    dirty_rects: &[cef::Rect],
    info: &cef::AcceleratedPaintInfo,
  ) {
    if let Some(handler) = &self.accelerated_paint {
      handler(element, dirty_rects, info);
      (self.inner.request_redraw)();
    }
  }

  pub(crate) fn set_screen(&self, origin: (i32, i32), rect: cef::Rect) {
    let mut state = self.inner.state.lock().unwrap();
    state.screen_origin = origin;
    state.screen_rect = rect;
  }

  pub(crate) fn screen(&self) -> ((i32, i32), cef::Rect) {
    let state = self.inner.state.lock().unwrap();
    (state.screen_origin, state.screen_rect.clone())
  }

  /// Returns the latest complete frames without copying their pixels.
  pub fn snapshot(&self) -> OffscreenSnapshot {
    let state = self.inner.state.lock().unwrap();
    OffscreenSnapshot {
      visible: state.visible,
      view: state.visible.then(|| state.view.clone()).flatten(),
      popup: (state.visible && state.popup_visible)
        .then(|| state.popup.clone())
        .flatten(),
      popup_visible: state.visible && state.popup_visible,
      popup_rect: OffscreenRect {
        x: (state.popup_rect.x as f64 * state.bounds.scale_factor).round() as i32,
        y: (state.popup_rect.y as f64 * state.bounds.scale_factor).round() as i32,
        width: (state.popup_rect.width as f64 * state.bounds.scale_factor).round() as u32,
        height: (state.popup_rect.height as f64 * state.bounds.scale_factor).round() as u32,
      },
    }
  }

  pub(crate) fn set_cursor(&self, cursor: cef::CursorType) {
    use cef::CursorType as C;
    use winit::cursor::CursorIcon as W;
    let icon = match cursor {
      C::CROSS => W::Crosshair,
      C::HAND => W::Pointer,
      C::IBEAM => W::Text,
      C::WAIT => W::Wait,
      C::HELP => W::Help,
      C::MOVE => W::Move,
      C::EASTRESIZE => W::EResize,
      C::NORTHRESIZE => W::NResize,
      C::NORTHEASTRESIZE => W::NeResize,
      C::NORTHWESTRESIZE => W::NwResize,
      C::SOUTHRESIZE => W::SResize,
      C::SOUTHEASTRESIZE => W::SeResize,
      C::SOUTHWESTRESIZE => W::SwResize,
      C::WESTRESIZE => W::WResize,
      C::NORTHSOUTHRESIZE => W::NsResize,
      C::EASTWESTRESIZE => W::EwResize,
      C::NORTHEASTSOUTHWESTRESIZE => W::NeswResize,
      C::NORTHWESTSOUTHEASTRESIZE => W::NwseResize,
      C::COLUMNRESIZE => W::ColResize,
      C::ROWRESIZE => W::RowResize,
      C::VERTICALTEXT => W::VerticalText,
      C::CELL => W::Cell,
      C::CONTEXTMENU => W::ContextMenu,
      C::ALIAS => W::Alias,
      C::PROGRESS => W::Progress,
      C::NODROP => W::NoDrop,
      C::COPY => W::Copy,
      C::NOTALLOWED => W::NotAllowed,
      C::ZOOMIN => W::ZoomIn,
      C::ZOOMOUT => W::ZoomOut,
      C::GRAB => W::Grab,
      C::GRABBING => W::Grabbing,
      _ => W::Default,
    };
    self.inner.state.lock().unwrap().cursor = (cursor != C::NONE, icon);
    (self.inner.request_redraw)();
  }

  pub(crate) fn cursor(&self) -> (bool, winit::cursor::CursorIcon) {
    self.inner.state.lock().unwrap().cursor
  }

  pub(crate) fn update_screen(&self, window: &dyn winit::window::Window) {
    let origin = window.surface_position();
    let scale = window.scale_factor();
    let (position, size) = window
      .current_monitor()
      .map(|monitor| {
        (
          monitor.position().unwrap_or_default(),
          monitor
            .current_video_mode()
            .map(|mode| mode.size())
            .unwrap_or_else(|| window.surface_size()),
        )
      })
      .unwrap_or((origin, window.surface_size()));
    self.set_screen(
      (origin.x, origin.y),
      cef::Rect {
        x: (position.x as f64 / scale).round() as i32,
        y: (position.y as f64 / scale).round() as i32,
        width: (size.width as f64 / scale).round().max(1.0) as i32,
        height: (size.height as f64 / scale).round().max(1.0) as i32,
      },
    );
  }

  pub(crate) fn bounds(&self) -> OffscreenBounds {
    self.inner.state.lock().unwrap().bounds
  }

  pub(crate) fn set_bounds(&self, bounds: OffscreenBounds) {
    self.inner.state.lock().unwrap().bounds = bounds;
  }

  pub(crate) fn set_visible(&self, visible: bool) {
    self.inner.state.lock().unwrap().visible = visible;
    (self.inner.request_redraw)();
  }

  pub(crate) fn is_visible(&self) -> bool {
    self.inner.state.lock().unwrap().visible
  }

  pub(crate) fn set_popup_visible(&self, visible: bool) {
    let mut state = self.inner.state.lock().unwrap();
    state.popup_visible = visible;
    if !visible {
      state.popup = None;
    }
    drop(state);
    (self.inner.request_redraw)();
  }

  pub(crate) fn set_popup_rect(&self, x: i32, y: i32, width: u32, height: u32) {
    let mut state = self.inner.state.lock().unwrap();
    state.popup_rect = OffscreenRect {
      x,
      y,
      width,
      height,
    };
    drop(state);
    (self.inner.request_redraw)();
  }

  pub(crate) fn set_ime_character_bounds(&self, bounds: &[cef::Rect]) {
    self.inner.state.lock().unwrap().ime_character_bounds = bounds
      .iter()
      .map(|rect| OffscreenRect {
        x: rect.x,
        y: rect.y,
        width: rect.width.max(0) as u32,
        height: rect.height.max(0) as u32,
      })
      .collect();
    (self.inner.request_redraw)();
  }

  pub(crate) fn ime_cursor_area(&self, cursor: usize) -> Option<OffscreenImeCursorArea> {
    let state = self.inner.state.lock().unwrap();
    let index = cursor
      .saturating_sub(1)
      .min(state.ime_character_bounds.len().checked_sub(1)?);
    let rect = state.ime_character_bounds[index];
    let scale = state.bounds.scale_factor;
    Some(OffscreenImeCursorArea {
      x: state.bounds.x + (f64::from(rect.x) * scale).round() as i32,
      y: state.bounds.y + (f64::from(rect.y) * scale).round() as i32,
      width: (f64::from(rect.width) * scale).round().max(1.0) as u32,
      height: (f64::from(rect.height) * scale).round().max(1.0) as u32,
    })
  }

  pub(crate) fn publish(
    &self,
    popup: bool,
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
    dirty_rects: &[cef::Rect],
  ) {
    let mut state = self.inner.state.lock().unwrap();
    let serial = state.next_serial;
    state.next_serial = state.next_serial.wrapping_add(1).max(1);
    let frame = OffscreenFrame {
      width,
      height,
      pixels,
      dirty_rects: Arc::from(dirty_rects),
      serial,
    };
    if popup {
      state.popup = Some(frame);
    } else {
      state.view = Some(frame);
    }
    drop(state);
    (self.inner.request_redraw)();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn offscreen_snapshots_keep_owned_pixels_across_paints() {
    let surface = OffscreenSurface::new(|| {});
    surface.publish(false, 1, 1, Arc::from([1, 2, 3, 255]), &[]);
    let first = surface.snapshot().view.unwrap();
    surface.publish(false, 1, 1, Arc::from([4, 5, 6, 255]), &[]);
    let second = surface.snapshot().view.unwrap();
    assert_eq!(&*first.pixels, &[1, 2, 3, 255]);
    assert_eq!(&*second.pixels, &[4, 5, 6, 255]);
    assert_ne!(first.serial, second.serial);
  }

  #[test]
  fn offscreen_popup_visibility_and_scale_follow_the_browser() {
    let surface = OffscreenSurface::new(|| {});
    surface.set_popup_rect(-2, 10, 30, 40);
    surface.set_popup_visible(true);
    surface.publish(true, 30, 40, Arc::from(vec![0; 30 * 40 * 4]), &[]);
    surface.set_bounds(OffscreenBounds {
      scale_factor: 2.0,
      ..Default::default()
    });
    let snapshot = surface.snapshot();
    assert_eq!(
      snapshot.popup_rect,
      OffscreenRect {
        x: -4,
        y: 20,
        width: 60,
        height: 80
      }
    );
    assert!(snapshot.popup.is_some());
    surface.set_visible(false);
    assert!(!surface.snapshot().popup_visible);
    assert!(surface.snapshot().popup.is_none());
    surface.set_visible(true);
    surface.set_popup_visible(false);
    assert!(surface.snapshot().popup.is_none());
  }

  #[test]
  fn offscreen_surface_cannot_be_shared_by_two_browsers() {
    let surface = OffscreenSurface::new(|| {});
    assert!(surface.attach());
    assert!(!surface.clone().attach());
  }

  #[test]
  fn offscreen_accelerated_callback_does_not_retain_a_cef_frame() {
    let called = Arc::new(AtomicBool::new(false));
    let received = called.clone();
    let surface = OffscreenSurface::new(|| {}).with_accelerated_paint(move |element, rects, _| {
      assert_eq!(element, cef::PaintElementType::VIEW);
      assert!(rects.is_empty());
      received.store(true, Ordering::Relaxed);
    });
    surface.accelerated_paint(cef::PaintElementType::VIEW, &[], &Default::default());
    assert!(called.load(Ordering::Relaxed));
    assert!(surface.snapshot().view.is_none());
  }

  #[test]
  fn ime_cursor_area_maps_cef_logical_bounds_into_parent_pixels() {
    let surface = OffscreenSurface::new(|| {});
    surface.set_bounds(OffscreenBounds {
      x: 40,
      y: 60,
      width: 800,
      height: 600,
      scale_factor: 1.5,
    });
    surface.set_ime_character_bounds(&[
      cef::Rect {
        x: 10,
        y: 20,
        width: 8,
        height: 12,
      },
      cef::Rect {
        x: 18,
        y: 20,
        width: 9,
        height: 12,
      },
    ]);

    assert_eq!(
      surface.ime_cursor_area(2),
      Some(OffscreenImeCursorArea {
        x: 67,
        y: 90,
        width: 14,
        height: 18,
      })
    );
  }
}
