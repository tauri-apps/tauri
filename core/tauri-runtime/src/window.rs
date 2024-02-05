// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webview windows and Tauri.

use crate::{
  http::{Request as HttpRequest, Response as HttpResponse},
  menu::{Menu, MenuEntry, MenuHash, MenuId},
  webview::{WebviewAttributes, WebviewIpcHandler},
  Dispatch, Runtime, UserEvent, WindowBuilder,
};
use serde::{Deserialize, Deserializer, Serialize};
use tauri_utils::{config::WindowConfig, Theme};
use url::Url;

use std::{
  collections::{HashMap, HashSet},
  hash::{Hash, Hasher},
  path::PathBuf,
  sync::{mpsc::Sender, Arc, Mutex},
};

type UriSchemeProtocol =
  dyn Fn(&HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> + Send + Sync + 'static;

type WebResourceRequestHandler = dyn Fn(&HttpRequest, &mut HttpResponse) + Send + Sync;

/// UI scaling utilities.
pub mod dpi;

/// An event from a window.
#[derive(Debug, Clone)]
pub enum WindowEvent {
  /// The size of the window has changed. Contains the client area's new dimensions.
  Resized(dpi::PhysicalSize<u32>),
  /// The position of the window has changed. Contains the window's new position.
  Moved(dpi::PhysicalPosition<i32>),
  /// The window has been requested to close.
  CloseRequested {
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
  /// An event associated with the file drop action.
  FileDrop(FileDropEvent),
  /// The system window theme has changed.
  ///
  /// Applications might wish to react to this to change the theme of the content of the window when the system changes the window theme.
  ThemeChanged(Theme),
}

/// The file drop event payload.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum FileDropEvent {
  /// The file(s) have been dragged onto the window, but have not been dropped yet.
  Hovered(Vec<PathBuf>),
  /// The file(s) have been dropped onto the window.
  Dropped(Vec<PathBuf>),
  /// The file drop was aborted.
  Cancelled,
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

/// Describes the appearance of the mouse cursor.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CursorIcon {
  /// The platform-dependent default cursor.
  Default,
  /// A simple crosshair.
  Crosshair,
  /// A hand (often used to indicate links in web browsers).
  Hand,
  /// Self explanatory.
  Arrow,
  /// Indicates something is to be moved.
  Move,
  /// Indicates text that may be selected or edited.
  Text,
  /// Program busy indicator.
  Wait,
  /// Help indicator (often rendered as a "?")
  Help,
  /// Progress indicator. Shows that processing is being done. But in contrast
  /// with "Wait" the user may still interact with the program. Often rendered
  /// as a spinning beach ball, or an arrow with a watch or hourglass.
  Progress,

  /// Cursor showing that something cannot be done.
  NotAllowed,
  ContextMenu,
  Cell,
  VerticalText,
  Alias,
  Copy,
  NoDrop,
  /// Indicates something can be grabbed.
  Grab,
  /// Indicates something is grabbed.
  Grabbing,
  AllScroll,
  ZoomIn,
  ZoomOut,

  /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
  /// is used when the movement starts from the south-east corner of the box.
  EResize,
  NResize,
  NeResize,
  NwResize,
  SResize,
  SeResize,
  SwResize,
  WResize,
  EwResize,
  NsResize,
  NeswResize,
  NwseResize,
  ColResize,
  RowResize,
}

impl<'de> Deserialize<'de> for CursorIcon {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(match s.to_lowercase().as_str() {
      "default" => CursorIcon::Default,
      "crosshair" => CursorIcon::Crosshair,
      "hand" => CursorIcon::Hand,
      "arrow" => CursorIcon::Arrow,
      "move" => CursorIcon::Move,
      "text" => CursorIcon::Text,
      "wait" => CursorIcon::Wait,
      "help" => CursorIcon::Help,
      "progress" => CursorIcon::Progress,
      "notallowed" => CursorIcon::NotAllowed,
      "contextmenu" => CursorIcon::ContextMenu,
      "cell" => CursorIcon::Cell,
      "verticaltext" => CursorIcon::VerticalText,
      "alias" => CursorIcon::Alias,
      "copy" => CursorIcon::Copy,
      "nodrop" => CursorIcon::NoDrop,
      "grab" => CursorIcon::Grab,
      "grabbing" => CursorIcon::Grabbing,
      "allscroll" => CursorIcon::AllScroll,
      "zoomin" => CursorIcon::ZoomIn,
      "zoomout" => CursorIcon::ZoomOut,
      "eresize" => CursorIcon::EResize,
      "nresize" => CursorIcon::NResize,
      "neresize" => CursorIcon::NeResize,
      "nwresize" => CursorIcon::NwResize,
      "sresize" => CursorIcon::SResize,
      "seresize" => CursorIcon::SeResize,
      "swresize" => CursorIcon::SwResize,
      "wresize" => CursorIcon::WResize,
      "ewresize" => CursorIcon::EwResize,
      "nsresize" => CursorIcon::NsResize,
      "neswresize" => CursorIcon::NeswResize,
      "nwseresize" => CursorIcon::NwseResize,
      "colresize" => CursorIcon::ColResize,
      "rowresize" => CursorIcon::RowResize,
      _ => CursorIcon::Default,
    })
  }
}

