#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[derive(tauri::FromTauriContext)]
#[config_path = "examples/multiwindow/src-tauri/tauri.conf.json"]
struct Context;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Context>::new()
    .setup(|webview_manager| async move {
      let current_webview = webview_manager.current_webview().unwrap().clone();
      let current_webview_ = current_webview.clone();
      let event_name = format!("window://{}", webview_manager.current_window_label());
      current_webview.listen(event_name.clone(), move |msg| {
        current_webview_
          .emit(&event_name, msg)
          .expect("failed to emit");
      });
    })
    .build()
    .unwrap()
    .run();
}
