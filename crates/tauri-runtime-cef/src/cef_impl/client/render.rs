// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::*;

use crate::{OffscreenEvent, OffscreenView};
use tauri_runtime::dpi::PhysicalSize;

wrap_render_handler! {
  pub(crate) struct TauriCefRenderHandler {
    view: OffscreenView,
  }

  impl RenderHandler {
    fn root_screen_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) -> i32 {
      let Some(rect) = rect else {
        return 0;
      };
      let window = &self.view.window;
      let scale = window.scale_factor();
      let position = window
        .outer_position()
        .unwrap_or_else(|_| window.surface_position())
        .to_logical::<f64>(scale);
      let size = window.outer_size().to_logical::<f64>(scale);
      *rect = Rect {
        x: position.x.round() as i32,
        y: position.y.round() as i32,
        width: size.width.round().max(1.0) as i32,
        height: size.height.round().max(1.0) as i32,
      };
      1
    }

    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
      let Some(rect) = rect else {
        return;
      };
      let bounds = self.view.physical_bounds();
      rect.x = 0;
      rect.y = 0;
      rect.width = (f64::from(bounds.width) / bounds.scale_factor)
        .round()
        .max(1.0) as i32;
      rect.height = (f64::from(bounds.height) / bounds.scale_factor)
        .round()
        .max(1.0) as i32;
    }

    fn screen_info(
      &self,
      _browser: Option<&mut Browser>,
      screen_info: Option<&mut ScreenInfo>,
    ) -> std::os::raw::c_int {
      let Some(screen_info) = screen_info else {
        return 0;
      };
      let bounds = self.view.physical_bounds();
      screen_info.device_scale_factor = bounds.scale_factor as f32;
      screen_info.depth = 32;
      screen_info.depth_per_component = 8;
      screen_info.is_monochrome = 0;
      let monitor = self.view.window.current_monitor();
      let origin = monitor
        .as_ref()
        .and_then(|monitor| monitor.position())
        .unwrap_or_default();
      let size = monitor
        .and_then(|monitor| monitor.current_video_mode())
        .map(|mode| mode.size())
        .unwrap_or_else(|| self.view.window.surface_size());
      screen_info.rect = Rect {
        x: (origin.x as f64 / bounds.scale_factor).round() as i32,
        y: (origin.y as f64 / bounds.scale_factor).round() as i32,
        width: (size.width as f64 / bounds.scale_factor).round().max(1.0) as i32,
        height: (size.height as f64 / bounds.scale_factor).round().max(1.0) as i32,
      };
      screen_info.available_rect = screen_info.rect.clone();
      1
    }

    fn screen_point(
      &self,
      _browser: Option<&mut Browser>,
      view_x: i32,
      view_y: i32,
      screen_x: Option<&mut i32>,
      screen_y: Option<&mut i32>,
    ) -> i32 {
      let (Some(screen_x), Some(screen_y)) = (screen_x, screen_y) else {
        return 0;
      };
      let scale = self.view.window.scale_factor();
      let bounds = self.view.bounds().to_logical::<f64, f64>(scale);
      let x = bounds.position.x + view_x as f64;
      let y = bounds.position.y + view_y as f64;
      #[cfg(target_os = "macos")]
      let Some(point) =
        crate::platform::macos::offscreen_screen_point(self.view.window.as_ref(), x, y)
      else {
        return 0;
      };
      #[cfg(windows)]
      let point = {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows::Win32::{
          Foundation::{HWND, POINT},
          Graphics::Gdi::ClientToScreen,
        };

        let Ok(handle) = self.view.window.window_handle() else {
          return 0;
        };
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
          return 0;
        };
        let mut point = POINT {
          x: (x * scale).round() as i32,
          y: (y * scale).round() as i32,
        };
        // winit's Windows surface_position is (0, 0), not the client offset
        // inside the decorated window. Let Win32 translate the client point.
        if !unsafe { ClientToScreen(HWND(handle.hwnd.get() as _), &mut point) }.as_bool() {
          return 0;
        }
        (point.x, point.y)
      };
      #[cfg(not(any(target_os = "macos", windows)))]
      let point = {
        let Ok(origin) = self.view.window.outer_position() else {
          return 0;
        };
        let offset = self.view.window.surface_position();
        (
          (origin.x as f64 + offset.x as f64 + x * scale).round() as i32,
          (origin.y as f64 + offset.y as f64 + y * scale).round() as i32,
        )
      };
      *screen_x = point.0;
      *screen_y = point.1;
      1
    }

    fn on_accelerated_paint(
      &self,
      _browser: Option<&mut Browser>,
      element: PaintElementType,
      dirty_rects: Option<&[Rect]>,
      info: Option<&AcceleratedPaintInfo>,
    ) {
      if let Some(info) = info {
        self.view.emit(OffscreenEvent::AcceleratedPaint {
          element,
          dirty_rects: dirty_rects.unwrap_or_default(),
          info,
        });
      }
    }

    fn on_popup_show(&self, browser: Option<&mut Browser>, show: std::os::raw::c_int) {
      self.view.popup_show(show != 0);
      // Applications may release popup textures while hidden. Reopening must
      // deliver pixels again instead of relying on a retained client-side image.
      if show != 0
        && let Some(host) = browser.and_then(|browser| browser.host())
      {
        host.invalidate(PaintElementType::POPUP);
      }
    }

    fn on_popup_size(&self, _browser: Option<&mut Browser>, rect: Option<&Rect>) {
      if let Some(rect) = rect {
        self.view.popup_size(rect);
      }
    }

    fn on_ime_composition_range_changed(
      &self,
      _browser: Option<&mut Browser>,
      _selected_range: Option<&Range>,
      character_bounds: Option<&[Rect]>,
    ) {
      self
        .view
        .set_ime_bounds(character_bounds.unwrap_or_default());
    }

    fn on_paint(
      &self,
      _browser: Option<&mut Browser>,
      type_: PaintElementType,
      dirty_rects: Option<&[Rect]>,
      buffer: *const u8,
      width: std::os::raw::c_int,
      height: std::os::raw::c_int,
    ) {
      // CEF lends the complete BGRA buffer and damage list for this callback.
      if let Some(event) = unsafe {
        paint_event(
          type_,
          dirty_rects.unwrap_or_default(),
          buffer,
          width,
          height,
        )
      } {
        self.view.emit(event);
      }
    }
  }
}

