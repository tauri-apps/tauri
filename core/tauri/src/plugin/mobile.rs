// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{PluginApi, PluginHandle};

use crate::Runtime;
#[cfg(target_os = "android")]
use crate::{
  runtime::RuntimeHandle,
  sealed::{ManagerBase, RuntimeOrDispatch},
};

use once_cell::sync::OnceCell;
use serde::de::DeserializeOwned;

use std::{
  collections::HashMap,
  fmt,
  sync::{mpsc::channel, Mutex},
};

type PendingPluginCallHandler =
  Box<dyn FnOnce(Result<serde_json::Value, serde_json::Value>) + Send + 'static>;

static PENDING_PLUGIN_CALLS: OnceCell<Mutex<HashMap<i32, PendingPluginCallHandler>>> =
  OnceCell::new();

/// Possible errors when invoking a plugin.
#[derive(Debug, thiserror::Error)]
pub enum PluginInvokeError {
  /// Failed to reach platform webview handle.
  #[error("the webview is unreachable")]
  UnreachableWebview,
  /// JNI error.
  #[cfg(target_os = "android")]
  #[error("jni error: {0}")]
  Jni(#[from] jni::errors::Error),
  /// Error returned from direct mobile plugin invoke.
  #[error(transparent)]
  InvokeRejected(#[from] ErrorResponse),
  /// Failed to deserialize response.
  #[error("failed to deserialize response: {0}")]
  CannotDeserializeResponse(serde_json::Error),
}

/// Glue between Rust and the Kotlin code that sends the plugin response back.
#[cfg(target_os = "android")]
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

/// Error response from the Kotlin and Swift backends.
#[derive(Debug, thiserror::Error, Clone, serde::Deserialize)]
pub struct ErrorResponse<T = ()> {
  /// Error code.
  pub code: Option<String>,
  /// Error message.
  pub message: Option<String>,
  /// Optional error data.
  #[serde(flatten)]
  pub data: T,
}

impl<T> fmt::Display for ErrorResponse<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(code) = &self.code {
      write!(f, "[{code}]")?;
      if self.message.is_some() {
        write!(f, " - ")?;
      }
    }
    if let Some(message) = &self.message {
      write!(f, "{message}")?;
    }
    Ok(())
  }
}

impl<R: Runtime, C: DeserializeOwned> PluginApi<R, C> {
  /// Registers an iOS plugin.
  #[cfg(target_os = "ios")]
  pub fn register_ios_plugin(
    &self,
    init_fn: unsafe fn() -> *const std::ffi::c_void,
  ) -> Result<PluginHandle<R>, PluginInvokeError> {
    if let Some(window) = self.handle.manager.windows().values().next() {
      let (tx, rx) = channel();
      let name = self.name;
      let config = self.raw_config.clone();
      window
        .with_webview(move |w| {
          unsafe {
            crate::ios::register_plugin(
              &name.into(),
              init_fn(),
              crate::ios::json_to_dictionary(&config) as _,
              w.inner() as _,
            )
          };
          tx.send(()).unwrap();
        })
        .map_err(|_| PluginInvokeError::UnreachableWebview)?;
      rx.recv().unwrap();
    } else {
      unsafe {
        crate::ios::register_plugin(
          &self.name.into(),
          init_fn(),
          crate::ios::json_to_dictionary(&self.raw_config) as _,
          std::ptr::null(),
        )
      };
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
  ) -> Result<PluginHandle<R>, PluginInvokeError> {
    use jni::{errors::Error as JniError, objects::JObject, JNIEnv};

    fn initialize_plugin<'a, R: Runtime>(
      env: JNIEnv<'a>,
      activity: JObject<'a>,
      webview: JObject<'a>,
      runtime_handle: &R::Handle,
      plugin_name: &'static str,
      plugin_class: String,
      plugin_config: &serde_json::Value,
    ) -> Result<(), JniError> {
      let plugin_manager = env
        .call_method(
          activity,
          "getPluginManager",
          "()Lapp/tauri/plugin/PluginManager;",
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
        "(Landroid/webkit/WebView;Ljava/lang/String;Lapp/tauri/plugin/Plugin;Lapp/tauri/plugin/JSObject;)V",
        &[
          webview.into(),
          env.new_string(plugin_name)?.into(),
          plugin.into(),
          crate::jni_helpers::to_jsobject::<R>(env, activity, runtime_handle, plugin_config)?
        ],
      )?;

      Ok(())
    }

    let plugin_class = format!("{}/{}", plugin_identifier.replace('.', "/"), class_name);
    let plugin_name = self.name;
    let plugin_config = self.raw_config.clone();
    let runtime_handle = self.handle.runtime_handle.clone();
    let (tx, rx) = channel();
    self
      .handle
      .runtime_handle
      .run_on_android_context(move |env, activity, webview| {
        let result = initialize_plugin::<R>(
          env,
          activity,
          webview,
          &runtime_handle,
          plugin_name,
          plugin_class,
          &plugin_config,
        );
        tx.send(result).unwrap();
      });

    rx.recv().unwrap()?;

    Ok(PluginHandle {
      name: self.name,
      handle: self.handle.clone(),
    })
  }
}

