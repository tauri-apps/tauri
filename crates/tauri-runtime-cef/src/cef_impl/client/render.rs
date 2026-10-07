// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use cef::*;

use crate::OffscreenSurface;

wrap_render_handler! {
  pub(crate) struct TauriCefRenderHandler {
    surface: OffscreenSurface,
  }

  impl RenderHandler {
    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
      let Some(rect) = rect else {
        return;
      };
      let bounds = self.surface.bounds();
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
      let bounds = self.surface.bounds();
      screen_info.device_scale_factor = bounds.scale_factor as f32;
      screen_info.depth = 32;
      screen_info.depth_per_component = 8;
      screen_info.is_monochrome = 0;
      screen_info.rect = self.surface.screen().1;
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
      let bounds = self.surface.bounds();
      let (origin, _) = self.surface.screen();
      let x = origin.0 as f64 + bounds.x as f64 + view_x as f64 * bounds.scale_factor;
      let y = origin.1 as f64 + bounds.y as f64 + view_y as f64 * bounds.scale_factor;
      let scale = if cfg!(target_os = "macos") {
        bounds.scale_factor
      } else {
        1.0
      };
      *screen_x = (x / scale).round() as i32;
      *screen_y = (y / scale).round() as i32;
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
        self.surface.accelerated_paint(element, dirty_rects.unwrap_or_default(), info);
      }
    }

    fn on_popup_show(&self, _browser: Option<&mut Browser>, show: std::os::raw::c_int) {
      self.surface.set_popup_visible(show != 0);
    }

    fn on_popup_size(&self, _browser: Option<&mut Browser>, rect: Option<&Rect>) {
      if let Some(rect) = rect {
        self.surface.set_popup_rect(
          rect.x,
          rect.y,
          rect.width.max(0) as u32,
          rect.height.max(0) as u32,
        );
      }
    }

    fn on_ime_composition_range_changed(
      &self,
      _browser: Option<&mut Browser>,
      _selected_range: Option<&Range>,
      character_bounds: Option<&[Rect]>,
    ) {
      self
        .surface
        .set_ime_character_bounds(character_bounds.unwrap_or_default());
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
      let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
        return;
      };
      let Some(length) = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .filter(|bytes| *bytes <= isize::MAX as usize)
      else {
        return;
      };
      if buffer.is_null() || length == 0 {
        return;
      }

      // CEF guarantees a tightly packed BGRA8 buffer containing `width * height`
      // pixels for the duration of this callback. Copy it before returning so no
      // CEF-owned pointer escapes the unsafe boundary.
      let pixels = unsafe { std::slice::from_raw_parts(buffer, length) };
      self.surface.publish(
        type_ == PaintElementType::POPUP,
        width,
        height,
        Arc::from(pixels),
        dirty_rects.unwrap_or_default(),
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn offscreen_paint_copies_the_borrowed_cef_buffer() {
    let surface = OffscreenSurface::new(|| {});
    let handler = TauriCefRenderHandler::new(surface.clone());
    let mut pixels = [1, 2, 3, 255, 4, 5, 6, 255];
    let damage = [Rect {
      x: 0,
      y: 0,
      width: 2,
      height: 1,
    }];
    handler.on_paint(
      None,
      PaintElementType::VIEW,
      Some(&damage),
      pixels.as_ptr(),
      2,
      1,
    );
    pixels.fill(0);
    let frame = surface.snapshot().view.unwrap();
    assert_eq!(&*frame.pixels, &[1, 2, 3, 255, 4, 5, 6, 255]);
    assert_eq!((frame.width, frame.height), (2, 1));
    assert_eq!(frame.dirty_rects[0].width, 2);
  }

  #[test]
  fn offscreen_invalid_paint_does_not_replace_the_current_frame() {
    let surface = OffscreenSurface::new(|| {});
    let handler = TauriCefRenderHandler::new(surface.clone());
    handler.on_paint(None, PaintElementType::VIEW, None, std::ptr::null(), 1, 1);
    handler.on_paint(None, PaintElementType::VIEW, None, std::ptr::null(), -1, 1);
    assert!(surface.snapshot().view.is_none());
  }
}
