// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A module containting builder types

mod check;
mod icon;
mod normal;
mod submenu;

pub use crate::about_metadata::AboutMetadataBuilder;
pub use check::*;
pub use icon::*;
pub use normal::*;
pub use submenu::*;
