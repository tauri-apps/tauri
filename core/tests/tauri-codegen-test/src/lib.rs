pub fn context() -> tauri::Context<tauri::utils::assets::EmbeddedAssets> {
  tauri::tauri_build_context!()
}
