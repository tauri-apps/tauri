// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webview windows and Tauri.

use crate::{
  webview::{FileDropHandler, WebviewAttributes, WebviewRpcHandler},
  Dispatch, Runtime, WindowBuilder,
};
use serde::Serialize;
use tauri_utils::config::WindowConfig;

use std::hash::{Hash, Hasher};

/// UI scaling utilities.
pub mod dpi;

/// An event from a window.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WindowEvent {
  /// The size of the window has changed. Contains the client area's new dimensions.
  Resized(dpi::PhysicalSize<u32>),
  /// The position of the window has changed. Contains the window's new position.
  Moved(dpi::PhysicalPosition<i32>),
  /// The window has been requested to close.
  CloseRequested,
  /// The window has been destroyed.
  Destroyed,
  /// The window gained or lost focus.
  ///
  /// The parameter is true if the window has gained focus, and false if it has lost focus.
  Focused(bool),
  /// The window's scale factor has changed.
  ///
  /// The following user actions can cause DPI changes:
  ///
  /// - Changing the display's resolution.
  /// - Changing the display's scale factor (e.g. in Control Panel on Windows).
  /// - Moving the window to a display with a different scale factor.
  ScaleFactorChanged {
    /// The new scale factor.
    scale_factor: f64,
    /// The window inner size.
    new_inner_size: dpi::PhysicalSize<u32>,
  },
}

/// A menu event.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuEvent {
  pub menu_item_id: u16,
}

/// A webview window that has yet to be built.
pub struct PendingWindow<R: Runtime> {
  /// The label that the window will be named.
  pub label: String,

  /// The [`WindowBuilder`] that the window will be created with.
  pub window_builder: <R::Dispatcher as Dispatch>::WindowBuilder,

  /// The [`WebviewAttributes`] that the webview will be created with.
  pub webview_attributes: WebviewAttributes,

  /// How to handle RPC calls on the webview window.
  pub rpc_handler: Option<WebviewRpcHandler<R>>,

  /// How to handle a file dropping onto the webview window.
  pub file_drop_handler: Option<FileDropHandler<R>>,

  /// The resolved URL to load on the webview.
  pub url: String,
}

impl<R: Runtime> PendingWindow<R> {
  /// Create a new [`PendingWindow`] with a label and starting url.
  pub fn new(
    window_builder: <R::Dispatcher as Dispatch>::WindowBuilder,
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> Self {
    Self {
      window_builder,
      webview_attributes,
      label: label.into(),
      rpc_handler: None,
      file_drop_handler: None,
      url: "tauri://localhost".to_string(),
    }
  }

  /// Create a new [`PendingWindow`] from a [`WindowConfig`] with a label and starting url.
  pub fn with_config(
    window_config: WindowConfig,
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> Self {
    Self {
      window_builder: <<R::Dispatcher as Dispatch>::WindowBuilder>::with_config(window_config),
      webview_attributes,
      label: label.into(),
      rpc_handler: None,
      file_drop_handler: None,
      url: "tauri://localhost".to_string(),
    }
  }
}

/// A webview window that is not yet managed by Tauri.
pub struct DetachedWindow<R: Runtime> {
  /// Name of the window
  pub label: String,

  /// The [`Dispatch`](crate::Dispatch) associated with the window.
  pub dispatcher: R::Dispatcher,
}

impl<R: Runtime> Clone for DetachedWindow<R> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
    }
  }
}

impl<R: Runtime> Hash for DetachedWindow<R> {
  /// Only use the [`DetachedWindow`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<R: Runtime> Eq for DetachedWindow<R> {}
impl<R: Runtime> PartialEq for DetachedWindow<R> {
  /// Only use the [`DetachedWindow`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}
