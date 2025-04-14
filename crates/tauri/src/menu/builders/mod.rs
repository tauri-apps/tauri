// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(desktop)]

//! A module containing menu builder types

mod menu;
pub use menu::MenuBuilder;
pub use menu::SubmenuBuilder;
mod normal;
pub use normal::MenuItemBuilder;
mod check;
pub use check::CheckMenuItemBuilder;
mod icon;
pub use icon::IconMenuItemBuilder;
