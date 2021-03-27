use serde_json::Value as JsonValue;
use std::{convert::TryFrom, path::PathBuf};

pub mod wry;

/// A icon definition.
pub enum Icon {
  /// Icon from file path.
  File(String),
  /// Icon from raw bytes.
  Raw(Vec<u8>),
}

/// Messages to dispatch to the application.
pub enum Message {
  // webview messages
  /// Eval a script on the webview.
  EvalScript(String),
  // window messages
  /// Updates the window resizable flag.
  SetResizable(bool),
  /// Updates the window title.
  SetTitle(String),
  /// Maximizes the window.
  Maximize,
  /// Unmaximizes the window.
  Unmaximize,
  /// Minimizes the window.
  Minimize,
  /// Unminimizes the window.
  Unminimize,
  /// Shows the window.
  Show,
  /// Hides the window.
  Hide,
  /// Updates the hasDecorations flag.
  SetDecorations(bool),
  /// Updates the window alwaysOnTop flag.
  SetAlwaysOnTop(bool),
  /// Updates the window width.
  SetWidth(f64),
  /// Updates the window height.
  SetHeight(f64),
  /// Resizes the window.
  Resize {
    /// New width.
    width: f64,
    /// New height.
    height: f64,
  },
  /// Updates the window min size.
  SetMinSize {
    /// New value for the window min width.
    min_width: f64,
    /// New value for the window min height.
    min_height: f64,
  },
  /// Updates the window max size.
  SetMaxSize {
    /// New value for the window max width.
    max_width: f64,
    /// New value for the window max height.
    max_height: f64,
  },
  /// Updates the X position.
  SetX(f64),
  /// Updates the Y position.
  SetY(f64),
  /// Updates the window position.
  SetPosition {
    /// New value for the window X coordinate.
    x: f64,
    /// New value for the window Y coordinate.
    y: f64,
  },
  /// Updates the window fullscreen state.
  SetFullscreen(bool),
  /// Updates the window icon.
  SetIcon(Icon),
}

pub struct WindowConfig(pub crate::api::config::WindowConfig);

pub trait AttributesPrivate: Sized {
  /// Sets the webview url.
  fn url(self, url: String) -> Self;
}

/// The webview builder.
pub trait Attributes: Sized {
  /// Expected icon format.
  type Icon: TryFrom<Icon, Error = crate::Error>;

  /// Initializes a new webview builder.
  fn new() -> Self;

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

/// Rpc handler.
pub type WebviewRpcHandler<D, L> = Box<dyn Fn(D, L, RpcRequest) + Send>;

/// Uses a custom handler to resolve file requests
pub struct CustomProtocol {
  /// Name of the protocol
  pub name: String,
  /// Handler for protocol
  pub handler: Box<dyn Fn(&str) -> crate::Result<Vec<u8>> + Send>,
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

/// File drop handler callback
/// Return `true` in the callback to block the OS' default behavior of handling a file drop..
pub type FileDropHandler = Box<dyn Fn(FileDropEvent) -> bool + Send>;
