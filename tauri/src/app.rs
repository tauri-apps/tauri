use std::marker::PhantomData;
use tauri_api::config::Config;
use tauri_api::private::AsTauriConfig;
use webview_official::Webview;

mod runner;

type InvokeHandler = Box<dyn FnMut(&mut Webview<'_>, &str) -> Result<(), String>>;
type Setup = Box<dyn FnMut(&mut Webview<'_>, String)>;

/// Configuration for the application's internal use.
pub(crate) struct AppConfig {
  pub config: Config,
  pub assets: &'static tauri_api::assets::Assets,
  pub index: &'static str,
}

impl AppConfig {
  pub fn new<Config: AsTauriConfig>() -> crate::Result<Self> {
    Ok(Self {
      config: serde_json::from_str(Config::raw_config())?,
      assets: Config::assets(),
      index: Config::raw_index(),
    })
  }
}

/// The application runner.
pub struct App {
  /// The JS message handler.
  invoke_handler: Option<InvokeHandler>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Setup>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
  /// The configuration the App was created with
  pub(crate) config: AppConfig,
}

impl App {
  /// Runs the app until it finishes.
  pub fn run(mut self) {
    runner::run(&mut self).expect("Failed to build webview");
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) fn run_invoke_handler(
    &mut self,
    webview: &mut Webview<'_>,
    arg: &str,
  ) -> Result<bool, String> {
    if let Some(ref mut invoke_handler) = self.invoke_handler {
      invoke_handler(webview, arg).map(|_| true)
    } else {
      Ok(false)
    }
  }

  /// Runs the setup callback if defined.
  pub(crate) fn run_setup(&mut self, webview: &mut Webview<'_>, source: String) {
    if let Some(ref mut setup) = self.setup {
      setup(webview, source);
    }
  }

  /// Returns the splashscreen HTML.
  pub fn splashscreen_html(&self) -> Option<&String> {
    self.splashscreen_html.as_ref()
  }
}

/// The App builder.
#[derive(Default)]
pub struct AppBuilder<Config: AsTauriConfig> {
  /// The JS message handler.
  invoke_handler: Option<InvokeHandler>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Setup>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
  /// The configuration used
  config: PhantomData<Config>,
}

impl<Config: AsTauriConfig> AppBuilder<Config> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      splashscreen_html: None,
      config: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F: FnMut(&mut Webview<'_>, &str) -> Result<(), String> + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  /// Defines the setup callback.
  pub fn setup<F: FnMut(&mut Webview<'_>, String) + 'static>(mut self, setup: F) -> Self {
    self.setup = Some(Box::new(setup));
    self
  }

  /// Defines the splashscreen HTML to render.
  pub fn splashscreen_html(mut self, html: &str) -> Self {
    self.splashscreen_html = Some(html.to_string());
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin(self, plugin: impl crate::plugin::Plugin + 'static) -> Self {
    crate::plugin::register(plugin);
    self
  }

  /// Builds the App.
  pub fn build(self) -> crate::Result<App> {
    Ok(App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      splashscreen_html: self.splashscreen_html,
      config: AppConfig::new::<Config>()?,
    })
  }
}
