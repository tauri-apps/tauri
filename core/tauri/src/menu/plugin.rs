// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use super::*;
use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  resources::ResourceId,
  AppHandle, IconDto, Manager, Runtime,
};

#[derive(Deserialize, Serialize)]
pub(crate) enum ItemKind {
  Menu,
  MenuItem,
  Predefined,
  Submenu,
  Check,
  Icon,
}

#[derive(Deserialize)]
pub(crate) struct AboutMetadata {
  pub name: Option<String>,
  pub version: Option<String>,
  pub short_version: Option<String>,
  pub authors: Option<Vec<String>>,
  pub comments: Option<String>,
  pub copyright: Option<String>,
  pub license: Option<String>,
  pub website: Option<String>,
  pub website_label: Option<String>,
  pub credits: Option<String>,
  pub icon: Option<IconDto>,
}

impl From<AboutMetadata> for super::AboutMetadata {
  fn from(value: AboutMetadata) -> Self {
    Self {
      name: value.name,
      version: value.version,
      short_version: value.short_version,
      authors: value.authors,
      comments: value.comments,
      copyright: value.copyright,
      license: value.license,
      website: value.website,
      website_label: value.website_label,
      credits: value.credits,
      icon: value.icon.map(Into::into),
    }
  }
}

#[derive(Deserialize)]
enum Predefined {
  Separator,
  Copy,
  Cut,
  Paste,
  SelectAll,
  Undo,
  Redo,
  Minimize,
  Maximize,
  Fullscreen,
  Hide,
  HideOthers,
  ShowAll,
  CloseWindow,
  Quit,
  About(Option<AboutMetadata>),
  Services,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct NewOptions {
  id: Option<MenuId>,
  text: Option<String>,
  enabled: Option<bool>,
  checked: Option<bool>,
  accelerator: Option<String>,
  #[serde(rename = "item")]
  predefined_item: Option<Predefined>,
  icon: Option<IconDto>,
  native_icon: Option<NativeIcon>,
  items: Option<Vec<(ResourceId, ItemKind)>>,
}

macro_rules! do_item {
  ($resources_table:ident, $rid:ident, $kind:ident, $ex:expr) => {
    match $kind {
      ItemKind::Submenu => {
        let item = $resources_table.get::<Submenu<R>>($rid)?;
        $ex(&*item)
      }
      ItemKind::MenuItem => {
        let item = $resources_table.get::<MenuItem<R>>($rid)?;
        $ex(&*item)
      }
      ItemKind::Predefined => {
        let item = $resources_table.get::<PredefinedMenuItem<R>>($rid)?;
        $ex(&*item)
      }
      ItemKind::Check => {
        let item = $resources_table.get::<CheckMenuItem<R>>($rid)?;
        $ex(&*item)
      }
      ItemKind::Icon => {
        let item = $resources_table.get::<IconMenuItem<R>>($rid)?;
        $ex(&*item)
      }
      _ => unreachable!(),
    }
  };
}

#[command(root = "crate")]
fn new<R: Runtime>(
  app: AppHandle<R>,
  kind: ItemKind,
  options: Option<NewOptions>,
) -> crate::Result<(ResourceId, MenuId)> {
  let options = options.unwrap_or_default();
  let mut resources_table = app.manager.resources_table();

  match kind {
    ItemKind::Menu => {
      let mut builder = MenuBuilder::new(&app);
      if let Some(id) = options.id {
        builder = builder.id(id);
      }
      if let Some(items) = options.items {
        for (rid, kind) in items {
          builder = do_item!(resources_table, rid, kind, |i| builder.item(i));
        }
      }
      let menu = builder.build()?;
      let id = menu.id().clone();
      let rid = resources_table.add(menu);
      Some((rid, id))
    }

    ItemKind::Submenu => {
      let mut builder = SubmenuBuilder::new(&app, options.text.unwrap_or_default());
      if let Some(id) = options.id {
        builder = builder.id(id);
      }
      if let Some(items) = options.items {
        for (rid, kind) in items {
          builder = do_item!(resources_table, rid, kind, |i| builder.item(i));
        }
      }

      let submenu = builder.build()?;
      let id = submenu.id().clone();
      let rid = resources_table.add(submenu);
      Some((rid, id))
    }

    ItemKind::MenuItem => {
      let mut builder = MenuItemBuilder::new(options.text.unwrap_or_default());
      if let Some(accelerator) = options.accelerator {
        builder = builder.accelerator(accelerator);
      }
      if let Some(enabled) = options.enabled {
        builder = builder.enabled(enabled);
      }
      let item = builder.build(&app);
      let id = item.id().clone();
      let rid = resources_table.add(item);
      Some((rid, id))
    }

    ItemKind::Predefined => {
      let item = match options.predefined_item.unwrap() {
        Predefined::Separator => PredefinedMenuItem::separator(&app),
        Predefined::Copy => PredefinedMenuItem::copy(&app, options.text.as_deref()),
        Predefined::Cut => PredefinedMenuItem::cut(&app, options.text.as_deref()),
        Predefined::Paste => PredefinedMenuItem::paste(&app, options.text.as_deref()),
        Predefined::SelectAll => PredefinedMenuItem::select_all(&app, options.text.as_deref()),
        Predefined::Undo => PredefinedMenuItem::undo(&app, options.text.as_deref()),
        Predefined::Redo => PredefinedMenuItem::redo(&app, options.text.as_deref()),
        Predefined::Minimize => PredefinedMenuItem::minimize(&app, options.text.as_deref()),
        Predefined::Maximize => PredefinedMenuItem::maximize(&app, options.text.as_deref()),
        Predefined::Fullscreen => PredefinedMenuItem::fullscreen(&app, options.text.as_deref()),
        Predefined::Hide => PredefinedMenuItem::hide(&app, options.text.as_deref()),
        Predefined::HideOthers => PredefinedMenuItem::hide_others(&app, options.text.as_deref()),
        Predefined::ShowAll => PredefinedMenuItem::show_all(&app, options.text.as_deref()),
        Predefined::CloseWindow => PredefinedMenuItem::close_window(&app, options.text.as_deref()),
        Predefined::Quit => PredefinedMenuItem::quit(&app, options.text.as_deref()),
        Predefined::About(metadata) => {
          PredefinedMenuItem::about(&app, options.text.as_deref(), metadata.map(Into::into))
        }
        Predefined::Services => PredefinedMenuItem::services(&app, options.text.as_deref()),
      };
      let id = item.id().clone();
      let rid = resources_table.add(item);
      Some((rid, id))
    }

    ItemKind::Check => {
      let mut builder = CheckMenuItemBuilder::new(options.text.unwrap_or_default());
      if let Some(accelerator) = options.accelerator {
        builder = builder.accelerator(accelerator);
      }
      if let Some(enabled) = options.enabled {
        builder = builder.enabled(enabled);
      }
      if let Some(checked) = options.checked {
        builder = builder.checked(checked);
      }
      let item = builder.build(&app);
      let id = item.id().clone();
      let rid = resources_table.add(item);
      Some((rid, id))
    }

    ItemKind::Icon => {
      let mut builder = IconMenuItemBuilder::new(options.text.unwrap_or_default());
      if let Some(accelerator) = options.accelerator {
        builder = builder.accelerator(accelerator);
      }
      if let Some(enabled) = options.enabled {
        builder = builder.enabled(enabled);
      }
      if let Some(native_icon) = options.native_icon {
        builder = builder.native_icon(native_icon.into());
      }
      if let Some(icon) = options.icon {
        builder = builder.icon(icon.into());
      }
      let item = builder.build(&app);
      let id = item.id().clone();
      let rid = resources_table.add(item);
      Some((rid, id))
    }
  };

  Ok((0, "".into()))
}

#[command(root = "crate")]
fn append<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  items: Vec<(ResourceId, ItemKind)>,
) -> crate::Result<()> {
  let resources_table = app.manager.resources_table();
  match kind {
    ItemKind::Menu => {
      let menu = resources_table.get::<Menu<R>>(rid)?;
      for (rid, kind) in items {
        do_item!(resources_table, rid, kind, |i| menu.append(i))?;
      }
    }
    ItemKind::Submenu => {
      let submenu = resources_table.get::<Submenu<R>>(rid)?;
      for (rid, kind) in items {
        do_item!(resources_table, rid, kind, |i| submenu.append(i))?;
      }
    }
    _ => unreachable!(),
  };

  Ok(())
}

