// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Finder layout metadata for DMGs built on non-macOS hosts.
//!
//! The macOS path lets Finder write the `.DS_Store` file after mounting the
//! image and arranging icons through AppleScript. Neither exists on Linux, so
//! this module writes the same file directly: the empty store shipped with
//! `node-ds-store` is used as the allocator/B-tree template, and a single leaf
//! page with the window and icon records is placed after it.

use std::path::{Path, PathBuf};

use crate::bundle::settings::Position;

/// Empty `.DS_Store` produced by Finder, used for the allocator metadata.
///
/// Copied from `node-ds-store`'s `assets/DSStore-clean`, which is also the
/// template `appdmg` (and therefore Electron) uses. `node-ds-store` is MIT
/// licensed, copyright (c) 2014 Linus Unnebäck.
const TEMPLATE: &[u8] = include_bytes!("DSStore-clean");

/// Offset of the leaf page contents. `DSStore-clean` stores it at buddy offset
/// `0x1000`, and the buddy region starts four bytes into the file.
const LEAF_PAGE_OFFSET: usize = 0x1004;
/// Offset of the record count in the B-tree master block (buddy offset `0x40`).
const RECORD_COUNT_OFFSET: usize = 0x4c;
const PAGE_SIZE: usize = 4096;
#[cfg(test)]
const PAGE_HEADER_LEN: usize = 8;

/// Finder metadata for one directory.
pub struct Layout<'a> {
  pub app_name: &'a str,
  pub window_size: (u32, u32),
  pub window_position: Option<Position>,
  pub app_position: Position,
  pub application_folder_position: Position,
}

/// Serialize the Finder records of a freshly created DMG volume.
pub fn build(layout: &Layout<'_>) -> crate::Result<Vec<u8>> {
  let mut records = vec![
    Entry::new(
      ".",
      "bwsp",
      Value::BinaryPlist(browser_window_settings(layout)),
    ),
    Entry::new(".", "icvp", Value::BinaryPlist(icon_view_options())),
    Entry::new(".", "vSrn", Value::Long(1)),
  ];

  records.push(Entry::new(
    "Applications",
    "Iloc",
    Value::IconLocation(layout.application_folder_position.clone()),
  ));
  records.push(Entry::new(
    layout.app_name,
    "Iloc",
    Value::IconLocation(layout.app_position.clone()),
  ));

  // Finder's B-tree orders records by case-insensitive name, then field.
  records.sort_by(|left, right| {
    let left_name = left.name.to_lowercase();
    let right_name = right.name.to_lowercase();
    left_name
      .cmp(&right_name)
      .then_with(|| left.field.cmp(right.field))
  });

  let mut page = Vec::with_capacity(PAGE_SIZE);
  page.extend_from_slice(&0u32.to_be_bytes());
  page.extend_from_slice(&(records.len() as u32).to_be_bytes());
  for record in &records {
    record.encode(&mut page)?;
  }

  if page.len() > PAGE_SIZE {
    return Err(crate::Error::GenericError(
      "Finder layout records do not fit into a .DS_Store page".into(),
    ));
  }

  let mut store = TEMPLATE.to_vec();
  store[RECORD_COUNT_OFFSET..RECORD_COUNT_OFFSET + 4]
    .copy_from_slice(&(records.len() as u32).to_be_bytes());
  store[LEAF_PAGE_OFFSET..LEAF_PAGE_OFFSET + page.len()].copy_from_slice(&page);
  Ok(store)
}

/// Write the `.DS_Store` file into the DMG staging directory.
pub fn write(directory: &Path, layout: &Layout<'_>) -> crate::Result<PathBuf> {
  let path = directory.join(".DS_Store");
  std::fs::write(&path, build(layout)?).map_err(|error| {
    crate::Error::GenericError(format!("failed to write {}: {error}", path.display()))
  })?;
  Ok(path)
}

