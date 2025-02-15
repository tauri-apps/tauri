// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::Deserialize;

use crate::{
  command,
  image::JsImage,
  ipc::Channel,
  menu::{plugin::ItemKind, Menu, Submenu},
  plugin::{Builder, TauriPlugin},
  resources::ResourceId,
  tray::TrayIconBuilder,
  AppHandle, Manager, Runtime, Webview,
};

use super::{TrayIcon, TrayIconEvent};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrayIconOptions {
  id: Option<String>,
  menu: Option<(ResourceId, ItemKind)>,
  icon: Option<JsImage>,
  tooltip: Option<String>,
  title: Option<String>,
  temp_dir_path: Option<PathBuf>,
  icon_as_template: Option<bool>,
  menu_on_left_click: Option<bool>,
  show_menu_on_left_click: Option<bool>,
}

#[command(root = "crate")]
fn new<R: Runtime>(
  webview: Webview<R>,
  options: TrayIconOptions,
  handler: Channel<TrayIconEvent>,
) -> crate::Result<(ResourceId, String)> {
  let mut builder = if let Some(id) = options.id {
    TrayIconBuilder::<R>::with_id(id)
  } else {
    TrayIconBuilder::<R>::new()
  };

  builder = builder.on_tray_icon_event(move |_tray, e| {
    let _ = handler.send(e);
  });

  let mut resources_table = webview.resources_table();

  if let Some((rid, kind)) = options.menu {
    match kind {
      ItemKind::Menu => {
        let menu = resources_table.get::<Menu<R>>(rid)?;
        builder = builder.menu(&*menu);
      }
      ItemKind::Submenu => {
        let submenu = resources_table.get::<Submenu<R>>(rid)?;
        builder = builder.menu(&*submenu);
      }
      _ => return Err(anyhow::anyhow!("unexpected menu item kind").into()),
    };
  }
  if let Some(icon) = options.icon {
    builder = builder.icon(icon.into_img(&resources_table)?.as_ref().clone());
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
  #[allow(deprecated)]
  if let Some(menu_on_left_click) = options.menu_on_left_click {
    builder = builder.menu_on_left_click(menu_on_left_click);
  }
  if let Some(show_menu_on_left_click) = options.show_menu_on_left_click {
    builder = builder.show_menu_on_left_click(show_menu_on_left_click);
  }

  let tray = builder.build(&webview)?;
  let id = tray.id().as_ref().to_string();
  let rid = resources_table.add(tray);

  Ok((rid, id))
}

#[command(root = "crate")]
fn get_by_id<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  id: &str,
) -> crate::Result<Option<ResourceId>> {
  let tray = app.tray_by_id(id);
  let maybe_rid = tray.map(|tray| {
    let mut resources_table = webview.resources_table();
    resources_table.add(tray)
  });
  Ok(maybe_rid)
}

#[command(root = "crate")]
fn remove_by_id<R: Runtime>(app: AppHandle<R>, id: &str) -> crate::Result<()> {
  app
    .remove_tray_by_id(id)
    .ok_or_else(|| anyhow::anyhow!("Can't find a tray associated with this id: {id}"))
    .map(|_| ())
    .map_err(Into::into)
}

#[command(root = "crate")]
fn set_icon<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  icon: Option<JsImage>,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  let icon = match icon {
    Some(i) => Some(i.into_img(&resources_table)?.as_ref().clone()),
    None => None,
  };
  tray.set_icon(icon)
}

#[command(root = "crate")]
fn set_menu<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  menu: Option<(ResourceId, ItemKind)>,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  if let Some((rid, kind)) = menu {
    match kind {
      ItemKind::Menu => {
        let menu = resources_table.get::<Menu<R>>(rid)?;
        tray.set_menu(Some((*menu).clone()))?;
      }
      ItemKind::Submenu => {
        let submenu = resources_table.get::<Submenu<R>>(rid)?;
        tray.set_menu(Some((*submenu).clone()))?;
      }
      _ => return Err(anyhow::anyhow!("unexpected menu item kind").into()),
    };
  } else {
    tray.set_menu(None::<Menu<R>>)?;
  }
  Ok(())
}

#[command(root = "crate")]
fn set_tooltip<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  tooltip: Option<String>,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  tray.set_tooltip(tooltip)
}

#[command(root = "crate")]
fn set_title<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  title: Option<String>,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  tray.set_title(title)
}

#[command(root = "crate")]
fn set_visible<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  visible: bool,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  tray.set_visible(visible)
}

#[command(root = "crate")]
fn set_temp_dir_path<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  path: Option<PathBuf>,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  tray.set_temp_dir_path(path)
}

#[command(root = "crate")]
fn set_icon_as_template<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  as_template: bool,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  tray.set_icon_as_template(as_template)
}

#[command(root = "crate")]
fn set_show_menu_on_left_click<R: Runtime>(
  webview: Webview<R>,
  rid: ResourceId,
  on_left: bool,
) -> crate::Result<()> {
  let resources_table = webview.resources_table();
  let tray = resources_table.get::<TrayIcon<R>>(rid)?;
  tray.set_show_menu_on_left_click(on_left)
}

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("tray")
    .invoke_handler(crate::generate_handler![
      new,
      get_by_id,
      remove_by_id,
      set_icon,
      set_menu,
      set_tooltip,
      set_title,
      set_visible,
      set_temp_dir_path,
      set_icon_as_template,
      set_show_menu_on_left_click,
    ])
    .build()
}