#[command(root = "crate")]
fn prepend<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  items: Vec<(ResourceId, ItemKind)>,
) -> crate::Result<()> {
  let resources_table = app.manager.resources_table();
  match kind {
    ItemKind::Menu => {
      let menu = resources_table.get::<Menu<R>>(rid)?;
      for (rid, kind) in items {
        do_item!(resources_table, rid, kind, |i| menu.prepend(i))?;
      }
    }
    ItemKind::Submenu => {
      let submenu = resources_table.get::<Submenu<R>>(rid)?;
      for (rid, kind) in items {
        do_item!(resources_table, rid, kind, |i| submenu.prepend(i))?;
      }
    }
    _ => unreachable!(),
  };

  Ok(())
}

#[command(root = "crate")]
fn insert<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  items: Vec<(ResourceId, ItemKind)>,
  mut position: usize,
) -> crate::Result<()> {
  let resources_table = app.manager.resources_table();
  match kind {
    ItemKind::Menu => {
      let menu = resources_table.get::<Menu<R>>(rid)?;
      for (rid, kind) in items {
        do_item!(resources_table, rid, kind, |i| menu.insert(i, position))?;
        position += 1
      }
    }
    ItemKind::Submenu => {
      let submenu = resources_table.get::<Submenu<R>>(rid)?;
      for (rid, kind) in items {
        do_item!(resources_table, rid, kind, |i| submenu.insert(i, position))?;
        position += 1
      }
    }
    _ => unreachable!(),
  };

  Ok(())
}

