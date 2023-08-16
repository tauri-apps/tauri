// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("menu")
    .invoke_handler(crate::generate_handler![])
    .build()
}
