// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! CEF-owned popup lifetimes. CEF keeps its native window and opener semantics;
//! the runtime owns observation, teardown, and the separate protocol observer.

use crate::{
  FrameNavigationState, NativeWindowToken,
  webview::{DevToolsProtocolHandler, Webview, WebviewSnapshot, add_dev_tools_observer},
};
use cef::*;
use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};
use tauri_runtime::dpi::{LogicalPosition, LogicalSize, Rect};

const MAX_POPUPS: usize = 128;

#[derive(Clone)]
pub(crate) struct PopupRequest {
  pub(crate) opener: FrameNavigationState,
  popup_id: i32,
  identity: Arc<()>,
}
impl PopupRequest {
  pub(crate) fn is_same(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.identity, &other.identity)
  }
}

struct Popup {
  browser: Browser,
  state: FrameNavigationState,
  opener: FrameNavigationState,
  closing: AtomicBool,
  _observer: Registration,
  dialogs: crate::dialog::DialogState,
}

pub(crate) struct PopupFamily {
  root: FrameNavigationState,
  closing: AtomicBool,
  popups: Mutex<Vec<Arc<Popup>>>,
  pending: Mutex<Vec<PopupRequest>>,
  windows: Mutex<Vec<(Window, NativeWindowToken)>>,
  pub(crate) handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
}

impl PopupFamily {
  pub(crate) fn new(root: FrameNavigationState) -> Self {
    Self {
      root,
      closing: AtomicBool::new(false),
      popups: Mutex::default(),
      pending: Mutex::default(),
      windows: Mutex::default(),
      handlers: Arc::default(),
    }
  }

  pub(crate) fn admits(&self, opener: &FrameNavigationState, browser_id: i32) -> bool {
    if self.closing.load(Ordering::Acquire) || !opener.has_browser_id(browser_id) {
      return false;
    }
    let Ok(popups) = self.popups.lock() else {
      return false;
    };
    popups.len() < MAX_POPUPS
      && (self.root.is_same_browser(opener)
        || popups.iter().any(|popup| {
          !popup.closing.load(Ordering::Acquire) && popup.state.is_same_browser(opener)
        }))
  }

  pub(crate) fn reserve(
    &self,
    opener: &FrameNavigationState,
    browser_id: i32,
    popup_id: i32,
  ) -> Option<PopupRequest> {
    if !self.admits(opener, browser_id) {
      return None;
    }
    let mut pending = self.pending.lock().ok()?;
    if pending.len() + self.popups.lock().ok()?.len() >= MAX_POPUPS
      || pending
        .iter()
        .any(|request| request.popup_id == popup_id && request.opener.is_same_browser(opener))
    {
      return None;
    }
    let request = PopupRequest {
      opener: opener.clone(),
      popup_id,
      identity: Arc::new(()),
    };
    pending.push(request.clone());
    Some(request)
  }

  pub(crate) fn abort(&self, opener: &FrameNavigationState, popup_id: i32) -> Option<PopupRequest> {
    let mut pending = self.pending.lock().ok()?;
    let index = pending
      .iter()
      .position(|request| request.popup_id == popup_id && request.opener.is_same_browser(opener))?;
    Some(pending.remove(index))
  }

  pub(crate) fn created(
    &self,
    browser: &Browser,
    request: &PopupRequest,
    state: &FrameNavigationState,
  ) {
    let reserved = self
      .pending
      .lock()
      .map(|mut pending| {
        let found = pending.iter().any(|entry| entry.is_same(request));
        pending.retain(|entry| !entry.is_same(request));
        found
      })
      .unwrap_or(false);
    let opener = &request.opener;
    let Some(host) = browser.host() else {
      return;
    };
    if !reserved || !self.admits(opener, host.opener_identifier()) {
      state.close();
      host.close_browser(1);
      return;
    }
    let dialogs = crate::dialog::DialogState::new(state.clone());
    let Some(observer) = add_dev_tools_observer(
      browser,
      self.handlers.clone(),
      Arc::default(),
      dialogs.clone(),
    ) else {
      state.close();
      host.close_browser(1);
      return;
    };
    let popup = Arc::new(Popup {
      browser: browser.clone(),
      state: state.clone(),
      opener: opener.clone(),
      closing: AtomicBool::new(false),
      _observer: observer,
      dialogs,
    });
    if let Ok(mut popups) = self.popups.lock() {
      popups.push(popup);
    } else {
      state.close();
      host.close_browser(1);
    }
  }

