// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use std::sync::atomic::Ordering;
use std::{
  collections::HashMap,
  fmt,
  sync::{Mutex, OnceLock, atomic::AtomicI32},
};

pub(super) type PluginResponse = Result<serde_json::Value, serde_json::Value>;

type PendingPluginCallHandler = Box<dyn FnOnce(PluginResponse) + Send + 'static>;

pub(super) static PENDING_PLUGIN_CALLS_ID: AtomicI32 = AtomicI32::new(0);
pub(super) static PENDING_PLUGIN_CALLS: OnceLock<Mutex<HashMap<i32, PendingPluginCallHandler>>> =
  OnceLock::new();

/// Possible errors when invoking a plugin.
#[derive(Debug, thiserror::Error)]
pub enum PluginInvokeError {
  /// Failed to reach platform webview handle.
  #[error("the webview is unreachable")]
  #[cfg_attr(not(mobile), allow(dead_code))]
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
  /// Failed to serialize request payload.
  #[error("failed to serialize payload: {0}")]
  #[cfg_attr(not(mobile), allow(dead_code))]
  CannotSerializePayload(serde_json::Error),
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
  #[cfg_attr(not(mobile), allow(dead_code))]
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

const ORIGIN_UNAVAILABLE: &str = "ORIGIN_UNAVAILABLE";

// Release the lock before producing or delivering the response to avoid deadlocks.
// Ignore completed calls without evaluating the response.
pub(super) fn complete_pending_call(id: i32, response: impl FnOnce() -> PluginResponse) {
  let handler = PENDING_PLUGIN_CALLS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .remove(&id);
  if let Some(handler) = handler {
    handler(response());
  }
}

// Reject the call if its dispatch closure is dropped before reaching the native bridge.
// Disarm after handing the call to the bridge.
pub(super) struct PendingCallGuard(Option<i32>);

impl PendingCallGuard {
  pub(super) fn new<F: FnOnce(PluginResponse) + Send + 'static>(handler: F) -> Self {
    let id: i32 = PENDING_PLUGIN_CALLS_ID.fetch_add(1, Ordering::Relaxed);
    PENDING_PLUGIN_CALLS
      .get_or_init(Default::default)
      .lock()
      .unwrap()
      .insert(id, Box::new(handler));
    Self(Some(id))
  }

  pub(super) fn disarm(mut self) -> i32 {
    self.0.take().expect("guard already disarmed")
  }
}

impl Drop for PendingCallGuard {
  fn drop(&mut self) {
    if let Some(id) = self.0.take() {
      complete_pending_call(id, || {
        Err(serde_json::json!({
          "code": ORIGIN_UNAVAILABLE,
          "message": "the webview that originated the call is no longer available",
        }))
      });
    }
  }
}

pub(super) fn parse_plugin_response<T: DeserializeOwned>(
  response: PluginResponse,
) -> Result<T, PluginInvokeError> {
  match response {
    Ok(r) => serde_json::from_value(r).map_err(PluginInvokeError::CannotDeserializeResponse),
    Err(r) => Err(
      serde_json::from_value::<ErrorResponse>(r)
        .map(Into::into)
        .map_err(PluginInvokeError::CannotDeserializeResponse)?,
    ),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use std::sync::mpsc::channel;

  #[test]
  fn plugin_responses_preserve_success_and_deserialization_errors() {
    assert_eq!(
      parse_plugin_response::<Vec<u32>>(Ok(json!([1, 2]))).unwrap(),
      vec![1, 2]
    );
    assert!(matches!(
      parse_plugin_response::<Vec<u32>>(Ok(json!("invalid"))),
      Err(PluginInvokeError::CannotDeserializeResponse(_))
    ));
    assert!(matches!(
      parse_plugin_response::<()>(Err(json!("invalid"))),
      Err(PluginInvokeError::CannotDeserializeResponse(_))
    ));
  }

  #[test]
  fn plugin_responses_preserve_legacy_and_contextual_rejections() {
    for code in [None, Some("ORIGIN_UNAVAILABLE"), Some("RESULT_PENDING")] {
      let response = json!({ "code": code, "message": "failure" });
      let error = parse_plugin_response::<()>(Err(response)).unwrap_err();
      let expected = code.map_or_else(
        || "failure".to_string(),
        |code| format!("[{code}] - failure"),
      );
      assert_eq!(error.to_string(), expected);
      match error {
        PluginInvokeError::InvokeRejected(error) => assert_eq!(error.code.as_deref(), code),
        error => panic!("unexpected error: {error}"),
      }
    }
  }

  #[test]
  fn pending_call_completes_once_after_releasing_the_lock() {
    let (tx, rx) = channel();
    let id = PENDING_PLUGIN_CALLS_ID.fetch_add(1, Ordering::Relaxed);
    PENDING_PLUGIN_CALLS
      .get_or_init(Default::default)
      .lock()
      .unwrap()
      .insert(
        id,
        Box::new(move |response| {
          assert!(PENDING_PLUGIN_CALLS.get().unwrap().try_lock().is_ok());
          tx.send(response).unwrap();
        }),
      );
    complete_pending_call(id, || Ok(json!(42)));
    complete_pending_call(id, || panic!("a duplicate response must not be evaluated"));
    assert_eq!(rx.try_recv().unwrap(), Ok(json!(42)));
    assert!(rx.try_recv().is_err());

    {
      let (tx, rx) = channel();
      let guard = PendingCallGuard::new(move |response| {
        assert!(PENDING_PLUGIN_CALLS.get().unwrap().try_lock().is_ok());
        tx.send(response).unwrap();
      });
      let id = guard.0.unwrap();
      drop(guard);
      let error = parse_plugin_response::<()>(rx.try_recv().unwrap()).unwrap_err();
      assert!(matches!(
        error,
        PluginInvokeError::InvokeRejected(error)
          if error.code.as_deref() == Some("ORIGIN_UNAVAILABLE")
      ));
      complete_pending_call(id, || {
        panic!("a dropped guard must complete the call only once")
      });
      assert!(rx.try_recv().is_err());

      let (tx, rx) = channel();
      let id = PendingCallGuard::new(move |response| tx.send(response).unwrap()).disarm();
      assert_eq!(rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty));
      complete_pending_call(id, || Ok(json!(42)));
      assert_eq!(rx.try_recv().unwrap(), Ok(json!(42)));
      assert!(rx.try_recv().is_err());
    }
  }
}
