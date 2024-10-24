// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
  UI::Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled},
};

use tao::platform::windows::WindowExtWindows;

impl super::WindowExt for tao::window::Window {
  fn set_enabled(&self, enabled: bool) {
    let _ = unsafe { EnableWindow(HWND(self.hwnd() as _), enabled) };
  }

  fn is_enabled(&self) -> bool {
    unsafe { IsWindowEnabled(HWND(self.hwnd() as _)) }.as_bool()
  }

  fn center(&self) {
    if let Some(monitor) = self.current_monitor() {
      let mut window_size = self.outer_size();

      if self.is_decorated() {
        let mut rect = RECT::default();
        let result = unsafe {
          DwmGetWindowAttribute(
            HWND(self.hwnd() as _),
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32,
          )
        };
        if result.is_ok() {
          window_size.height = (rect.bottom - rect.top) as u32;
        }
      }

      let new_pos = super::calculate_window_center_position(window_size, monitor);
      self.set_outer_position(new_pos);
    }
  }

  fn draw_surface(
    &self,
    surface: &mut softbuffer::Surface<
      std::sync::Arc<tao::window::Window>,
      std::sync::Arc<tao::window::Window>,
    >,
    background_color: Option<tao::window::RGBA>,
  ) {
    let size = self.inner_size();
    if let (Some(width), Some(height)) = (
      std::num::NonZeroU32::new(size.width),
      std::num::NonZeroU32::new(size.height),
    ) {
      surface.resize(width, height).unwrap();
      let mut buffer = surface.buffer_mut().unwrap();
      let color = background_color
        .map(|(r, g, b, _)| (b as u32) | ((g as u32) << 8) | ((r as u32) << 16))
        .unwrap_or(0);
      buffer.fill(color);
      let _ = buffer.present();
    }
  }
}
