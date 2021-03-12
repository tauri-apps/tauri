#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[derive(tauri::FromTauriContext)]
#[config_path = "examples/helloworld/src-tauri/tauri.conf.json"]
struct Context;

#[tauri::command]
fn my_custom_command(argument: String) {
  println!("{}", argument);
}

fn main() {
  tauri::AppBuilder::<Context>::new()
    .invoke_handler(tauri::generate_handler![my_custom_command])
    .build()
    .unwrap()
    // Ugly fix to pass PKG_NAME & PKG_VERSION to the updater
    // The best would be to test correctly with bundled app
    // if CARGO_PKG_VERSION return correct version in libraries
    // when we use examples run it return tauri version
    .meta(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    .run();
}
