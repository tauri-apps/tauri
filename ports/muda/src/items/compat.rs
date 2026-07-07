// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::AboutMetadata;

#[derive(Debug, Clone)]
pub struct CompatStandardItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub icon: Option<Vec<u8>>,
    pub about_metadata: Option<AboutMetadata>,
}

#[derive(Debug, Clone)]
pub struct CompatCheckmarkItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
}

#[derive(Debug, Clone)]
pub struct CompatSubMenuItem {
    pub label: String,
    pub enabled: bool,
    pub submenu: Vec<Arc<ArcSwap<CompatMenuItem>>>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum CompatMenuItem {
    Standard(CompatStandardItem),
    Checkmark(CompatCheckmarkItem),
    SubMenu(CompatSubMenuItem),
    Separator,
}

impl From<CompatStandardItem> for CompatMenuItem {
    fn from(item: CompatStandardItem) -> Self {
        CompatMenuItem::Standard(item)
    }
}

impl From<CompatCheckmarkItem> for CompatMenuItem {
    fn from(item: CompatCheckmarkItem) -> Self {
        CompatMenuItem::Checkmark(item)
    }
}

impl From<CompatSubMenuItem> for CompatMenuItem {
    fn from(item: CompatSubMenuItem) -> Self {
        CompatMenuItem::SubMenu(item)
    }
}

pub(crate) fn strip_mnemonic(text: impl AsRef<str>) -> String {
    text.as_ref()
        .replace("&&", "[~~]")
        .replace('&', "")
        .replace("[~~]", "&")
}
