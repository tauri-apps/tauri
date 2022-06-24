// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[napi_derive::napi]
pub fn run(args: Vec<String>, bin_name: Option<String>) {
  tauri_cli::run(args, bin_name);
}
