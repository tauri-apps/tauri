use super::{PluginStore, SizeHint, Webview, WebviewBuilder, WebviewDispatcher};
use once_cell::sync::Lazy;

#[derive(Default)]
pub struct WebviewOfficialBuilder {
  title: Option<String>,
  url: Option<String>,
  init: Option<String>,
  eval: Option<String>,
  size: (usize, usize, SizeHint),
  debug: bool,
}

impl WebviewBuilder for WebviewOfficialBuilder {
  type WebviewObject = webview_official::Webview;

  fn new() -> Self {
    WebviewOfficialBuilder::default()
  }

  fn bind<F>(self, _name: &str, _f: F) -> Self
  where
    F: FnMut(<<Self as WebviewBuilder>::WebviewObject as Webview>::Dispatcher, &str, &str),
  {
    self
  }

  fn debug(mut self, debug: bool) -> Self {
    self.debug = debug;
    self
  }

  fn title(mut self, title: &str) -> Self {
    self.title = Some(title.to_string());
    self
  }

  fn url(mut self, url: &str) -> Self {
    self.url = Some(url.to_string());
    self
  }

  fn init(mut self, init: &str) -> Self {
    self.init = Some(init.to_string());
    self
  }

  fn width(mut self, width: usize) -> Self {
    self.size.0 = width;
    self
  }

  fn height(mut self, height: usize) -> Self {
    self.size.1 = height;
    self
  }

  fn resizable(mut self, hint: SizeHint) -> Self {
    self.size.2 = hint;
    self
  }

  fn finish(self) -> Self::WebviewObject {
    let mut w = webview_official::Webview::create(self.debug, None);
    if let Some(title) = self.title {
      w.set_title(&title);
    }

    if let Some(init) = self.init {
      w.init(&init);
    }

    if let Some(url) = self.url {
      w.navigate(&url);
    }

    if let Some(eval) = self.eval {
      w.eval(&eval);
    }

    w.set_size(
      self.size.0 as i32,
      self.size.1 as i32,
      match self.size.2 {
        SizeHint::NONE => webview_official::SizeHint::NONE,
        SizeHint::MIN => webview_official::SizeHint::MIN,
        SizeHint::MAX => webview_official::SizeHint::MAX,
        SizeHint::FIXED => webview_official::SizeHint::FIXED,
      },
    );

    w
  }
}

impl WebviewDispatcher for webview_official::Webview {
  fn eval(&mut self, js: &str) {
    webview_official::Webview::eval(self, js);
  }
}

impl Webview for webview_official::Webview {
  type Builder = WebviewOfficialBuilder;
  type Dispatcher = webview_official::Webview;

  fn plugin_store() -> &'static PluginStore<Self::Dispatcher> {
    static PLUGINS: Lazy<PluginStore<webview_official::Webview>> = Lazy::new(Default::default);
    &PLUGINS
  }

  fn init(&mut self, js: &str) {
    self.init(js);
  }

  fn set_title(&mut self, title: &str) {
    self.set_title(title);
  }

  fn set_size(&mut self, width: i32, height: i32, hint: SizeHint) {
    self.set_size(
      width,
      height,
      match hint {
        SizeHint::NONE => webview_official::SizeHint::NONE,
        SizeHint::MIN => webview_official::SizeHint::MIN,
        SizeHint::MAX => webview_official::SizeHint::MAX,
        SizeHint::FIXED => webview_official::SizeHint::FIXED,
      },
    );
  }

  fn terminate(&mut self) {
    self.terminate();
  }

  fn eval(&mut self, js: &str) {
    self.eval(js);
  }

  fn dispatch<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self) + Send + 'static,
  {
    self.dispatch(f);
  }

  fn dispatcher(&mut self) -> Self::Dispatcher {
    self.clone()
  }

  fn run(&mut self) {
    self.run();
  }
}
