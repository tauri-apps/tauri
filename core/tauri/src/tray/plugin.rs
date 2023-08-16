// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{path::PathBuf, sync::Arc};

use serde::Deserialize;

use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  resources::ResourceId,
  tray::TrayIconBuilder,
  AppHandle, Icon, Manager, Runtime,
};

use super::TrayIcon;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum IconDto {
  #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
  File(std::path::PathBuf),
  #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
  Raw(Vec<u8>),
  Rgba {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
  },
}

impl From<IconDto> for Icon {
  fn from(icon: IconDto) -> Self {
    match icon {
      #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
      IconDto::File(path) => Self::File(path),
      #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
      IconDto::Raw(raw) => Self::Raw(raw),
      IconDto::Rgba {
        rgba,
        width,
        height,
      } => Self::Rgba {
        rgba,
        width,
        height,
      },
    }
  }
}

#[derive(Deserialize)]
struct TrayIconOptions {
  id: Option<String>,
  menu: Option<ResourceId>,
  icon: Option<IconDto>,
  tooltip: Option<String>,
  title: Option<String>,
  temp_dir_path: Option<PathBuf>,
  icon_as_template: Option<bool>,
  menu_on_left_click: Option<bool>,
}

#[command(root = "crate")]
fn new<R: Runtime>(
  app: AppHandle<R>,
  options: TrayIconOptions,
) -> crate::Result<(ResourceId, String)> {
  let mut builder = if let Some(id) = options.id {
    TrayIconBuilder::<R>::with_id(id)
  } else {
    TrayIconBuilder::<R>::new()
  };

  builder = builder.on_tray_event(|tray, e| {
    let _ = tray.app_handle().emit_all("tauri://tray", e);
  });

  if let Some(_menu) = options.menu {
    // builder = builder.menu(menu.into()); TODO
  }
  if let Some(icon) = options.icon {
    builder = builder.icon(icon.into());
  }
  if let Some(tooltip) = options.tooltip {
    builder = builder.tooltip(tooltip);
  }
  if let Some(title) = options.title {
    builder = builder.title(title);
  }
  if let Some(temp_dir_path) = options.temp_dir_path {
    builder = builder.temp_dir_path(temp_dir_path);
  }
  if let Some(icon_as_template) = options.icon_as_template {
    builder = builder.icon_as_template(icon_as_template);
  }
  if let Some(menu_on_left_click) = options.menu_on_left_click {
    builder = builder.menu_on_left_click(menu_on_left_click);
  }

  let tray = builder.build(&app)?;
  let id = tray.id().as_ref().to_string();
  let mut resources_table = app.manager.resources_table();
  let rid = resources_table.add(tray);

  Ok((rid, id))
}

fn with_tray<R: Runtime, T, F: FnOnce(Arc<TrayIcon<R>>) -> crate::Result<T>>(
  app: &AppHandle<R>,
  rid: ResourceId,
  f: F,
) -> crate::Result<T> {
  let resources_table = app.manager.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  f(tray)
}

#[command(root = "crate")]
fn set_icon<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  icon: Option<IconDto>,
) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_icon(icon.map(Into::into)))
}

#[command(root = "crate")]
fn set_menu<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  menu: Option<ResourceId>,
) -> crate::Result<()> {
  // TODO
  Ok(())
}

#[command(root = "crate")]
fn set_tooltip<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  tooltip: Option<String>,
) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_tooltip(tooltip))
}

#[command(root = "crate")]
fn set_title<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  title: Option<String>,
) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_title(title))
}

#[command(root = "crate")]
fn set_visible<R: Runtime>(app: AppHandle<R>, rid: ResourceId, visible: bool) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_visible(visible))
}

#[command(root = "crate")]
fn set_temp_dir_path<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  path: Option<PathBuf>,
) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_temp_dir_path(path))
}

#[command(root = "crate")]
fn set_icon_as_template<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  as_template: bool,
) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_icon_as_template(as_template))
}

#[command(root = "crate")]
fn set_show_menu_on_left_click<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  on_left: bool,
) -> crate::Result<()> {
  with_tray(&app, rid, |tray| tray.set_show_menu_on_left_click(on_left))
}

#[command(root = "crate")]
fn destroy<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> crate::Result<()> {
  app.manager.resources_table().close(rid)
}

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("tray")
    .invoke_handler(crate::generate_handler![
      new,
      set_icon,
      set_menu,
      set_tooltip,
      set_title,
      set_visible,
      set_temp_dir_path,
      set_icon_as_template,
      set_show_menu_on_left_click,
      destroy
    ])
    .build()
}
