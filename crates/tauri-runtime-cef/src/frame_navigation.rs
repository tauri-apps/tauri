// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Native document generations shared by every CEF webview.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use std::sync::{Arc, Mutex};

const MAX_NATIVE_FRAMES: usize = 256;
const MAX_NATIVE_FRAME_ID_BYTES: usize = 512;

#[derive(Debug, Default)]
struct NativeFrameState {
  browser_id: Option<i32>,
  generation: u64,
  exhausted: bool,
  loading: bool,
  observed_load_state: bool,
  main_frame: Option<String>,
  frames: BTreeMap<String, bool>,
}

/// Read-only navigation state for one exact native CEF browser lifetime. The CEF UI thread
/// advances it before input submission can be admitted on that same thread.
/// No lock is held while calling CEF or awaiting a renderer response.
#[derive(Clone, Debug)]
pub struct FrameNavigationState {
  state: Arc<Mutex<NativeFrameState>>,
}

/// Opaque process-local proof of an observed native browser/document lifetime.
/// Compare it with `WebviewSnapshot::document` in the final UI-thread callback
/// before an effect. A token alone does not authorize an account or profile.
#[derive(Clone)]
pub struct NativeDocumentToken {
  state: Arc<Mutex<NativeFrameState>>,
  generation: u64,
}

impl PartialEq for NativeDocumentToken {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.state, &other.state) && self.generation == other.generation
  }
}
impl Eq for NativeDocumentToken {}

impl std::fmt::Debug for NativeDocumentToken {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("NativeDocumentToken")
      .finish_non_exhaustive()
  }
}

impl FrameNavigationState {
  pub(crate) fn new() -> Self {
    Self {
      state: Arc::default(),
    }
  }

  /// Identifies the same native browser lifetime independently of navigation.
  /// A replacement browser never matches, even if a caller reuses its label.
  pub fn is_same_browser(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.state, &other.state)
  }

  pub(crate) fn observe_document(&self, browser: &cef::Browser) -> Option<NativeDocumentToken> {
    use cef::ImplBrowser;
    let generation = self.ready_generation()?;
    if browser.is_valid() == 0 || browser.frame_count() > MAX_NATIVE_FRAMES {
      return None;
    }
    let mut identifiers = cef::CefStringList::new();
    browser.frame_identifiers(Some(&mut identifiers));
    self
      .admits_native_frames(
        generation,
        browser.identifier(),
        browser.is_loading() != 0,
        &identifiers.into_iter().collect(),
      )
      .then(|| NativeDocumentToken {
        state: Arc::clone(&self.state),
        generation,
      })
  }

  /// Captures an observed document only when every known frame is attached and
  /// native load completion has been observed. Compare with the final native
  /// snapshot before dispatch; this read does not query current CEF frame IDs.
  pub fn document(&self) -> Option<NativeDocumentToken> {
    self
      .ready_generation()
      .map(|generation| NativeDocumentToken {
        state: Arc::clone(&self.state),
        generation,
      })
  }

  fn ready_generation(&self) -> Option<u64> {
    let state = self.state.lock().ok()?;
    Self::ready(&state).then_some(state.generation)
  }

  fn ready(state: &NativeFrameState) -> bool {
    !state.exhausted
      && state.observed_load_state
      && !state.loading
      && !state.frames.is_empty()
      && state.frames.values().all(|attached| *attached)
      && state
        .main_frame
        .as_ref()
        .is_some_and(|id| state.frames.get(id) == Some(&true))
  }

  pub(crate) fn on_frame_event(&self, event: &crate::FrameEvent) {
    self.apply(
      event.browser_id,
      &event.frame_id,
      event.is_main,
      &event.kind,
    );
  }

  fn apply(&self, browser_id: i32, frame_id: &str, is_main: bool, kind: &crate::FrameEventKind) {
    use crate::FrameEventKind;

    let Ok(mut state) = self.state.lock() else {
      return;
    };
    if state.exhausted {
      return;
    }
    // Runtime-managed popups can inherit an opener's event handlers. Their
    // browser identity never advances or grants authority over the opener.
    if state
      .browser_id
      .is_some_and(|expected| expected != browser_id)
    {
      return;
    }
    let browser_wide = matches!(
      kind,
      FrameEventKind::MainFrameChanged
        | FrameEventKind::LoadingStateChanged { .. }
        | FrameEventKind::RendererTerminated
    );
    if frame_id.is_empty() && !browser_wide || frame_id.len() > MAX_NATIVE_FRAME_ID_BYTES {
      state.exhausted = true;
      return;
    }
    state.browser_id = Some(browser_id);
    let Some(generation) = state.generation.checked_add(1) else {
      state.exhausted = true;
      return;
    };
    state.generation = generation;
    match kind {
      FrameEventKind::Created => {
        state.frames.insert(frame_id.to_string(), false);
      }
      FrameEventKind::Attached => {
        state.frames.insert(frame_id.to_string(), true);
      }
      FrameEventKind::Detached | FrameEventKind::Destroyed => {
        state.frames.remove(frame_id);
        if state.main_frame.as_deref() == Some(frame_id) {
          state.main_frame = None;
        }
      }
      FrameEventKind::MainFrameChanged => {
        state.main_frame = is_main.then(|| frame_id.to_string());
      }
      FrameEventKind::NavigationStarted { .. } => {
        state.loading = true;
      }
      FrameEventKind::LoadingStateChanged { is_loading } => {
        state.observed_load_state = true;
        state.loading = *is_loading;
      }
      FrameEventKind::DocumentCommitted { .. }
      | FrameEventKind::NavigationFailed { .. }
      | FrameEventKind::AddressChanged { .. } => {}
      FrameEventKind::RendererTerminated => {
        state.frames.clear();
        state.main_frame = None;
        state.loading = true;
        state.observed_load_state = false;
      }
    }
    if state.frames.len() > MAX_NATIVE_FRAMES {
      state.exhausted = true;
    }
  }

  /// Final native admission compares the exact current CEF frame identities,
  /// not just their count. A replacement cannot reuse a document capability.
  pub(crate) fn admits_native_frames(
    &self,
    expected: u64,
    browser_id: i32,
    is_loading: bool,
    frame_ids: &BTreeSet<String>,
  ) -> bool {
    let Ok(state) = self.state.lock() else {
      return false;
    };
    Self::ready(&state)
      && state.generation == expected
      && state.browser_id == Some(browser_id)
      && !is_loading
      && state.frames.keys().eq(frame_ids.iter())
  }
}

