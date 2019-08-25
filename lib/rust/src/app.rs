mod runner;
use tauri_ui::WebView;

//type FnMut(&mut InvokeHandler<WebView<'_, ()>>, &str) = FnMut(&mut FnMut(&mut InvokeHandler<WebView<'_, ()>>, &str)<WebView<'_, ()>>, &str);

pub struct App {
  invoke_handler: Option<Box<dyn FnMut(&mut WebView<'_, ()>, &str)>>,
}

impl App {
  pub fn run(mut self) {
    runner::run(&mut self);
  }

  pub fn run_invoke_handler(&mut self, webview: &mut WebView<'_, ()>, arg: &str) {
    match self.invoke_handler {
      Some(ref mut invoke_handler) => {
        invoke_handler(webview, arg);
      }
      None => {}
    }
  }
}

pub struct AppBuilder {
  invoke_handler: Option<Box<dyn FnMut(&mut WebView<'_, ()>, &str)>>,
}

impl AppBuilder {
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
    }
  }

  pub fn extension(self, ext: impl crate::extension::Extension + 'static) -> Self {
    crate::extension::register(ext);
    self
  }

  pub fn invoke_handler<F: FnMut(&mut WebView<'_, ()>, &str) + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  pub fn build(self) -> App {
    App {
      invoke_handler: self.invoke_handler,
    }
  }
}
