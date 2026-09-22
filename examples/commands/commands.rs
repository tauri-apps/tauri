// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{State, command};

#[command]
pub fn cmd(_argument: String) {}

#[command]
pub fn invoke(_argument: String) {}

#[command]
pub fn message(_argument: String) {}

#[command]
pub fn resolver(_argument: String) {}

#[command]
pub fn simple_command(the_argument: String) {
  println!("{the_argument}");
}

#[command]
pub fn stateful_command(the_argument: Option<String>, state: State<'_, super::MyState>) {
  println!("{:?} {:?}", the_argument, *state);
}

#[command(rename = "renamed_command_in_mod_new")]
pub fn renamed_command_in_mod() {
  println!("renamed command in mod called");
}