fn browser_window_settings(layout: &Layout<'_>) -> plist::Value {
  let (width, height) = layout.window_size;
  let origin = layout
    .window_position
    .as_ref()
    .map(|position| (position.x, position.y))
    // Matches the top-left corner `bundle_dmg.sh` uses when no position is set.
    .unwrap_or((10, 60));

  let mut settings = plist::Dictionary::new();
  settings.insert("ContainerShowSidebar".into(), plist::Value::Boolean(false));
  settings.insert("ShowPathbar".into(), plist::Value::Boolean(false));
  settings.insert("ShowSidebar".into(), plist::Value::Boolean(false));
  settings.insert("ShowStatusBar".into(), plist::Value::Boolean(false));
  settings.insert("ShowTabView".into(), plist::Value::Boolean(false));
  settings.insert("ShowToolbar".into(), plist::Value::Boolean(false));
  settings.insert("SidebarWidth".into(), plist::Value::Real(0.0));
  settings.insert(
    "WindowBounds".into(),
    plist::Value::String(format!(
      "{{{{{}, {}}}, {{{width}, {height}}}}}",
      origin.0, origin.1
    )),
  );
  plist::Value::Dictionary(settings)
}

fn icon_view_options() -> plist::Value {
  let mut options = plist::Dictionary::new();
  options.insert("arrangeBy".into(), plist::Value::String("none".into()));
  // Finder's default background. A background image would need an HFS+ alias.
  options.insert("backgroundType".into(), plist::Value::Integer(0.into()));
  options.insert("backgroundColorRed".into(), plist::Value::Real(1.0));
  options.insert("backgroundColorGreen".into(), plist::Value::Real(1.0));
  options.insert("backgroundColorBlue".into(), plist::Value::Real(1.0));
  options.insert("gridOffsetX".into(), plist::Value::Real(0.0));
  options.insert("gridOffsetY".into(), plist::Value::Real(0.0));
  options.insert("gridSpacing".into(), plist::Value::Real(100.0));
  options.insert("iconSize".into(), plist::Value::Real(128.0));
  options.insert("labelOnBottom".into(), plist::Value::Boolean(true));
  options.insert("showIconPreview".into(), plist::Value::Boolean(true));
  options.insert("showItemInfo".into(), plist::Value::Boolean(false));
  options.insert("textSize".into(), plist::Value::Real(16.0));
  options.insert("viewOptionsVersion".into(), plist::Value::Integer(1.into()));
  plist::Value::Dictionary(options)
}

struct Entry {
  name: String,
  field: &'static str,
  value: Value,
}

enum Value {
  Long(u32),
  IconLocation(Position),
  BinaryPlist(plist::Value),
}

impl Entry {
  fn new(name: impl Into<String>, field: &'static str, value: Value) -> Self {
    Entry {
      name: name.into(),
      field,
      value,
    }
  }

