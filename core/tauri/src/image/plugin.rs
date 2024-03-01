// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::plugin::{Builder, TauriPlugin};
use crate::{command, AppHandle, Image, Manager, ResourceId, Runtime};

use std::sync::Arc;

use super::JsImage;

#[command(root = "crate")]
fn new<R: Runtime>(
  app: AppHandle<R>,
  rgba: Vec<u8>,
  width: u32,
  height: u32,
) -> crate::Result<ResourceId> {
  let image = JsImage::Rgba {
    rgba: &rgba,
    width,
    height,
  }
  .into_img(&app)?;
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(Arc::try_unwrap(image).unwrap().to_owned());
  Ok(rid)
}

#[command(root = "crate")]
fn from_bytes<R: Runtime>(app: AppHandle<R>, bytes: Vec<u8>) -> crate::Result<ResourceId> {
  let image = JsImage::Bytes(&bytes).into_img(&app)?;
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(Arc::try_unwrap(image).unwrap().to_owned());
  Ok(rid)
}

#[command(root = "crate")]
fn from_path<R: Runtime>(app: AppHandle<R>, path: std::path::PathBuf) -> crate::Result<ResourceId> {
  let image = JsImage::Path(path).into_img(&app)?.to_owned();
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(Arc::try_unwrap(image).unwrap());
  Ok(rid)
}

#[command(root = "crate")]
fn rgba<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> crate::Result<Vec<u8>> {
  let resources_table = app.resources_table();
  let image = resources_table.get::<Image<'_>>(rid)?;
  Ok(image.rgba().to_vec())
}

#[command(root = "crate")]
fn width<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> crate::Result<u32> {
  let resources_table = app.resources_table();
  let image = resources_table.get::<Image<'_>>(rid)?;
  Ok(image.width())
}

#[command(root = "crate")]
fn height<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> crate::Result<u32> {
  let resources_table = app.resources_table();
  let image = resources_table.get::<Image<'_>>(rid)?;
  Ok(image.height())
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("image")
    .invoke_handler(crate::generate_handler![
      new, from_bytes, from_path, rgba, width, height
    ])
    .build()
}