#[command(root = "crate")]
fn remove<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  item: (ResourceId, ItemKind),
) -> crate::Result<()> {
  let (item_rid, item_kind) = item;
  let resources_table = app.manager.resources_table();
  match kind {
    ItemKind::Menu => {
      let menu = resources_table.get::<Menu<R>>(rid)?;
      do_item!(resources_table, item_rid, item_kind, |i| menu.remove(i))?;
    }
    ItemKind::Submenu => {
      let submenu = resources_table.get::<Submenu<R>>(rid)?;
      do_item!(resources_table, item_rid, item_kind, |i| submenu.remove(i))?;
    }
    _ => unreachable!(),
  };

  Ok(())
}

macro_rules! make_item_resource {
  ($resources_table:ident, $item:ident) => {{
    let id = $item.id().clone();
    let (rid, kind) = match $item {
      MenuItemKind::MenuItem(i) => ($resources_table.add(i), ItemKind::MenuItem),
      MenuItemKind::Submenu(i) => ($resources_table.add(i), ItemKind::Submenu),
      MenuItemKind::Predefined(i) => ($resources_table.add(i), ItemKind::Predefined),
      MenuItemKind::Check(i) => ($resources_table.add(i), ItemKind::Check),
      MenuItemKind::Icon(i) => ($resources_table.add(i), ItemKind::Icon),
    };
    (rid, id, kind)
  }};
}

#[command(root = "crate")]
fn remove_at<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  position: usize,
) -> crate::Result<Option<(ResourceId, MenuId, ItemKind)>> {
  let mut resources_table = app.manager.resources_table();
  match kind {
    ItemKind::Menu => {
      let menu = resources_table.get::<Menu<R>>(rid)?;
      if let Some(item) = menu.remove_at(position)? {
        return Ok(Some(make_item_resource!(resources_table, item)));
      }
    }
    ItemKind::Submenu => {
      let submenu = resources_table.get::<Submenu<R>>(rid)?;
      if let Some(item) = submenu.remove_at(position)? {
        return Ok(Some(make_item_resource!(resources_table, item)));
      }
    }
    _ => unreachable!(),
  };

  Ok(None)
}

#[command(root = "crate")]
fn items<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
) -> crate::Result<Vec<(ResourceId, MenuId, ItemKind)>> {
  let mut resources_table = app.manager.resources_table();
  let items = match kind {
    ItemKind::Menu => resources_table.get::<Menu<R>>(rid)?.items()?,
    ItemKind::Submenu => resources_table.get::<Submenu<R>>(rid)?.items()?,
    _ => unreachable!(),
  };

  Ok(
    items
      .into_iter()
      .map(|i| make_item_resource!(resources_table, i))
      .collect::<Vec<_>>(),
  )
}

#[command(root = "crate")]
fn get<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  id: MenuId,
) -> crate::Result<Option<(ResourceId, MenuId, ItemKind)>> {
  let mut resources_table = app.manager.resources_table();
  match kind {
    ItemKind::Menu => {
      let menu = resources_table.get::<Menu<R>>(rid)?;
      if let Some(item) = menu.get(&id) {
        return Ok(Some(make_item_resource!(resources_table, item)));
      }
    }
    ItemKind::Submenu => {
      let submenu = resources_table.get::<Submenu<R>>(rid)?;
      if let Some(item) = submenu.get(&id) {
        return Ok(Some(make_item_resource!(resources_table, item)));
      }
    }
    _ => unreachable!(),
  };

  Ok(None)
}

#[command(root = "crate")]
fn popup<R: Runtime>(
  app: AppHandle<R>,
  rid: ResourceId,
  kind: ItemKind,
  window: String,
  position: (u32, u32),
) -> crate::Result<()> {
  if let Some(window) = app.get_window(&window) {
    let position = crate::Position::Logical(position.into());
    let resources_table = app.manager.resources_table();
    match kind {
      ItemKind::Menu => {
        let menu = resources_table.get::<Menu<R>>(rid)?;
        menu.popup_at(window, position)?;
      }
      ItemKind::Submenu => {
        let submenu = resources_table.get::<Submenu<R>>(rid)?;
        submenu.popup_at(window, position)?;
      }
      _ => unreachable!(),
    };
  }

  Ok(())
}

#[command(root = "crate")]
fn default<R: Runtime>(app: AppHandle<R>) -> crate::Result<(ResourceId, MenuId)> {
  let mut resources_table = app.manager.resources_table();
  let menu = Menu::default(&app)?;
  let id = menu.id().clone();
  let rid = resources_table.add(menu);
  Ok((rid, id))
}

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("menu")
    .invoke_handler(crate::generate_handler![
      new, append, prepend, insert, remove, remove_at, items, get, popup, default
    ])
    .build()
}