// The caller must keep the CEF image buffer valid throughout `'a`.
unsafe fn paint_event<'a>(
  element: PaintElementType,
  dirty_rects: &'a [Rect],
  buffer: *const u8,
  width: i32,
  height: i32,
) -> Option<OffscreenEvent<'a>> {
  let (width, height) = (u32::try_from(width).ok()?, u32::try_from(height).ok()?);
  let length = usize::try_from(width.checked_mul(height)?.checked_mul(4)?).ok()?;
  if buffer.is_null() || length == 0 || length > isize::MAX as usize {
    return None;
  }
  // SAFETY: validated length, and the caller guarantees callback-scoped CEF storage.
  let pixels = unsafe { std::slice::from_raw_parts(buffer, length) };
  Some(OffscreenEvent::Paint {
    element,
    dirty_rects,
    pixels,
    size: PhysicalSize::new(width, height),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn offscreen_paint_lends_pixels_without_copying() {
    let pixels = [1, 2, 3, 255, 4, 5, 6, 255];
    let damage = [Rect {
      x: 0,
      y: 0,
      width: 2,
      height: 1,
    }];
    let event =
      unsafe { paint_event(PaintElementType::VIEW, &damage, pixels.as_ptr(), 2, 1) }.unwrap();
    let OffscreenEvent::Paint {
      pixels: borrowed,
      size,
      dirty_rects,
      ..
    } = event
    else {
      panic!("expected software frame")
    };
    assert_eq!(borrowed.as_ptr(), pixels.as_ptr());
    assert_eq!(borrowed, &pixels);
    assert_eq!(size, PhysicalSize::new(2, 1));
    assert_eq!(dirty_rects.as_ptr(), damage.as_ptr());
  }

  #[test]
  fn offscreen_invalid_paint_never_constructs_a_slice() {
    for (width, height) in [(0, 0), (-1, 2), (i32::MAX, i32::MAX)] {
      assert!(
        unsafe { paint_event(PaintElementType::VIEW, &[], std::ptr::null(), width, height) }
          .is_none()
      );
    }
    assert!(unsafe { paint_event(PaintElementType::VIEW, &[], std::ptr::null(), 2, 2) }.is_none());
  }
}
