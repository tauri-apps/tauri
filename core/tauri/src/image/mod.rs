// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod plugin;

use std::borrow::Cow;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::{Manager, Resource, ResourceId, Runtime};

/// An RGBA Image in row-major order from top to bottom.
#[derive(Debug, Clone)]
pub struct Image<'a> {
  rgba: Cow<'a, [u8]>,
  width: u32,
  height: u32,
}

impl Resource for Image<'static> {}

impl<'a> Image<'a> {
  /// Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height.
  pub const fn new(rgba: &'a [u8], width: u32, height: u32) -> Self {
    Self {
      rgba: Cow::Borrowed(rgba),
      width,
      height,
    }
  }

  /// Creates a new image using the provided png bytes.
  #[cfg(feature = "image-png")]
  #[cfg_attr(docsrs, doc(cfg(feature = "image-png")))]
  pub fn from_png_bytes(bytes: &[u8]) -> std::io::Result<Self> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info()?;
    let mut buffer = Vec::new();
    while let Ok(Some(row)) = reader.next_row() {
      buffer.extend(row.data());
    }
    Ok(Self {
      rgba: Cow::Owned(buffer),
      width: reader.info().width,
      height: reader.info().height,
    })
  }

  /// Creates a new image using the provided ico bytes.
  #[cfg(feature = "image-ico")]
  #[cfg_attr(docsrs, doc(cfg(feature = "image-ico")))]
  pub fn from_ico_bytes(bytes: &[u8]) -> std::io::Result<Self> {
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(&bytes))?;
    let first = icon_dir.entries().first().ok_or_else(|| {
      Error::new(
        ErrorKind::NotFound,
        "Couldn't find any icons inside provided ico bytes",
      )
    })?;

    let rgba = first.decode()?.rgba_data().to_vec();

    Ok(Self {
      rgba: Cow::Owned(rgba),
      width: first.width(),
      height: first.height(),
    })
  }

  /// Creates a new image using the provided bytes.
  ///
  /// Only `ico` and `png` are supported (based on activated feature flag).
  #[cfg(any(feature = "image-ico", feature = "image-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "image-ico", feature = "image-png"))))]
  pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
    let extension = infer::get(bytes)
      .expect("could not determine icon extension")
      .extension();

    match extension {
      #[cfg(feature = "image-ico")]
      "ico" => Self::from_ico_bytes(bytes),
      #[cfg(feature = "image-png")]
      "png" => Self::from_png_bytes(bytes),
      _ => {
        let supported = [
          #[cfg(feature = "image-png")]
          "'png'",
          #[cfg(feature = "image-ico")]
          "'ico'",
        ];

        Err(Error::new(
          ErrorKind::InvalidInput,
          format!(
            "Unexpected image format, expected {}, found '{extension}'. Please check the `image-*` Cargo features on the tauri crate to see if Tauri has optional support for this format.",
            if supported.is_empty() {
              "''".to_string()
            } else {
              supported.join(" or ")
            }
          ),
        ))
      }
    }
  }

  /// Creates a new image using the provided path.
  ///
  /// Only `ico` and `png` are supported (based on activated feature flag).
  #[cfg(any(feature = "image-ico", feature = "image-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "image-ico", feature = "image-png"))))]
  pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
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

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum JsImage<'a> {
  Path(std::path::PathBuf),
  Bytes(&'a [u8]),
  Resource(ResourceId),
  Rgba {
    rgba: &'a [u8],
    width: u32,
    height: u32,
  },
}

impl<'a> JsImage<'a> {
  pub fn into_img<R: Runtime, M: Manager<R>>(self, app: &M) -> crate::Result<Arc<Image<'a>>> {
    match self {
      Self::Resource(rid) => {
        let resources_table = app.resources_table();
        resources_table.get::<Image<'static>>(rid)
      }
      #[cfg(any(feature = "image-ico", feature = "image-png"))]
      Self::Path(path) => Image::from_path(path).map(Arc::new).map_err(Into::into),

      #[cfg(any(feature = "image-ico", feature = "image-png"))]
      Self::Bytes(bytes) => Image::from_bytes(bytes).map(Arc::new).map_err(Into::into),

      Self::Rgba {
        rgba,
        width,
        height,
      } => Ok(Arc::new(Image::new(rgba, width, height))),

      #[cfg(not(any(feature = "image-ico", feature = "image-png")))]
      _ => Err(
        Error::new(
          ErrorKind::InvalidInput,
          format!(
            "expected RGBA image data, found {}",
            match img {
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
