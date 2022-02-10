// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use napi::{Error, Result, Status};

#[napi_derive::napi]
pub fn run(args: Vec<String>, bin_name: Option<String>) -> Result<()> {
  tauri_cli::run(args, bin_name).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}