  /// Revoke descendants before asking CEF to close them. The native close
  /// callbacks remove their browser references; no family lock crosses CEF.
  pub(crate) fn closed(&self, state: &FrameNavigationState, browser_id: i32) {
    if !state.has_browser_id(browser_id) {
      return;
    }
    state.close();
    let root_closed = self.root.is_same_browser(state);
    if root_closed {
      self.closing.store(true, Ordering::Release);
    }
    let descendants = {
      let Ok(mut popups) = self.popups.lock() else {
        return;
      };
      popups.retain(|popup| !popup.state.is_same_browser(state));
      revoke_descendants(
        state,
        root_closed,
        popups
          .iter()
          .enumerate()
          .map(|(index, popup)| (index, &popup.state, &popup.opener, &popup.closing)),
      )
      .into_iter()
      .map(|index| Arc::clone(&popups[index]))
      .collect::<Vec<_>>()
    };
    for popup in descendants {
      if popup.browser.is_valid() != 0
        && let Some(host) = popup.browser.host()
      {
        host.close_browser(1);
      }
    }
  }

  /// Window close and app shutdown own the whole family, so teardown never
  /// depends on the root having observed a native browser identity. `closed`
  /// keeps its exact-identity guard for the per-browser native callback; a root
  /// that was exhausted before its first frame event would otherwise leave every
  /// popup open and the event loop waiting on them forever.
  pub(crate) fn close_all(&self) {
    self.closing.store(true, Ordering::Release);
    self.root.close();
    let descendants = {
      let Ok(popups) = self.popups.lock() else {
        return;
      };
      revoke_descendants(
        &self.root,
        true,
        popups
          .iter()
          .enumerate()
          .map(|(index, popup)| (index, &popup.state, &popup.opener, &popup.closing)),
      )
      .into_iter()
      .map(|index| Arc::clone(&popups[index]))
      .collect::<Vec<_>>()
    };
    for popup in descendants {
      if popup.browser.is_valid() != 0
        && let Some(host) = popup.browser.host()
      {
        host.close_browser(1);
      }
    }
  }

  pub(crate) fn observe(&self) -> Vec<Webview> {
    if self.closing.load(Ordering::Acquire) {
      return Vec::new();
    }
    let popups = self
      .popups
      .lock()
      .map(|popups| popups.clone())
      .unwrap_or_default();
    // Several popup browser tabs may share one CEF window. Window identity is
    // owned by the family, never minted independently for each tab.
    let mut windows = self
      .windows
      .lock()
      .map(|windows| windows.clone())
      .unwrap_or_default();
    windows.retain(|(window, _)| window.is_valid() != 0 && window.is_closed() == 0);
    let observations = popups
      .iter()
      .filter_map(|popup| {
        if popup.closing.load(Ordering::Acquire) || popup.browser.is_valid() == 0 {
          return None;
        }
        let mut browser = popup.browser.clone();
        let view =
          browser_view_get_for_browser(Some(&mut browser)).filter(|view| view.is_valid() != 0);
        let observed_window = view
          .as_ref()
          .and_then(ImplView::window)
          .filter(|window| window.is_valid() != 0 && window.is_closed() == 0);
        let window = observed_window.as_ref().and_then(|window| {
          if let Some((_, token)) = windows
            .iter()
            .find(|(previous, _)| window.is_same(Some(&mut View::from(previous))) != 0)
          {
            return Some(token.clone());
          }
          if windows.len() >= MAX_POPUPS {
            return None;
          }
          let token = NativeWindowToken::new();
          windows.push((window.clone(), token.clone()));
          Some(token)
        });
        let bounds = view.as_ref().map(|view| {
          let rect = view.bounds();
          Rect {
            position: LogicalPosition::new(rect.x, rect.y).into(),
            size: LogicalSize::new(rect.width, rect.height).into(),
          }
        });
        let visible = view
          .as_ref()
          .zip(observed_window.as_ref())
          .map(|(view, window)| {
            view.is_drawn() != 0 && window.is_visible() != 0 && window.is_minimized() == 0
          });
        let document = popup.state.observe_document(&browser);
        let dialogs = popup.dialogs.snapshot(document.as_ref());
        let snapshot = WebviewSnapshot {
          browser_id: browser.identifier(),
          dialogs,
          document,
          window_label: None,
          window,
          parent_matches: observed_window.as_ref().map(|_| true),
          bounds,
          visible,
        };
        let mut native = Webview::new(browser, snapshot, popup.state.clone());
        native.set_opener(popup.opener.clone());
        Some(native)
      })
      .collect();
    if let Ok(mut retained) = self.windows.lock() {
      *retained = windows;
    }
    observations
  }
}

