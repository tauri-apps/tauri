// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri plugin extension to expand Tauri functionality.

use crate::{
  http::{Request as HttpRequest, Response as HttpResponse},
  manager::CustomProtocol,
  utils::config::PluginConfig,
  AppHandle, Invoke, InvokeHandler, PageLoadPayload, RunEvent, Runtime, Window,
};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use tauri_macros::default_runtime;

use std::{collections::HashMap, fmt, sync::Arc};

/// The result type of Tauri plugin module.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The plugin interface.
pub trait Plugin<R: Runtime>: Send {
  /// The plugin name. Used as key on the plugin config object.
  fn name(&self) -> &'static str;

  /// Initializes the plugin.
  #[allow(unused_variables)]
  fn initialize(&mut self, app: &AppHandle<R>, config: JsonValue) -> Result<()> {
    Ok(())
  }

  /// Add the provided JavaScript to a list of scripts that should be run after the global object has been created,
  /// but before the HTML document has been parsed and before any other script included by the HTML document is run.
  ///
  /// Since it runs on all top-level document and child frame page navigations,
  /// it's recommended to check the `window.location` to guard your script from running on unexpected origins.
  ///
  /// The script is wrapped into its own context with `(function () { /* your script here */ })();`,
  /// so global variables must be assigned to `window` instead of implicitly declared.
  fn initialization_script(&self) -> Option<String> {
    None
  }

  /// Callback invoked when the webview is created.
  #[allow(unused_variables)]
  fn created(&mut self, window: Window<R>) {}

  /// Callback invoked when the webview performs a navigation to a page.
  #[allow(unused_variables)]
  fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {}

  /// Callback invoked when the event loop receives a new event.
  #[allow(unused_variables)]
  fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {}

  /// Extend commands to [`crate::Builder::invoke_handler`].
  #[allow(unused_variables)]
  fn extend_api(&mut self, invoke: Invoke<R>) {}
}

type SetupHook<R> = dyn FnOnce(&AppHandle<R>) -> Result<()> + Send;
type SetupWithConfigHook<R, T> = dyn FnOnce(&AppHandle<R>, T) -> Result<()> + Send;
type OnWebviewReady<R> = dyn FnMut(Window<R>) + Send;
type OnEvent<R> = dyn FnMut(&AppHandle<R>, &RunEvent) + Send;
type OnPageLoad<R> = dyn FnMut(Window<R>, PageLoadPayload) + Send;
type OnDrop<R> = dyn FnOnce(AppHandle<R>) + Send;

/// Builds a [`TauriPlugin`].
///
/// This Builder offers a more concise way to construct Tauri plugins than implementing the Plugin trait directly.
///
/// # Conventions
///
/// When using the Builder Pattern it is encouraged to export a function called `init` that constructs and returns the plugin.
/// While plugin authors can provide every possible way to construct a plugin,
/// sticking to the `init` function convention helps users to quickly identify the correct function to call.
///
/// ```rust
/// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
///
/// pub fn init<R: Runtime>() -> TauriPlugin<R> {
///   Builder::new("example")
///     .build()
/// }
/// ```
///
/// When plugins expose more complex configuration options, it can be helpful to provide a Builder instead:
///
/// ```rust
/// use tauri::{plugin::{Builder as PluginBuilder, TauriPlugin}, Runtime};
///
/// pub struct Builder {
///   option_a: String,
///   option_b: String,
///   option_c: bool
/// }
///
/// impl Default for Builder {
///   fn default() -> Self {
///     Self {
///       option_a: "foo".to_string(),
///       option_b: "bar".to_string(),
///       option_c: false
///     }
///   }
/// }
///
/// impl Builder {
///   pub fn new() -> Self {
///     Default::default()
///   }
///
///   pub fn option_a(mut self, option_a: String) -> Self {
///     self.option_a = option_a;
///     self
///   }
///
///   pub fn option_b(mut self, option_b: String) -> Self {
///     self.option_b = option_b;
///     self
///   }
///
///   pub fn option_c(mut self, option_c: bool) -> Self {
///     self.option_c = option_c;
///     self
///   }
///
///   pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
///     PluginBuilder::new("example")
///       .setup(move |app_handle| {
///         // use the options here to do stuff
///         println!("a: {}, b: {}, c: {}", self.option_a, self.option_b, self.option_c);
///
///         Ok(())
///       })
///       .build()
///   }
/// }
/// ```
pub struct Builder<R: Runtime, C: DeserializeOwned = ()> {
  name: &'static str,
  invoke_handler: Box<InvokeHandler<R>>,
  setup: Option<Box<SetupHook<R>>>,
  setup_with_config: Option<Box<SetupWithConfigHook<R, C>>>,
  js_init_script: Option<String>,
  on_page_load: Box<OnPageLoad<R>>,
  on_webview_ready: Box<OnWebviewReady<R>>,
  on_event: Box<OnEvent<R>>,
  on_drop: Option<Box<OnDrop<R>>>,
  uri_scheme_protocols: HashMap<String, Arc<CustomProtocol<R>>>,
}

