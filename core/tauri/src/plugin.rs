// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri plugin extension to expand Tauri functionality.

#[cfg(target_os = "android")]
use crate::{
  runtime::RuntimeHandle,
  sealed::{ManagerBase, RuntimeOrDispatch},
};
use crate::{
  utils::config::PluginConfig, AppHandle, Invoke, InvokeHandler, PageLoadPayload, RunEvent,
  Runtime, Window,
};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use tauri_macros::default_runtime;

use std::{collections::HashMap, fmt, result::Result as StdResult};

/// The result type of Tauri plugin module.
pub type Result<T> = StdResult<T, Box<dyn std::error::Error>>;

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
  fn extend_api(&mut self, invoke: Invoke<R>) -> bool {
    false
  }
}

type SetupHook<R, C> = dyn FnOnce(&AppHandle<R>, PluginApi<R, C>) -> Result<()> + Send;
type OnWebviewReady<R> = dyn FnMut(Window<R>) + Send;
type OnEvent<R> = dyn FnMut(&AppHandle<R>, &RunEvent) + Send;
type OnPageLoad<R> = dyn FnMut(Window<R>, PageLoadPayload) + Send;
type OnDrop<R> = dyn FnOnce(AppHandle<R>) + Send;

#[cfg(mobile)]
type PendingPluginCallHandler =
  Box<dyn FnOnce(StdResult<serde_json::Value, serde_json::Value>) + Send + 'static>;

#[cfg(mobile)]
static PENDING_PLUGIN_CALLS: once_cell::sync::OnceCell<
  std::sync::Mutex<HashMap<i32, PendingPluginCallHandler>>,
> = once_cell::sync::OnceCell::new();

#[cfg(target_os = "android")]
#[doc(hidden)]
pub fn handle_android_plugin_response(
  env: jni::JNIEnv<'_>,
  id: i32,
  success: jni::objects::JString<'_>,
  error: jni::objects::JString<'_>,
) {
  let (payload, is_ok): (serde_json::Value, bool) = match (
    env
      .is_same_object(success, jni::objects::JObject::default())
      .unwrap_or_default(),
    env
      .is_same_object(error, jni::objects::JObject::default())
      .unwrap_or_default(),
  ) {
    // both null
    (true, true) => (serde_json::Value::Null, true),
    // error null
    (false, true) => (
      serde_json::from_str(env.get_string(success).unwrap().to_str().unwrap()).unwrap(),
      true,
    ),
    // success null
    (true, false) => (
      serde_json::from_str(env.get_string(error).unwrap().to_str().unwrap()).unwrap(),
      false,
    ),
    // both are set - impossible in the Kotlin code
    (false, false) => unreachable!(),
  };

  if let Some(handler) = PENDING_PLUGIN_CALLS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .remove(&id)
  {
    handler(if is_ok { Ok(payload) } else { Err(payload) });
  }
}

/// A handle to a plugin.
#[derive(Clone)]
#[allow(dead_code)]
pub struct PluginHandle<R: Runtime> {
  name: &'static str,
  handle: AppHandle<R>,
}

impl<R: Runtime> PluginHandle<R> {
  /// Executes the given mobile method.
  #[cfg(mobile)]
  pub fn run_mobile_plugin<T: serde::de::DeserializeOwned, E: serde::de::DeserializeOwned>(
    &self,
    method: impl AsRef<str>,
    payload: impl serde::Serialize,
  ) -> crate::Result<StdResult<T, E>> {
    #[cfg(target_os = "ios")]
    {
      Ok(self.run_ios_plugin(method, payload))
    }
    #[cfg(target_os = "android")]
    {
      self.run_android_plugin(method, payload).map_err(Into::into)
    }
  }

  /// Executes the given iOS method.
  #[cfg(target_os = "ios")]
  fn run_ios_plugin<T: serde::de::DeserializeOwned, E: serde::de::DeserializeOwned>(
    &self,
    method: impl AsRef<str>,
    payload: impl serde::Serialize,
  ) -> StdResult<T, E> {
    use std::{
      ffi::CStr,
      os::raw::{c_char, c_int},
      sync::mpsc::channel,
    };

    let id: i32 = rand::random();
    let (tx, rx) = channel();
    PENDING_PLUGIN_CALLS
      .get_or_init(Default::default)
      .lock()
      .unwrap()
      .insert(
        id,
        Box::new(move |arg| {
          tx.send(arg).unwrap();
        }),
      );

    unsafe {
      extern "C" fn plugin_method_response_handler(
        id: c_int,
        success: c_int,
        payload: *const c_char,
      ) {
        let payload = unsafe {
          assert!(!payload.is_null());
          CStr::from_ptr(payload)
        };

        if let Some(handler) = PENDING_PLUGIN_CALLS
          .get_or_init(Default::default)
          .lock()
          .unwrap()
          .remove(&id)
        {
          let payload = serde_json::from_str(payload.to_str().unwrap()).unwrap();
          handler(if success == 1 {
            Ok(payload)
          } else {
            Err(payload)
          });
        }
      }

      crate::ios::run_plugin_method(
        id,
        &self.name.into(),
        &method.as_ref().into(),
        crate::ios::json_to_dictionary(serde_json::to_value(payload).unwrap()),
        plugin_method_response_handler,
      );
    }
    rx.recv()
      .unwrap()
      .map(|r| serde_json::from_value(r).unwrap())
      .map_err(|e| serde_json::from_value(e).unwrap())
  }

