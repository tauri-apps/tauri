use super::{PluginStore, SizeHint, Webview, WebviewBuilder};
use once_cell::sync::Lazy;

type WebviewRef = webview_official::Webview;

/// The webview implementation from https://github.com/webview/webview
#[derive(Clone)]
pub struct WebviewOfficial(WebviewRef);

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
  type WebviewObject = WebviewOfficial;

  fn new() -> Self {
    WebviewOfficialBuilder::default()
  }

  fn debug(&mut self, debug: bool) -> &mut Self {
    self.debug = debug;
    self
  }

  fn title(&mut self, title: &str) -> &mut Self {
    self.title = Some(title.to_string());
    self
  }

  fn url(&mut self, url: &str) -> &mut Self {
    self.url = Some(url.to_string());
    self
  }

  fn init(&mut self, init: &str) -> &mut Self {
    self.init = Some(init.to_string());
    self
  }

  fn width(&mut self, width: usize) -> &mut Self {
    self.size.0 = width;
    self
  }

  fn height(&mut self, height: usize) -> &mut Self {
    self.size.1 = height;
    self
  }

  fn resizable(&mut self, hint: SizeHint) -> &mut Self {
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

    WebviewOfficial(w)
  }
}

impl Webview for WebviewOfficial {
  type Builder = WebviewOfficialBuilder;

  fn plugin_store() -> &'static PluginStore<Self> {
    static PLUGINS: Lazy<PluginStore<WebviewOfficial>> = Lazy::new(Default::default);
    &PLUGINS
  }

  fn init(&mut self, js: &str) {
    self.0.init(js);
  }

  fn set_title(&mut self, title: &str) {
    self.0.set_title(title);
  }

  fn set_size(&mut self, width: i32, height: i32, hint: SizeHint) {
    self.0.set_size(
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
    self.0.terminate();
  }

  fn eval(&mut self, js: &str) {
    self.0.eval(js);
  }

  fn dispatch<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self) + Send + 'static,
  {
    self.0.dispatch(move |w| {
      f(&mut Self(w));
    });
  }

  fn bind<F>(&mut self, name: &str, f: F)
  where
    F: FnMut(&str, &str),
  {
    self.0.bind(name, f);
  }

  fn run(&mut self) {
    self.0.run();
  }
}
