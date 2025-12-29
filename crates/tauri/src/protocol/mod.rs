// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Custom protocol handlers

#[cfg(feature = "protocol-asset")]
pub mod asset;
#[cfg(feature = "isolation")]
pub mod isolation;
pub mod tauri;
