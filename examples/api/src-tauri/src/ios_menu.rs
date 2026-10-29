// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
  AppHandle, Emitter, Runtime,
  menu::{CheckMenuItemBuilder, IconMenuItemBuilder, Menu, MenuItem, NativeIcon, SubmenuBuilder},
};

/// Fills the iPadOS menu bar, starting from the default menu.
pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
  let menu = Menu::default(app)?;

  // add our commands to the File menu
  let file_menu = menu
    .items()?
    .into_iter()
    .filter_map(|item| item.as_submenu().cloned())
    .find(|submenu| submenu.text().is_ok_and(|text| text == "File"));
  if let Some(file_menu) = file_menu {
    file_menu.insert_items(
      &[
        &MenuItem::with_id(app, "import", "Import…", true, Some("CmdOrCtrl+I"))?,
        &MenuItem::with_id(app, "export", "Export…", true, Some("CmdOrCtrl+Shift+E"))?,
      ],
      0,
    )?;
  }

  let dashboard_menu = SubmenuBuilder::new(app, "Dashboard")
    .text("rename", "Rename")
    .item(
      &IconMenuItemBuilder::with_id("duplicate", "Duplicate")
        .native_icon(NativeIcon::MultipleDocuments)
        .build(app)?,
    )
    .separator()
    .item(
      &CheckMenuItemBuilder::with_id("grid", "Show Grid")
        .checked(true)
        .accelerator("CmdOrCtrl+G")
        .build(app)?,
    )
    .item(
      &SubmenuBuilder::new(app, "Sort By")
        .text("sort-name", "Name")
        .text("sort-date", "Date")
        .build()?,
    )
    .separator()
    .item(
      &IconMenuItemBuilder::with_id("delete", "Delete")
        .native_icon(NativeIcon::TrashEmpty)
        .enabled(false)
        .build(app)?,
    )
    .build()?;
  menu.insert(&dashboard_menu, menu.items()?.len().saturating_sub(2))?;

  app.set_menu(menu)?;
  app.on_menu_event(|app, event| {
    log::info!("menu event: {:?}", event.id());
    let _ = app.emit("menu-event", event.id().0.clone());
  });

  Ok(())
}