impl Default for CursorIcon {
  fn default() -> Self {
    CursorIcon::Default
  }
}

/// A webview window that has yet to be built.
pub struct PendingWindow<T: UserEvent, R: Runtime<T>> {
  /// The label that the window will be named.
  pub label: String,

  /// The [`WindowBuilder`] that the window will be created with.
  pub window_builder: <R::Dispatcher as Dispatch<T>>::WindowBuilder,

  /// The [`WebviewAttributes`] that the webview will be created with.
  pub webview_attributes: WebviewAttributes,

  pub uri_scheme_protocols: HashMap<String, Box<UriSchemeProtocol>>,

  // Whether custom protocols on windows should use http://<scheme>.localhost/ instead of https://<scheme>.localhost/
  pub http_scheme: bool,

  /// How to handle IPC calls on the webview window.
  pub ipc_handler: Option<WebviewIpcHandler<T, R>>,

  /// Maps runtime id to a string menu id.
  pub menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,

  /// A HashMap mapping JS event names with associated listener ids.
  pub js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u32>>>>,

  /// A handler to decide if incoming url is allowed to navigate.
  pub navigation_handler: Option<Box<dyn Fn(Url) -> bool + Send>>,

  pub web_resource_request_handler: Option<Box<WebResourceRequestHandler>>,

  /// The URL to load on the webview.
  pub url: Url,
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

impl<T: UserEvent, R: Runtime<T>> PendingWindow<T, R> {
  /// Create a new [`PendingWindow`] with a label and starting url.
  pub fn new(
    window_builder: <R::Dispatcher as Dispatch<T>>::WindowBuilder,
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> crate::Result<Self> {
    let mut menu_ids = HashMap::new();
    if let Some(menu) = window_builder.get_menu() {
      get_menu_ids(&mut menu_ids, menu);
    }
    let label = label.into();
    if !is_label_valid(&label) {
      Err(crate::Error::InvalidWindowLabel)
    } else {
      Ok(Self {
        window_builder,
        webview_attributes,
        uri_scheme_protocols: Default::default(),
        label,
        ipc_handler: None,
        menu_ids: Arc::new(Mutex::new(menu_ids)),
        js_event_listeners: Default::default(),
        navigation_handler: Default::default(),
        web_resource_request_handler: Default::default(),
        // This will never fail
        url: Url::parse("tauri://localhost").unwrap(),
        http_scheme: false,
      })
    }
  }

  /// Create a new [`PendingWindow`] from a [`WindowConfig`] with a label and starting url.
  pub fn with_config(
    window_config: WindowConfig,
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> crate::Result<Self> {
    let window_builder =
      <<R::Dispatcher as Dispatch<T>>::WindowBuilder>::with_config(window_config);
    let mut menu_ids = HashMap::new();
    if let Some(menu) = window_builder.get_menu() {
      get_menu_ids(&mut menu_ids, menu);
    }
    let label = label.into();
    if !is_label_valid(&label) {
      Err(crate::Error::InvalidWindowLabel)
    } else {
      Ok(Self {
        window_builder,
        webview_attributes,
        uri_scheme_protocols: Default::default(),
        label,
        ipc_handler: None,
        menu_ids: Arc::new(Mutex::new(menu_ids)),
        js_event_listeners: Default::default(),
        navigation_handler: Default::default(),
        web_resource_request_handler: Default::default(),
        // This will never fail
        url: Url::parse("tauri://localhost").unwrap(),
        http_scheme: false,
      })
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
pub struct DetachedWindow<T: UserEvent, R: Runtime<T>> {
  /// Name of the window
  pub label: String,

  /// The [`Dispatch`](crate::Dispatch) associated with the window.
  pub dispatcher: R::Dispatcher,

  /// Maps runtime id to a string menu id.
  pub menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,

  /// A HashMap mapping JS event names with associated listener ids.
  pub js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u32>>>>,
}

impl<T: UserEvent, R: Runtime<T>> Clone for DetachedWindow<T, R> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
      menu_ids: self.menu_ids.clone(),
      js_event_listeners: self.js_event_listeners.clone(),
    }
  }
}

impl<T: UserEvent, R: Runtime<T>> Hash for DetachedWindow<T, R> {
  /// Only use the [`DetachedWindow`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<T: UserEvent, R: Runtime<T>> Eq for DetachedWindow<T, R> {}
impl<T: UserEvent, R: Runtime<T>> PartialEq for DetachedWindow<T, R> {
  /// Only use the [`DetachedWindow`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}
