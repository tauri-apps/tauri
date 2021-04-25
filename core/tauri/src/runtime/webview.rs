// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Items specific to the [`Runtime`](crate::runtime::Runtime)'s webview.

use crate::runtime::Icon;
use crate::{api::config::WindowConfig, runtime::window::DetachedWindow};
use serde_json::Value as JsonValue;
use std::{convert::TryFrom, path::PathBuf};

/// Do **NOT** implement this trait except for use in a custom [`Runtime`](crate::runtime::Runtime).
///
/// This trait is separate from [`Attributes`] to prevent "accidental" implementation.
pub trait AttributesBase: Sized {}

/// A builder for all attributes related to a single webview.
///
/// This trait is only meant to be implemented by a custom [`Runtime`](crate::runtime::Runtime)
/// and not by applications.
pub trait Attributes: AttributesBase {
  /// Expected icon format.
  type Icon: TryFrom<Icon, Error = crate::Error>;

  /// Initializes a new webview builder.
  fn new() -> Self;

  /// Initializes a new webview builder from a [`WindowConfig`]
  fn with_config(config: WindowConfig) -> Self;

  /// Sets the init script.
  fn initialization_script(self, init: &str) -> Self;

  /// The horizontal position of the window's top left corner.
  fn x(self, x: f64) -> Self;

  /// The vertical position of the window's top left corner.
  fn y(self, y: f64) -> Self;

  /// Window width.
  fn width(self, width: f64) -> Self;

  /// Window height.
  fn height(self, height: f64) -> Self;

  /// Window min width.
  fn min_width(self, min_width: f64) -> Self;

  /// Window min height.
  fn min_height(self, min_height: f64) -> Self;

  /// Window max width.
  fn max_width(self, max_width: f64) -> Self;

  /// Window max height.
  fn max_height(self, max_height: f64) -> Self;

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
  fn icon(self, icon: Self::Icon) -> Self;

  /// Whether the icon was set or not.
  fn has_icon(&self) -> bool;

  /// User data path for the webview. Actually only supported on Windows.
  fn user_data_path(self, user_data_path: Option<PathBuf>) -> Self;

  /// Sets the webview url.
  fn url(self, url: String) -> Self;

  /// Whether the webview protocol handler is defined or not.
  fn has_webview_protocol(&self, name: &str) -> bool;

  /// Registers a webview protocol handler.
  fn register_webview_protocol<
    N: Into<String>,
    H: Fn(&str) -> crate::Result<Vec<u8>> + Send + Sync + 'static,
  >(
    self,
    name: N,
    handler: H,
  ) -> Self;

  /// The full attributes.
  fn build(self) -> Self;
}

/// Rpc request.
pub struct RpcRequest {
  /// RPC command.
  pub command: String,
  /// Params.
  pub params: Option<JsonValue>,
}

/// Uses a custom handler to resolve file requests
pub struct CustomProtocol {
  /// Handler for protocol
  pub handler: Box<dyn Fn(&str) -> crate::Result<Vec<u8>> + Send + Sync>,
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
