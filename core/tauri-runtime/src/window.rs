// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webview windows and Tauri.

use crate::{
  webview::{WebviewAttributes, WebviewIpcHandler},
  Dispatch, Runtime, UserEvent, WindowBuilder,
};

use http::{Request as HttpRequest, Response as HttpResponse};
use serde::{Deserialize, Deserializer};
use tauri_utils::{config::WindowConfig, Theme};
use url::Url;

use std::{
  borrow::Cow,
  collections::HashMap,
  hash::{Hash, Hasher},
  marker::PhantomData,
  path::PathBuf,
  sync::mpsc::Sender,
};

use self::dpi::PhysicalPosition;

type UriSchemeProtocol = dyn Fn(HttpRequest<Vec<u8>>, Box<dyn FnOnce(HttpResponse<Cow<'static, [u8]>>) + Send>)
  + Send
  + Sync
  + 'static;

type WebResourceRequestHandler =
  dyn Fn(HttpRequest<Vec<u8>>, &mut HttpResponse<Cow<'static, [u8]>>) + Send + Sync;

type NavigationHandler = dyn Fn(&Url) -> bool + Send;

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
  Hovered {
    paths: Vec<PathBuf>,
    /// The position of the mouse cursor.
    position: PhysicalPosition<f64>,
  },
  /// The file(s) have been dropped onto the window.
  Dropped {
    paths: Vec<PathBuf>,
    /// The position of the mouse cursor.
    position: PhysicalPosition<f64>,
  },
  /// The file drop was aborted.
  Cancelled,
}

/// Describes the appearance of the mouse cursor.
#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CursorIcon {
  /// The platform-dependent default cursor.
  #[default]
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

#[cfg(target_os = "android")]
pub struct CreationContext<'a, 'b> {
  pub env: &'a mut jni::JNIEnv<'b>,
  pub activity: &'a jni::objects::JObject<'b>,
  pub webview: &'a jni::objects::JObject<'b>,
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

  /// How to handle IPC calls on the webview window.
  pub ipc_handler: Option<WebviewIpcHandler<T, R>>,

  /// A handler to decide if incoming url is allowed to navigate.
  pub navigation_handler: Option<Box<NavigationHandler>>,

  /// The resolved URL to load on the webview.
  pub url: String,

  #[cfg(target_os = "android")]
  #[allow(clippy::type_complexity)]
  pub on_webview_created:
    Option<Box<dyn Fn(CreationContext<'_, '_>) -> Result<(), jni::errors::Error> + Send>>,

  pub web_resource_request_handler: Option<Box<WebResourceRequestHandler>>,
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
        navigation_handler: Default::default(),
        url: "tauri://localhost".to_string(),
        #[cfg(target_os = "android")]
        on_webview_created: None,
        web_resource_request_handler: Default::default(),
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
        navigation_handler: Default::default(),
        url: "tauri://localhost".to_string(),
        #[cfg(target_os = "android")]
        on_webview_created: None,
        web_resource_request_handler: Default::default(),
      })
    }
  }

  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(HttpRequest<Vec<u8>>, Box<dyn FnOnce(HttpResponse<Cow<'static, [u8]>>) + Send>)
      + Send
      + Sync
      + 'static,
  >(
    &mut self,
    uri_scheme: N,
    protocol: H,
  ) {
    let uri_scheme = uri_scheme.into();
    self
      .uri_scheme_protocols
      .insert(uri_scheme, Box::new(protocol));
  }

  #[cfg(target_os = "android")]
  pub fn on_webview_created<
    F: Fn(CreationContext<'_, '_>) -> Result<(), jni::errors::Error> + Send + 'static,
  >(
    mut self,
    f: F,
  ) -> Self {
    self.on_webview_created.replace(Box::new(f));
    self
  }
}

/// A webview window that is not yet managed by Tauri.
#[derive(Debug)]
pub struct DetachedWindow<T: UserEvent, R: Runtime<T>> {
  /// Name of the window
  pub label: String,

  /// The [`Dispatch`](crate::Dispatch) associated with the window.
  pub dispatcher: R::Dispatcher,
}

impl<T: UserEvent, R: Runtime<T>> Clone for DetachedWindow<T, R> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
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

/// A raw window type that contains fields to access
/// the HWND on Windows, gtk::ApplicationWindow on Linux and
/// NSView on macOS.
pub struct RawWindow<'a> {
  #[cfg(windows)]
  pub hwnd: isize,
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub gtk_window: &'a gtk::ApplicationWindow,
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub default_vbox: Option<&'a gtk::Box>,
  pub _marker: &'a PhantomData<()>,
}
