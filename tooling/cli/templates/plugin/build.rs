use std::process::exit;

fn main() {
  if let Err(error) = tauri_build::mobile::PluginBuilder::new()
    .android_path("android")
    .ios_path("ios")
    .run()
  {
    println!("{error:#}");
    exit(1);
  }
}
