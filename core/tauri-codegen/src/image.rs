// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::embedded_assets::{ensure_out_dir, EmbeddedAssetsError, EmbeddedAssetsResult};
use proc_macro2::TokenStream;
use quote::quote;
use std::{ffi::OsStr, io::Cursor, path::Path, path::PathBuf};

/// The format the Icon is consumed as.
pub(crate) enum IconFormat {
  /// The image, completely unmodified.
  Raw,

  /// RGBA raw data, meant to be consumed by [`tauri::image::Image`].
  Image { width: u32, height: u32 },
}

pub struct CachedIcon {
  /// Relative path from `$OUT_DIR` to the cached file.
  path: PathBuf,

  /// How the icon is meant to be consumed.
  format: IconFormat,
}

impl TryFrom<&PathBuf> for CachedIcon {
  type Error = EmbeddedAssetsError;

  /// Read and cache the file in `$OUT_DIR`.
  ///
  /// This only supports the [`IconFormat::Image`] format.
  fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
    Self::try_from(path.as_path())
  }
}

impl TryFrom<&Path> for CachedIcon {
  type Error = EmbeddedAssetsError;

  /// Read and cache the file in `$OUT_DIR`.
  ///
  /// This only supports the [`IconFormat::Image`] format.
  fn try_from(path: &Path) -> Result<Self, Self::Error> {
    match path.extension().map(OsStr::to_string_lossy).as_deref() {
      Some("png") => Self::try_from_png(path),
      Some("ico") => Self::try_from_ico(path),
      unknown => Err(EmbeddedAssetsError::InvalidImageExtension {
        extension: unknown.unwrap_or_default().into(),
        path: path.to_path_buf(),
      }),
    }
  }
}

impl CachedIcon {
  fn open(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("failed to open icon {}: {}", path.display(), e))
  }

  /// Cache the icon without any manipulation.
  pub fn try_from_raw(path: &Path) -> EmbeddedAssetsResult<Self> {
    let buf = Self::open(path);
    Self::cache(&buf).map(|path| Self {
      path,
      format: IconFormat::Raw,
    })
  }

  /// Cache an ICO icon as RGBA data, see [`ImageFormat::Image`].
  pub fn try_from_ico(path: &Path) -> EmbeddedAssetsResult<Self> {
    let buf = Self::open(path);

    let icon_dir = ico::IconDir::read(Cursor::new(&buf))
      .unwrap_or_else(|e| panic!("failed to parse icon {}: {}", path.display(), e));

    let entry = &icon_dir.entries()[0];
    let rgba = entry
      .decode()
      .unwrap_or_else(|e| panic!("failed to decode icon {}: {}", path.display(), e))
      .rgba_data()
      .to_vec();

    Self::cache(&rgba).map(|path| Self {
      path,
      format: IconFormat::Image {
        width: entry.width(),
        height: entry.height(),
      },
    })
  }

  /// Cache a PNG icon as RGBA data, see [`ImageFormat::Image`].
  pub fn try_from_png(path: &Path) -> EmbeddedAssetsResult<Self> {
    let buf = Self::open(path);
    let decoder = png::Decoder::new(Cursor::new(&buf));
    let mut reader = decoder
      .read_info()
      .unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e));

    if reader.output_color_type().0 != png::ColorType::Rgba {
      panic!("icon {} is not RGBA", path.display());
    }

    let mut rgba = Vec::with_capacity(reader.output_buffer_size());
    while let Ok(Some(row)) = reader.next_row() {
      rgba.extend(row.data());
    }

    Self::cache(&rgba).map(|path| Self {
      path,
      format: IconFormat::Image {
        width: reader.info().width,
        height: reader.info().height,
      },
    })
  }

  /// Cache the data to `$OUT_DIR`, only if it does not already exist.
  ///
  /// Due to using a checksum as the filename, an existing file should be the exact same content
  /// as the data being checked.
  fn cache(buf: &[u8]) -> EmbeddedAssetsResult<PathBuf> {
    let hash = crate::checksum(buf).map_err(EmbeddedAssetsError::Hex)?;
    let filename = PathBuf::from(hash);
    let path = ensure_out_dir()?.join(&filename);
    if let Ok(existing) = std::fs::read(&path) {
      if existing == buf {
        return Ok(filename);
      }
    }

    std::fs::write(&path, buf).map_err(|error| EmbeddedAssetsError::AssetWrite {
      path: path.to_owned(),
      error,
    })?;

    Ok(filename)
  }

  /// Generate the code to read the image from `$OUT_DIR`.
  pub fn codegen(&self, root: &TokenStream) -> TokenStream {
    let path = self.path.to_string_lossy();
    let raw = quote! {
      ::std::include_bytes!(::std::concat!(::std::env!("OUT_DIR"), "/", #path))
    };

    match self.format {
      IconFormat::Raw => raw,
      IconFormat::Image { width, height } => quote! {
        #root::image::Image::new(#raw, #width, #height)
      },
    }
  }
}
