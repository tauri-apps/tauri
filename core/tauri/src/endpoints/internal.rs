// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{runtime::window::Window, Params};
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  ValidateSalt { salt: String },
}

impl Cmd {
  pub fn run<P: Params>(self, window: Window<P>) -> crate::Result<InvokeResponse> {
    match self {
      Self::ValidateSalt { salt } => Ok(window.verify_salt(salt).into()),
    }
  }
}
