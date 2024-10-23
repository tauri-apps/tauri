// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Serialize;

use crate::plugin::{Builder, TauriPlugin};
use crate::Manager;
use crate::{command, image::Image, ResourceId, Runtime, Webview};

#[command(root = "crate")]
fn new<R: Runtime>(
  webview: Webview<R>,
  rgba: Vec<u8>,
  width: u32,
  height: u32,
) -> crate::Result<ResourceId> {
  let image = Image::new_owned(rgba, width, height);
  let mut resources_table = webview.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}

#[cfg(any(feature = "image-ico", feature = "image-png"))]
#[command(root = "crate")]
fn from_bytes<R: Runtime>(webview: Webview<R>, bytes: Vec<u8>) -> crate::Result<ResourceId> {
  let image = Image::from_bytes(&bytes)?.to_owned();
  let mut resources_table = webview.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}

#[cfg(not(any(feature = "image-ico", feature = "image-png")))]
#[command(root = "crate")]
fn from_bytes() -> std::result::Result<(), &'static str> {
  Err("from_bytes is only supported if the `image-ico` or `image-png` Cargo features are enabled")
}

#[cfg(any(feature = "image-ico", feature = "image-png"))]
#[command(root = "crate")]
fn from_path<R: Runtime>(
  webview: Webview<R>,
  path: std::path::PathBuf,
) -> crate::Result<ResourceId> {
  let image = Image::from_path(path)?.to_owned();
  let mut resources_table = webview.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}

#[cfg(not(any(feature = "image-ico", feature = "image-png")))]
#[command(root = "crate")]
fn from_path() -> std::result::Result<(), &'static str> {
  Err("from_path is only supported if the `image-ico` or `image-png` Cargo features are enabled")
}

#[command(root = "crate")]
fn rgba<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<Vec<u8>> {
  let resources_table = webview.resources_table();
  let image = resources_table.get::<Image<'_>>(rid)?;
  Ok(image.rgba().to_vec())
}

#[derive(Serialize)]
struct Size {
  width: u32,
  height: u32,
}

#[command(root = "crate")]
fn size<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<Size> {
  let resources_table = webview.resources_table();
  let image = resources_table.get::<Image<'_>>(rid)?;
  Ok(Size {
    width: image.width(),
    height: image.height(),
  })
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("image")
    .invoke_handler(crate::generate_handler![
      new, from_bytes, from_path, rgba, size
    ])
    .build()
}