impl<R: Runtime, C: DeserializeOwned> Builder<R, C> {
  /// Creates a new Plugin builder.
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      setup: None,
      setup_with_config: None,
      js_init_script: None,
      invoke_handler: Box::new(|_| ()),
      on_page_load: Box::new(|_, _| ()),
      on_webview_ready: Box::new(|_| ()),
      on_event: Box::new(|_, _| ()),
      on_drop: None,
      uri_scheme_protocols: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  /// It is recommended you use the [tauri::generate_handler] to generate the input to this method, as the input type is not considered stable yet.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
  ///
  /// #[tauri::command]
  /// async fn foobar<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<(), String> {
  ///   println!("foobar");
  ///
  ///   Ok(())
  /// }
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("example")
  ///     .invoke_handler(tauri::generate_handler![foobar])
  ///     .build()
  /// }
  ///
  /// ```
  /// [tauri::generate_handler]: ../macro.generate_handler.html
  #[must_use]
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(Invoke<R>) + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// Sets the provided JavaScript to be run after the global object has been created,
  /// but before the HTML document has been parsed and before any other script included by the HTML document is run.
  ///
  /// Since it runs on all top-level document and child frame page navigations,
  /// it's recommended to check the `window.location` to guard your script from running on unexpected origins.
  ///
  /// The script is wrapped into its own context with `(function () { /* your script here */ })();`,
  /// so global variables must be assigned to `window` instead of implicitly declared.
  ///
  /// Note that calling this function multiple times overrides previous values.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
  ///
  /// const INIT_SCRIPT: &str = r#"
  ///   if (window.location.origin === 'https://tauri.app') {
  ///     console.log("hello world from js init script");
  ///
  ///     window.__MY_CUSTOM_PROPERTY__ = { foo: 'bar' };
  ///   }
  /// "#;
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("example")
  ///     .js_init_script(INIT_SCRIPT.to_string())
  ///     .build()
  /// }
  /// ```
  #[must_use]
  pub fn js_init_script(mut self, js_init_script: String) -> Self {
    self.js_init_script = Some(js_init_script);
    self
  }

  /// Define a closure that runs when the plugin is registered.
  ///
  /// This is a convenience function around [setup_with_config], without the need to specify a configuration object.
  ///
  /// The closure gets called before the [setup_with_config] closure.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime, Manager};
  /// use std::path::PathBuf;
  ///
  /// #[derive(Debug, Default)]
  /// struct PluginState {
  ///    dir: Option<PathBuf>
  /// }
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  /// Builder::new("example")
  ///   .setup(|app_handle| {
  ///     app_handle.manage(PluginState::default());
  ///
  ///     Ok(())
  ///   })
  ///   .build()
  /// }
  /// ```
  ///
  /// [setup_with_config]: struct.Builder.html#method.setup_with_config
  #[must_use]
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: FnOnce(&AppHandle<R>) -> Result<()> + Send + 'static,
  {
    self.setup.replace(Box::new(setup));
    self
  }

