// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Items specific to the [`Runtime`](crate::Runtime)'s webview.

use crate::{window::DetachedWindow, Icon};

#[cfg(feature = "menu")]
use crate::menu::Menu;

use serde::Deserialize;
use serde_json::Value as JsonValue;
use tauri_utils::config::{WindowConfig, WindowUrl};

#[cfg(windows)]
use winapi::shared::windef::HWND;

use std::{collections::HashMap, path::PathBuf};

type UriSchemeProtocol =
  dyn Fn(&str) -> Result<Vec<u8>, Box<dyn std::error::Error>> + Send + Sync + 'static;

/// The attributes used to create an webview.
pub struct WebviewAttributes {
  pub url: WindowUrl,
  pub initialization_scripts: Vec<String>,
  pub data_directory: Option<PathBuf>,
  pub uri_scheme_protocols: HashMap<String, Box<UriSchemeProtocol>>,
  pub file_drop_handler_enabled: bool,
}

impl WebviewAttributes {
  /// Initializes the default attributes for a webview.
  pub fn new(url: WindowUrl) -> Self {
    Self {
      url,
      initialization_scripts: Vec::new(),
      data_directory: None,
      uri_scheme_protocols: Default::default(),
      file_drop_handler_enabled: true,
    }
  }

  /// Sets the init script.
  pub fn initialization_script(mut self, script: &str) -> Self {
    self.initialization_scripts.push(script.to_string());
    self
  }

  /// Data directory for the webview.
  pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
    self.data_directory.replace(data_directory);
    self
  }

  /// Whether the webview URI scheme protocol is defined or not.
  pub fn has_uri_scheme_protocol(&self, name: &str) -> bool {
    self.uri_scheme_protocols.contains_key(name)
  }

  /// Registers a webview protocol handler.
  /// Leverages [setURLSchemeHandler](https://developer.apple.com/documentation/webkit/wkwebviewconfiguration/2875766-seturlschemehandler) on macOS,
  /// [AddWebResourceRequestedFilter](https://docs.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.addwebresourcerequestedfilter?view=webview2-dotnet-1.0.774.44) on Windows
  /// and [webkit-web-context-register-uri-scheme](https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebContext.html#webkit-web-context-register-uri-scheme) on Linux.
  ///
  /// # Arguments
  ///
  /// * `uri_scheme` The URI scheme to register, such as `example`.
  /// * `protocol` the protocol associated with the given URI scheme. It's a function that takes an URL such as `example://localhost/asset.css`.
  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(&str) -> Result<Vec<u8>, Box<dyn std::error::Error>> + Send + Sync + 'static,
  >(
    mut self,
    uri_scheme: N,
    protocol: H,
  ) -> Self {
    let uri_scheme = uri_scheme.into();
    self
      .uri_scheme_protocols
      .insert(uri_scheme, Box::new(move |data| (protocol)(data)));
    self
  }

  /// Disables the file drop handler. This is required to use drag and drop APIs on the front end on Windows.
  pub fn disable_file_drop_handler(mut self) -> Self {
    self.file_drop_handler_enabled = false;
    self
  }
}

/// Do **NOT** implement this trait except for use in a custom [`Runtime`](crate::Runtime).
///
/// This trait is separate from [`WindowBuilder`] to prevent "accidental" implementation.
pub trait WindowBuilderBase: Sized {}

/// A builder for all attributes related to a single webview.
///
/// This trait is only meant to be implemented by a custom [`Runtime`](crate::Runtime)
/// and not by applications.
pub trait WindowBuilder: WindowBuilderBase {
  /// Initializes a new window attributes builder.
  fn new() -> Self;

  /// Initializes a new webview builder from a [`WindowConfig`]
  fn with_config(config: WindowConfig) -> Self;

  /// Sets the menu for the window.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  fn menu(self, menu: Menu) -> Self;

  /// Show window in the center of the screen.
  fn center(self) -> Self;

  /// The initial position of the window's.
  fn position(self, x: f64, y: f64) -> Self;

  /// Window size.
  fn inner_size(self, min_width: f64, min_height: f64) -> Self;

  /// Window min inner size.
  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self;

  /// Window max inner size.
  fn max_inner_size(self, min_width: f64, min_height: f64) -> Self;

  /// Whether the window is resizable or not.
  fn resizable(self, resizable: bool) -> Self;

  /// The title of the window in the title bar.
  fn title<S: Into<String>>(self, title: S) -> Self;

  /// Whether to start the window in fullscreen or not.
  fn fullscreen(self, fullscreen: bool) -> Self;

  /// Whether the window will be initially hidden or focused.
  fn focus(self) -> Self;

  /// Whether the window should be maximized upon creation.
  fn maximized(self, maximized: bool) -> Self;

  /// Whether the window should be immediately visible upon creation.
  fn visible(self, visible: bool) -> Self;

  /// Whether the the window should be transparent. If this is true, writing colors
  /// with alpha values different than `1.0` will produce a transparent window.
  fn transparent(self, transparent: bool) -> Self;

  /// Whether the window should have borders and bars.
  fn decorations(self, decorations: bool) -> Self;

  /// Whether the window should always be on top of other windows.
  fn always_on_top(self, always_on_top: bool) -> Self;

  /// Sets the window icon.
  fn icon(self, icon: Icon) -> crate::Result<Self>;

  /// Sets whether or not the window icon should be added to the taskbar.
  fn skip_taskbar(self, skip: bool) -> Self;

  /// Sets a parent to the window to be created.
  ///
  /// A child window has the WS_CHILD style and is confined to the client area of its parent window.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#child-windows>
  #[cfg(windows)]
  fn parent_window(self, parent: HWND) -> Self;

  /// Set an owner to the window to be created.
  ///
  /// From MSDN:
  /// - An owned window is always above its owner in the z-order.
  /// - The system automatically destroys an owned window when its owner is destroyed.
  /// - An owned window is hidden when its owner is minimized.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows>
  #[cfg(windows)]
  fn owner_window(self, owner: HWND) -> Self;

  /// Whether the icon was set or not.
  fn has_icon(&self) -> bool;

  /// Whether the menu was set or not.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  fn has_menu(&self) -> bool;
}

/// Rpc request.
pub struct RpcRequest {
  /// RPC command.
  pub command: String,
  /// Params.
  pub params: Option<JsonValue>,
}

/// Uses a custom URI scheme handler to resolve file requests
pub struct CustomProtocol {
  /// Handler for protocol
  #[allow(clippy::type_complexity)]
  pub protocol: Box<dyn Fn(&str) -> Result<Vec<u8>, Box<dyn std::error::Error>> + Send + Sync>,
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

/// Rpc handler.
pub type WebviewRpcHandler<R> = Box<dyn Fn(DetachedWindow<R>, RpcRequest) + Send>;

/// File drop handler callback
/// Return `true` in the callback to block the OS' default behavior of handling a file drop.
pub type FileDropHandler<R> = Box<dyn Fn(FileDropEvent, DetachedWindow<R>) -> bool + Send>;

#[derive(Deserialize)]
pub struct InvokePayload {
  #[serde(rename = "__tauriModule")]
  pub tauri_module: Option<String>,
  pub callback: String,
  pub error: String,
  #[serde(rename = "__invokeKey")]
  pub key: u32,
  #[serde(flatten)]
  pub inner: JsonValue,
}