#[cfg(test)]
mod tests {
  use crate::FrameEventKind as Event;

  use super::*;

  fn apply(barrier: &FrameNavigationState, frame: &str, kind: Event) {
    barrier.apply(1, frame, frame == "main", &kind);
  }

  fn ready_main() -> FrameNavigationState {
    let barrier = FrameNavigationState::new();
    apply(&barrier, "main", Event::Created);
    apply(&barrier, "main", Event::MainFrameChanged);
    apply(&barrier, "main", Event::Attached);
    apply(
      &barrier,
      "main",
      Event::LoadingStateChanged { is_loading: false },
    );
    barrier
  }

  fn admits(barrier: &FrameNavigationState, generation: u64, frames: &[&str]) -> bool {
    barrier.admits_native_frames(
      generation,
      1,
      false,
      &frames.iter().map(|id| id.to_string()).collect(),
    )
  }

  #[test]
  fn child_navigation_replacement_and_detach_revoke_prior_documents() {
    let barrier = ready_main();
    let main_generation = barrier.ready_generation().unwrap();
    assert!(admits(&barrier, main_generation, &["main"]));
    apply(&barrier, "child-a", Event::Created);
    assert!(barrier.ready_generation().is_none());
    apply(&barrier, "child-a", Event::Attached);
    let child_generation = barrier.ready_generation().unwrap();
    assert!(!admits(&barrier, main_generation, &["main", "child-a"]));
    assert!(admits(&barrier, child_generation, &["main", "child-a"]));
    let url = url::Url::parse("https://example.test/frame").unwrap();
    apply(
      &barrier,
      "child-a",
      Event::NavigationStarted { url: url.clone() },
    );
    assert!(barrier.ready_generation().is_none());
    apply(&barrier, "child-a", Event::DocumentCommitted { url });
    assert!(barrier.ready_generation().is_none());
    apply(
      &barrier,
      "main",
      Event::LoadingStateChanged { is_loading: false },
    );
    assert!(!admits(&barrier, child_generation, &["main", "child-a"]));
    let loaded_generation = barrier.ready_generation().unwrap();
    assert!(admits(&barrier, loaded_generation, &["main", "child-a"]));
    // Even an equal native frame count must reject different identities.
    assert!(!admits(&barrier, loaded_generation, &["main", "child-b"]));
    apply(&barrier, "child-a", Event::Detached);
    apply(&barrier, "child-b", Event::Created);
    apply(&barrier, "child-b", Event::Attached);
    assert!(!admits(&barrier, loaded_generation, &["main", "child-b"]));
    let replacement_generation = barrier.ready_generation().unwrap();
    assert!(admits(
      &barrier,
      replacement_generation,
      &["main", "child-b"]
    ));
    apply(&barrier, "child-b", Event::Detached);
    assert!(!admits(&barrier, replacement_generation, &["main"]));
    assert!(admits(
      &barrier,
      barrier.ready_generation().unwrap(),
      &["main"]
    ));
  }