  /// Define a closure that runs when the plugin is registered, accepting a configuration object set on `tauri.conf.json > plugins > yourPluginName`.
  ///
  /// If your plugin is not pulling a configuration object from `tauri.conf.json`, use [setup].
  ///
  /// The closure gets called after the [setup] closure.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// #[derive(serde::Deserialize)]
  /// struct Config {
  ///   api_url: String,
  /// }
  ///
  /// fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R, Config> {
  ///   tauri::plugin::Builder::<R, Config>::new("api")
  ///     .setup_with_config(|_app_handle, config| {
  ///       println!("config: {:?}", config.api_url);
  ///       Ok(())
  ///     })
  ///     .build()
  /// }
  ///
  /// tauri::Builder::default().plugin(init());
  /// ```
  ///
  /// [setup]: struct.Builder.html#method.setup
  #[must_use]
  pub fn setup_with_config<F>(mut self, setup_with_config: F) -> Self
  where
    F: FnOnce(&AppHandle<R>, C) -> Result<()> + Send + 'static,
  {
    self.setup_with_config.replace(Box::new(setup_with_config));
    self
  }

  /// Callback invoked when the webview performs a navigation to a page.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("example")
  ///     .on_page_load(|window, payload| {
  ///       println!("Loaded URL {} in window {}", payload.url(), window.label());
  ///     })
  ///     .build()
  /// }
  /// ```
  #[must_use]
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: FnMut(Window<R>, PageLoadPayload) + Send + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Callback invoked when the webview is created.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("example")
  ///     .on_webview_ready(|window| {
  ///       println!("created window {}", window.label());
  ///     })
  ///     .build()
  /// }
  /// ```
  #[must_use]
  pub fn on_webview_ready<F>(mut self, on_webview_ready: F) -> Self
  where
    F: FnMut(Window<R>) + Send + 'static,
  {
    self.on_webview_ready = Box::new(on_webview_ready);
    self
  }

  /// Callback invoked when the event loop receives a new event.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, RunEvent, Runtime};
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("example")
  ///     .on_event(|app_handle, event| {
  ///       match event {
  ///         RunEvent::ExitRequested { api, .. } => {
  ///           // Prevents the app from exiting.
  ///           // This will cause the core thread to continue running in the background even without any open windows.
  ///           api.prevent_exit();
  ///         }
  ///         // Ignore all other cases.
  ///         _ => {}
  ///       }
  ///     })
  ///     .build()
  /// }
  /// ```
  #[must_use]
  pub fn on_event<F>(mut self, on_event: F) -> Self
  where
    F: FnMut(&AppHandle<R>, &RunEvent) + Send + 'static,
  {
    self.on_event = Box::new(on_event);
    self
  }

  /// Callback invoked when the plugin is dropped.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("example")
  ///     .on_drop(|app| {
  ///       println!("plugin has been dropped and is no longer running");
  ///       // you can run cleanup logic here
  ///     })
  ///     .build()
  /// }
  /// ```
  #[must_use]
  pub fn on_drop<F>(mut self, on_drop: F) -> Self
  where
    F: FnOnce(AppHandle<R>) + Send + 'static,
  {
    self.on_drop.replace(Box::new(on_drop));
    self
  }

