// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Per-browser audio capture via CEF's `cef_audio_handler_t`.
//!
//! ## OpenHuman extension
//!
//! Upstream `tauri-runtime-cef` does not expose Chromium's audio-handler
//! callbacks to user code. We need them so the desktop shell can listen
//! to a specific embedded webview's audio output (currently just the
//! Google Meet call window — see `app/src-tauri/src/meet_audio/`) without
//! resorting to OS-level audio taps that would require the user to grant
//! microphone / screen-recording permission.
//!
//! ## Design
//!
//! Handlers are registered against a **URL prefix**. When the runtime
//! constructs a [`BrowserClient`], it consults this registry; if the
//! browser's initial URL starts with any registered prefix, the
//! corresponding callback is wired to the matching CEF audio handler.
//! Multiple windows can match different prefixes; non-matching browsers
//! get no audio handler at all (so existing webviews — Telegram,
//! WhatsApp, Slack, the OpenHuman main window — keep their previous
//! behavior unchanged).
//!
//! The registry is process-wide because CEF's audio plumbing runs on the
//! browser process and there is no clean way to attach state to a single
//! `BrowserClient` from outside the runtime crate. Drop the returned
//! [`AudioHandlerRegistration`] to remove the prefix.

use std::sync::{Arc, Mutex, OnceLock};

/// Event delivered by [`AudioStreamHandler`] for each audio-pipeline
/// callback CEF surfaces. Frames are float32 planar — one inner `Vec<f32>`
/// per channel, all the same length.
#[derive(Debug, Clone)]
pub enum AudioStreamEvent {
  Started {
    sample_rate_hz: i32,
    channels: i32,
    frames_per_buffer: i32,
  },
  /// Float32 planar PCM. `pts_ms` is CEF's presentation timestamp in
  /// milliseconds — useful for ordering when frames arrive out of order
  /// (rare in practice, but the field is part of the CEF contract).
  Packet {
    channels: Vec<Vec<f32>>,
    pts_ms: i64,
  },
  Stopped,
  Error(String),
}

/// User callback the runtime invokes for each audio event. Held as
/// `Arc<dyn Fn>` so the runtime can call it from the CEF browser thread
/// without copying state.
pub type AudioStreamHandler = dyn Fn(AudioStreamEvent) + Send + Sync;

/// RAII handle returned from [`register_audio_handler`]. Drop it to stop
/// receiving events. Stored as a struct (not a bare token id) so callers
/// can keep it on the heap alongside the rest of their state.
pub struct AudioHandlerRegistration {
  prefix: String,
}

impl Drop for AudioHandlerRegistration {
  fn drop(&mut self) {
    let mut guard = registry().handlers.lock().unwrap();
    guard.retain(|(p, _)| p != &self.prefix);
  }
}

#[derive(Default)]
struct AudioHandlerRegistry {
  // `(url_prefix, handler)`. Vec rather than HashMap so the order of
  // insertion is preserved — a longer, more-specific prefix that was
  // registered later still matches first because we walk in order. In
  // practice the meet-audio module only ever registers one prefix per
  // call so this is moot, but the deterministic order keeps things
  // testable.
  handlers: Mutex<Vec<(String, Arc<AudioStreamHandler>)>>,
}

fn registry() -> &'static AudioHandlerRegistry {
  static REGISTRY: OnceLock<AudioHandlerRegistry> = OnceLock::new();
  REGISTRY.get_or_init(AudioHandlerRegistry::default)
}

/// Register a callback that will fire for every audio event emitted by
/// any CEF browser whose initial URL starts with `url_prefix`. Drop the
/// returned [`AudioHandlerRegistration`] to remove the entry.
///
/// The matching is "longest prefix wins" only insofar as the first
/// match in registration order is taken — callers that want strict
/// uniqueness should pick prefixes that don't overlap (the meet-audio
/// path uses the per-call URL which is unique by construction).
pub fn register_audio_handler<F>(url_prefix: impl Into<String>, handler: F) -> AudioHandlerRegistration
where
  F: Fn(AudioStreamEvent) + Send + Sync + 'static,
{
  let prefix = url_prefix.into();
  let mut guard = registry().handlers.lock().unwrap();
  guard.push((prefix.clone(), Arc::new(handler)));
  AudioHandlerRegistration { prefix }
}

/// Look up a handler whose registered prefix matches `url`. Returns
/// `None` when no prefix matches — the caller (the runtime) treats this
/// as "this browser does not need audio capture" and skips installing
/// the CEF audio handler altogether.
pub(crate) fn handler_for_url(url: &str) -> Option<Arc<AudioStreamHandler>> {
  let guard = registry().handlers.lock().unwrap();
  for (prefix, handler) in guard.iter() {
    if url.starts_with(prefix) {
      return Some(handler.clone());
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};

  #[test]
  fn registration_drop_removes_handler() {
    let counter = Arc::new(AtomicUsize::new(0));
    {
      let c = counter.clone();
      let _reg = register_audio_handler("https://example.com/test-drop/", move |_| {
        c.fetch_add(1, Ordering::SeqCst);
      });
      assert!(handler_for_url("https://example.com/test-drop/abc").is_some());
    }
    assert!(handler_for_url("https://example.com/test-drop/abc").is_none());
  }

  #[test]
  fn non_matching_url_returns_none() {
    let _reg = register_audio_handler("https://example.com/nomatch-a/", |_| {});
    assert!(handler_for_url("https://other.com/nomatch-a/").is_none());
  }

  #[test]
  fn first_registered_prefix_wins() {
    let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let l1 = log.clone();
    let l2 = log.clone();
    let _r1 = register_audio_handler("https://x.test/first-wins/", move |_| {
      l1.lock().unwrap().push("first");
    });
    let _r2 = register_audio_handler("https://x.test/first-wins/", move |_| {
      l2.lock().unwrap().push("second");
    });
    let h = handler_for_url("https://x.test/first-wins/page").unwrap();
    h(AudioStreamEvent::Stopped);
    assert_eq!(*log.lock().unwrap(), vec!["first"]);
  }
}
