#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  #[cfg(target_os = "ios")]
  let mut requested_scenes = 0u32;

  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(move |_app_handle, _event| {
      // the system requests a scene when the user opens a new app window,
      // e.g. from the iPadOS app switcher; create a webview window for it,
      // otherwise the new scene stays empty
      #[cfg(target_os = "ios")]
      if let tauri::RunEvent::SceneRequested { .. } = _event {
        requested_scenes += 1;
        tauri::WebviewWindowBuilder::new(
          _app_handle,
          format!("scene-{requested_scenes}"),
          tauri::WebviewUrl::default(),
        )
        .build()
        .expect("failed to create window for the requested scene");
      }
    });
}
