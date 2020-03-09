use web_view::WebView;

mod runner;

type InvokeHandler<T> = Box<dyn FnMut(&mut WebView<'_, T>, &str)>;
type Setup<T> = Box<dyn FnMut(&mut WebView<'_, T>, String)>;

pub struct App<T: 'static> {
  invoke_handler: Option<InvokeHandler<T>>,
  setup: Option<Setup<T>>,
  splashscreen_html: Option<String>,
  user_data: Option<T>
}

impl<T> App<T> {
  pub fn run(mut self) {
    runner::run(&mut self).expect("Failed to build webview");
  }

  pub(crate) fn run_invoke_handler(&mut self, webview: &mut WebView<'_, T>, arg: &str) {
    if let Some(ref mut invoke_handler) = self.invoke_handler {
      invoke_handler(webview, arg);
    }
  }

  pub(crate) fn run_setup(&mut self, webview: &mut WebView<'_, T>, source: String) {
    if let Some(ref mut setup) = self.setup {
      setup(webview, source);
    }
  }

  pub fn splashscreen_html(&self) -> Option<&String> {
    self.splashscreen_html.as_ref()
  }
}

#[derive(Default)]
pub struct AppBuilder<T: 'static> {
  invoke_handler: Option<InvokeHandler<T>>,
  setup: Option<Setup<T>>,
  splashscreen_html: Option<String>,
  user_data: Option<T>
}

// Default builder with no user_data
impl AppBuilder<()> {
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      splashscreen_html: None,
      user_data: Some(())
    }
  }
}

impl<T> AppBuilder<T> {
  pub fn new_with_user_data(user_data: T) -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      splashscreen_html: None,
      user_data: Some(user_data)
    }
  }

  pub fn invoke_handler<F: FnMut(&mut WebView<'_, T>, &str) + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  pub fn setup<F: FnMut(&mut WebView<'_, T>, String) + 'static>(mut self, setup: F) -> Self {
    self.setup = Some(Box::new(setup));
    self
  }

  pub fn splashscreen_html(mut self, html: &str) -> Self {
    self.splashscreen_html = Some(html.to_string());
    self
  }

  pub fn build(self) -> App<T> {
    App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      splashscreen_html: self.splashscreen_html,
      user_data: self.user_data,
    }
  }
}
