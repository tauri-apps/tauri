// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};

use crate::icon::{BadIcon, RgbaIcon};

/// An icon used for the window titlebar, taskbar, etc.
#[derive(Debug, Clone)]
pub struct PlatformIcon {
  raw: Vec<u8>,
  width: i32,
  height: i32,
  row_stride: i32,
}

impl From<PlatformIcon> for Pixbuf {
  fn from(icon: PlatformIcon) -> Self {
    Pixbuf::from_mut_slice(
      icon.raw,
      Colorspace::Rgb,
      true,
      8,
      icon.width,
      icon.height,
      icon.row_stride,
    )
  }
}

impl PlatformIcon {
  /// Creates an `Icon` from 32bpp RGBA data.
  ///
  /// The length of `rgba` must be divisible by 4, and `width * height` must equal
  /// `rgba.len() / 4`. Otherwise, this will return a `BadIcon` error.
  pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
    let icon = RgbaIcon::from_rgba(rgba, width, height)?;
    let row_stride = Pixbuf::calculate_rowstride(
      Colorspace::Rgb,
      true,
      8,
      icon.width as i32,
      icon.height as i32,
    );
    Ok(Self {
      raw: icon.rgba,
      width: icon.width as i32,
      height: icon.height as i32,
      row_stride,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_rgba_rejects_non_rgba_buffer() {
    let error = PlatformIcon::from_rgba(vec![0; 3], 1, 1).unwrap_err();

    assert!(matches!(
      error,
      BadIcon::ByteCountNotDivisibleBy4 { byte_count: 3 }
    ));
  }

  #[test]
  fn from_rgba_rejects_dimension_mismatch() {
    let error = PlatformIcon::from_rgba(vec![0; 4], 2, 1).unwrap_err();

    assert!(matches!(
      error,
      BadIcon::DimensionsVsPixelCount {
        width: 2,
        height: 1,
        width_x_height: 2,
        pixel_count: 1,
      }
    ));
  }
}