  /// Registers a URI scheme protocol available to all webviews.
  /// Leverages [setURLSchemeHandler](https://developer.apple.com/documentation/webkit/wkwebviewconfiguration/2875766-seturlschemehandler) on macOS,
  /// [AddWebResourceRequestedFilter](https://docs.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.addwebresourcerequestedfilter?view=webview2-dotnet-1.0.774.44) on Windows
  /// and [webkit-web-context-register-uri-scheme](https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebContext.html#webkit-web-context-register-uri-scheme) on Linux.
  ///
  /// # Known limitations
  ///
  /// URI scheme protocols are registered when the webview is created. Due to this limitation, if the plugin is registed after a webview has been created, this protocol won't be available.
  ///
  /// # Arguments
  ///
  /// * `uri_scheme` The URI scheme to register, such as `example`.
  /// * `protocol` the protocol associated with the given URI scheme. It's a function that takes an URL such as `example://localhost/asset.css`.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{plugin::{Builder, TauriPlugin}, Runtime};
  ///
  /// fn init<R: Runtime>() -> TauriPlugin<R> {
  ///   Builder::new("myplugin")
  ///     .register_uri_scheme_protocol("myscheme", |app, req| {
  ///       tauri::http::ResponseBuilder::new().body(Vec::new())
  ///     })
  ///     .build()
  /// }
  /// ```
  #[must_use]
  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(
        &AppHandle<R>,
        &HttpRequest,
      ) -> std::result::Result<HttpResponse, Box<dyn std::error::Error>>
      + Send
      + Sync
      + 'static,
  >(
    mut self,
    uri_scheme: N,
    protocol: H,
  ) -> Self {
    self.uri_scheme_protocols.insert(
      uri_scheme.into(),
      Arc::new(CustomProtocol {
        protocol: Box::new(protocol),
      }),
    );
    self
  }

  /// Builds the [TauriPlugin].
  pub fn build(self) -> TauriPlugin<R, C> {
    TauriPlugin {
      name: self.name,
      app: None,
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      setup_with_config: self.setup_with_config,
      js_init_script: self.js_init_script,
      on_page_load: self.on_page_load,
      on_webview_ready: self.on_webview_ready,
      on_event: self.on_event,
      on_drop: self.on_drop,
      uri_scheme_protocols: self.uri_scheme_protocols,
    }
  }
}

/// Plugin struct that is returned by the [`Builder`]. Should only be constructed through the builder.
pub struct TauriPlugin<R: Runtime, C: DeserializeOwned = ()> {
  name: &'static str,
  app: Option<AppHandle<R>>,
  invoke_handler: Box<InvokeHandler<R>>,
  setup: Option<Box<SetupHook<R>>>,
  setup_with_config: Option<Box<SetupWithConfigHook<R, C>>>,
  js_init_script: Option<String>,
  on_page_load: Box<OnPageLoad<R>>,
  on_webview_ready: Box<OnWebviewReady<R>>,
  on_event: Box<OnEvent<R>>,
  on_drop: Option<Box<OnDrop<R>>>,
  uri_scheme_protocols: HashMap<String, Arc<CustomProtocol<R>>>,
}

impl<R: Runtime, C: DeserializeOwned> Drop for TauriPlugin<R, C> {
  fn drop(&mut self) {
    if let (Some(on_drop), Some(app)) = (self.on_drop.take(), self.app.take()) {
      on_drop(app);
    }
  }
}

impl<R: Runtime, C: DeserializeOwned> Plugin<R> for TauriPlugin<R, C> {
  fn name(&self) -> &'static str {
    self.name
  }

  fn initialize(&mut self, app: &AppHandle<R>, config: JsonValue) -> Result<()> {
    self.app.replace(app.clone());
    if let Some(s) = self.setup.take() {
      (s)(app)?;
    }
    if let Some(s) = self.setup_with_config.take() {
      (s)(app, serde_json::from_value(config)?)?;
    }
    for (uri_scheme, protocol) in &self.uri_scheme_protocols {
      app
        .manager
        .register_uri_scheme_protocol(uri_scheme, protocol.clone())
    }
    Ok(())
  }

  fn initialization_script(&self) -> Option<String> {
    self.js_init_script.clone()
  }

  fn created(&mut self, window: Window<R>) {
    (self.on_webview_ready)(window)
  }

  fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {
    (self.on_page_load)(window, payload)
  }

  fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
    (self.on_event)(app, event)
  }

  fn extend_api(&mut self, invoke: Invoke<R>) {
    (self.invoke_handler)(invoke)
  }
}

