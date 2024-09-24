// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  embedded_assets::{EmbeddedAssetsError, EmbeddedAssetsResult},
  Cached,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::{ffi::OsStr, io::Cursor, path::Path};

/// The format the Icon is consumed as.
pub(crate) enum IconFormat {
  /// The image, completely unmodified.
  Raw,

  /// RGBA raw data, meant to be consumed by [`tauri::image::Image`].
  Image { width: u32, height: u32 },
}

pub struct CachedIcon {
  cache: Cached,
  format: IconFormat,
  root: TokenStream,
}

impl CachedIcon {
  pub fn new(root: &TokenStream, icon: &Path) -> EmbeddedAssetsResult<Self> {
    match icon.extension().map(OsStr::to_string_lossy).as_deref() {
      Some("png") => Self::new_png(root, icon),
      Some("ico") => Self::new_ico(root, icon),
      unknown => Err(EmbeddedAssetsError::InvalidImageExtension {
        extension: unknown.unwrap_or_default().into(),
        path: icon.to_path_buf(),
      }),
    }
  }

  /// Cache the icon without any manipulation.
  pub fn new_raw(root: &TokenStream, icon: &Path) -> EmbeddedAssetsResult<Self> {
    let buf = Self::open(icon);
    Cached::try_from(buf).map(|cache| Self {
      cache,
      root: root.clone(),
      format: IconFormat::Raw,
    })
  }

  /// Cache an ICO icon as RGBA data, see [`ImageFormat::Image`].
  pub fn new_ico(root: &TokenStream, icon: &Path) -> EmbeddedAssetsResult<Self> {
    let buf = Self::open(icon);

    let icon_dir = ico::IconDir::read(Cursor::new(&buf))
      .unwrap_or_else(|e| panic!("failed to parse icon {}: {}", icon.display(), e));

    let entry = &icon_dir.entries()[0];
    let rgba = entry
      .decode()
      .unwrap_or_else(|e| panic!("failed to decode icon {}: {}", icon.display(), e))
      .rgba_data()
      .to_vec();

    Cached::try_from(rgba).map(|cache| Self {
      cache,
      root: root.clone(),
      format: IconFormat::Image {
        width: entry.width(),
        height: entry.height(),
      },
    })
  }

  /// Cache a PNG icon as RGBA data, see [`ImageFormat::Image`].
  pub fn new_png(root: &TokenStream, icon: &Path) -> EmbeddedAssetsResult<Self> {
    let buf = Self::open(icon);
    let decoder = png::Decoder::new(Cursor::new(&buf));
    let mut reader = decoder
      .read_info()
      .unwrap_or_else(|e| panic!("failed to read icon {}: {}", icon.display(), e));

    if reader.output_color_type().0 != png::ColorType::Rgba {
      panic!("icon {} is not RGBA", icon.display());
    }

    let mut rgba = Vec::with_capacity(reader.output_buffer_size());
    while let Ok(Some(row)) = reader.next_row() {
      rgba.extend(row.data());
    }

    Cached::try_from(rgba).map(|cache| Self {
      cache,
      root: root.clone(),
      format: IconFormat::Image {
        width: reader.info().width,
        height: reader.info().height,
      },
    })
  }

  fn open(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("failed to open icon {}: {}", path.display(), e))
  }
}

impl ToTokens for CachedIcon {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let root = &self.root;
    let cache = &self.cache;
    let raw = quote!(::std::include_bytes!(#cache));
    tokens.append_all(match self.format {
      IconFormat::Raw => raw,
      IconFormat::Image { width, height } => {
        quote!(#root::image::Image::new(#raw, #width, #height))
      }
    })
  }
}