  /// Executes the given Android method.
  #[cfg(target_os = "android")]
  fn run_android_plugin<T: serde::de::DeserializeOwned, E: serde::de::DeserializeOwned>(
    &self,
    method: impl AsRef<str>,
    payload: impl serde::Serialize,
  ) -> StdResult<StdResult<T, E>, jni::errors::Error> {
    use jni::{errors::Error as JniError, objects::JObject, JNIEnv};

    fn run<R: Runtime>(
      id: i32,
      plugin: &'static str,
      method: String,
      payload: serde_json::Value,
      runtime_handle: &R::Handle,
      env: JNIEnv<'_>,
      activity: JObject<'_>,
    ) -> StdResult<(), JniError> {
      let data = crate::jni_helpers::to_jsobject::<R>(env, activity, runtime_handle, payload)?;
      let plugin_manager = env
        .call_method(
          activity,
          "getPluginManager",
          "()Lapp/tauri/plugin/PluginManager;",
          &[],
        )?
        .l()?;

      env.call_method(
        plugin_manager,
        "runPluginMethod",
        "(ILjava/lang/String;Ljava/lang/String;Lapp/tauri/plugin/JSObject;)V",
        &[
          id.into(),
          env.new_string(plugin)?.into(),
          env.new_string(&method)?.into(),
          data.into(),
        ],
      )?;

      Ok(())
    }

    let handle = match self.handle.runtime() {
      RuntimeOrDispatch::Runtime(r) => r.handle(),
      RuntimeOrDispatch::RuntimeHandle(h) => h,
      _ => unreachable!(),
    };

    let id: i32 = rand::random();
    let plugin_name = self.name;
    let method = method.as_ref().to_string();
    let payload = serde_json::to_value(payload).unwrap();
    let handle_ = handle.clone();

    let (tx, rx) = std::sync::mpsc::channel();
    let tx_ = tx.clone();
    PENDING_PLUGIN_CALLS
      .get_or_init(Default::default)
      .lock()
      .unwrap()
      .insert(
        id,
        Box::new(move |arg| {
          tx.send(Ok(arg)).unwrap();
        }),
      );

    handle.run_on_android_context(move |env, activity, _webview| {
      if let Err(e) = run::<R>(id, plugin_name, method, payload, &handle_, env, activity) {
        tx_.send(Err(e)).unwrap();
      }
    });

    rx.recv().unwrap().map(|response| {
      response
        .map(|r| serde_json::from_value(r).unwrap())
        .map_err(|e| serde_json::from_value(e).unwrap())
    })
  }
}

/// Api exposed to the plugin setup hook.
#[derive(Clone)]
#[allow(dead_code)]
pub struct PluginApi<R: Runtime, C: DeserializeOwned> {
  handle: AppHandle<R>,
  name: &'static str,
  config: C,
}

impl<R: Runtime, C: DeserializeOwned> PluginApi<R, C> {
  /// Returns the plugin configuration.
  pub fn config(&self) -> &C {
    &self.config
  }

  /// Registers an iOS plugin.
  #[cfg(target_os = "ios")]
  pub fn register_ios_plugin(
    &self,
    init_fn: unsafe extern "C" fn(cocoa::base::id),
  ) -> crate::Result<PluginHandle<R>> {
    if let Some(window) = self.windows().values().next() {
      window.with_webview(move |w| {
        unsafe { init_fn(w.inner()) };
      })?;
    } else {
      unsafe { init_fn(cocoa::base::nil) };
    }
    Ok(PluginHandle {
      name: self.name,
      handle: self.handle.clone(),
    })
  }

  /// Registers an Android plugin.
  #[cfg(target_os = "android")]
  pub fn register_android_plugin(
    &self,
    plugin_identifier: &str,
    class_name: &str,
  ) -> crate::Result<PluginHandle<R>> {
    use jni::{errors::Error as JniError, objects::JObject, JNIEnv};

    fn initialize_plugin<'a, R: Runtime>(
      env: JNIEnv<'a>,
      activity: JObject<'a>,
      webview: JObject<'a>,
      runtime_handle: &R::Handle,
      plugin_name: &'static str,
      plugin_class: String,
    ) -> StdResult<(), JniError> {
      let plugin_manager = env
        .call_method(
          activity,
          "getPluginManager",
          format!("()Lapp/tauri/plugin/PluginManager;"),
          &[],
        )?
        .l()?;

      // instantiate plugin
      let plugin_class = runtime_handle.find_class(env, activity, plugin_class)?;
      let plugin = env.new_object(
        plugin_class,
        "(Landroid/app/Activity;)V",
        &[activity.into()],
      )?;

      // load plugin
      env.call_method(
        plugin_manager,
        "load",
        format!("(Landroid/webkit/WebView;Ljava/lang/String;Lapp/tauri/plugin/Plugin;)V"),
        &[
          webview.into(),
          env.new_string(plugin_name)?.into(),
          plugin.into(),
        ],
      )?;

      Ok(())
    }

