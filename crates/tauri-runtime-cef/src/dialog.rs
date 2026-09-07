// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Native protocol observations of CEF's existing JavaScript dialog UI.
//! No custom dialog handler, callback, message text, or prompt value is retained.
//!
//! These events reach the runtime through the DevTools `Page` domain it enables
//! on every browser. Enabling that domain observes dialogs without taking them
//! over: Chromium notifies each enabled `PageHandler` and *then* still runs the
//! browser's own dialog manager, and CEF always supplies one for its browsers.
//! A dialog only stalls a page when no browser handler exists at all, which the
//! protocol itself reports as `hasBrowserHandler == false`. The runtime
//! therefore never has to answer a dialog with `Page.handleJavaScriptDialog`.

use crate::{FrameNavigationState, NativeDocumentToken};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum NativeDialogKind {
  Alert,
  Confirm,
  Prompt,
  BeforeUnload,
}

/// Opaque lifetime of one native JavaScript dialog notification. Compare with
/// the current UI-thread snapshot before submitting any dialog action.
#[derive(Clone)]
pub struct NativeDialogToken(Arc<()>);
impl PartialEq for NativeDialogToken {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0)
  }
}
impl Eq for NativeDialogToken {}
impl std::fmt::Debug for NativeDialogToken {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NativeDialogToken").finish_non_exhaustive()
  }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct NativeDialogSnapshot {
  pub token: NativeDialogToken,
  pub kind: NativeDialogKind,
  /// Chromium reports a non-DevTools dialog handler. This does not establish
  /// native visibility, the exact button label, or the current prompt value.
  pub has_browser_handler: bool,
}

#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct NativeDialogObservation {
  /// A dialog event was observed for this document. False never means absent.
  pub known: bool,
  pub dialog: Option<NativeDialogSnapshot>,
}

#[derive(Default)]
struct ObservedDialog {
  document: Option<NativeDocumentToken>,
  observation: NativeDialogObservation,
}

#[derive(Clone)]
pub(crate) struct DialogState {
  browser: FrameNavigationState,
  state: Arc<Mutex<ObservedDialog>>,
}
impl DialogState {
  pub(crate) fn new(browser: FrameNavigationState) -> Self {
    Self {
      browser,
      state: Arc::default(),
    }
  }
  pub(crate) fn accepts_browser(&self, id: i32) -> bool {
    self.browser.has_browser_id(id)
  }
  pub(crate) fn snapshot(&self, document: Option<&NativeDocumentToken>) -> NativeDialogObservation {
    self
      .state
      .lock()
      .ok()
      .filter(|state| state.document.as_ref() == document)
      .map(|state| state.observation.clone())
      .unwrap_or_default()
  }
  pub(crate) fn on_event(&self, method: &str, params: &[u8]) {
    let observation = match method {
      "Page.javascriptDialogOpening" => {
        #[derive(serde::Deserialize)]
        struct Opening {
          #[serde(rename = "type")]
          kind: NativeDialogKind,
          #[serde(rename = "hasBrowserHandler")]
          has_browser_handler: bool,
        }
        // Dialog payloads contain arbitrary page text. Bound parsing and retain
        // only the native kind/handler facts; malformed input revokes old refs.
        let opening = (params.len() <= 262_144)
          .then(|| serde_json::from_slice::<Opening>(params).ok())
          .flatten();
        opening
          .map(|opening| NativeDialogObservation {
            known: true,
            dialog: Some(NativeDialogSnapshot {
              token: NativeDialogToken(Arc::new(())),
              kind: opening.kind,
              has_browser_handler: opening.has_browser_handler,
            }),
          })
          .unwrap_or_default()
      }
      "Page.javascriptDialogClosed" => NativeDialogObservation {
        known: true,
        dialog: None,
      },
      _ => return,
    };
    let document = self.browser.document();
    if let Ok(mut state) = self.state.lock() {
      *state = ObservedDialog {
        document,
        observation,
      };
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn ready() -> FrameNavigationState {
    let browser = FrameNavigationState::new();
    for kind in [
      crate::FrameEventKind::Created,
      crate::FrameEventKind::Attached,
      crate::FrameEventKind::MainFrameChanged,
      crate::FrameEventKind::LoadingStateChanged { is_loading: false },
    ] {
      browser.on_frame_event(&crate::FrameEvent {
        browser_id: 1,
        frame_id: "main".into(),
        is_main: true,
        kind,
      });
    }
    browser
  }
  const CONFIRM: &[u8] = br#"{"type":"confirm","hasBrowserHandler":true,"message":"non-secret fixture","defaultPrompt":"omitted fixture"}"#;
  #[test]
  fn exact_dialog_lifetimes_rotate_on_reopen_and_do_not_cross_browsers() {
    let browser = ready();
    let state = DialogState::new(browser.clone());
    let document = browser.document().unwrap();
    assert!(!state.snapshot(Some(&document)).known);
    state.on_event("Page.javascriptDialogOpening", CONFIRM);
    let first = state.snapshot(Some(&document)).dialog.unwrap();
    assert_eq!(first.kind, NativeDialogKind::Confirm);
    assert!(first.has_browser_handler);
    state.on_event("Page.javascriptDialogClosed", b"unread prompt value");
    assert!(state.snapshot(Some(&document)).known);
    assert!(state.snapshot(Some(&document)).dialog.is_none());
    state.on_event("Page.javascriptDialogOpening", CONFIRM);
    assert_ne!(
      state.snapshot(Some(&document)).dialog.unwrap().token,
      first.token
    );
    let other = DialogState::new(ready());
    other.on_event("Page.javascriptDialogOpening", CONFIRM);
    assert_ne!(
      other
        .state
        .lock()
        .unwrap()
        .observation
        .dialog
        .as_ref()
        .unwrap()
        .token,
      first.token
    );
    assert!(!state.accepts_browser(2));
  }
  #[test]
  fn navigation_unknown_protocol_and_missing_handler_facts_fail_closed() {
    let browser = ready();
    let state = DialogState::new(browser.clone());
    let document = browser.document().unwrap();
    state.on_event("Page.javascriptDialogOpening", CONFIRM);
    browser.close();
    assert!(!state.snapshot(browser.document().as_ref()).known);
    state.on_event("Page.javascriptDialogOpening", br#"{"type":"confirm"}"#);
    assert!(!state.snapshot(None).known);
    state.on_event(
      "Page.javascriptDialogOpening",
      br#"{"type":"unknown","hasBrowserHandler":true}"#,
    );
    assert!(state.snapshot(None).dialog.is_none());
    assert!(state.snapshot(Some(&document)).dialog.is_none());
  }
}
