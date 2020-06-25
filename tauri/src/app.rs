use web_view::WebView;

mod runner;

type InvokeHandler = Box<dyn FnMut(&mut WebView<'_, ()>, &str) -> Result<(), String>>;
type Setup = Box<dyn FnMut(&mut WebView<'_, ()>, String)>;

pub struct App {
  invoke_handler: Option<InvokeHandler>,
  setup: Option<Setup>,
  splashscreen_html: Option<String>,
  name: Option<String>,
  version: Option<String>,
}

#[derive(Clone)]
pub struct AppMeta {
  pub name: String,
  pub version: String,
}

impl App {
  pub fn run(mut self) {
    runner::run(&mut self).expect("Failed to build webview");
  }

  pub(crate) fn run_invoke_handler(
    &mut self,
    webview: &mut WebView<'_, ()>,
    arg: &str,
  ) -> Result<bool, String> {
    if let Some(ref mut invoke_handler) = self.invoke_handler {
      invoke_handler(webview, arg).map(|_| true)
    } else {
      Ok(false)
    }
  }

  pub(crate) fn run_setup(&mut self, webview: &mut WebView<'_, ()>, source: String) {
    if let Some(ref mut setup) = self.setup {
      setup(webview, source);
    }
  }

  pub fn splashscreen_html(&self) -> Option<&String> {
    self.splashscreen_html.as_ref()
  }

  pub fn meta(&self) -> Option<AppMeta> {
    if self.version.is_some() && self.name.is_some() {
      Some(AppMeta {
        version: self
          .version
          .clone()
          .expect("Something wrong with app version extraction"),
        name: self
          .name
          .clone()
          .expect("Something wrong with app name extraction"),
      })
    } else {
      None
    }
  }
}

#[derive(Default)]
pub struct AppBuilder {
  invoke_handler: Option<InvokeHandler>,
  setup: Option<Setup>,
  splashscreen_html: Option<String>,
  name: Option<String>,
  version: Option<String>,
}

impl AppBuilder {
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      splashscreen_html: None,
      name: None,
      version: None,
    }
  }

  pub fn invoke_handler<F: FnMut(&mut WebView<'_, ()>, &str) -> Result<(), String> + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  pub fn setup<F: FnMut(&mut WebView<'_, ()>, String) + 'static>(mut self, setup: F) -> Self {
    self.setup = Some(Box::new(setup));
    self
  }

  pub fn splashscreen_html(mut self, html: &str) -> Self {
    self.splashscreen_html = Some(html.to_string());
    self
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = Some(name.to_string());
    self
  }

  pub fn version(mut self, version: &str) -> Self {
    self.version = Some(version.to_string());
    self
  }

  pub fn build(self) -> App {
    App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      splashscreen_html: self.splashscreen_html,
      name: self.name,
      version: self.version,
    }
  }
}
