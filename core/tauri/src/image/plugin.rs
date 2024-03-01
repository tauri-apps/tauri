// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::plugin::{Builder, TauriPlugin};
use crate::{command, AppHandle, Image, Manager, ResourceId, Runtime};

use super::JsImage;

#[command(root = "crate")]
fn new<R: Runtime>(app: AppHandle<R>, image: JsImage<'_>) -> crate::Result<ResourceId> {
  let image: Image<'_> = image.try_into()?;
  let image = image.to_owned();
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}
#[command(root = "crate")]
fn from_png_bytes<R: Runtime>(app: AppHandle<R>, image: JsImage<'_>) -> crate::Result<ResourceId> {
  let image: Image<'_> = image.try_into()?;
  let image = image.to_owned();
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}
#[command(root = "crate")]
fn from_ico_bytes<R: Runtime>(app: AppHandle<R>, image: JsImage<'_>) -> crate::Result<ResourceId> {
  let image: Image<'_> = image.try_into()?;
  let image = image.to_owned();
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}
#[command(root = "crate")]
fn from_bytes<R: Runtime>(app: AppHandle<R>, image: JsImage<'_>) -> crate::Result<ResourceId> {
  let image: Image<'_> = image.try_into()?;
  let image = image.to_owned();
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(image);
  Ok(rid)
}
#[command(root = "crate")]
fn from_path<R: Runtime>(app: AppHandle<R>, image: JsImage<'_>) -> crate::Result<ResourceId> {
  let image: Image<'_> = image.try_into()?;
  let image = image.to_owned();
  let mut resources_table = app.resources_table();
  let rid = resources_table.add(image);
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
      new,
      from_png_bytes,
      from_ico_bytes,
      from_bytes,
      from_path,
      rgba,
      width,
      height
    ])
    .build()
}
