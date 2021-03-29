#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[tauri::command]
fn my_custom_command(argument: String) {
  println!("{}", argument);
}

fn main() {
  let ctx = tauri::generate_context!();

  tauri::AppBuilder::default()
    .setup(|_app| {
      println!("hello from the setup function!");
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![my_custom_command])
    .build(ctx)
    .run()
    .expect("error while running application");
}
