mod cmd;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
  tauri::AppBuilder::new()
    .setup(|webview, _| {
      let handle = webview.handle();
      tauri::event::listen(String::from("hello"), move |_| {
        tauri::event::emit(&handle, String::from("reply"), Some("{ msg: 'TEST' }".to_string()));
      });
      webview.eval("window.onTauriInit && window.onTauriInit()").unwrap();
    })
     .invoke_handler(|webview, arg| {
      use cmd::Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => {
          Err(e.to_string())
        }
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            Exit { } => {
              webview.exit();
            }
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}
