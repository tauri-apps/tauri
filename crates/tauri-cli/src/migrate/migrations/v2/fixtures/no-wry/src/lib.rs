pub fn app() -> tauri::App<tauri::test::MockRuntime> {
  tauri::test::mock_builder()
    .build(tauri::generate_context!())
    .unwrap()
}

pub fn builder() -> tauri::Builder<tauri::Wry> {
  tauri::Builder::default()
}
