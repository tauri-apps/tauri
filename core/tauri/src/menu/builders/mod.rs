// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(desktop)]

//! A module containting menu builder types

pub use muda::AboutMetadataBuilder;

mod menu;
pub use menu::MenuBuilder;
mod normal;
pub use normal::MenuItemBuilder;
mod submenu;
pub use submenu::SubmenuBuilder;
mod check;
pub use check::CheckMenuItemBuilder;
mod icon;
pub use icon::IconMenuItemBuilder;
