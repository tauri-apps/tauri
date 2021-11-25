use tauri::{plugin::{TauriPlugin, Builder}, Runtime};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("hello-world")
    .setup(|_| {
      println!("hello world from plugin");
      Ok(())
    })
    .build()
}
