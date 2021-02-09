mod cmd;

use tauri::ApplicationDispatcherExt;

#[derive(tauri::FromTauriContext)]
struct Context;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Config>::new()
    .setup(|dispatcher, _| async move {
      let mut dispatcher_ = dispatcher.clone();
      tauri::event::listen(String::from("hello"), move |_| {
        tauri::event::emit(
          &mut dispatcher_,
          String::from("reply"),
          Some("{ msg: 'TEST' }".to_string()),
        )
        .unwrap();
      });
      dispatcher.eval("window.onTauriInit && window.onTauriInit()");
    })
    .invoke_handler(|dispatcher, arg| async move {
      use cmd::Cmd::*;
      match serde_json::from_str(&arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            Exit {} => {
              // TODO dispatcher.terminate();
            }
          }
          Ok(())
        }
      }
    })
    .build()
    .unwrap()
    .run();
}