    let plugin_class = format!("{}/{}", plugin_identifier.replace(".", "/"), class_name);
    let plugin_name = self.name;
    let runtime_handle = self.handle.runtime_handle.clone();
    self
      .handle
      .runtime_handle
      .run_on_android_context(move |env, activity, webview| {
        let _ = initialize_plugin::<R>(
          env,
          activity,
          webview,
          &runtime_handle,
          plugin_name,
          plugin_class,
        );
      });

    Ok(PluginHandle {
      name: self.name,
      handle: self.handle.clone(),
    })
  }
}

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
  setup: Option<Box<SetupHook<R, C>>>,
  js_init_script: Option<String>,
  on_page_load: Box<OnPageLoad<R>>,
  on_webview_ready: Box<OnWebviewReady<R>>,
  on_event: Box<OnEvent<R>>,
  on_drop: Option<Box<OnDrop<R>>>,
}

impl<R: Runtime, C: DeserializeOwned> Builder<R, C> {
  /// Creates a new Plugin builder.
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      setup: None,
      js_init_script: None,
      invoke_handler: Box::new(|_| false),
      on_page_load: Box::new(|_, _| ()),
      on_webview_ready: Box::new(|_| ()),
      on_event: Box::new(|_, _| ()),
      on_drop: None,
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
    F: Fn(Invoke<R>) -> bool + Send + Sync + 'static,
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
  ///   .setup(|app, api| {
  ///     app.manage(PluginState::default());
  ///
  ///     Ok(())
  ///   })
  ///   .build()
  /// }
  /// ```
  #[must_use]
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: FnOnce(&AppHandle<R>, PluginApi<R, C>) -> Result<()> + Send + 'static,
  {
    self.setup.replace(Box::new(setup));
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

  /// Builds the [TauriPlugin].
  pub fn build(self) -> TauriPlugin<R, C> {
    TauriPlugin {
      name: self.name,
      app: None,
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      js_init_script: self.js_init_script,
      on_page_load: self.on_page_load,
      on_webview_ready: self.on_webview_ready,
      on_event: self.on_event,
      on_drop: self.on_drop,
    }
  }
}

/// Plugin struct that is returned by the [`Builder`]. Should only be constructed through the builder.
pub struct TauriPlugin<R: Runtime, C: DeserializeOwned = ()> {
  name: &'static str,
  app: Option<AppHandle<R>>,
  invoke_handler: Box<InvokeHandler<R>>,
  setup: Option<Box<SetupHook<R, C>>>,
  js_init_script: Option<String>,
  on_page_load: Box<OnPageLoad<R>>,
  on_webview_ready: Box<OnWebviewReady<R>>,
  on_event: Box<OnEvent<R>>,
  on_drop: Option<Box<OnDrop<R>>>,
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
      (s)(
        app,
        PluginApi {
          name: self.name,
          handle: app.clone(),
          config: serde_json::from_value(config)?,
        },
      )?;
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

  fn extend_api(&mut self, invoke: Invoke<R>) -> bool {
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
  pub fn register<P: Plugin<R> + 'static>(&mut self, plugin: P) -> bool {
    self.store.insert(plugin.name(), Box::new(plugin)).is_some()
  }

  /// Removes the plugin with the given name from the store.
  pub fn unregister(&mut self, plugin: &'static str) -> bool {
    self.store.remove(plugin).is_some()
  }

  /// Initializes all plugins in the store.
  pub(crate) fn initialize(
    &mut self,
    app: &AppHandle<R>,
    config: &PluginConfig,
  ) -> crate::Result<()> {
    self.store.values_mut().try_for_each(|plugin| {
      plugin
        .initialize(
          app,
          config.0.get(plugin.name()).cloned().unwrap_or_default(),
        )
        .map_err(|e| crate::Error::PluginInitialization(plugin.name().to_string(), e.to_string()))
    })
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
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.created(window.clone()))
  }

  /// Runs the on_page_load hook for all plugins in the store.
  pub(crate) fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.on_page_load(window.clone(), payload.clone()))
  }

  /// Runs the on_event hook for all plugins in the store.
  pub(crate) fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.on_event(app, event))
  }

  /// Runs the plugin `extend_api` hook if it exists. Returns whether the invoke message was handled or not.
  ///
  /// The message is not handled when the plugin exists **and** the command does not.
  pub(crate) fn extend_api(&mut self, plugin: &str, invoke: Invoke<R>) -> bool {
    if let Some(plugin) = self.store.get_mut(plugin) {
      plugin.extend_api(invoke)
    } else {
      invoke.resolver.reject(format!("plugin {plugin} not found"));
      true
    }
  }
}
