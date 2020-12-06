mod cmd;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
  tauri::AppBuilder::new()
    .setup(|webview, _| {
      let mut webview_ = webview.as_mut();
      tauri::event::listen(String::from("hello"), move |_| {
        tauri::event::emit(
          &mut webview_,
          String::from("reply"),
          Some("{ msg: 'TEST' }".to_string()),
        )
        .unwrap();
      });
      webview.eval("window.onTauriInit && window.onTauriInit()");
    })
    .invoke_handler(|webview, arg| {
      use cmd::Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            Exit {} => {
              webview.terminate();
            }
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}
