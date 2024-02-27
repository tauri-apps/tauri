fn main() {
  tauri_plugin::Builder::new()
    .android_path("android")
    .ios_path("ios")
    .build();
}
