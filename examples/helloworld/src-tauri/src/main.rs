#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[tauri::command]
fn my_custom_command(argument: String) {
  println!("{}", argument);
}

fn main() {
  let _build = tauri::tauri_build_context!();
  let _macro = tauri::generate_tauri_context!();

  tauri::AppBuilder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command])
    .build(_macro)
    .run();
}
