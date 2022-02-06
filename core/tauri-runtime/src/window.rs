// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webview windows and Tauri.

use crate::{
  http::{Request as HttpRequest, Response as HttpResponse},
  menu::{Menu, MenuEntry, MenuHash, MenuId},
  webview::{FileDropHandler, WebviewAttributes, WebviewIpcHandler},
  Dispatch, Runtime, WindowBuilder,
};
use serde::Serialize;
use tauri_utils::config::WindowConfig;

use std::{
  collections::{HashMap, HashSet},
  hash::{Hash, Hasher},
  sync::{mpsc::Sender, Arc, Mutex},
};

type UriSchemeProtocol =
  dyn Fn(&HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> + Send + Sync + 'static;

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
  CloseRequested {
    /// The window label.
    label: String,
    /// A signal sender. If a `true` value is emitted, the window won't be closed.
    signal_tx: Sender<bool>,
  },
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuEvent {
  pub menu_item_id: u16,
}

fn get_menu_ids(map: &mut HashMap<MenuHash, MenuId>, menu: &Menu) {
  for item in &menu.items {
    match item {
      MenuEntry::CustomItem(c) => {
        map.insert(c.id, c.id_str.clone());
      }
      MenuEntry::Submenu(s) => get_menu_ids(map, &s.inner),
      _ => {}
    }
  }
}

/// A webview window that has yet to be built.
pub struct PendingWindow<R: Runtime> {
  /// The label that the window will be named.
  pub label: String,

  /// The [`WindowBuilder`] that the window will be created with.
  pub window_builder: <R::Dispatcher as Dispatch>::WindowBuilder,

  /// The [`WebviewAttributes`] that the webview will be created with.
  pub webview_attributes: WebviewAttributes,

  pub uri_scheme_protocols: HashMap<String, Box<UriSchemeProtocol>>,

  /// How to handle IPC calls on the webview window.
  pub ipc_handler: Option<WebviewIpcHandler<R>>,

  /// How to handle a file dropping onto the webview window.
  pub file_drop_handler: Option<FileDropHandler<R>>,

  /// The resolved URL to load on the webview.
  pub url: String,

  /// Maps runtime id to a string menu id.
  pub menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,

  /// A HashMap mapping JS event names with associated listener ids.
  pub js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u64>>>>,
}

pub fn is_label_valid(label: &str) -> bool {
  label
    .chars()
    .all(|c| char::is_alphanumeric(c) || c == '-' || c == '/' || c == ':' || c == '_')
}

pub fn assert_label_is_valid(label: &str) {
  assert!(
    is_label_valid(label),
    "Window label must include only alphanumeric characters, `-`, `/`, `:` and `_`."
  );
}

impl<R: Runtime> PendingWindow<R> {
  /// Create a new [`PendingWindow`] with a label and starting url.
  pub fn new(
    window_builder: <R::Dispatcher as Dispatch>::WindowBuilder,
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> Self {
    let mut menu_ids = HashMap::new();
    if let Some(menu) = window_builder.get_menu() {
      get_menu_ids(&mut menu_ids, menu);
    }
    let label = label.into();
    assert_label_is_valid(&label);
    Self {
      window_builder,
      webview_attributes,
      uri_scheme_protocols: Default::default(),
      label,
      ipc_handler: None,
      file_drop_handler: None,
      url: "tauri://localhost".to_string(),
      menu_ids: Arc::new(Mutex::new(menu_ids)),
      js_event_listeners: Default::default(),
    }
  }

  /// Create a new [`PendingWindow`] from a [`WindowConfig`] with a label and starting url.
  pub fn with_config(
    window_config: WindowConfig,
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> Self {
    let window_builder = <<R::Dispatcher as Dispatch>::WindowBuilder>::with_config(window_config);
    let mut menu_ids = HashMap::new();
    if let Some(menu) = window_builder.get_menu() {
      get_menu_ids(&mut menu_ids, menu);
    }
    let label = label.into();
    assert_label_is_valid(&label);
    Self {
      window_builder,
      webview_attributes,
      uri_scheme_protocols: Default::default(),
      label,
      ipc_handler: None,
      file_drop_handler: None,
      url: "tauri://localhost".to_string(),
      menu_ids: Arc::new(Mutex::new(menu_ids)),
      js_event_listeners: Default::default(),
    }
  }

  #[must_use]
  pub fn set_menu(mut self, menu: Menu) -> Self {
    let mut menu_ids = HashMap::new();
    get_menu_ids(&mut menu_ids, &menu);
    *self.menu_ids.lock().unwrap() = menu_ids;
    self.window_builder = self.window_builder.menu(menu);
    self
  }

  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(&HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> + Send + Sync + 'static,
  >(
    &mut self,
    uri_scheme: N,
    protocol: H,
  ) {
    let uri_scheme = uri_scheme.into();
    self
      .uri_scheme_protocols
      .insert(uri_scheme, Box::new(move |data| (protocol)(data)));
  }
}

/// Key for a JS event listener.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsEventListenerKey {
  /// The associated window label.
  pub window_label: Option<String>,
  /// The event name.
  pub event: String,
}

/// A webview window that is not yet managed by Tauri.
#[derive(Debug)]
pub struct DetachedWindow<R: Runtime> {
  /// Name of the window
  pub label: String,

  /// The [`Dispatch`](crate::Dispatch) associated with the window.
  pub dispatcher: R::Dispatcher,

  /// Maps runtime id to a string menu id.
  pub menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,

  /// A HashMap mapping JS event names with associated listener ids.
  pub js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u64>>>>,
}

impl<R: Runtime> Clone for DetachedWindow<R> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
      menu_ids: self.menu_ids.clone(),
      js_event_listeners: self.js_event_listeners.clone(),
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
