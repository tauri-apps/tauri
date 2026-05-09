// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Icon, IsMenuItem, MenuId, NativeIcon, Submenu};

/// A builder type for [`Submenu`]
#[derive(Clone, Default)]
pub struct SubmenuBuilder<'a> {
    text: String,
    enabled: bool,
    id: Option<MenuId>,
    items: Vec<&'a dyn IsMenuItem>,
    icon: Option<Icon>,
    native_icon: Option<NativeIcon>,
}

impl std::fmt::Debug for SubmenuBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubmenuBuilder")
            .field("text", &self.text)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl<'a> SubmenuBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the id this submenu.
    pub fn id(mut self, id: MenuId) -> Self {
        self.id.replace(id);
        self
    }

    /// Set the text for this submenu.
    ///
    /// See [`Submenu::set_text`] for more info.
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = text.into();
        self
    }

    /// Enable or disable this submenu.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add an item to this submenu.
    pub fn item(mut self, item: &'a dyn IsMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add these items to this submenu.
    pub fn items(mut self, items: &[&'a dyn IsMenuItem]) -> Self {
        self.items.extend_from_slice(items);
        self
    }

    /// Set an icon for this submenu.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set a native icon for this submenu.
    pub fn native_icon(mut self, icon: NativeIcon) -> Self {
        self.native_icon = Some(icon);
        self
    }

    /// Build this menu item.
    pub fn build(self) -> crate::Result<Submenu> {
        let submenu = if let Some(id) = self.id {
            Submenu::with_id_and_items(id, self.text, self.enabled, &self.items)?
        } else {
            Submenu::with_items(self.text, self.enabled, &self.items)?
        };

        if let Some(icon) = self.icon {
            submenu.set_icon(Some(icon));
        }

        if let Some(native_icon) = self.native_icon {
            submenu.set_native_icon(Some(native_icon));
        }

        Ok(submenu)
    }
}