/// Plugin collection type.
#[default_runtime(crate::Wry, wry)]
pub(crate) struct PluginStore<R: Runtime> {
  store: HashMap<&'static str, Box<dyn Plugin<R>>>,
}

impl<R: Runtime> fmt::Debug for PluginStore<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("PluginStore")
      .field("plugins", &self.store.keys())
      .finish()
  }
}

impl<R: Runtime> Default for PluginStore<R> {
  fn default() -> Self {
    Self {
      store: HashMap::new(),
    }
  }
}

impl<R: Runtime> PluginStore<R> {
  /// Adds a plugin to the store.
  ///
  /// Returns `true` if a plugin with the same name is already in the store.
  pub fn register(&mut self, plugin: Box<dyn Plugin<R>>) -> bool {
    self.store.insert(plugin.name(), plugin).is_some()
  }

  /// Removes the plugin with the given name from the store.
  pub fn unregister(&mut self, plugin: &'static str) -> bool {
    self.store.remove(plugin).is_some()
  }

  /// Initializes the given plugin.
  pub(crate) fn initialize(
    &self,
    plugin: &mut Box<dyn Plugin<R>>,
    app: &AppHandle<R>,
    config: &PluginConfig,
  ) -> crate::Result<()> {
    initialize(plugin, app, config)
  }

  /// Initializes all plugins in the store.
  pub(crate) fn initialize_all(
    &mut self,
    app: &AppHandle<R>,
    config: &PluginConfig,
  ) -> crate::Result<()> {
    self
      .store
      .values_mut()
      .try_for_each(|plugin| initialize(plugin, app, config))
  }

  /// Generates an initialization script from all plugins in the store.
  pub(crate) fn initialization_script(&self) -> String {
    self
      .store
      .values()
      .filter_map(|p| p.initialization_script())
      .fold(String::new(), |acc, script| {
        format!("{acc}\n(function () {{ {script} }})();")
      })
  }

  /// Runs the created hook for all plugins in the store.
  pub(crate) fn created(&mut self, window: Window<R>) {
    self.store.values_mut().for_each(|plugin| {
      #[cfg(feature = "tracing")]
      let _span = tracing::trace_span!("plugin::hooks::created", name = plugin.name()).entered();
      plugin.created(window.clone())
    })
  }

  /// Runs the on_page_load hook for all plugins in the store.
  pub(crate) fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {
    self.store.values_mut().for_each(|plugin| {
      #[cfg(feature = "tracing")]
      let _span =
        tracing::trace_span!("plugin::hooks::on_page_load", name = plugin.name()).entered();
      plugin.on_page_load(window.clone(), payload.clone())
    })
  }

  /// Runs the on_event hook for all plugins in the store.
  pub(crate) fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.on_event(app, event))
  }

  pub(crate) fn extend_api(&mut self, mut invoke: Invoke<R>) {
    let command = invoke.message.command.replace("plugin:", "");
    let mut tokens = command.split('|');
    // safe to unwrap: split always has a least one item
    let target = tokens.next().unwrap();

    if let Some(plugin) = self.store.get_mut(target) {
      invoke.message.command = tokens
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(String::new);
      #[cfg(feature = "tracing")]
      let _span = tracing::trace_span!("plugin::hooks::ipc", name = plugin.name()).entered();
      plugin.extend_api(invoke);
    } else {
      invoke.resolver.reject(format!("plugin {target} not found"));
    }
  }
}

#[cfg_attr(feature = "tracing", tracing::instrument(name = "plugin::hooks::initialize", skip(plugin), fields(name = plugin.name())))]
fn initialize<R: Runtime>(
  plugin: &mut Box<dyn Plugin<R>>,
  app: &AppHandle<R>,
  config: &PluginConfig,
) -> crate::Result<()> {
  plugin
    .initialize(
      app,
      config.0.get(plugin.name()).cloned().unwrap_or_default(),
    )
    .map_err(|e| crate::Error::PluginInitialization(plugin.name().to_string(), e.to_string()))
}
