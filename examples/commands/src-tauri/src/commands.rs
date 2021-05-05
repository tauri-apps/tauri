// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[tauri::command]
pub fn simple_command(argument: String) {
  println!("{}", argument);
}

#[tauri::command]
pub fn stateful_command(argument: Option<String>, state: tauri::State<'_, super::MyState>) {
  println!("{:?} {:?}", argument, state.inner());
}
