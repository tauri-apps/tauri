// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use super::mobile_response::{ErrorResponse, PluginInvokeError};
use super::{
  PluginApi, PluginHandle,
  mobile_response::{
    PendingCallGuard, PluginResponse, complete_pending_call, parse_plugin_response,
  },
};

use crate::{AppHandle, Runtime, Webview, ipc::Channel};
#[cfg(target_os = "android")]
use crate::{
  runtime::RuntimeHandle,
  sealed::{ManagerBase, RuntimeOrDispatch},
};

#[cfg(mobile)]
use tokio::sync::oneshot;

use serde::{Serialize, de::DeserializeOwned};

use std::{
  collections::HashMap,
  sync::{Mutex, OnceLock, mpsc::channel},
};

static CHANNELS: OnceLock<Mutex<HashMap<u32, Channel<serde_json::Value>>>> = OnceLock::new();

pub(crate) fn register_channel(channel: Channel<serde_json::Value>) {
  CHANNELS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .insert(channel.id(), channel);
}

/// Glue between Rust and the Kotlin code that sends the plugin response back.
#[cfg(target_os = "android")]
pub fn handle_android_plugin_response(
  env: &mut jni::JNIEnv<'_>,
  id: i32,
  success: jni::objects::JString<'_>,
  error: jni::objects::JString<'_>,
) {
  let (payload, is_ok): (serde_json::Value, bool) = match (
    env
      .is_same_object(&success, jni::objects::JObject::default())
      .unwrap_or_default(),
    env
      .is_same_object(&error, jni::objects::JObject::default())
      .unwrap_or_default(),
  ) {
    // both null
    (true, true) => (serde_json::Value::Null, true),
    // error null
    (false, true) => (
      serde_json::from_str(env.get_string(&success).unwrap().to_str().unwrap()).unwrap(),
      true,
    ),
    // success null
    (true, false) => (
      serde_json::from_str(env.get_string(&error).unwrap().to_str().unwrap()).unwrap(),
      false,
    ),
    // both are set - impossible in the Kotlin code
    (false, false) => unreachable!(),
  };

  // Drop the lock before invoking the handler: it delivers the command response
  // to the webview (which can block on the UI thread), and holding
  // PENDING_PLUGIN_CALLS across that call deadlocks a concurrent `run_command`.
  complete_pending_call(id, || if is_ok { Ok(payload) } else { Err(payload) });
}

/// Glue between Rust and the Kotlin code that sends the channel data.
#[cfg(target_os = "android")]
pub fn send_channel_data(
  env: &mut jni::JNIEnv<'_>,
  channel_id: i64,
  data_str: jni::objects::JString<'_>,
) {
  let data: serde_json::Value =
    serde_json::from_str(env.get_string(&data_str).unwrap().to_str().unwrap()).unwrap();

  // Clone the channel out and drop the lock before send(): send() can block
  // delivering to the webview, and holding CHANNELS across it deadlocks a
  // concurrent channel registration/send.
  let channel = CHANNELS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .get(&(channel_id as u32))
    .cloned();
  if let Some(channel) = channel {
    let _ = channel.send(data);
  }
}

