mod cmd;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
  tauri::AppBuilder::new()
    .setup(|_webview| {
      let handle = _webview.handle();
      tauri::event::listen(String::from("hello"), move |_| {
        tauri::event::emit(&handle, String::from("reply"), "{ msg: 'TEST' }".to_string());
      });
    })
    .build()
    .run();
}