  fn encode(&self, out: &mut Vec<u8>) -> crate::Result<()> {
    let name_len = u32::try_from(self.name.encode_utf16().count()).map_err(|_| {
      crate::Error::GenericError(format!("Finder record name is too long: {}", self.name))
    })?;
    out.extend_from_slice(&name_len.to_be_bytes());
    for unit in self.name.encode_utf16() {
      out.extend_from_slice(&unit.to_be_bytes());
    }
    out.extend_from_slice(self.field.as_bytes());

    match &self.value {
      Value::Long(value) => {
        out.extend_from_slice(b"long");
        out.extend_from_slice(&value.to_be_bytes());
      }
      Value::IconLocation(position) => {
        out.extend_from_slice(b"blob");
        out.extend_from_slice(&16u32.to_be_bytes());
        out.extend_from_slice(&position.x.to_be_bytes());
        out.extend_from_slice(&position.y.to_be_bytes());
        out.extend_from_slice(&0xffff_ffffu32.to_be_bytes());
        out.extend_from_slice(&0xffff_0000u32.to_be_bytes());
      }
      Value::BinaryPlist(value) => {
        let mut bytes = Vec::new();
        value.to_writer_binary(&mut bytes).map_err(|error| {
          crate::Error::GenericError(format!("failed to encode Finder plist: {error}"))
        })?;
        let len = u32::try_from(bytes.len())
          .map_err(|_| crate::Error::GenericError("Finder plist is too large".into()))?;
        out.extend_from_slice(b"blob");
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&bytes);
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn record_names(store: &[u8]) -> Vec<(String, String, String)> {
    assert_eq!(&store[4..8], b"Bud1");
    let count = u32::from_be_bytes(
      store[RECORD_COUNT_OFFSET..RECORD_COUNT_OFFSET + 4]
        .try_into()
        .unwrap(),
    ) as usize;
    let mut offset = LEAF_PAGE_OFFSET + PAGE_HEADER_LEN;
    let mut records = Vec::new();
    for _ in 0..count {
      let name_len = u32::from_be_bytes(store[offset..offset + 4].try_into().unwrap()) as usize;
      offset += 4;
      let name = String::from_utf16(
        &store[offset..offset + name_len * 2]
          .chunks_exact(2)
          .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
          .collect::<Vec<_>>(),
      )
      .unwrap();
      offset += name_len * 2;
      let field = std::str::from_utf8(&store[offset..offset + 4])
        .unwrap()
        .to_string();
      offset += 4;
      let kind = std::str::from_utf8(&store[offset..offset + 4]).unwrap();
      offset += 4;
      let length = match kind {
        "blob" => {
          let length = u32::from_be_bytes(store[offset..offset + 4].try_into().unwrap()) as usize;
          offset += 4 + length;
          length
        }
        "long" => {
          offset += 4;
          4
        }
        other => panic!("unexpected value type {other}"),
      };
      assert!(length > 0);
      records.push((name, field, kind.to_string()));
    }
    records
  }

  fn layout() -> Layout<'static> {
    Layout {
      app_name: "Tauri App.app",
      window_size: (660, 400),
      window_position: None,
      app_position: Position { x: 180, y: 170 },
      application_folder_position: Position { x: 480, y: 170 },
    }
  }

  #[test]
  fn writes_window_and_icon_records() {
    let store = build(&layout()).unwrap();
    assert_eq!(store.len(), TEMPLATE.len());
    let records = record_names(&store);
    let names: Vec<_> = records.iter().map(|(name, _, _)| name.as_str()).collect();
    assert_eq!(names, [".", ".", ".", "Applications", "Tauri App.app"]);

    let fields: Vec<_> = records.iter().map(|(_, field, _)| field.as_str()).collect();
    assert_eq!(fields, ["bwsp", "icvp", "vSrn", "Iloc", "Iloc"]);
  }

  #[test]
  fn encodes_the_configured_positions() {
    let store = build(&layout()).unwrap();
    let mut offset = LEAF_PAGE_OFFSET + PAGE_HEADER_LEN;
    let mut positions = Vec::new();
    for _ in 0..5 {
      let name_len = u32::from_be_bytes(store[offset..offset + 4].try_into().unwrap()) as usize;
      offset += 4;
      let name = String::from_utf16(
        &store[offset..offset + name_len * 2]
          .chunks_exact(2)
          .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
          .collect::<Vec<_>>(),
      )
      .unwrap();
      offset += name_len * 2;
      let field = std::str::from_utf8(&store[offset..offset + 4]).unwrap();
      offset += 4;
      let kind = std::str::from_utf8(&store[offset..offset + 4]).unwrap();
      offset += 4;
      match kind {
        "blob" => {
          let length = u32::from_be_bytes(store[offset..offset + 4].try_into().unwrap()) as usize;
          offset += 4;
          if field == "Iloc" {
            let x = u32::from_be_bytes(store[offset..offset + 4].try_into().unwrap());
            let y = u32::from_be_bytes(store[offset + 4..offset + 8].try_into().unwrap());
            positions.push((name, x, y));
          }
          offset += length;
        }
        "long" => offset += 4,
        other => panic!("unexpected value type {other}"),
      }
    }
    assert_eq!(
      positions,
      [
        ("Applications".to_string(), 480, 170),
        ("Tauri App.app".to_string(), 180, 170),
      ]
    );
  }
}
