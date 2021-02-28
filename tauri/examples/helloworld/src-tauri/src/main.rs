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
    .run();
}
