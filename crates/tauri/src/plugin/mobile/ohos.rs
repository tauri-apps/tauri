// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  ErrorResponse, PendingPluginCallHandler, PluginInvokeError, PENDING_PLUGIN_CALLS,
  PENDING_PLUGIN_CALLS_ID,
};
use crate::{AppHandle, Manager, Runtime};
use napi_ohos::{
  bindgen_prelude::Function,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Status,
};
use std::sync::{
  atomic::{AtomicU64, Ordering},
  Mutex,
};
use std::thread::{self, ThreadId};

struct Invoker {
  generation: u64,
  callback: ThreadsafeFunction<String, (), String, Status, false>,
  main_thread: ThreadId,
  files: std::path::PathBuf,
  cache: std::path::PathBuf,
  temp: std::path::PathBuf,
}

struct Session(u64);
static GENERATION: AtomicU64 = AtomicU64::new(1);
static INVOKER: Mutex<Option<Invoker>> = Mutex::new(None);

fn unavailable(message: &str) -> PluginInvokeError {
  ErrorResponse {
    code: Some("OHOS_PLUGIN_UNAVAILABLE".into()),
    message: Some(message.into()),
    data: (),
  }
  .into()
}

pub(super) fn register_app<R: Runtime>(app: &AppHandle<R>) -> Result<(), PluginInvokeError> {
  let invoker = INVOKER.lock().unwrap();
  let invoker = invoker
    .as_ref()
    .ok_or_else(|| unavailable("Initialize the OHOS plugin bridge before starting Tauri"))?;
  if let Some(session) = app.try_state::<Session>() {
    if session.0 != invoker.generation {
      return Err(unavailable(
        "This application handle belongs to a destroyed Ability",
      ));
    }
  } else {
    app.manage(Session(invoker.generation));
  }
  Ok(())
}

pub(super) fn ensure_worker_thread() -> Result<(), PluginInvokeError> {
  if INVOKER
    .lock()
    .unwrap()
    .as_ref()
    .is_some_and(|v| v.main_thread == thread::current().id())
  {
    return Err(unavailable(
      "Synchronous plugin calls cannot block the ArkTS thread; use run_mobile_plugin_async",
    ));
  }
  Ok(())
}

/// Called by the native Ability before RustAbility.onCreate, never by web content.
pub fn initialize_ohos_plugin_bridge(
  callback: Function<'_, String, ()>,
  files: String,
  cache: String,
  temp: String,
) -> napi_ohos::Result<()> {
  for path in [&files, &cache, &temp] {
    if !std::path::Path::new(path).is_absolute() {
      return Err(napi_ohos::Error::from_reason(
        "OHOS sandbox directories must be absolute",
      ));
    }
  }
  let callback = callback
    .build_threadsafe_function::<String>()
    .callee_handled::<false>()
    .build()?;
  let mut invoker = INVOKER.lock().unwrap();
  if invoker.is_some() {
    return Err(napi_ohos::Error::from_reason(
      "OHOS plugin bridge is already initialized",
    ));
  }
  *invoker = Some(Invoker {
    generation: GENERATION.fetch_add(1, Ordering::Relaxed),
    callback,
    main_thread: thread::current().id(),
    files: files.into(),
    cache: cache.into(),
    temp: temp.into(),
  });
  Ok(())
}

pub(super) fn invoke<R: Runtime>(
  app: &AppHandle<R>,
  name: &str,
  command: &str,
  payload: serde_json::Value,
  handler: PendingPluginCallHandler,
) -> Result<(), PluginInvokeError> {
  let invoker = INVOKER.lock().unwrap();
  let Some(invoker) = invoker.as_ref() else {
    return Err(unavailable("OHOS Ability is not active"));
  };
  if app.try_state::<Session>().map(|session| session.0) != Some(invoker.generation) {
    return Err(unavailable(
      "This application handle belongs to a destroyed Ability",
    ));
  }
  let id = PENDING_PLUGIN_CALLS_ID.fetch_add(1, Ordering::Relaxed);
  let request = serde_json::json!({ "id": id, "plugin": name, "command": command, "payload": payload.to_string() });
  let pending = PENDING_PLUGIN_CALLS.get_or_init(Default::default);
  pending.lock().unwrap().insert(id, handler);
  // Queue onto ArkTS, never call UI APIs from the Rust event-loop/IPC threads.
  let status = invoker
    .callback
    .call(request.to_string(), ThreadsafeFunctionCallMode::NonBlocking);
  if status != Status::Ok {
    pending.lock().unwrap().remove(&id);
    return Err(unavailable(&format!(
      "Could not queue OHOS plugin call: {status:?}"
    )));
  }
  Ok(())
}

/// Late replies after Ability teardown are ignored; callbacks run outside the map lock.
pub fn ohos_plugin_response(id: i32, success: bool, payload: String) -> napi_ohos::Result<()> {
  let response = match serde_json::from_str(&payload) {
    Ok(value) => {
      if success {
        Ok(value)
      } else {
        Err(value)
      }
    }
    Err(error) => {
      Err(serde_json::json!({ "code": "INVALID_RESPONSE", "message": error.to_string() }))
    }
  };
  let callback = PENDING_PLUGIN_CALLS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .remove(&id);
  if let Some(callback) = callback {
    callback(response);
  }
  Ok(())
}

/// Reject every pending call instead of leaving blocking Rust receivers waiting forever.
pub fn close_ohos_plugin_bridge() {
  INVOKER.lock().unwrap().take();
  let callbacks: Vec<_> = PENDING_PLUGIN_CALLS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .drain()
    .map(|(_, callback)| callback)
    .collect();
  for callback in callbacks {
    callback(Err(
      serde_json::json!({ "code": "ABILITY_DESTROYED", "message": "The OHOS Ability was destroyed" }),
    ));
  }
}

/// Resolves a directory supplied by the active native Ability.
pub fn ohos_plugin_directory(kind: &str) -> crate::Result<std::path::PathBuf> {
  let invoker = INVOKER.lock().unwrap();
  let invoker = invoker.as_ref().ok_or(crate::Error::UnknownPath)?;
  match kind {
    "files" => Ok(invoker.files.clone()),
    "cache" => Ok(invoker.cache.clone()),
    "temp" => Ok(invoker.temp.clone()),
    _ => Err(crate::Error::UnknownPath),
  }
}
