// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

pub enum PlistKind {
  Path(PathBuf),
  Plist(plist::Value),
}

impl From<PathBuf> for PlistKind {
  fn from(p: PathBuf) -> Self {
    Self::Path(p)
  }
}
impl From<plist::Value> for PlistKind {
  fn from(p: plist::Value) -> Self {
    Self::Plist(p)
  }
}

impl From<plist::Dictionary> for PlistKind {
  fn from(p: plist::Dictionary) -> Self {
    Self::Plist(p.into())
  }
}

pub fn merge(src: Vec<PlistKind>) -> crate::Result<plist::Value> {
  let mut merged_plist = plist::Dictionary::new();

  for plist_kind in src {
    let plist = match plist_kind {
      PlistKind::Path(p) => plist::Value::from_file(p),
      PlistKind::Plist(v) => Ok(v),
    };
    if let Ok(src_plist) = plist {
      if let Some(dict) = src_plist.into_dictionary() {
        for (key, value) in dict {
          merged_plist.insert(key, value);
        }
      }
    }
  }

  Ok(plist::Value::Dictionary(merged_plist))
}
