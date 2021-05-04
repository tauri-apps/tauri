// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "1s"
)]

#[derive(Debug)]
struct MyState {
  value: u64,
  label: String,
}

#[tauri::command]
fn simple_command(argument: String) {
  println!("{}", argument);
}

#[tauri::command]
fn stateful_command(argument: Option<String>, state: tauri::State<'_, MyState>) {
  println!("{:?} {:?}", argument, state.inner());
}

// ------------------------ Commands using Window ------------------------
#[tauri::command]
fn window_label(window: tauri::Window<impl tauri::Params<Label = String>>) {
  println!("window label: {}", window.label());
}

// Async commands

#[tauri::command]
async fn async_simple_command(argument: String) {
  println!("{}", argument);
}

#[tauri::command]
async fn async_stateful_command(argument: Option<String>, state: tauri::State<'_, MyState>) {
  println!("{:?} {:?}", argument, state.inner());
}

// ------------------------ Commands returning Result ------------------------

type Result<T> = std::result::Result<T, ()>;

#[tauri::command]
fn simple_command_with_result(argument: String) -> Result<String> {
  println!("{}", argument);
  (!argument.is_empty()).then(|| argument).ok_or(())
}

#[tauri::command]
fn stateful_command_with_result(
  argument: Option<String>,
  state: tauri::State<'_, MyState>,
) -> Result<String> {
  println!("{:?} {:?}", argument, state.inner());
  argument.ok_or(())
}

// Async commands

#[tauri::command]
async fn async_simple_command_with_result(argument: String) -> Result<String> {
  println!("{}", argument);
  Ok(argument)
}

#[tauri::command]
async fn async_stateful_command_with_result(
  argument: Option<String>,
  state: tauri::State<'_, MyState>,
) -> Result<String> {
  println!("{:?} {:?}", argument, state.inner());
  Ok(argument.unwrap_or_else(|| "".to_string()))
}

fn main() {
  tauri::Builder::default()
    .manage(MyState {
      value: 0,
      label: "Tauri!".into(),
    })
    .invoke_handler(tauri::generate_handler![
      window_label,
      simple_command,
      stateful_command,
      async_simple_command,
      async_stateful_command,
      simple_command_with_result,
      stateful_command_with_result,
      async_simple_command_with_result,
      async_stateful_command_with_result
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