impl<R: Runtime, C: DeserializeOwned> PluginApi<R, C> {
  /// Registers an iOS plugin.
  #[cfg(all(target_os = "ios", feature = "wry"))]
  pub fn register_ios_plugin(
    &self,
    init_fn: unsafe fn() -> *const std::ffi::c_void,
  ) -> Result<PluginHandle<R>, PluginInvokeError> {
    if let Some(webview) = self.handle.manager.webviews().values().next() {
      let (tx, rx) = channel();
      let name = self.name;
      let config = self.raw_config.clone();
      webview
        .with_webview(move |w| {
          unsafe {
            crate::ios::register_plugin(
              &name.into(),
              init_fn(),
              &serde_json::to_string(&config).unwrap().as_str().into(),
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
          &serde_json::to_string(&self.raw_config)
            .unwrap()
            .as_str()
            .into(),
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
    use jni::{JNIEnv, errors::Error as JniError, objects::JObject};

    fn initialize_plugin<R: Runtime>(
      env: &mut JNIEnv<'_>,
      activity: &JObject<'_>,
      webview: &JObject<'_>,
      runtime_handle: &R::Handle,
      plugin_name: &'static str,
      plugin_class: String,
      plugin_config: &serde_json::Value,
    ) -> Result<(), JniError> {
      // instantiate plugin
      let plugin_class = runtime_handle.find_class(env, activity, plugin_class)?;
      let plugin = env.new_object(
        plugin_class,
        "(Landroid/app/Activity;)V",
        &[activity.into()],
      )?;

      // load plugin

      let plugin_manager = android_plugin_manager(env, activity)?;

      let plugin_name = env.new_string(plugin_name)?;
      let config = env.new_string(serde_json::to_string(plugin_config).unwrap())?;
      env.call_method(
        plugin_manager,
        "load",
        "(Landroid/webkit/WebView;Ljava/lang/String;Lapp/tauri/plugin/Plugin;Ljava/lang/String;)V",
        &[
          webview.into(),
          (&plugin_name).into(),
          (&plugin).into(),
          (&config).into(),
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
  /// Executes the given mobile command.
  /// This is an async optimized variant of run_mobile_plugin
  pub async fn run_mobile_plugin_async<T: DeserializeOwned>(
    &self,
    command: impl AsRef<str>,
    payload: impl Serialize,
  ) -> Result<T, PluginInvokeError> {
    let (tx, rx) = oneshot::channel();
    run_command(
      self.name,
      &self.handle,
      None,
      command,
      serde_json::to_value(payload).map_err(PluginInvokeError::CannotSerializePayload)?,
      move |response| {
        tx.send(response).unwrap();
      },
    )?;

    parse_plugin_response(rx.await.unwrap())
  }

  /// Executes the given mobile command.
  pub fn run_mobile_plugin<T: DeserializeOwned>(
    &self,
    command: impl AsRef<str>,
    payload: impl Serialize,
  ) -> Result<T, PluginInvokeError> {
    let (tx, rx) = channel();
    run_command(
      self.name,
      &self.handle,
      None,
      command,
      serde_json::to_value(payload).map_err(PluginInvokeError::CannotSerializePayload)?,
      move |response| {
        tx.send(response).unwrap();
      },
    )?;

    parse_plugin_response(rx.recv().unwrap())
  }
}

#[cfg(target_os = "ios")]
pub(crate) fn run_command<R: Runtime, C: AsRef<str>, F: FnOnce(PluginResponse) + Send + 'static>(
  name: &str,
  _handle: &AppHandle<R>,
  _origin: Option<&Webview<R>>,
  command: C,
  payload: serde_json::Value,
  handler: F,
) -> Result<(), PluginInvokeError> {
  let name = name.to_string();
  let command = command.as_ref().to_string();
  let payload =
    serde_json::to_string(&payload).map_err(PluginInvokeError::CannotSerializePayload)?;
  let guard = PendingCallGuard::new(handler);

  #[cfg(feature = "wry")]
  if let Some(webview) = _origin {
    // Dropping the closure on dispatch failure lets the guard reject the call.
    let _ = webview.with_webview(move |w| {
      let id = guard.disarm();
      unsafe {
        crate::ios::run_plugin_command_with_context(
          id,
          &name.as_str().into(),
          &command.as_str().into(),
          &payload.as_str().into(),
          crate::ios::PluginMessageCallback(plugin_command_response_handler),
          crate::ios::ChannelSendDataCallback(send_channel_data_handler),
          w.view_controller() as _,
        );
      }
    });
    return Ok(());
  }

  let id = guard.disarm();
  unsafe {
    crate::ios::run_plugin_command(
      id,
      &name.as_str().into(),
      &command.as_str().into(),
      &payload.as_str().into(),
      crate::ios::PluginMessageCallback(plugin_command_response_handler),
      crate::ios::ChannelSendDataCallback(send_channel_data_handler),
    );
  }

  Ok(())
}

#[cfg(target_os = "ios")]
extern "C" fn plugin_command_response_handler(
  id: std::os::raw::c_int,
  success: std::os::raw::c_int,
  payload: *const std::os::raw::c_char,
) {
  let payload = unsafe {
    assert!(!payload.is_null());
    std::ffi::CStr::from_ptr(payload)
  };

  complete_pending_call(id, || {
    let json = payload.to_str().unwrap();
    match serde_json::from_str(json) {
      Ok(payload) => {
        if success == 1 {
          Ok(payload)
        } else {
          Err(payload)
        }
      }
      Err(err) => Err(format!("{err}, data: {}", json).into()),
    }
  });
}

#[cfg(target_os = "ios")]
extern "C" fn send_channel_data_handler(
  id: std::os::raw::c_ulonglong,
  payload: *const std::os::raw::c_char,
) {
  let payload = unsafe {
    assert!(!payload.is_null());
    std::ffi::CStr::from_ptr(payload)
  };

  // Clone the channel out and drop the lock before send(): send() can block
  // delivering to the webview, and holding CHANNELS across it deadlocks a
  // concurrent channel registration/send.
  let channel = CHANNELS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .get(&(id as u32))
    .cloned();
  if let Some(channel) = channel {
    let payload: serde_json::Value = serde_json::from_str(payload.to_str().unwrap()).unwrap();
    let _ = channel.send(payload);
  }
}

#[cfg(target_os = "android")]
fn android_plugin_manager<'local>(
  env: &mut jni::JNIEnv<'local>,
  activity: &jni::objects::JObject<'_>,
) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
  env
    .call_method(
      activity,
      "getPluginManager",
      "()Lapp/tauri/plugin/PluginManager;",
      &[],
    )?
    .l()
}

#[cfg(target_os = "android")]
pub(crate) fn run_command<R: Runtime, C: AsRef<str>, F: FnOnce(PluginResponse) + Send + 'static>(
  name: &str,
  handle: &AppHandle<R>,
  origin: Option<&Webview<R>>,
  command: C,
  payload: serde_json::Value,
  handler: F,
) -> Result<(), PluginInvokeError> {
  use jni::{JNIEnv, errors::Error as JniError, objects::JObject};

  fn run(
    id: i32,
    plugin: &str,
    command: &str,
    payload: &str,
    env: &mut JNIEnv<'_>,
    activity: &JObject<'_>,
    contextual: bool,
  ) -> Result<(), JniError> {
    let plugin = env.new_string(plugin)?;
    let command = env.new_string(command)?;
    let data = env.new_string(payload)?;
    let plugin_manager = android_plugin_manager(env, activity)?;
    let args = [
      id.into(),
      (&plugin).into(),
      (&command).into(),
      (&data).into(),
      activity.into(),
    ];
    let (method, signature, args) = if contextual {
      (
        "runCommandWithContext",
        "(ILjava/lang/String;Ljava/lang/String;Ljava/lang/String;Landroid/app/Activity;)V",
        &args[..],
      )
    } else {
      (
        "runCommand",
        "(ILjava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
        &args[..4],
      )
    };

    env.call_method(plugin_manager, method, signature, args)?;

    Ok(())
  }

  let name = name.to_string();
  let command = command.as_ref().to_string();
  let payload =
    serde_json::to_string(&payload).map_err(PluginInvokeError::CannotSerializePayload)?;
  let guard = PendingCallGuard::new(handler);
  let contextual = origin.is_some();
  let invoke = move |id, env: &mut JNIEnv<'_>, activity: &JObject<'_>| {
    if let Err(e) = run(id, &name, &command, &payload, env, activity, contextual) {
      complete_pending_call(id, || {
        Err(if contextual {
          serde_json::json!({ "message": e.to_string() })
        } else {
          e.to_string().into()
        })
      });
    }
  };

  #[cfg(feature = "wry")]
  if let Some(webview) = origin {
    // Dropping the closure on dispatch failure lets the guard reject the call.
    let _ = webview.with_webview(move |w| {
      w.jni_handle().exec(move |env, activity, _webview| {
        // Leave the guard armed if the activity is no longer available.
        if activity.is_null() {
          return;
        }
        invoke(guard.disarm(), env, activity);
      });
    });
    return Ok(());
  }

  let id = guard.disarm();
  let handle = match handle.runtime() {
    RuntimeOrDispatch::Runtime(r) => r.handle(),
    RuntimeOrDispatch::RuntimeHandle(h) => h,
    _ => unreachable!(),
  };

  handle.run_on_android_context(move |env, activity, _webview| {
    invoke(id, env, activity);
  });

  Ok(())
}

#[cfg(feature = "wry")]
impl<R: Runtime> PluginHandle<R> {
  /// Executes the given mobile command on behalf of the given webview.
  ///
  /// Unlike [`Self::run_mobile_plugin`], the native plugin gets the `UIViewController` (iOS) or
  /// `Activity` (Android) hosting that webview through `invoke.viewController` / `invoke.activity`,
  /// and `invoke.isContextual` is set. If the webview is gone before the call reaches the native
  /// plugin, the call is rejected with the `ORIGIN_UNAVAILABLE` error code.
  ///
  /// For a [`WebviewWindow`](crate::WebviewWindow), pass `window.as_ref()`.
  ///
  /// This is a blocking operation and must *NOT* be used on the UI (main) thread that hosts the
  /// webview: the call is dispatched to that thread, so waiting there would deadlock.
  pub fn run_mobile_plugin_with_webview<T: DeserializeOwned>(
    &self,
    webview: &Webview<R>,
    command: impl AsRef<str>,
    payload: impl Serialize,
  ) -> Result<T, PluginInvokeError> {
    let (tx, rx) = channel();
    run_command(
      self.name,
      &self.handle,
      Some(webview),
      command,
      serde_json::to_value(payload).map_err(PluginInvokeError::CannotSerializePayload)?,
      move |response| {
        tx.send(response).unwrap();
      },
    )?;

    parse_plugin_response(rx.recv().unwrap())
  }

  /// Async variant of [`Self::run_mobile_plugin_with_webview`].
  pub async fn run_mobile_plugin_async_with_webview<T: DeserializeOwned>(
    &self,
    webview: &Webview<R>,
    command: impl AsRef<str>,
    payload: impl Serialize,
  ) -> Result<T, PluginInvokeError> {
    let (tx, rx) = oneshot::channel();
    run_command(
      self.name,
      &self.handle,
      Some(webview),
      command,
      serde_json::to_value(payload).map_err(PluginInvokeError::CannotSerializePayload)?,
      move |response| {
        let _ = tx.send(response);
      },
    )?;

    parse_plugin_response(rx.await.unwrap())
  }
}
