// Copyright 2014-2021 The winit contributors
// Copyright 2021-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use crate::icon::BadIcon;

/// An icon used for the window titlebar, taskbar, etc.
///
/// Stores raw PNG bytes to be `Send + Sync` safe. The GTK `BytesIcon`
/// is created lazily when needed on the GTK main thread.
#[derive(Debug, Clone)]
pub struct PlatformIcon {
    png_data: Vec<u8>,
}

// PlatformIcon is Send + Sync because it only contains Vec<u8>
// The BytesIcon is created lazily on the GTK main thread when needed
unsafe impl Send for PlatformIcon {}
unsafe impl Sync for PlatformIcon {}

impl PlatformIcon {
    /// Creates an `Icon` from 32bpp RGBA data.
    ///
    /// The length of `rgba` must be divisible by 4, and `width * height` must equal
    /// `rgba.len() / 4`. Otherwise, this will return a `BadIcon` error.
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        let mut png_data = Vec::with_capacity(rgba.len());

        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(BadIcon::PngEncodingError)?;
        writer
            .write_image_data(&rgba)
            .map_err(BadIcon::PngEncodingError)?;
        writer.finish().map_err(BadIcon::PngEncodingError)?;

        Ok(Self { png_data })
    }

    /// Creates a GTK `BytesIcon` from the stored PNG data.
    ///
    /// This should only be called on the GTK main thread.
    pub fn to_bytes_icon(&self) -> gtk::gio::BytesIcon {
        let bytes = gtk::glib::Bytes::from(&self.png_data);
        gtk::gio::BytesIcon::new(&bytes)
    }

    #[cfg(all(feature = "linux-ksni", target_os = "linux"))]
    pub(crate) fn png_data(&self) -> Vec<u8> {
        self.png_data.clone()
    }
}
