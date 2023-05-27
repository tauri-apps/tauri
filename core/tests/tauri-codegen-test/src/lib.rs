// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use tauri_utils::{assets::EmbeddedAssets, Context};

pub fn context() -> Context<EmbeddedAssets> {
  tauri::build_script_context!()
}
