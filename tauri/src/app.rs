use web_view::WebView;

mod runner;

//type FnMut(&mut InvokeHandler<WebView<'_, ()>>, &str) = FnMut(&mut FnMut(&mut InvokeHandler<WebView<'_, ()>>, &str)<WebView<'_, ()>>, &str);

pub struct App {
  invoke_handler: Option<Box<dyn FnMut(&mut WebView<'_, ()>, &str)>>,
  setup: Option<Box<dyn FnMut(&mut WebView<'_, ()>)>>,
}

impl App {
  pub fn run(mut self) {
    runner::run(&mut self).expect("Failed to build webview");
  }

  pub(crate) fn run_invoke_handler(&mut self, webview: &mut WebView<'_, ()>, arg: &str) {
    match self.invoke_handler {
      Some(ref mut invoke_handler) => {
        invoke_handler(webview, arg);
      }
      None => {}
    }
  }

  pub(crate) fn run_setup(&mut self, webview: &mut WebView<'_, ()>) {
    match self.setup {
      Some(ref mut setup) => {
        setup(webview);
      }
      None => {}
    }
  }
}

pub struct AppBuilder {
  invoke_handler: Option<Box<dyn FnMut(&mut WebView<'_, ()>, &str)>>,
  setup: Option<Box<dyn FnMut(&mut WebView<'_, ()>)>>,
}

impl AppBuilder {
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
    }
  }

  pub fn invoke_handler<F: FnMut(&mut WebView<'_, ()>, &str) + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  pub fn setup<F: FnMut(&mut WebView<'_, ()>) + 'static>(mut self, setup: F) -> Self {
    self.setup = Some(Box::new(setup));
    self
  }

  pub fn build(self) -> App {
    App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
    }
  }
}
