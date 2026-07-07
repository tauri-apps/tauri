// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fs::File, io::BufWriter, path::Path};

use crate::icon::{BadIcon, PIXEL_SIZE};

#[derive(Debug, Clone)]
pub struct PlatformIcon {
    rgba: Vec<u8>,
    width: i32,
    height: i32,
}

impl PlatformIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        if !rgba.len().is_multiple_of(PIXEL_SIZE) {
            return Err(BadIcon::ByteCountNotDivisibleBy4 {
                byte_count: rgba.len(),
            });
        }

        let pixel_count = rgba.len() / PIXEL_SIZE;
        let width_x_height = width as usize * height as usize;
        if pixel_count != width_x_height {
            return Err(BadIcon::DimensionsVsPixelCount {
                width,
                height,
                width_x_height,
                pixel_count,
            });
        }

        Ok(Self {
            rgba,
            width: width as i32,
            height: height as i32,
        })
    }

    pub fn write_to_png(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let png = File::create(path)?;
        let w = &mut BufWriter::new(png);

        let mut encoder = png::Encoder::new(w, self.width as _, self.height as _);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.rgba)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rgba_rejects_invalid_byte_count() {
        assert!(matches!(
            PlatformIcon::from_rgba(vec![0, 0, 0], 1, 1),
            Err(BadIcon::ByteCountNotDivisibleBy4 { byte_count: 3 })
        ));
    }

    #[test]
    fn from_rgba_rejects_dimension_mismatch() {
        assert!(matches!(
            PlatformIcon::from_rgba(vec![0, 0, 0, 0], 2, 1),
            Err(BadIcon::DimensionsVsPixelCount {
                width: 2,
                height: 1,
                width_x_height: 2,
                pixel_count: 1,
            })
        ));
    }
}