  #[test]
  fn pending_cancelled_navigation_waits_for_native_loading_completion() {
    let barrier = ready_main();
    let url = url::Url::parse("https://example.test/repeated").unwrap();
    apply(
      &barrier,
      "main",
      Event::NavigationStarted { url: url.clone() },
    );
    apply(
      &barrier,
      "main",
      Event::NavigationStarted { url: url.clone() },
    );
    apply(&barrier, "main", Event::NavigationFailed { url });
    assert!(barrier.ready_generation().is_none());
    apply(
      &barrier,
      "main",
      Event::LoadingStateChanged { is_loading: false },
    );
    assert!(barrier.ready_generation().is_some());
  }

  #[test]
  fn foreign_browser_unobserved_frames_and_exhaustion_fail_closed() {
    let barrier = ready_main();
    let generation = barrier.ready_generation().unwrap();
    barrier.apply(2, "popup", true, &Event::Created);
    assert_eq!(barrier.ready_generation(), Some(generation));
    assert!(!barrier.admits_native_frames(
      generation,
      2,
      false,
      &BTreeSet::from(["main".to_string()])
    ));
    assert!(!admits(&barrier, generation, &["main", "unobserved"]));
    barrier.state.lock().unwrap().generation = u64::MAX;
    apply(&barrier, "main", Event::Attached);
    assert!(barrier.ready_generation().is_none());
    apply(
      &barrier,
      "main",
      Event::LoadingStateChanged { is_loading: false },
    );
    assert!(barrier.ready_generation().is_none());
  }
  #[test]
  fn missing_main_frame_and_renderer_termination_revoke_prior_generations() {
    let barrier = ready_main();
    let generation = barrier.ready_generation().unwrap();
    barrier.apply(1, "", false, &Event::MainFrameChanged);
    assert!(barrier.ready_generation().is_none());
    apply(&barrier, "main", Event::MainFrameChanged);
    assert_ne!(barrier.ready_generation().unwrap(), generation);
    barrier.apply(1, "", false, &Event::RendererTerminated);
    assert!(barrier.ready_generation().is_none());
    barrier.apply(
      1,
      "",
      false,
      &Event::LoadingStateChanged { is_loading: false },
    );
    assert!(barrier.ready_generation().is_none());
    apply(&barrier, "replacement", Event::Created);
    apply(&barrier, "replacement", Event::Attached);
    barrier.apply(1, "replacement", true, &Event::MainFrameChanged);
    assert_ne!(barrier.ready_generation().unwrap(), generation);
  }
  #[test]
  fn document_tokens_cannot_cross_native_lifetimes_or_navigation() {
    let first = ready_main();
    let second = ready_main();
    assert_eq!(first.ready_generation(), second.ready_generation());
    assert!(first.is_same_browser(&first.clone()));
    assert!(!first.is_same_browser(&second));
    let before = first.document().unwrap();
    assert_eq!(before, first.clone().document().unwrap());
    assert_ne!(before, second.document().unwrap());
    apply(&first, "child", Event::Created);
    assert!(first.document().is_none());
    apply(&first, "child", Event::Attached);
    assert_ne!(before, first.document().unwrap());
  }
}