/// The graph walk is independent of CEF calls. Revoke every descendant before
/// returning handles for native close, including when callbacks arrive out of order.
fn revoke_descendants<'a>(
  state: &FrameNavigationState,
  root_closed: bool,
  nodes: impl Iterator<
    Item = (
      usize,
      &'a FrameNavigationState,
      &'a FrameNavigationState,
      &'a AtomicBool,
    ),
  > + Clone,
) -> Vec<usize> {
  let mut revoked = vec![state.clone()];
  let mut indices = Vec::new();
  loop {
    let before = indices.len();
    for (index, state, opener, closing) in nodes.clone() {
      if !closing.load(Ordering::Acquire)
        && (root_closed || revoked.iter().any(|parent| parent.is_same_browser(opener)))
      {
        closing.store(true, Ordering::Release);
        state.close();
        revoked.push(state.clone());
        indices.push(index);
      }
    }
    if indices.len() == before {
      return indices;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn created(id: i32) -> FrameNavigationState {
    let state = FrameNavigationState::new();
    state.on_frame_event(&crate::FrameEvent {
      browser_id: id,
      frame_id: "main".into(),
      is_main: true,
      kind: crate::FrameEventKind::Created,
    });
    state
  }
  #[test]
  fn closing_an_opener_revokes_only_its_exact_descendants() {
    let root = created(1);
    let first = created(2);
    let nested = created(3);
    let sibling = created(4);
    // Deliberately put a descendant before its opener.
    let nodes = [
      (nested.clone(), first.clone(), AtomicBool::new(false)),
      (first.clone(), root.clone(), AtomicBool::new(false)),
      (sibling.clone(), root.clone(), AtomicBool::new(false)),
    ];
    let refs = || {
      nodes
        .iter()
        .enumerate()
        .map(|(index, (state, opener, closing))| (index, state, opener, closing))
    };
    assert!(revoke_descendants(&created(2), false, refs()).is_empty());
    assert_eq!(revoke_descendants(&first, false, refs()), vec![0]);
    assert!(nodes[0].2.load(Ordering::Acquire));
    assert!(!nodes[2].2.load(Ordering::Acquire));
    assert_eq!(revoke_descendants(&root, true, refs()), vec![1, 2]);
    assert!(nodes.iter().all(|node| node.2.load(Ordering::Acquire)));
    assert!(revoke_descendants(&root, true, refs()).is_empty());
  }
  #[test]
  fn pending_popups_are_bounded_and_retained_until_exact_creation_or_abort() {
    let root = created(1);
    let family = PopupFamily::new(root.clone());
    let first = family.reserve(&root, 1, 7).unwrap();
    assert!(family.reserve(&root, 1, 7).is_none());
    assert!(family.abort(&created(1), 7).is_none());
    for id in 8..(7 + MAX_POPUPS as i32) {
      assert!(family.reserve(&root, 1, id).is_some());
    }
    assert!(family.reserve(&root, 1, 1000).is_none());
    family.closed(&root, 1);
    assert_eq!(family.pending.lock().unwrap().len(), MAX_POPUPS);
    assert!(family.abort(&root, 7).unwrap().is_same(&first));
    assert!(family.abort(&root, 7).is_none());
    assert!(family.reserve(&root, 1, 1000).is_none());
  }
  #[test]
  fn popup_admission_requires_the_live_exact_native_opener() {
    let root = created(1);
    let family = PopupFamily::new(root.clone());
    assert!(family.admits(&root, 1));
    assert!(!family.admits(&root, 2));
    assert!(!family.admits(&created(1), 1));
    assert!(!family.admits(&FrameNavigationState::new(), 1));
    family.closed(&root, 2);
    assert!(family.admits(&root, 1));
    family.closed(&root, 1);
    assert!(!family.admits(&root, 1));
    assert!(family.observe().is_empty());
  }
  #[test]
  fn teardown_closes_a_family_whose_root_never_observed_a_browser_id() {
    // A root that never observed a native frame event has no browser identity.
    let root = FrameNavigationState::new();
    assert!(!root.has_browser_id(1));
    let family = PopupFamily::new(root.clone());
    // The per-browser native callback cannot match an unobserved identity.
    family.closed(&root, 1);
    assert!(!family.closing.load(Ordering::Acquire));
    // Window close and app shutdown own the family outright and must not.
    family.close_all();
    assert!(family.closing.load(Ordering::Acquire));
    assert!(family.observe().is_empty());
  }
}