impl<R: Runtime> PluginHandle<R> {
  /// Executes the given mobile method.
  pub fn run_mobile_plugin<T: serde::de::DeserializeOwned>(
    &self,
    method: impl AsRef<str>,
    payload: impl serde::Serialize,
  ) -> Result<T, PluginInvokeError> {
    #[cfg(target_os = "ios")]
    {
      self.run_ios_plugin(method, payload).map_err(Into::into)
    }
    #[cfg(target_os = "android")]
    {
      self.run_android_plugin(method, payload).map_err(Into::into)
    }
  }

  // Executes the given iOS method.
  #[cfg(target_os = "ios")]
  fn run_ios_plugin<T: serde::de::DeserializeOwned>(
    &self,
    method: impl AsRef<str>,
    payload: impl serde::Serialize,
  ) -> Result<T, PluginInvokeError> {
    use std::{
      ffi::CStr,
      os::raw::{c_char, c_int},
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
        crate::ios::json_to_dictionary(&serde_json::to_value(payload).unwrap()) as _,
        crate::ios::PluginMessageCallback(plugin_method_response_handler),
      );
    }

    let response = rx.recv().unwrap();
    match response {
      Ok(r) => serde_json::from_value(r).map_err(PluginInvokeError::CannotDeserializeResponse),
      Err(r) => Err(
        serde_json::from_value::<ErrorResponse>(r)
          .map(Into::into)
          .map_err(PluginInvokeError::CannotDeserializeResponse)?,
      ),
    }
  }

  // Executes the given Android method.
  #[cfg(target_os = "android")]
  fn run_android_plugin<T: serde::de::DeserializeOwned>(
    &self,
    method: impl AsRef<str>,
    payload: impl serde::Serialize,
  ) -> Result<T, PluginInvokeError> {
    use jni::{errors::Error as JniError, objects::JObject, JNIEnv};

    fn run<R: Runtime>(
      id: i32,
      plugin: &'static str,
      method: String,
      payload: &serde_json::Value,
      runtime_handle: &R::Handle,
      env: JNIEnv<'_>,
      activity: JObject<'_>,
    ) -> Result<(), JniError> {
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
        "runCommand",
        "(ILjava/lang/String;Ljava/lang/String;Lapp/tauri/plugin/JSObject;)V",
        &[
          id.into(),
          env.new_string(plugin)?.into(),
          env.new_string(&method)?.into(),
          data,
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

    let (tx, rx) = channel();
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
      if let Err(e) = run::<R>(id, plugin_name, method, &payload, &handle_, env, activity) {
        tx_.send(Err(e)).unwrap();
      }
    });

    let response = rx.recv().unwrap()?;
    match response {
      Ok(r) => serde_json::from_value(r).map_err(PluginInvokeError::CannotDeserializeResponse),
      Err(r) => Err(
        serde_json::from_value::<ErrorResponse>(r)
          .map(Into::into)
          .map_err(PluginInvokeError::CannotDeserializeResponse)?,
      ),
    }
  }
}
