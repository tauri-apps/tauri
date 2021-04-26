// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Items specific to the [`Runtime`](crate::runtime::Runtime)'s webview.

use crate::runtime::Icon;
use crate::{
  api::config::{WindowConfig, WindowUrl},
  runtime::window::DetachedWindow,
};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, path::PathBuf};

type UriSchemeProtocol = dyn Fn(&str) -> crate::Result<Vec<u8>> + Send + Sync + 'static;

/// The attributes used to create an webview.
pub struct WebviewAttributes {
  pub(crate) url: WindowUrl,
  pub(crate) initialization_scripts: Vec<String>,
  pub(crate) data_directory: Option<PathBuf>,
  pub(crate) uri_scheme_protocols: HashMap<String, Box<UriSchemeProtocol>>,
}

impl WebviewAttributes {
  /// Initializes the default attributes for a webview.
  pub fn new(url: WindowUrl) -> Self {
    Self {
      url,
      initialization_scripts: Vec::new(),
      data_directory: None,
      uri_scheme_protocols: Default::default(),
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
    H: Fn(&str) -> crate::Result<Vec<u8>> + Send + Sync + 'static,
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
}

/// Do **NOT** implement this trait except for use in a custom [`Runtime`](crate::runtime::Runtime).
///
/// This trait is separate from [`WindowAttributes`] to prevent "accidental" implementation.
pub trait WindowAttributesBase: Sized {}

/// A builder for all attributes related to a single webview.
///
/// This trait is only meant to be implemented by a custom [`Runtime`](crate::runtime::Runtime)
/// and not by applications.
pub trait WindowAttributes: WindowAttributesBase {
  /// Initializes a new window attributes builder.
  fn new() -> Self;

  /// Initializes a new webview builder from a [`WindowConfig`]
  fn with_config(config: WindowConfig) -> Self;

  /// The horizontal position of the window's top left corner.
  fn x(self, x: f64) -> Self;

  /// The vertical position of the window's top left corner.
  fn y(self, y: f64) -> Self;

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

  /// Whether the icon was set or not.
  fn has_icon(&self) -> bool;
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
  pub protocol: Box<dyn Fn(&str) -> crate::Result<Vec<u8>> + Send + Sync>,
}

/// The file drop event payload.
#[derive(Debug, Clone)]
pub enum FileDropEvent {
  /// The file(s) have been dragged onto the window, but have not been dropped yet.
  Hovered(Vec<PathBuf>),
  /// The file(s) have been dropped onto the window.
  Dropped(Vec<PathBuf>),
  /// The file drop was aborted.
  Cancelled,
}

/// Rpc handler.
pub(crate) type WebviewRpcHandler<M> = Box<dyn Fn(DetachedWindow<M>, RpcRequest) + Send>;

/// File drop handler callback
/// Return `true` in the callback to block the OS' default behavior of handling a file drop.
pub(crate) type FileDropHandler<M> = Box<dyn Fn(FileDropEvent, DetachedWindow<M>) -> bool + Send>;
