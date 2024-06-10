// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::embedded_assets::{ensure_out_dir, EmbeddedAssetsError, EmbeddedAssetsResult};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::path::Path;
use syn::{punctuated::Punctuated, Ident, PathArguments, PathSegment, Token};

pub fn include_image_codegen(path: &Path) -> EmbeddedAssetsResult<TokenStream> {
  let out_dir = ensure_out_dir()?;

  let mut segments = Punctuated::new();
  segments.push(PathSegment {
    ident: Ident::new("tauri", Span::call_site()),
    arguments: PathArguments::None,
  });
  let root = syn::Path {
    leading_colon: Some(Token![::](Span::call_site())),
    segments,
  };

  image_icon(&root.to_token_stream(), &out_dir, path)
}

pub(crate) fn image_icon(
  root: &TokenStream,
  out_dir: &Path,
  path: &Path,
) -> EmbeddedAssetsResult<TokenStream> {
  let extension = path.extension().unwrap_or_default();
  if extension == "ico" {
    ico_icon(root, out_dir, path)
  } else if extension == "png" {
    png_icon(root, out_dir, path)
  } else {
    Err(EmbeddedAssetsError::InvalidImageExtension {
      extension: extension.into(),
      path: path.to_path_buf(),
    })
  }
}

pub(crate) fn raw_icon(out_dir: &Path, path: &Path) -> EmbeddedAssetsResult<TokenStream> {
  let bytes =
    std::fs::read(path).unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e));

  let out_path = out_dir.join(path.file_name().unwrap());
  write_if_changed(&out_path, &bytes).map_err(|error| EmbeddedAssetsError::AssetWrite {
    path: path.to_owned(),
    error,
  })?;

  let icon_path = path.file_name().unwrap().to_str().unwrap().to_string();
  let icon = quote!(::std::option::Option::Some(
    include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_path)).to_vec()
  ));
  Ok(icon)
}

pub(crate) fn ico_icon(
  root: &TokenStream,
  out_dir: &Path,
  path: &Path,
) -> EmbeddedAssetsResult<TokenStream> {
  let file = std::fs::File::open(path)
    .unwrap_or_else(|e| panic!("failed to open icon {}: {}", path.display(), e));
  let icon_dir = ico::IconDir::read(file)
    .unwrap_or_else(|e| panic!("failed to parse icon {}: {}", path.display(), e));
  let entry = &icon_dir.entries()[0];
  let rgba = entry
    .decode()
    .unwrap_or_else(|e| panic!("failed to decode icon {}: {}", path.display(), e))
    .rgba_data()
    .to_vec();
  let width = entry.width();
  let height = entry.height();

  let icon_file_name = path.file_name().unwrap();
  let out_path = out_dir.join(icon_file_name);
  write_if_changed(&out_path, &rgba).map_err(|error| EmbeddedAssetsError::AssetWrite {
    path: path.to_owned(),
    error,
  })?;

  let icon_file_name = icon_file_name.to_str().unwrap();
  let icon = quote!(#root::image::Image::new(include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_file_name)), #width, #height));
  Ok(icon)
}

pub(crate) fn png_icon(
  root: &TokenStream,
  out_dir: &Path,
  path: &Path,
) -> EmbeddedAssetsResult<TokenStream> {
  let file = std::fs::File::open(path)
    .unwrap_or_else(|e| panic!("failed to open icon {}: {}", path.display(), e));
  let decoder = png::Decoder::new(file);
  let mut reader = decoder
    .read_info()
    .unwrap_or_else(|e| panic!("failed to read icon {}: {}", path.display(), e));

  let (color_type, _) = reader.output_color_type();

  if color_type != png::ColorType::Rgba {
    panic!("icon {} is not RGBA", path.display());
  }

  let mut buffer: Vec<u8> = Vec::new();
  while let Ok(Some(row)) = reader.next_row() {
    buffer.extend(row.data());
  }
  let width = reader.info().width;
  let height = reader.info().height;

  let icon_file_name = path.file_name().unwrap();
  let out_path = out_dir.join(icon_file_name);
  write_if_changed(&out_path, &buffer).map_err(|error| EmbeddedAssetsError::AssetWrite {
    path: path.to_owned(),
    error,
  })?;

  let icon_file_name = icon_file_name.to_str().unwrap();
  let icon = quote!(#root::image::Image::new(include_bytes!(concat!(std::env!("OUT_DIR"), "/", #icon_file_name)), #width, #height));
  Ok(icon)
}

pub(crate) fn write_if_changed(out_path: &Path, data: &[u8]) -> std::io::Result<()> {
  if let Ok(curr) = std::fs::read(out_path) {
    if curr == data {
      return Ok(());
    }
  }
  std::fs::write(out_path, data)
}
