use tauri::ApplicationDispatcherExt;

#[derive(tauri::FromTauriContext)]
struct Context;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Context>::new()
    .setup(|webview_manager| async move {
      let mut webview_manager_ = webview_manager.clone();
      tauri::event::listen(String::from("hello"), move |_| {
        tauri::event::emit(
          &webview_manager_,
          String::from("reply"),
          Some("{ msg: 'TEST' }".to_string()),
        )
        .unwrap();
      });
      webview_manager
        .current_webview()
        .eval("window.onTauriInit && window.onTauriInit()");
    })
    .invoke_handler(|webview_manager, command, _arg| async move {
      if &command == "exit" {
        webview_manager.close().unwrap();
      }
    })
    .build()
    .unwrap()
    .run();
}
