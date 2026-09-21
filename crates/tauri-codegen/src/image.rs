// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  Cached,
  embedded_assets::{EmbeddedAssetsError, EmbeddedAssetsResult},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};
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

    let entry = largest_ico_entry(&icon_dir)
      .unwrap_or_else(|| panic!("icon {} has no entries", icon.display()));
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

    let mut rgba = Vec::with_capacity(reader.output_buffer_size().unwrap());
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

/// Picks the entry of an ICO file to embed: the largest one (and, for equal sizes, the deepest).
///
/// ICO files conventionally store entries smallest-first, so taking the first entry would give
/// the OS a 16x16 image to upscale; the largest entry lets it downscale a high-resolution source instead.
fn largest_ico_entry(icon_dir: &ico::IconDir) -> Option<&ico::IconDirEntry> {
  icon_dir
    .entries()
    .iter()
    .max_by_key(|e| (e.width() * e.height(), e.bits_per_pixel()))
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

#[cfg(test)]
mod tests {
  use super::largest_ico_entry;
  use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
  use std::io::Cursor;

  fn image(size: u32, alpha: u8) -> IconImage {
    let rgba = [255, 0, 0, alpha].repeat((size * size) as usize);
    IconImage::from_rgba_data(size, size, rgba)
  }

  /// Builds an ICO with an opaque entry for each size, in the given order, and reads it back
  /// through the same parser `CachedIcon::new_ico` uses.
  fn icon_dir(sizes: &[u32]) -> IconDir {
    let mut dir = IconDir::new(ResourceType::Icon);
    for &size in sizes {
      let entry = if size == 256 {
        IconDirEntry::encode_as_png(&image(size, 255)).unwrap()
      } else {
        IconDirEntry::encode_as_bmp(&image(size, 255)).unwrap()
      };
      dir.add_entry(entry);
    }
    let mut buf = Vec::new();
    dir.write(&mut buf).unwrap();
    IconDir::read(Cursor::new(buf)).unwrap()
  }

  #[test]
  fn picks_largest_entry_regardless_of_order() {
    // the entry order `tauri icon` used to generate, with the PNG-compressed 256 entry last
    let dir = icon_dir(&[32, 16, 24, 48, 64, 256]);
    let entry = largest_ico_entry(&dir).unwrap();
    assert_eq!((entry.width(), entry.height()), (256, 256));
    assert!(entry.is_png());

    // largest first, in the middle and alone
    for sizes in [&[64, 48, 16][..], &[16, 64, 48], &[64]] {
      let dir = icon_dir(sizes);
      let entry = largest_ico_entry(&dir).unwrap();
      assert_eq!((entry.width(), entry.height()), (64, 64), "{sizes:?}");
    }
  }

  #[test]
  fn picks_deepest_entry_for_equal_sizes() {
    // a single opaque color encodes as a low-depth BMP, a translucent one needs 32bpp
    let shallow = IconDirEntry::encode_as_bmp(&image(32, 255)).unwrap();
    let deep = IconDirEntry::encode_as_bmp(&image(32, 128)).unwrap();
    assert!(shallow.bits_per_pixel() < deep.bits_per_pixel());
    let deep_bpp = deep.bits_per_pixel();

    for entries in [[shallow.clone(), deep.clone()], [deep, shallow]] {
      let mut dir = IconDir::new(ResourceType::Icon);
      for entry in entries {
        dir.add_entry(entry);
      }
      let entry = largest_ico_entry(&dir).unwrap();
      assert_eq!((entry.width(), entry.height()), (32, 32));
      assert_eq!(entry.bits_per_pixel(), deep_bpp);
    }
  }

  #[test]
  fn selected_entry_decodes_at_its_own_size() {
    let dir = icon_dir(&[16, 256]);
    let entry = largest_ico_entry(&dir).unwrap();
    let decoded = entry.decode().unwrap();
    assert_eq!((decoded.width(), decoded.height()), (256, 256));
    assert_eq!(decoded.rgba_data().len(), 256 * 256 * 4);
  }

  #[test]
  fn no_entries() {
    assert!(largest_ico_entry(&IconDir::new(ResourceType::Icon)).is_none());
  }
}
