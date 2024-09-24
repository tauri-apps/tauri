// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Image types used by this crate and also referenced by the JavaScript API layer.

pub(crate) mod plugin;

use std::borrow::Cow;
use std::sync::Arc;

use crate::{Resource, ResourceId, ResourceTable};

/// An RGBA Image in row-major order from top to bottom.
#[derive(Debug, Clone)]
pub struct Image<'a> {
  rgba: Cow<'a, [u8]>,
  width: u32,
  height: u32,
}

impl Resource for Image<'static> {}

impl Image<'static> {
  /// Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height.
  ///
  /// Similar to [`Self::new`] but avoids cloning the rgba data to get an owned Image.
  pub const fn new_owned(rgba: Vec<u8>, width: u32, height: u32) -> Self {
    Self {
      rgba: Cow::Owned(rgba),
      width,
      height,
    }
  }
}

impl<'a> Image<'a> {
  /// Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height.
  pub const fn new(rgba: &'a [u8], width: u32, height: u32) -> Self {
    Self {
      rgba: Cow::Borrowed(rgba),
      width,
      height,
    }
  }

  /// Creates a new image using the provided bytes.
  ///
  /// Only `ico` and `png` are supported (based on activated feature flag).
  #[cfg(any(feature = "image-ico", feature = "image-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "image-ico", feature = "image-png"))))]
  pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
    use image::GenericImageView;

    let img = image::load_from_memory(bytes)?;
    let pixels = img
      .pixels()
      .flat_map(|(_, _, pixel)| pixel.0)
      .collect::<Vec<_>>();
    Ok(Self {
      rgba: Cow::Owned(pixels),
      width: img.width(),
      height: img.height(),
    })
  }

  /// Creates a new image using the provided path.
  ///
  /// Only `ico` and `png` are supported (based on activated feature flag).
  #[cfg(any(feature = "image-ico", feature = "image-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "image-ico", feature = "image-png"))))]
  pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Self> {
    let bytes = std::fs::read(path)?;
    Self::from_bytes(&bytes)
  }

  /// Returns the RGBA data for this image, in row-major order from top to bottom.
  pub fn rgba(&'a self) -> &'a [u8] {
    &self.rgba
  }

  /// Returns the width of this image.
  pub fn width(&self) -> u32 {
    self.width
  }

  /// Returns the height of this image.
  pub fn height(&self) -> u32 {
    self.height
  }

  /// Convert into a 'static owned [`Image`].
  /// This will allocate.
  pub fn to_owned(self) -> Image<'static> {
    Image {
      rgba: match self.rgba {
        Cow::Owned(v) => Cow::Owned(v),
        Cow::Borrowed(v) => Cow::Owned(v.to_vec()),
      },
      height: self.height,
      width: self.width,
    }
  }
}

impl<'a> From<Image<'a>> for crate::runtime::Icon<'a> {
  fn from(img: Image<'a>) -> Self {
    Self {
      rgba: img.rgba,
      width: img.width,
      height: img.height,
    }
  }
}

#[cfg(desktop)]
impl TryFrom<Image<'_>> for muda::Icon {
  type Error = crate::Error;

  fn try_from(img: Image<'_>) -> Result<Self, Self::Error> {
    muda::Icon::from_rgba(img.rgba.to_vec(), img.width, img.height).map_err(Into::into)
  }
}

#[cfg(all(desktop, feature = "tray-icon"))]
impl TryFrom<Image<'_>> for tray_icon::Icon {
  type Error = crate::Error;

  fn try_from(img: Image<'_>) -> Result<Self, Self::Error> {
    tray_icon::Icon::from_rgba(img.rgba.to_vec(), img.width, img.height).map_err(Into::into)
  }
}

/// An image type that accepts file paths, raw bytes, previously loaded images and image objects.
/// This type is meant to be used along the [transformImage](https://v2.tauri.app/reference/javascript/api/namespaceimage/#transformimage) API.
///
/// # Stability
///
/// The stability of the variants are not guaranteed, and matching against them is not recommended.
/// Use [`JsImage::into_img`] instead.
#[derive(serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum JsImage {
  /// A reference to a image in the filesystem.
  #[non_exhaustive]
  Path(std::path::PathBuf),
  /// Image from raw bytes.
  #[non_exhaustive]
  Bytes(Vec<u8>),
  /// An image that was previously loaded with the API and is stored in the resource table.
  #[non_exhaustive]
  Resource(ResourceId),
  /// Raw RGBA definition of an image.
  #[non_exhaustive]
  Rgba {
    /// Image bytes.
    rgba: Vec<u8>,
    /// Image width.
    width: u32,
    /// Image height.
    height: u32,
  },
}

impl JsImage {
  /// Converts this intermediate image format into an actual [`Image`].
  ///
  /// This will retrieve the image from the passed [`ResourceTable`] if it is [`JsImage::Resource`]
  /// and will return an error if it doesn't exist in the passed [`ResourceTable`] so make sure
  /// the passed [`ResourceTable`] is the same one used to store the image, usually this should be
  /// the webview [resources table](crate::webview::Webview::resources_table).
  pub fn into_img(self, resources_table: &ResourceTable) -> crate::Result<Arc<Image<'_>>> {
    match self {
      Self::Resource(rid) => resources_table.get::<Image<'static>>(rid),
      #[cfg(any(feature = "image-ico", feature = "image-png"))]
      Self::Path(path) => Image::from_path(path).map(Arc::new).map_err(Into::into),

      #[cfg(any(feature = "image-ico", feature = "image-png"))]
      Self::Bytes(bytes) => Image::from_bytes(&bytes).map(Arc::new).map_err(Into::into),

      Self::Rgba {
        rgba,
        width,
        height,
      } => Ok(Arc::new(Image::new_owned(rgba, width, height))),

      #[cfg(not(any(feature = "image-ico", feature = "image-png")))]
      _ => Err(
        std::io::Error::new(
          std::io::ErrorKind::InvalidInput,
          format!(
            "expected RGBA image data, found {}",
            match self {
              JsImage::Path(_) => "a file path",
              JsImage::Bytes(_) => "raw bytes",
              _ => unreachable!(),
            }
          ),
        )
        .into(),
      ),
    }
  }
}
