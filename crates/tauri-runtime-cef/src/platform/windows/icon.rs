// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::Icon;
use windows::Win32::UI::WindowsAndMessaging::{CreateIcon, HICON};

pub fn icon_to_hicon(icon: Icon<'static>) -> Option<HICON> {
  let width = icon.width;
  let height = icon.height;
  let mut rgba = icon.rgba.into_owned();
  if width == 0 || height == 0 || rgba.len() != width as usize * height as usize * 4 {
    return None;
  }

  let mut and_mask = Vec::with_capacity(width as usize * height as usize);
  for pixel in rgba.chunks_exact_mut(4) {
    and_mask.push(pixel[3].wrapping_sub(u8::MAX));
    pixel.swap(0, 2);
  }

  unsafe {
    CreateIcon(
      None,
      width as i32,
      height as i32,
      1,
      32,
      and_mask.as_ptr(),
      rgba.as_ptr(),
    )
    .ok()
  }
}
