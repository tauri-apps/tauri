// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Type-erased [`Runtime`].
//!
//! [`DynRuntime`] wraps any [`Runtime`] implementation behind trait objects so the concrete
//! runtime can be selected by the application when building it, instead of being part of the
//! generic parameters of every type that interacts with the runtime.
//!
//! The concrete runtime is selected through [`DynRuntimeInitAttrs::new`], which takes the
//! [`RuntimeSpecificInitAttrs`] of the runtime to use (e.g. `tauri_runtime_wry::Wry`).
//! Runtime-specific APIs can still be reached by downcasting the wrappers
//! (see [`DynRuntimeHandle::downcast_ref`], [`DynWebviewDispatcher::downcast_ref`],
//! [`DynWindowDispatcher::downcast_ref`] and [`DynWebview::downcast_ref`]).

use std::{
  any::{Any, type_name},
  fmt,
  marker::PhantomData,
  sync::Arc,
};

use raw_window_handle::{DisplayHandle, HandleError, WindowHandle};
use tauri_utils::{
  Theme,
  config::{Color, Config, WindowConfig},
};
use url::Url;

#[cfg(target_os = "macos")]
use crate::ActivationPolicy;
use crate::{
  Cookie, DeviceEventFilter, Error, EventLoopProxy, Icon, ProgressBarState, ResizeDirection,
  Result, RunEvent, Runtime, RuntimeHandle, RuntimeInitArgs, RuntimeSpecificInitAttrs,
  UserAttentionType, UserEvent, WebviewDispatch, WebviewEventId, WindowDispatch, WindowEventId,
  dpi::{PhysicalPosition, PhysicalSize, Position, Rect, Size},
  monitor::Monitor,
  webview::{
    DetachedWebview, NewWindowFeatures, NewWindowHandler, PendingWebview, WebviewIpcHandler,
  },
  window::{
    CursorIcon, DetachedWindow, DetachedWindowWebview, PendingWindow, RawWindow, WebviewEvent,
    WindowBuilder, WindowBuilderBase, WindowEvent, WindowId, WindowSizeConstraints,
  },
};

type AfterWindowCreation = Box<dyn Fn(RawWindow<'_>) + Send>;
type RunCallback<T> = Box<dyn FnMut(RunEvent<T>)>;
type MainThreadTask = Box<dyn FnOnce() + Send>;

fn mismatch<Expected: ?Sized>(what: &str) -> Error {
  Error::RuntimeTypeMismatch(format!(
    "expected {what} of type `{}`",
    type_name::<Expected>()
  ))
}

// ---------------------------------------------------------------------------
// Erased values
// ---------------------------------------------------------------------------

/// The platform webview handle of the selected runtime, exposed through [`WebviewDispatch::with_webview`].
///
/// Downcast it to the runtime's webview type (e.g. `tauri_runtime_wry::Webview`) to reach the platform APIs.
pub struct DynWebview(Box<dyn Any>);

impl DynWebview {
  /// Wraps a runtime webview handle.
  pub fn new<W: Any>(webview: W) -> Self {
    Self(Box::new(webview))
  }

  /// Whether the inner webview is of type `W`.
  pub fn is<W: Any>(&self) -> bool {
    self.0.is::<W>()
  }

  /// Returns a reference to the inner webview if it is of type `W`.
  pub fn downcast_ref<W: Any>(&self) -> Option<&W> {
    self.0.downcast_ref()
  }

  /// Returns a mutable reference to the inner webview if it is of type `W`.
  pub fn downcast_mut<W: Any>(&mut self) -> Option<&mut W> {
    self.0.downcast_mut()
  }

  /// Unwraps the inner webview if it is of type `W`.
  pub fn downcast<W: Any>(self) -> std::result::Result<W, Self> {
    self.0.downcast::<W>().map(|w| *w).map_err(Self)
  }

  /// Returns the inner boxed webview.
  pub fn into_inner(self) -> Box<dyn Any> {
    self.0
  }
}

impl fmt::Debug for DynWebview {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynWebview").finish_non_exhaustive()
  }
}

/// Information about the webview that initiated a new window request, for the selected runtime.
///
/// Downcast it to the runtime's opener type (e.g. `tauri_runtime_wry::NewWindowOpener`) to inspect it.
pub struct DynWindowOpener(Box<dyn Any + Send + Sync>);

impl DynWindowOpener {
  /// Wraps a runtime window opener.
  pub fn new<O: Any + Send + Sync>(opener: O) -> Self {
    Self(Box::new(opener))
  }

  /// Whether the inner opener is of type `O`.
  pub fn is<O: Any>(&self) -> bool {
    self.0.is::<O>()
  }

  /// Returns a reference to the inner opener if it is of type `O`.
  pub fn downcast_ref<O: Any>(&self) -> Option<&O> {
    self.0.downcast_ref()
  }

  /// Unwraps the inner opener, failing if it is not of type `O`.
  pub fn downcast<O: Any>(self) -> Result<O> {
    self
      .0
      .downcast::<O>()
      .map(|o| *o)
      .map_err(|_| mismatch::<O>("window opener"))
  }
}

impl fmt::Debug for DynWindowOpener {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynWindowOpener").finish_non_exhaustive()
  }
}

/// A runtime-specific webview attribute for the selected runtime.
///
/// Wrap the runtime's attribute type (e.g. `tauri_runtime_wry::WebviewAttribute`) with [`DynWebviewAttribute::new`].
pub struct DynWebviewAttribute(Box<dyn Any + Send + Sync>);

impl DynWebviewAttribute {
  /// Wraps a runtime webview attribute.
  pub fn new<A: Any + Send + Sync>(attribute: A) -> Self {
    Self(Box::new(attribute))
  }

  /// Whether the inner attribute is of type `A`.
  pub fn is<A: Any>(&self) -> bool {
    self.0.is::<A>()
  }

  /// Returns a reference to the inner attribute if it is of type `A`.
  pub fn downcast_ref<A: Any>(&self) -> Option<&A> {
    self.0.downcast_ref()
  }

  /// Unwraps the inner attribute, failing if it is not of type `A`.
  pub fn downcast<A: Any>(self) -> Result<A> {
    self
      .0
      .downcast::<A>()
      .map(|a| *a)
      .map_err(|_| mismatch::<A>("webview attribute"))
  }
}

impl fmt::Debug for DynWebviewAttribute {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynWebviewAttribute")
      .finish_non_exhaustive()
  }
}

// ---------------------------------------------------------------------------
// Window builder
// ---------------------------------------------------------------------------

type RuntimeBuilderCustomizer = Arc<dyn Fn(&mut dyn Any) + Send + Sync>;

#[derive(Clone)]
enum WindowBuilderOp {
  Center,
  Position(f64, f64),
  InnerSize(f64, f64),
  MinInnerSize(f64, f64),
  MaxInnerSize(f64, f64),
  InnerSizeConstraints(WindowSizeConstraints),
  PreventOverflow,
  PreventOverflowWithMargin(Size),
  Resizable(bool),
  Maximizable(bool),
  Minimizable(bool),
  Closable(bool),
  Title(String),
  Fullscreen(bool),
  Focused(bool),
  Focusable(bool),
  Maximized(bool),
  Visible(bool),
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  Transparent(bool),
  Decorations(bool),
  AlwaysOnBottom(bool),
  AlwaysOnTop(bool),
  VisibleOnAllWorkspaces(bool),
  ContentProtected(bool),
  Icon(Icon<'static>),
  SkipTaskbar(bool),
  BackgroundColor(Color),
  Shadow(bool),
  #[cfg(windows)]
  Owner(windows::Win32::Foundation::HWND),
  #[cfg(windows)]
  Parent(windows::Win32::Foundation::HWND),
  #[cfg(target_os = "macos")]
  Parent(*mut std::ffi::c_void),
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  TransientFor(gtk::Window),
  #[cfg(windows)]
  DragAndDrop(bool),
  #[cfg(target_os = "macos")]
  TitleBarStyle(tauri_utils::TitleBarStyle),
  #[cfg(target_os = "macos")]
  TrafficLightPosition(Position),
  #[cfg(target_os = "macos")]
  HiddenTitle(bool),
  #[cfg(target_os = "macos")]
  TabbingIdentifier(String),
  Theme(Option<Theme>),
  WindowClassname(String),
  NoRedirectionBitmap(bool),
  #[cfg(target_os = "android")]
  ActivityName(String),
  #[cfg(target_os = "android")]
  CreatedByActivityName(String),
  #[cfg(target_os = "ios")]
  RequestedBySceneIdentifier(String),
  Customize(RuntimeBuilderCustomizer),
}

impl fmt::Debug for WindowBuilderOp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Center => f.write_str("Center"),
      Self::Position(x, y) => f.debug_tuple("Position").field(x).field(y).finish(),
      Self::InnerSize(w, h) => f.debug_tuple("InnerSize").field(w).field(h).finish(),
      Self::MinInnerSize(w, h) => f.debug_tuple("MinInnerSize").field(w).field(h).finish(),
      Self::MaxInnerSize(w, h) => f.debug_tuple("MaxInnerSize").field(w).field(h).finish(),
      Self::InnerSizeConstraints(c) => f.debug_tuple("InnerSizeConstraints").field(c).finish(),
      Self::PreventOverflow => f.write_str("PreventOverflow"),
      Self::PreventOverflowWithMargin(m) => {
        f.debug_tuple("PreventOverflowWithMargin").field(m).finish()
      }
      Self::Resizable(v) => f.debug_tuple("Resizable").field(v).finish(),
      Self::Maximizable(v) => f.debug_tuple("Maximizable").field(v).finish(),
      Self::Minimizable(v) => f.debug_tuple("Minimizable").field(v).finish(),
      Self::Closable(v) => f.debug_tuple("Closable").field(v).finish(),
      Self::Title(v) => f.debug_tuple("Title").field(v).finish(),
      Self::Fullscreen(v) => f.debug_tuple("Fullscreen").field(v).finish(),
      Self::Focused(v) => f.debug_tuple("Focused").field(v).finish(),
      Self::Focusable(v) => f.debug_tuple("Focusable").field(v).finish(),
      Self::Maximized(v) => f.debug_tuple("Maximized").field(v).finish(),
      Self::Visible(v) => f.debug_tuple("Visible").field(v).finish(),
      #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
      Self::Transparent(v) => f.debug_tuple("Transparent").field(v).finish(),
      Self::Decorations(v) => f.debug_tuple("Decorations").field(v).finish(),
      Self::AlwaysOnBottom(v) => f.debug_tuple("AlwaysOnBottom").field(v).finish(),
      Self::AlwaysOnTop(v) => f.debug_tuple("AlwaysOnTop").field(v).finish(),
      Self::VisibleOnAllWorkspaces(v) => f.debug_tuple("VisibleOnAllWorkspaces").field(v).finish(),
      Self::ContentProtected(v) => f.debug_tuple("ContentProtected").field(v).finish(),
      Self::Icon(i) => f
        .debug_struct("Icon")
        .field("width", &i.width)
        .field("height", &i.height)
        .finish(),
      Self::SkipTaskbar(v) => f.debug_tuple("SkipTaskbar").field(v).finish(),
      Self::BackgroundColor(v) => f.debug_tuple("BackgroundColor").field(v).finish(),
      Self::Shadow(v) => f.debug_tuple("Shadow").field(v).finish(),
      #[cfg(windows)]
      Self::Owner(v) => f.debug_tuple("Owner").field(v).finish(),
      #[cfg(any(windows, target_os = "macos"))]
      Self::Parent(v) => f.debug_tuple("Parent").field(v).finish(),
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      Self::TransientFor(v) => f.debug_tuple("TransientFor").field(v).finish(),
      #[cfg(windows)]
      Self::DragAndDrop(v) => f.debug_tuple("DragAndDrop").field(v).finish(),
      #[cfg(target_os = "macos")]
      Self::TitleBarStyle(v) => f.debug_tuple("TitleBarStyle").field(v).finish(),
      #[cfg(target_os = "macos")]
      Self::TrafficLightPosition(v) => f.debug_tuple("TrafficLightPosition").field(v).finish(),
      #[cfg(target_os = "macos")]
      Self::HiddenTitle(v) => f.debug_tuple("HiddenTitle").field(v).finish(),
      #[cfg(target_os = "macos")]
      Self::TabbingIdentifier(v) => f.debug_tuple("TabbingIdentifier").field(v).finish(),
      Self::Theme(v) => f.debug_tuple("Theme").field(v).finish(),
      Self::WindowClassname(v) => f.debug_tuple("WindowClassname").field(v).finish(),
      Self::NoRedirectionBitmap(v) => f.debug_tuple("NoRedirectionBitmap").field(v).finish(),
      #[cfg(target_os = "android")]
      Self::ActivityName(v) => f.debug_tuple("ActivityName").field(v).finish(),
      #[cfg(target_os = "android")]
      Self::CreatedByActivityName(v) => f.debug_tuple("CreatedByActivityName").field(v).finish(),
      #[cfg(target_os = "ios")]
      Self::RequestedBySceneIdentifier(v) => f
        .debug_tuple("RequestedBySceneIdentifier")
        .field(v)
        .finish(),
      Self::Customize(_) => f.write_str("Customize"),
    }
  }
}

/// A [`WindowBuilder`] for [`DynRuntime`].
///
/// The concrete window builder type is only known when the runtime creates the window,
/// so this builder records the requested attributes and replays them on the runtime's
/// builder at that point (see [`DynWindowBuilder::apply`]).
#[derive(Debug, Clone, Default)]
pub struct DynWindowBuilder {
  config: Option<WindowConfig>,
  ops: Vec<WindowBuilderOp>,
}

// SAFETY: like the runtime window builders this wraps, the platform handles recorded here
// (parent windows) are only used on the main thread when the window is created.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for DynWindowBuilder {}

impl DynWindowBuilder {
  /// Registers a callback that customizes the runtime's window builder when this builder is applied.
  ///
  /// The callback receives the runtime's window builder as a [`dyn Any`](Any), which must be downcast
  /// to the builder type of the runtime in use. This is how runtime-specific builder APIs
  /// are reached through the type-erased runtime.
  #[must_use]
  pub fn customize<F: Fn(&mut dyn Any) + Send + Sync + 'static>(mut self, f: F) -> Self {
    self.ops.push(WindowBuilderOp::Customize(Arc::new(f)));
    self
  }

  /// Replays the recorded attributes on a window builder of the concrete runtime.
  pub fn apply<B: WindowBuilder>(self) -> Result<B> {
    let mut builder = match &self.config {
      Some(config) => B::with_config(config),
      None => B::new(),
    };
    for op in self.ops {
      builder = match op {
        WindowBuilderOp::Center => builder.center(),
        WindowBuilderOp::Position(x, y) => builder.position(x, y),
        WindowBuilderOp::InnerSize(w, h) => builder.inner_size(w, h),
        WindowBuilderOp::MinInnerSize(w, h) => builder.min_inner_size(w, h),
        WindowBuilderOp::MaxInnerSize(w, h) => builder.max_inner_size(w, h),
        WindowBuilderOp::InnerSizeConstraints(c) => builder.inner_size_constraints(c),
        WindowBuilderOp::PreventOverflow => builder.prevent_overflow(),
        WindowBuilderOp::PreventOverflowWithMargin(m) => builder.prevent_overflow_with_margin(m),
        WindowBuilderOp::Resizable(v) => builder.resizable(v),
        WindowBuilderOp::Maximizable(v) => builder.maximizable(v),
        WindowBuilderOp::Minimizable(v) => builder.minimizable(v),
        WindowBuilderOp::Closable(v) => builder.closable(v),
        WindowBuilderOp::Title(v) => builder.title(v),
        WindowBuilderOp::Fullscreen(v) => builder.fullscreen(v),
        WindowBuilderOp::Focused(v) => builder.focused(v),
        WindowBuilderOp::Focusable(v) => builder.focusable(v),
        WindowBuilderOp::Maximized(v) => builder.maximized(v),
        WindowBuilderOp::Visible(v) => builder.visible(v),
        #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
        WindowBuilderOp::Transparent(v) => builder.transparent(v),
        WindowBuilderOp::Decorations(v) => builder.decorations(v),
        WindowBuilderOp::AlwaysOnBottom(v) => builder.always_on_bottom(v),
        WindowBuilderOp::AlwaysOnTop(v) => builder.always_on_top(v),
        WindowBuilderOp::VisibleOnAllWorkspaces(v) => builder.visible_on_all_workspaces(v),
        WindowBuilderOp::ContentProtected(v) => builder.content_protected(v),
        WindowBuilderOp::Icon(icon) => builder.icon(icon)?,
        WindowBuilderOp::SkipTaskbar(v) => builder.skip_taskbar(v),
        WindowBuilderOp::BackgroundColor(v) => builder.background_color(v),
        WindowBuilderOp::Shadow(v) => builder.shadow(v),
        #[cfg(windows)]
        WindowBuilderOp::Owner(v) => builder.owner(v),
        #[cfg(any(windows, target_os = "macos"))]
        WindowBuilderOp::Parent(v) => builder.parent(v),
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        WindowBuilderOp::TransientFor(v) => builder.transient_for(&v),
        #[cfg(windows)]
        WindowBuilderOp::DragAndDrop(v) => builder.drag_and_drop(v),
        #[cfg(target_os = "macos")]
        WindowBuilderOp::TitleBarStyle(v) => builder.title_bar_style(v),
        #[cfg(target_os = "macos")]
        WindowBuilderOp::TrafficLightPosition(v) => builder.traffic_light_position(v),
        #[cfg(target_os = "macos")]
        WindowBuilderOp::HiddenTitle(v) => builder.hidden_title(v),
        #[cfg(target_os = "macos")]
        WindowBuilderOp::TabbingIdentifier(v) => builder.tabbing_identifier(&v),
        WindowBuilderOp::Theme(v) => builder.theme(v),
        WindowBuilderOp::WindowClassname(v) => builder.window_classname(v),
        WindowBuilderOp::NoRedirectionBitmap(v) => builder.no_redirection_bitmap(v),
        #[cfg(target_os = "android")]
        WindowBuilderOp::ActivityName(v) => builder.activity_name(v),
        #[cfg(target_os = "android")]
        WindowBuilderOp::CreatedByActivityName(v) => builder.created_by_activity_name(v),
        #[cfg(target_os = "ios")]
        WindowBuilderOp::RequestedBySceneIdentifier(v) => builder.requested_by_scene_identifier(v),
        WindowBuilderOp::Customize(f) => {
          f(&mut builder);
          builder
        }
      };
    }
    Ok(builder)
  }

  fn push(mut self, op: WindowBuilderOp) -> Self {
    self.ops.push(op);
    self
  }
}

impl WindowBuilderBase for DynWindowBuilder {}

impl WindowBuilder for DynWindowBuilder {
  fn new() -> Self {
    Self::default()
  }

  fn with_config(config: &WindowConfig) -> Self {
    Self {
      config: Some(config.clone()),
      ops: Vec::new(),
    }
  }

  fn center(self) -> Self {
    self.push(WindowBuilderOp::Center)
  }

  fn position(self, x: f64, y: f64) -> Self {
    self.push(WindowBuilderOp::Position(x, y))
  }

  fn inner_size(self, width: f64, height: f64) -> Self {
    self.push(WindowBuilderOp::InnerSize(width, height))
  }

  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self {
    self.push(WindowBuilderOp::MinInnerSize(min_width, min_height))
  }

  fn max_inner_size(self, max_width: f64, max_height: f64) -> Self {
    self.push(WindowBuilderOp::MaxInnerSize(max_width, max_height))
  }

  fn inner_size_constraints(self, constraints: WindowSizeConstraints) -> Self {
    self.push(WindowBuilderOp::InnerSizeConstraints(constraints))
  }

  fn prevent_overflow(self) -> Self {
    self.push(WindowBuilderOp::PreventOverflow)
  }

  fn prevent_overflow_with_margin(self, margin: Size) -> Self {
    self.push(WindowBuilderOp::PreventOverflowWithMargin(margin))
  }

  fn resizable(self, resizable: bool) -> Self {
    self.push(WindowBuilderOp::Resizable(resizable))
  }

  fn maximizable(self, maximizable: bool) -> Self {
    self.push(WindowBuilderOp::Maximizable(maximizable))
  }

  fn minimizable(self, minimizable: bool) -> Self {
    self.push(WindowBuilderOp::Minimizable(minimizable))
  }

  fn closable(self, closable: bool) -> Self {
    self.push(WindowBuilderOp::Closable(closable))
  }

  fn title<S: Into<String>>(self, title: S) -> Self {
    self.push(WindowBuilderOp::Title(title.into()))
  }

  fn fullscreen(self, fullscreen: bool) -> Self {
    self.push(WindowBuilderOp::Fullscreen(fullscreen))
  }

  fn focused(self, focused: bool) -> Self {
    self.push(WindowBuilderOp::Focused(focused))
  }

  fn focusable(self, focusable: bool) -> Self {
    self.push(WindowBuilderOp::Focusable(focusable))
  }

  fn maximized(self, maximized: bool) -> Self {
    self.push(WindowBuilderOp::Maximized(maximized))
  }

  fn visible(self, visible: bool) -> Self {
    self.push(WindowBuilderOp::Visible(visible))
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  fn transparent(self, transparent: bool) -> Self {
    self.push(WindowBuilderOp::Transparent(transparent))
  }

  fn decorations(self, decorations: bool) -> Self {
    self.push(WindowBuilderOp::Decorations(decorations))
  }

  fn always_on_bottom(self, always_on_bottom: bool) -> Self {
    self.push(WindowBuilderOp::AlwaysOnBottom(always_on_bottom))
  }

  fn always_on_top(self, always_on_top: bool) -> Self {
    self.push(WindowBuilderOp::AlwaysOnTop(always_on_top))
  }

  fn visible_on_all_workspaces(self, visible_on_all_workspaces: bool) -> Self {
    self.push(WindowBuilderOp::VisibleOnAllWorkspaces(
      visible_on_all_workspaces,
    ))
  }

  fn content_protected(self, protected: bool) -> Self {
    self.push(WindowBuilderOp::ContentProtected(protected))
  }

  fn icon(self, icon: Icon) -> Result<Self> {
    let expected_len = (icon.width as usize)
      .saturating_mul(icon.height as usize)
      .saturating_mul(4);
    if icon.rgba.len() != expected_len {
      return Err(Error::InvalidIcon(
        format!(
          "the icon RGBA buffer has {} bytes but {}x{} pixels require {expected_len}",
          icon.rgba.len(),
          icon.width,
          icon.height
        )
        .into(),
      ));
    }
    Ok(self.push(WindowBuilderOp::Icon(icon.into_owned())))
  }

  fn skip_taskbar(self, skip: bool) -> Self {
    self.push(WindowBuilderOp::SkipTaskbar(skip))
  }

  fn background_color(self, color: Color) -> Self {
    self.push(WindowBuilderOp::BackgroundColor(color))
  }

  fn shadow(self, enable: bool) -> Self {
    self.push(WindowBuilderOp::Shadow(enable))
  }

  #[cfg(windows)]
  fn owner(self, owner: windows::Win32::Foundation::HWND) -> Self {
    self.push(WindowBuilderOp::Owner(owner))
  }

  #[cfg(windows)]
  fn parent(self, parent: windows::Win32::Foundation::HWND) -> Self {
    self.push(WindowBuilderOp::Parent(parent))
  }

  #[cfg(target_os = "macos")]
  fn parent(self, parent: *mut std::ffi::c_void) -> Self {
    self.push(WindowBuilderOp::Parent(parent))
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn transient_for(self, parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    use gtk::glib::Cast;
    self.push(WindowBuilderOp::TransientFor(parent.clone().upcast()))
  }

  #[cfg(windows)]
  fn drag_and_drop(self, enabled: bool) -> Self {
    self.push(WindowBuilderOp::DragAndDrop(enabled))
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(self, style: tauri_utils::TitleBarStyle) -> Self {
    self.push(WindowBuilderOp::TitleBarStyle(style))
  }

  #[cfg(target_os = "macos")]
  fn traffic_light_position<P: Into<Position>>(self, position: P) -> Self {
    self.push(WindowBuilderOp::TrafficLightPosition(position.into()))
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(self, hidden: bool) -> Self {
    self.push(WindowBuilderOp::HiddenTitle(hidden))
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(self, identifier: &str) -> Self {
    self.push(WindowBuilderOp::TabbingIdentifier(identifier.to_string()))
  }

  fn theme(self, theme: Option<Theme>) -> Self {
    self.push(WindowBuilderOp::Theme(theme))
  }

  fn has_icon(&self) -> bool {
    self
      .ops
      .iter()
      .any(|op| matches!(op, WindowBuilderOp::Icon(_)))
  }

  fn get_theme(&self) -> Option<Theme> {
    self
      .ops
      .iter()
      .rev()
      .find_map(|op| match op {
        WindowBuilderOp::Theme(theme) => Some(*theme),
        _ => None,
      })
      .unwrap_or_else(|| self.config.as_ref().and_then(|config| config.theme))
  }

  fn window_classname<S: Into<String>>(self, window_classname: S) -> Self {
    self.push(WindowBuilderOp::WindowClassname(window_classname.into()))
  }

  fn no_redirection_bitmap(self, enable: bool) -> Self {
    self.push(WindowBuilderOp::NoRedirectionBitmap(enable))
  }

  #[cfg(target_os = "android")]
  fn activity_name<S: Into<String>>(self, class_name: S) -> Self {
    self.push(WindowBuilderOp::ActivityName(class_name.into()))
  }

  #[cfg(target_os = "android")]
  fn created_by_activity_name<S: Into<String>>(self, class_name: S) -> Self {
    self.push(WindowBuilderOp::CreatedByActivityName(class_name.into()))
  }

  #[cfg(target_os = "ios")]
  fn requested_by_scene_identifier<S: Into<String>>(self, identifier: S) -> Self {
    self.push(WindowBuilderOp::RequestedBySceneIdentifier(
      identifier.into(),
    ))
  }
}

// ---------------------------------------------------------------------------
// Conversions between the erased and the concrete pending/detached types
// ---------------------------------------------------------------------------

fn pending_window_from_dyn<T: UserEvent, R: Runtime<T>>(
  pending: PendingWindow<T, DynRuntime<T>>,
) -> Result<PendingWindow<T, R>> {
  let PendingWindow {
    label,
    window_builder,
    webview,
  } = pending;
  Ok(PendingWindow {
    label,
    window_builder: window_builder.apply()?,
    webview: webview.map(pending_webview_from_dyn::<T, R>).transpose()?,
  })
}

fn pending_webview_from_dyn<T: UserEvent, R: Runtime<T>>(
  pending: PendingWebview<T, DynRuntime<T>>,
) -> Result<PendingWebview<T, R>> {
  let PendingWebview {
    label,
    webview_attributes,
    opener,
    platform_specific_attributes,
    uri_scheme_protocols,
    ipc_handler,
    navigation_handler,
    new_window_handler,
    document_title_changed_handler,
    address_changed_handler,
    url,
    #[cfg(target_os = "android")]
    on_webview_created,
    web_resource_request_handler,
    on_page_load_handler,
    download_handler,
    permission_request_handler,
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    on_web_content_process_terminate_handler,
  } = pending;

  let opener = opener
    .map(|opener| opener.downcast::<R::WindowOpener>())
    .transpose()?;

  let platform_specific_attributes = platform_specific_attributes
    .into_iter()
    .map(|attribute| attribute.downcast::<R::PlatformSpecificWebviewAttribute>())
    .collect::<Result<Vec<_>>>()?;

  let ipc_handler = ipc_handler.map(|handler| -> WebviewIpcHandler<T, R> {
    Box::new(move |webview, request| handler(detached_webview_into_dyn(webview), request))
  });

  let new_window_handler = new_window_handler.map(|handler| -> Box<NewWindowHandler<T, R>> {
    Box::new(move |url, features| handler(url, new_window_features_into_dyn(features)))
  });

  Ok(PendingWebview {
    label,
    webview_attributes,
    opener,
    platform_specific_attributes,
    uri_scheme_protocols,
    ipc_handler,
    navigation_handler,
    new_window_handler,
    document_title_changed_handler,
    address_changed_handler,
    url,
    #[cfg(target_os = "android")]
    on_webview_created,
    web_resource_request_handler,
    on_page_load_handler,
    download_handler,
    permission_request_handler,
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    on_web_content_process_terminate_handler,
  })
}

fn new_window_features_into_dyn<T: UserEvent, R: Runtime<T>>(
  features: NewWindowFeatures<T, R>,
) -> NewWindowFeatures<T, DynRuntime<T>> {
  let size = features.size();
  let position = features.position();
  NewWindowFeatures::new(size, position, DynWindowOpener::new(features.into_opener()))
}

fn detached_window_into_dyn<T: UserEvent, R: Runtime<T>>(
  window: DetachedWindow<T, R>,
) -> DetachedWindow<T, DynRuntime<T>> {
  DetachedWindow {
    id: window.id,
    label: window.label,
    dispatcher: DynWindowDispatcher::new(window.dispatcher),
    webview: window.webview.map(|webview| DetachedWindowWebview {
      webview: detached_webview_into_dyn(webview.webview),
      use_https_scheme: webview.use_https_scheme,
      devtools: webview.devtools,
    }),
  }
}

fn detached_webview_into_dyn<T: UserEvent, R: Runtime<T>>(
  webview: DetachedWebview<T, R>,
) -> DetachedWebview<T, DynRuntime<T>> {
  DetachedWebview {
    label: webview.label,
    dispatcher: DynWebviewDispatcher::new(webview.dispatcher),
  }
}

// ---------------------------------------------------------------------------
// Event loop proxy
// ---------------------------------------------------------------------------

trait ErasedEventLoopProxy<T: UserEvent>: fmt::Debug + Send + Sync {
  fn send_event(&self, event: T) -> Result<()>;
}

impl<T: UserEvent, P: EventLoopProxy<T>> ErasedEventLoopProxy<T> for P {
  fn send_event(&self, event: T) -> Result<()> {
    EventLoopProxy::send_event(self, event)
  }
}

/// The [`EventLoopProxy`] of [`DynRuntime`].
#[derive(Debug)]
pub struct DynEventLoopProxy<T: UserEvent> {
  inner: Arc<dyn ErasedEventLoopProxy<T>>,
}

impl<T: UserEvent> Clone for DynEventLoopProxy<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T: UserEvent> DynEventLoopProxy<T> {
  fn new<P: EventLoopProxy<T> + 'static>(proxy: P) -> Self {
    Self {
      inner: Arc::new(proxy),
    }
  }
}

impl<T: UserEvent> EventLoopProxy<T> for DynEventLoopProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self.inner.send_event(event)
  }
}

// ---------------------------------------------------------------------------
// Runtime handle
// ---------------------------------------------------------------------------

trait ErasedRuntimeHandle<T: UserEvent>: fmt::Debug + Send + Sync + Any {
  fn create_proxy(&self) -> DynEventLoopProxy<T>;
  #[cfg(target_os = "macos")]
  fn set_activation_policy(&self, activation_policy: ActivationPolicy) -> Result<()>;
  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool) -> Result<()>;
  fn request_exit(&self, code: i32) -> Result<()>;
  fn create_window(
    &self,
    pending: PendingWindow<T, DynRuntime<T>>,
    after_window_creation: Option<AfterWindowCreation>,
  ) -> Result<DetachedWindow<T, DynRuntime<T>>>;
  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, DynRuntime<T>>,
  ) -> Result<DetachedWebview<T, DynRuntime<T>>>;
  fn run_on_main_thread(&self, f: MainThreadTask) -> Result<()>;
  fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError>;
  fn primary_monitor(&self) -> Result<Option<Monitor>>;
  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>>;
  fn available_monitors(&self) -> Result<Vec<Monitor>>;
  fn cursor_position(&self) -> Result<PhysicalPosition<f64>>;
  fn set_theme(&self, theme: Option<Theme>);
  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()>;
  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()>;
  fn set_device_event_filter(&self, filter: DeviceEventFilter);
  fn custom_scheme_url(&self, scheme: &str, https: bool) -> String;
  fn webview_version(&self) -> Result<String>;
  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &self,
    env: &mut jni::JNIEnv<'a>,
    activity: &jni::objects::JObject<'_>,
    name: String,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error>;
  #[cfg(target_os = "android")]
  fn run_on_android_context(
    &self,
    f: Box<dyn FnOnce(&mut jni::JNIEnv, &jni::objects::JObject, &jni::objects::JObject) + Send>,
  );
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn fetch_data_store_identifiers(&self, cb: Box<dyn FnOnce(Vec<[u8; 16]>) + Send>) -> Result<()>;
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn remove_data_store(&self, uuid: [u8; 16], cb: Box<dyn FnOnce(Result<()>) + Send>)
  -> Result<()>;
  fn as_any(&self) -> &dyn Any;
}

impl<T: UserEvent, H: RuntimeHandle<T>> ErasedRuntimeHandle<T> for H {
  fn create_proxy(&self) -> DynEventLoopProxy<T> {
    DynEventLoopProxy::new(RuntimeHandle::create_proxy(self))
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&self, activation_policy: ActivationPolicy) -> Result<()> {
    RuntimeHandle::set_activation_policy(self, activation_policy)
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool) -> Result<()> {
    RuntimeHandle::set_dock_visibility(self, visible)
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    RuntimeHandle::request_exit(self, code)
  }

  fn create_window(
    &self,
    pending: PendingWindow<T, DynRuntime<T>>,
    after_window_creation: Option<AfterWindowCreation>,
  ) -> Result<DetachedWindow<T, DynRuntime<T>>> {
    let pending = pending_window_from_dyn::<T, H::Runtime>(pending)?;
    RuntimeHandle::create_window(self, pending, after_window_creation).map(detached_window_into_dyn)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, DynRuntime<T>>,
  ) -> Result<DetachedWebview<T, DynRuntime<T>>> {
    let pending = pending_webview_from_dyn::<T, H::Runtime>(pending)?;
    RuntimeHandle::create_webview(self, window_id, pending).map(detached_webview_into_dyn)
  }

  fn run_on_main_thread(&self, f: MainThreadTask) -> Result<()> {
    RuntimeHandle::run_on_main_thread(self, f)
  }

  fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
    RuntimeHandle::display_handle(self)
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    RuntimeHandle::primary_monitor(self)
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    RuntimeHandle::monitor_from_point(self, x, y)
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    RuntimeHandle::available_monitors(self)
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    RuntimeHandle::cursor_position(self)
  }

  fn set_theme(&self, theme: Option<Theme>) {
    RuntimeHandle::set_theme(self, theme)
  }

  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()> {
    RuntimeHandle::show(self)
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()> {
    RuntimeHandle::hide(self)
  }

  fn set_device_event_filter(&self, filter: DeviceEventFilter) {
    RuntimeHandle::set_device_event_filter(self, filter)
  }

  fn custom_scheme_url(&self, scheme: &str, https: bool) -> String {
    RuntimeHandle::custom_scheme_url(self, scheme, https)
  }

  fn webview_version(&self) -> Result<String> {
    RuntimeHandle::webview_version(self)
  }

  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &self,
    env: &mut jni::JNIEnv<'a>,
    activity: &jni::objects::JObject<'_>,
    name: String,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error> {
    RuntimeHandle::find_class(self, env, activity, name)
  }

  #[cfg(target_os = "android")]
  fn run_on_android_context(
    &self,
    f: Box<dyn FnOnce(&mut jni::JNIEnv, &jni::objects::JObject, &jni::objects::JObject) + Send>,
  ) {
    RuntimeHandle::run_on_android_context(self, f)
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn fetch_data_store_identifiers(&self, cb: Box<dyn FnOnce(Vec<[u8; 16]>) + Send>) -> Result<()> {
    RuntimeHandle::fetch_data_store_identifiers(self, cb)
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn remove_data_store(
    &self,
    uuid: [u8; 16],
    cb: Box<dyn FnOnce(Result<()>) + Send>,
  ) -> Result<()> {
    RuntimeHandle::remove_data_store(self, uuid, cb)
  }

  fn as_any(&self) -> &dyn Any {
    self
  }
}

/// The [`RuntimeHandle`] of [`DynRuntime`].
#[derive(Debug)]
pub struct DynRuntimeHandle<T: UserEvent> {
  inner: Arc<dyn ErasedRuntimeHandle<T>>,
}

impl<T: UserEvent> Clone for DynRuntimeHandle<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T: UserEvent> DynRuntimeHandle<T> {
  /// Wraps a runtime handle.
  pub fn new<H: RuntimeHandle<T>>(handle: H) -> Self {
    Self {
      inner: Arc::new(handle),
    }
  }

  /// Whether the wrapped handle is of type `H`.
  pub fn is<H: RuntimeHandle<T>>(&self) -> bool {
    self.inner.as_any().is::<H>()
  }

  /// Returns a reference to the wrapped handle if it is of type `H`.
  pub fn downcast_ref<H: RuntimeHandle<T>>(&self) -> Option<&H> {
    self.inner.as_any().downcast_ref()
  }
}

impl<T: UserEvent> RuntimeHandle<T> for DynRuntimeHandle<T> {
  type Runtime = DynRuntime<T>;

  fn create_proxy(&self) -> DynEventLoopProxy<T> {
    self.inner.create_proxy()
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&self, activation_policy: ActivationPolicy) -> Result<()> {
    self.inner.set_activation_policy(activation_policy)
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool) -> Result<()> {
    self.inner.set_dock_visibility(visible)
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    self.inner.request_exit(code)
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.inner.create_window(
      pending,
      after_window_creation.map(|f| Box::new(f) as AfterWindowCreation),
    )
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.inner.create_webview(window_id, pending)
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.inner.run_on_main_thread(Box::new(f))
  }

  fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
    self.inner.display_handle()
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    self.inner.primary_monitor()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    self.inner.monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    self.inner.available_monitors()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    self.inner.cursor_position()
  }

  fn set_theme(&self, theme: Option<Theme>) {
    self.inner.set_theme(theme)
  }

  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()> {
    self.inner.show()
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()> {
    self.inner.hide()
  }

  fn set_device_event_filter(&self, filter: DeviceEventFilter) {
    self.inner.set_device_event_filter(filter)
  }

  fn custom_scheme_url(&self, scheme: &str, https: bool) -> String {
    self.inner.custom_scheme_url(scheme, https)
  }

  fn webview_version(&self) -> Result<String> {
    self.inner.webview_version()
  }

  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &self,
    env: &mut jni::JNIEnv<'a>,
    activity: &jni::objects::JObject<'_>,
    name: impl Into<String>,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error> {
    self.inner.find_class(env, activity, name.into())
  }

  #[cfg(target_os = "android")]
  fn run_on_android_context<F>(&self, f: F)
  where
    F: FnOnce(&mut jni::JNIEnv, &jni::objects::JObject, &jni::objects::JObject) + Send + 'static,
  {
    self.inner.run_on_android_context(Box::new(f))
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn fetch_data_store_identifiers<F: FnOnce(Vec<[u8; 16]>) + Send + 'static>(
    &self,
    cb: F,
  ) -> Result<()> {
    self.inner.fetch_data_store_identifiers(Box::new(cb))
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn remove_data_store<F: FnOnce(Result<()>) + Send + 'static>(
    &self,
    uuid: [u8; 16],
    cb: F,
  ) -> Result<()> {
    self.inner.remove_data_store(uuid, Box::new(cb))
  }
}

// ---------------------------------------------------------------------------
// Window dispatcher
// ---------------------------------------------------------------------------

trait ErasedWindowDispatch<T: UserEvent>: fmt::Debug + Send + Sync + Any {
  fn box_clone(&self) -> Box<dyn ErasedWindowDispatch<T>>;
  fn run_on_main_thread(&self, f: MainThreadTask) -> Result<()>;
  fn on_window_event(&self, f: Box<dyn Fn(&WindowEvent) + Send>) -> WindowEventId;
  fn scale_factor(&self) -> Result<f64>;
  fn inner_position(&self) -> Result<PhysicalPosition<i32>>;
  fn outer_position(&self) -> Result<PhysicalPosition<i32>>;
  fn inner_size(&self) -> Result<PhysicalSize<u32>>;
  fn outer_size(&self) -> Result<PhysicalSize<u32>>;
  fn is_fullscreen(&self) -> Result<bool>;
  fn is_minimized(&self) -> Result<bool>;
  fn is_maximized(&self) -> Result<bool>;
  fn is_focused(&self) -> Result<bool>;
  fn is_decorated(&self) -> Result<bool>;
  fn is_resizable(&self) -> Result<bool>;
  fn is_maximizable(&self) -> Result<bool>;
  fn is_minimizable(&self) -> Result<bool>;
  fn is_closable(&self) -> Result<bool>;
  fn is_visible(&self) -> Result<bool>;
  fn is_enabled(&self) -> Result<bool>;
  fn is_always_on_top(&self) -> Result<bool>;
  fn title(&self) -> Result<String>;
  fn current_monitor(&self) -> Result<Option<Monitor>>;
  fn primary_monitor(&self) -> Result<Option<Monitor>>;
  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>>;
  fn available_monitors(&self) -> Result<Vec<Monitor>>;
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow>;
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn default_vbox(&self) -> Result<gtk::Box>;
  #[cfg(target_os = "android")]
  fn activity_name(&self) -> Result<String>;
  #[cfg(target_os = "ios")]
  fn scene_identifier(&self) -> Result<String>;
  fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError>;
  fn theme(&self) -> Result<Theme>;
  fn center(&self) -> Result<()>;
  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()>;
  fn create_window(
    &mut self,
    pending: PendingWindow<T, DynRuntime<T>>,
    after_window_creation: Option<AfterWindowCreation>,
  ) -> Result<DetachedWindow<T, DynRuntime<T>>>;
  fn create_webview(
    &mut self,
    pending: PendingWebview<T, DynRuntime<T>>,
  ) -> Result<DetachedWebview<T, DynRuntime<T>>>;
  fn set_resizable(&self, resizable: bool) -> Result<()>;
  fn set_enabled(&self, enabled: bool) -> Result<()>;
  fn set_maximizable(&self, maximizable: bool) -> Result<()>;
  fn set_minimizable(&self, minimizable: bool) -> Result<()>;
  fn set_closable(&self, closable: bool) -> Result<()>;
  fn set_title(&self, title: String) -> Result<()>;
  fn maximize(&self) -> Result<()>;
  fn unmaximize(&self) -> Result<()>;
  fn minimize(&self) -> Result<()>;
  fn unminimize(&self) -> Result<()>;
  fn show(&self) -> Result<()>;
  fn hide(&self) -> Result<()>;
  fn close(&self) -> Result<()>;
  fn destroy(&self) -> Result<()>;
  fn set_decorations(&self, decorations: bool) -> Result<()>;
  fn set_shadow(&self, enable: bool) -> Result<()>;
  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()>;
  fn set_always_on_top(&self, always_on_top: bool) -> Result<()>;
  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()>;
  fn set_background_color(&self, color: Option<Color>) -> Result<()>;
  fn set_content_protected(&self, protected: bool) -> Result<()>;
  fn set_size(&self, size: Size) -> Result<()>;
  fn set_min_size(&self, size: Option<Size>) -> Result<()>;
  fn set_max_size(&self, size: Option<Size>) -> Result<()>;
  fn set_size_constraints(&self, constraints: WindowSizeConstraints) -> Result<()>;
  fn set_position(&self, position: Position) -> Result<()>;
  fn set_fullscreen(&self, fullscreen: bool) -> Result<()>;
  #[cfg(target_os = "macos")]
  fn set_simple_fullscreen(&self, enable: bool) -> Result<()>;
  fn set_focus(&self) -> Result<()>;
  fn set_focusable(&self, focusable: bool) -> Result<()>;
  fn set_icon(&self, icon: Icon<'_>) -> Result<()>;
  fn set_skip_taskbar(&self, skip: bool) -> Result<()>;
  fn set_cursor_grab(&self, grab: bool) -> Result<()>;
  fn set_cursor_visible(&self, visible: bool) -> Result<()>;
  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()>;
  fn set_cursor_position(&self, position: Position) -> Result<()>;
  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()>;
  fn start_dragging(&self) -> Result<()>;
  fn start_resize_dragging(&self, direction: ResizeDirection) -> Result<()>;
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()>;
  fn set_badge_label(&self, label: Option<String>) -> Result<()>;
  fn set_overlay_icon(&self, icon: Option<Icon<'_>>) -> Result<()>;
  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()>;
  fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> Result<()>;
  fn set_traffic_light_position(&self, position: Position) -> Result<()>;
  fn set_theme(&self, theme: Option<Theme>) -> Result<()>;
  fn as_any(&self) -> &dyn Any;
}

impl<T: UserEvent, D: WindowDispatch<T>> ErasedWindowDispatch<T> for D {
  fn box_clone(&self) -> Box<dyn ErasedWindowDispatch<T>> {
    Box::new(self.clone())
  }

  fn run_on_main_thread(&self, f: MainThreadTask) -> Result<()> {
    WindowDispatch::run_on_main_thread(self, f)
  }

  fn on_window_event(&self, f: Box<dyn Fn(&WindowEvent) + Send>) -> WindowEventId {
    WindowDispatch::on_window_event(self, f)
  }

  fn scale_factor(&self) -> Result<f64> {
    WindowDispatch::scale_factor(self)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    WindowDispatch::inner_position(self)
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    WindowDispatch::outer_position(self)
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    WindowDispatch::inner_size(self)
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    WindowDispatch::outer_size(self)
  }

  fn is_fullscreen(&self) -> Result<bool> {
    WindowDispatch::is_fullscreen(self)
  }

  fn is_minimized(&self) -> Result<bool> {
    WindowDispatch::is_minimized(self)
  }

  fn is_maximized(&self) -> Result<bool> {
    WindowDispatch::is_maximized(self)
  }

  fn is_focused(&self) -> Result<bool> {
    WindowDispatch::is_focused(self)
  }

  fn is_decorated(&self) -> Result<bool> {
    WindowDispatch::is_decorated(self)
  }

  fn is_resizable(&self) -> Result<bool> {
    WindowDispatch::is_resizable(self)
  }

  fn is_maximizable(&self) -> Result<bool> {
    WindowDispatch::is_maximizable(self)
  }

  fn is_minimizable(&self) -> Result<bool> {
    WindowDispatch::is_minimizable(self)
  }

  fn is_closable(&self) -> Result<bool> {
    WindowDispatch::is_closable(self)
  }

  fn is_visible(&self) -> Result<bool> {
    WindowDispatch::is_visible(self)
  }

  fn is_enabled(&self) -> Result<bool> {
    WindowDispatch::is_enabled(self)
  }

  fn is_always_on_top(&self) -> Result<bool> {
    WindowDispatch::is_always_on_top(self)
  }

  fn title(&self) -> Result<String> {
    WindowDispatch::title(self)
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    WindowDispatch::current_monitor(self)
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    WindowDispatch::primary_monitor(self)
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    WindowDispatch::monitor_from_point(self, x, y)
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    WindowDispatch::available_monitors(self)
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    WindowDispatch::gtk_window(self)
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn default_vbox(&self) -> Result<gtk::Box> {
    WindowDispatch::default_vbox(self)
  }

  #[cfg(target_os = "android")]
  fn activity_name(&self) -> Result<String> {
    WindowDispatch::activity_name(self)
  }

  #[cfg(target_os = "ios")]
  fn scene_identifier(&self) -> Result<String> {
    WindowDispatch::scene_identifier(self)
  }

  fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
    WindowDispatch::window_handle(self)
  }

  fn theme(&self) -> Result<Theme> {
    WindowDispatch::theme(self)
  }

  fn center(&self) -> Result<()> {
    WindowDispatch::center(self)
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    WindowDispatch::request_user_attention(self, request_type)
  }

  fn create_window(
    &mut self,
    pending: PendingWindow<T, DynRuntime<T>>,
    after_window_creation: Option<AfterWindowCreation>,
  ) -> Result<DetachedWindow<T, DynRuntime<T>>> {
    let pending = pending_window_from_dyn::<T, D::Runtime>(pending)?;
    WindowDispatch::create_window(self, pending, after_window_creation)
      .map(detached_window_into_dyn)
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, DynRuntime<T>>,
  ) -> Result<DetachedWebview<T, DynRuntime<T>>> {
    let pending = pending_webview_from_dyn::<T, D::Runtime>(pending)?;
    WindowDispatch::create_webview(self, pending).map(detached_webview_into_dyn)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    WindowDispatch::set_resizable(self, resizable)
  }

  fn set_enabled(&self, enabled: bool) -> Result<()> {
    WindowDispatch::set_enabled(self, enabled)
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    WindowDispatch::set_maximizable(self, maximizable)
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    WindowDispatch::set_minimizable(self, minimizable)
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    WindowDispatch::set_closable(self, closable)
  }

  fn set_title(&self, title: String) -> Result<()> {
    WindowDispatch::set_title(self, title)
  }

  fn maximize(&self) -> Result<()> {
    WindowDispatch::maximize(self)
  }

  fn unmaximize(&self) -> Result<()> {
    WindowDispatch::unmaximize(self)
  }

  fn minimize(&self) -> Result<()> {
    WindowDispatch::minimize(self)
  }

  fn unminimize(&self) -> Result<()> {
    WindowDispatch::unminimize(self)
  }

  fn show(&self) -> Result<()> {
    WindowDispatch::show(self)
  }

  fn hide(&self) -> Result<()> {
    WindowDispatch::hide(self)
  }

  fn close(&self) -> Result<()> {
    WindowDispatch::close(self)
  }

  fn destroy(&self) -> Result<()> {
    WindowDispatch::destroy(self)
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    WindowDispatch::set_decorations(self, decorations)
  }

  fn set_shadow(&self, enable: bool) -> Result<()> {
    WindowDispatch::set_shadow(self, enable)
  }

  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    WindowDispatch::set_always_on_bottom(self, always_on_bottom)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    WindowDispatch::set_always_on_top(self, always_on_top)
  }

  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    WindowDispatch::set_visible_on_all_workspaces(self, visible_on_all_workspaces)
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    WindowDispatch::set_background_color(self, color)
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    WindowDispatch::set_content_protected(self, protected)
  }

  fn set_size(&self, size: Size) -> Result<()> {
    WindowDispatch::set_size(self, size)
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    WindowDispatch::set_min_size(self, size)
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    WindowDispatch::set_max_size(self, size)
  }

  fn set_size_constraints(&self, constraints: WindowSizeConstraints) -> Result<()> {
    WindowDispatch::set_size_constraints(self, constraints)
  }

  fn set_position(&self, position: Position) -> Result<()> {
    WindowDispatch::set_position(self, position)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    WindowDispatch::set_fullscreen(self, fullscreen)
  }

  #[cfg(target_os = "macos")]
  fn set_simple_fullscreen(&self, enable: bool) -> Result<()> {
    WindowDispatch::set_simple_fullscreen(self, enable)
  }

  fn set_focus(&self) -> Result<()> {
    WindowDispatch::set_focus(self)
  }

  fn set_focusable(&self, focusable: bool) -> Result<()> {
    WindowDispatch::set_focusable(self, focusable)
  }

  fn set_icon(&self, icon: Icon<'_>) -> Result<()> {
    WindowDispatch::set_icon(self, icon)
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    WindowDispatch::set_skip_taskbar(self, skip)
  }

  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    WindowDispatch::set_cursor_grab(self, grab)
  }

  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    WindowDispatch::set_cursor_visible(self, visible)
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    WindowDispatch::set_cursor_icon(self, icon)
  }

  fn set_cursor_position(&self, position: Position) -> Result<()> {
    WindowDispatch::set_cursor_position(self, position)
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    WindowDispatch::set_ignore_cursor_events(self, ignore)
  }

  fn start_dragging(&self) -> Result<()> {
    WindowDispatch::start_dragging(self)
  }

  fn start_resize_dragging(&self, direction: ResizeDirection) -> Result<()> {
    WindowDispatch::start_resize_dragging(self, direction)
  }

  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()> {
    WindowDispatch::set_badge_count(self, count, desktop_filename)
  }

  fn set_badge_label(&self, label: Option<String>) -> Result<()> {
    WindowDispatch::set_badge_label(self, label)
  }

  fn set_overlay_icon(&self, icon: Option<Icon<'_>>) -> Result<()> {
    WindowDispatch::set_overlay_icon(self, icon)
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    WindowDispatch::set_progress_bar(self, progress_state)
  }

  fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> Result<()> {
    WindowDispatch::set_title_bar_style(self, style)
  }

  fn set_traffic_light_position(&self, position: Position) -> Result<()> {
    WindowDispatch::set_traffic_light_position(self, position)
  }

  fn set_theme(&self, theme: Option<Theme>) -> Result<()> {
    WindowDispatch::set_theme(self, theme)
  }

  fn as_any(&self) -> &dyn Any {
    self
  }
}

/// The [`WindowDispatch`] of [`DynRuntime`].
#[derive(Debug)]
pub struct DynWindowDispatcher<T: UserEvent> {
  inner: Box<dyn ErasedWindowDispatch<T>>,
}

impl<T: UserEvent> Clone for DynWindowDispatcher<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.box_clone(),
    }
  }
}

impl<T: UserEvent> DynWindowDispatcher<T> {
  /// Wraps a window dispatcher.
  pub fn new<D: WindowDispatch<T>>(dispatcher: D) -> Self {
    Self {
      inner: Box::new(dispatcher),
    }
  }

  /// Whether the wrapped dispatcher is of type `D`.
  pub fn is<D: WindowDispatch<T>>(&self) -> bool {
    self.inner.as_any().is::<D>()
  }

  /// Returns a reference to the wrapped dispatcher if it is of type `D`.
  pub fn downcast_ref<D: WindowDispatch<T>>(&self) -> Option<&D> {
    self.inner.as_any().downcast_ref()
  }
}

impl<T: UserEvent> WindowDispatch<T> for DynWindowDispatcher<T> {
  type Runtime = DynRuntime<T>;
  type WindowBuilder = DynWindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.inner.run_on_main_thread(Box::new(f))
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> WindowEventId {
    self.inner.on_window_event(Box::new(f))
  }

  fn scale_factor(&self) -> Result<f64> {
    self.inner.scale_factor()
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    self.inner.inner_position()
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    self.inner.outer_position()
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    self.inner.inner_size()
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    self.inner.outer_size()
  }

  fn is_fullscreen(&self) -> Result<bool> {
    self.inner.is_fullscreen()
  }

  fn is_minimized(&self) -> Result<bool> {
    self.inner.is_minimized()
  }

  fn is_maximized(&self) -> Result<bool> {
    self.inner.is_maximized()
  }

  fn is_focused(&self) -> Result<bool> {
    self.inner.is_focused()
  }

  fn is_decorated(&self) -> Result<bool> {
    self.inner.is_decorated()
  }

  fn is_resizable(&self) -> Result<bool> {
    self.inner.is_resizable()
  }

  fn is_maximizable(&self) -> Result<bool> {
    self.inner.is_maximizable()
  }

  fn is_minimizable(&self) -> Result<bool> {
    self.inner.is_minimizable()
  }

  fn is_closable(&self) -> Result<bool> {
    self.inner.is_closable()
  }

  fn is_visible(&self) -> Result<bool> {
    self.inner.is_visible()
  }

  fn is_enabled(&self) -> Result<bool> {
    self.inner.is_enabled()
  }

  fn is_always_on_top(&self) -> Result<bool> {
    self.inner.is_always_on_top()
  }

  fn title(&self) -> Result<String> {
    self.inner.title()
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    self.inner.current_monitor()
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    self.inner.primary_monitor()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    self.inner.monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    self.inner.available_monitors()
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    self.inner.gtk_window()
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn default_vbox(&self) -> Result<gtk::Box> {
    self.inner.default_vbox()
  }

  #[cfg(target_os = "android")]
  fn activity_name(&self) -> Result<String> {
    self.inner.activity_name()
  }

  #[cfg(target_os = "ios")]
  fn scene_identifier(&self) -> Result<String> {
    self.inner.scene_identifier()
  }

  fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
    self.inner.window_handle()
  }

  fn theme(&self) -> Result<Theme> {
    self.inner.theme()
  }

  fn center(&self) -> Result<()> {
    self.inner.center()
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    self.inner.request_user_attention(request_type)
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.inner.create_window(
      pending,
      after_window_creation.map(|f| Box::new(f) as AfterWindowCreation),
    )
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.inner.create_webview(pending)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    self.inner.set_resizable(resizable)
  }

  fn set_enabled(&self, enabled: bool) -> Result<()> {
    self.inner.set_enabled(enabled)
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    self.inner.set_maximizable(maximizable)
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    self.inner.set_minimizable(minimizable)
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    self.inner.set_closable(closable)
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    self.inner.set_title(title.into())
  }

  fn maximize(&self) -> Result<()> {
    self.inner.maximize()
  }

  fn unmaximize(&self) -> Result<()> {
    self.inner.unmaximize()
  }

  fn minimize(&self) -> Result<()> {
    self.inner.minimize()
  }

  fn unminimize(&self) -> Result<()> {
    self.inner.unminimize()
  }

  fn show(&self) -> Result<()> {
    self.inner.show()
  }

  fn hide(&self) -> Result<()> {
    self.inner.hide()
  }

  fn close(&self) -> Result<()> {
    self.inner.close()
  }

  fn destroy(&self) -> Result<()> {
    self.inner.destroy()
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    self.inner.set_decorations(decorations)
  }

  fn set_shadow(&self, enable: bool) -> Result<()> {
    self.inner.set_shadow(enable)
  }

  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    self.inner.set_always_on_bottom(always_on_bottom)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    self.inner.set_always_on_top(always_on_top)
  }

  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    self
      .inner
      .set_visible_on_all_workspaces(visible_on_all_workspaces)
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.inner.set_background_color(color)
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    self.inner.set_content_protected(protected)
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.inner.set_size(size)
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    self.inner.set_min_size(size)
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    self.inner.set_max_size(size)
  }

  fn set_size_constraints(&self, constraints: WindowSizeConstraints) -> Result<()> {
    self.inner.set_size_constraints(constraints)
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.inner.set_position(position)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self.inner.set_fullscreen(fullscreen)
  }

  #[cfg(target_os = "macos")]
  fn set_simple_fullscreen(&self, enable: bool) -> Result<()> {
    self.inner.set_simple_fullscreen(enable)
  }

  fn set_focus(&self) -> Result<()> {
    self.inner.set_focus()
  }

  fn set_focusable(&self, focusable: bool) -> Result<()> {
    self.inner.set_focusable(focusable)
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    self.inner.set_icon(icon)
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    self.inner.set_skip_taskbar(skip)
  }

  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    self.inner.set_cursor_grab(grab)
  }

  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    self.inner.set_cursor_visible(visible)
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    self.inner.set_cursor_icon(icon)
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()> {
    self.inner.set_cursor_position(position.into())
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    self.inner.set_ignore_cursor_events(ignore)
  }

  fn start_dragging(&self) -> Result<()> {
    self.inner.start_dragging()
  }

  fn start_resize_dragging(&self, direction: ResizeDirection) -> Result<()> {
    self.inner.start_resize_dragging(direction)
  }

  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()> {
    self.inner.set_badge_count(count, desktop_filename)
  }

  fn set_badge_label(&self, label: Option<String>) -> Result<()> {
    self.inner.set_badge_label(label)
  }

  fn set_overlay_icon(&self, icon: Option<Icon>) -> Result<()> {
    self.inner.set_overlay_icon(icon)
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    self.inner.set_progress_bar(progress_state)
  }

  fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> Result<()> {
    self.inner.set_title_bar_style(style)
  }

  fn set_traffic_light_position(&self, position: Position) -> Result<()> {
    self.inner.set_traffic_light_position(position)
  }

  fn set_theme(&self, theme: Option<Theme>) -> Result<()> {
    self.inner.set_theme(theme)
  }
}

// ---------------------------------------------------------------------------
// Webview dispatcher
// ---------------------------------------------------------------------------

trait ErasedWebviewDispatch<T: UserEvent>: fmt::Debug + Send + Sync + Any {
  fn box_clone(&self) -> Box<dyn ErasedWebviewDispatch<T>>;
  fn run_on_main_thread(&self, f: MainThreadTask) -> Result<()>;
  fn on_webview_event(&self, f: Box<dyn Fn(&WebviewEvent) + Send>) -> WebviewEventId;
  fn with_webview(&self, f: Box<dyn FnOnce(DynWebview) + Send>) -> Result<()>;
  #[cfg(target_os = "ios")]
  fn with_ios_webview(
    &self,
    f: Box<dyn FnOnce(crate::webview::IosWebviewHandle) + Send>,
  ) -> Result<()>;
  fn open_devtools(&self);
  fn close_devtools(&self);
  fn is_devtools_open(&self) -> Result<bool>;
  fn url(&self) -> Result<String>;
  fn bounds(&self) -> Result<Rect>;
  fn position(&self) -> Result<PhysicalPosition<i32>>;
  fn size(&self) -> Result<PhysicalSize<u32>>;
  fn navigate(&self, url: Url) -> Result<()>;
  fn reload(&self) -> Result<()>;
  fn go_back(&self) -> Result<()>;
  fn can_go_back(&self) -> Result<bool>;
  fn go_forward(&self) -> Result<()>;
  fn can_go_forward(&self) -> Result<bool>;
  fn print(&self) -> Result<()>;
  fn close(&self) -> Result<()>;
  fn set_bounds(&self, bounds: Rect) -> Result<()>;
  fn set_size(&self, size: Size) -> Result<()>;
  fn set_position(&self, position: Position) -> Result<()>;
  fn set_focus(&self) -> Result<()>;
  fn hide(&self) -> Result<()>;
  fn show(&self) -> Result<()>;
  fn eval_script(&self, script: String) -> Result<()>;
  fn eval_script_with_callback(
    &self,
    script: String,
    callback: Box<dyn Fn(String) + Send>,
  ) -> Result<()>;
  fn reparent(&self, window_id: WindowId) -> Result<()>;
  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>>;
  fn cookies(&self) -> Result<Vec<Cookie<'static>>>;
  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()>;
  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()>;
  fn set_auto_resize(&self, auto_resize: bool) -> Result<()>;
  fn set_zoom(&self, scale_factor: f64) -> Result<()>;
  fn set_background_color(&self, color: Option<Color>) -> Result<()>;
  fn clear_all_browsing_data(&self) -> Result<()>;
  fn as_any(&self) -> &dyn Any;
}

impl<T: UserEvent, D: WebviewDispatch<T>> ErasedWebviewDispatch<T> for D {
  fn box_clone(&self) -> Box<dyn ErasedWebviewDispatch<T>> {
    Box::new(self.clone())
  }

  fn run_on_main_thread(&self, f: MainThreadTask) -> Result<()> {
    WebviewDispatch::run_on_main_thread(self, f)
  }

  fn on_webview_event(&self, f: Box<dyn Fn(&WebviewEvent) + Send>) -> WebviewEventId {
    WebviewDispatch::on_webview_event(self, f)
  }

  fn with_webview(&self, f: Box<dyn FnOnce(DynWebview) + Send>) -> Result<()> {
    WebviewDispatch::with_webview(self, move |webview| f(DynWebview::new(webview)))
  }

  #[cfg(target_os = "ios")]
  fn with_ios_webview(
    &self,
    f: Box<dyn FnOnce(crate::webview::IosWebviewHandle) + Send>,
  ) -> Result<()> {
    WebviewDispatch::with_ios_webview(self, f)
  }

  fn open_devtools(&self) {
    WebviewDispatch::open_devtools(self)
  }

  fn close_devtools(&self) {
    WebviewDispatch::close_devtools(self)
  }

  fn is_devtools_open(&self) -> Result<bool> {
    WebviewDispatch::is_devtools_open(self)
  }

  fn url(&self) -> Result<String> {
    WebviewDispatch::url(self)
  }

  fn bounds(&self) -> Result<Rect> {
    WebviewDispatch::bounds(self)
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    WebviewDispatch::position(self)
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    WebviewDispatch::size(self)
  }

  fn navigate(&self, url: Url) -> Result<()> {
    WebviewDispatch::navigate(self, url)
  }

  fn reload(&self) -> Result<()> {
    WebviewDispatch::reload(self)
  }

  fn go_back(&self) -> Result<()> {
    WebviewDispatch::go_back(self)
  }

  fn can_go_back(&self) -> Result<bool> {
    WebviewDispatch::can_go_back(self)
  }

  fn go_forward(&self) -> Result<()> {
    WebviewDispatch::go_forward(self)
  }

  fn can_go_forward(&self) -> Result<bool> {
    WebviewDispatch::can_go_forward(self)
  }

  fn print(&self) -> Result<()> {
    WebviewDispatch::print(self)
  }

  fn close(&self) -> Result<()> {
    WebviewDispatch::close(self)
  }

  fn set_bounds(&self, bounds: Rect) -> Result<()> {
    WebviewDispatch::set_bounds(self, bounds)
  }

  fn set_size(&self, size: Size) -> Result<()> {
    WebviewDispatch::set_size(self, size)
  }

  fn set_position(&self, position: Position) -> Result<()> {
    WebviewDispatch::set_position(self, position)
  }

  fn set_focus(&self) -> Result<()> {
    WebviewDispatch::set_focus(self)
  }

  fn hide(&self) -> Result<()> {
    WebviewDispatch::hide(self)
  }

  fn show(&self) -> Result<()> {
    WebviewDispatch::show(self)
  }

  fn eval_script(&self, script: String) -> Result<()> {
    WebviewDispatch::eval_script(self, script)
  }

  fn eval_script_with_callback(
    &self,
    script: String,
    callback: Box<dyn Fn(String) + Send>,
  ) -> Result<()> {
    WebviewDispatch::eval_script_with_callback(self, script, callback)
  }

  fn reparent(&self, window_id: WindowId) -> Result<()> {
    WebviewDispatch::reparent(self, window_id)
  }

  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>> {
    WebviewDispatch::cookies_for_url(self, url)
  }

  fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
    WebviewDispatch::cookies(self)
  }

  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    WebviewDispatch::set_cookie(self, cookie)
  }

  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    WebviewDispatch::delete_cookie(self, cookie)
  }

  fn set_auto_resize(&self, auto_resize: bool) -> Result<()> {
    WebviewDispatch::set_auto_resize(self, auto_resize)
  }

  fn set_zoom(&self, scale_factor: f64) -> Result<()> {
    WebviewDispatch::set_zoom(self, scale_factor)
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    WebviewDispatch::set_background_color(self, color)
  }

  fn clear_all_browsing_data(&self) -> Result<()> {
    WebviewDispatch::clear_all_browsing_data(self)
  }

  fn as_any(&self) -> &dyn Any {
    self
  }
}

/// The [`WebviewDispatch`] of [`DynRuntime`].
#[derive(Debug)]
pub struct DynWebviewDispatcher<T: UserEvent> {
  inner: Box<dyn ErasedWebviewDispatch<T>>,
}

impl<T: UserEvent> Clone for DynWebviewDispatcher<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.box_clone(),
    }
  }
}

impl<T: UserEvent> DynWebviewDispatcher<T> {
  /// Wraps a webview dispatcher.
  pub fn new<D: WebviewDispatch<T>>(dispatcher: D) -> Self {
    Self {
      inner: Box::new(dispatcher),
    }
  }

  /// Whether the wrapped dispatcher is of type `D`.
  pub fn is<D: WebviewDispatch<T>>(&self) -> bool {
    self.inner.as_any().is::<D>()
  }

  /// Returns a reference to the wrapped dispatcher if it is of type `D`.
  pub fn downcast_ref<D: WebviewDispatch<T>>(&self) -> Option<&D> {
    self.inner.as_any().downcast_ref()
  }
}

impl<T: UserEvent> WebviewDispatch<T> for DynWebviewDispatcher<T> {
  type Runtime = DynRuntime<T>;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.inner.run_on_main_thread(Box::new(f))
  }

  fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) -> WebviewEventId {
    self.inner.on_webview_event(Box::new(f))
  }

  fn with_webview<F: FnOnce(DynWebview) + Send + 'static>(&self, f: F) -> Result<()> {
    self.inner.with_webview(Box::new(f))
  }

  #[cfg(target_os = "ios")]
  fn with_ios_webview<F: FnOnce(crate::webview::IosWebviewHandle) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    self.inner.with_ios_webview(Box::new(f))
  }

  fn open_devtools(&self) {
    self.inner.open_devtools()
  }

  fn close_devtools(&self) {
    self.inner.close_devtools()
  }

  fn is_devtools_open(&self) -> Result<bool> {
    self.inner.is_devtools_open()
  }

  fn url(&self) -> Result<String> {
    self.inner.url()
  }

  fn bounds(&self) -> Result<Rect> {
    self.inner.bounds()
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    self.inner.position()
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    self.inner.size()
  }

  fn navigate(&self, url: Url) -> Result<()> {
    self.inner.navigate(url)
  }

  fn reload(&self) -> Result<()> {
    self.inner.reload()
  }

  fn go_back(&self) -> Result<()> {
    self.inner.go_back()
  }

  fn can_go_back(&self) -> Result<bool> {
    self.inner.can_go_back()
  }

  fn go_forward(&self) -> Result<()> {
    self.inner.go_forward()
  }

  fn can_go_forward(&self) -> Result<bool> {
    self.inner.can_go_forward()
  }

  fn print(&self) -> Result<()> {
    self.inner.print()
  }

  fn close(&self) -> Result<()> {
    self.inner.close()
  }

  fn set_bounds(&self, bounds: Rect) -> Result<()> {
    self.inner.set_bounds(bounds)
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.inner.set_size(size)
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.inner.set_position(position)
  }

  fn set_focus(&self) -> Result<()> {
    self.inner.set_focus()
  }

  fn hide(&self) -> Result<()> {
    self.inner.hide()
  }

  fn show(&self) -> Result<()> {
    self.inner.show()
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    self.inner.eval_script(script.into())
  }

  fn eval_script_with_callback<S: Into<String>>(
    &self,
    script: S,
    callback: impl Fn(String) + Send + 'static,
  ) -> Result<()> {
    self
      .inner
      .eval_script_with_callback(script.into(), Box::new(callback))
  }

  fn reparent(&self, window_id: WindowId) -> Result<()> {
    self.inner.reparent(window_id)
  }

  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>> {
    self.inner.cookies_for_url(url)
  }

  fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
    self.inner.cookies()
  }

  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.inner.set_cookie(cookie)
  }

  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.inner.delete_cookie(cookie)
  }

  fn set_auto_resize(&self, auto_resize: bool) -> Result<()> {
    self.inner.set_auto_resize(auto_resize)
  }

  fn set_zoom(&self, scale_factor: f64) -> Result<()> {
    self.inner.set_zoom(scale_factor)
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.inner.set_background_color(color)
  }

  fn clear_all_browsing_data(&self) -> Result<()> {
    self.inner.clear_all_browsing_data()
  }
}

// ---------------------------------------------------------------------------
// Runtime init attributes
// ---------------------------------------------------------------------------

trait ErasedRuntimeInitAttrs<T: UserEvent>: Send + Sync {
  fn apply_config(&mut self, config: &Config) -> Result<()>;
  fn build(self: Box<Self>, args: RuntimeInitArgs<()>) -> Result<Box<dyn ErasedRuntime<T>>>;
  #[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn build_any_thread(
    self: Box<Self>,
    args: RuntimeInitArgs<()>,
  ) -> Result<Box<dyn ErasedRuntime<T>>>;
}

struct RuntimeInitAttrs<T: UserEvent, A: RuntimeSpecificInitAttrs<T>> {
  attrs: A,
  _marker: PhantomData<fn() -> T>,
}

impl<T: UserEvent, A: RuntimeSpecificInitAttrs<T>> ErasedRuntimeInitAttrs<T>
  for RuntimeInitAttrs<T, A>
{
  fn apply_config(&mut self, config: &Config) -> Result<()> {
    self.attrs.apply_config(config)
  }

  fn build(self: Box<Self>, args: RuntimeInitArgs<()>) -> Result<Box<dyn ErasedRuntime<T>>> {
    let (args, ()) = args.with_attrs(self.attrs);
    <A::Runtime as Runtime<T>>::new(args)
      .map(|runtime| Box::new(runtime) as Box<dyn ErasedRuntime<T>>)
  }

  #[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn build_any_thread(
    self: Box<Self>,
    args: RuntimeInitArgs<()>,
  ) -> Result<Box<dyn ErasedRuntime<T>>> {
    let (args, ()) = args.with_attrs(self.attrs);
    <A::Runtime as Runtime<T>>::new_any_thread(args)
      .map(|runtime| Box::new(runtime) as Box<dyn ErasedRuntime<T>>)
  }
}

/// The [`RuntimeSpecificInitAttrs`] of [`DynRuntime`].
///
/// Wraps the attributes of the concrete runtime to use, which is how the runtime is selected.
/// The default value selects no runtime, in which case initializing the [`DynRuntime`] fails
/// with [`Error::RuntimeNotConfigured`].
pub struct DynRuntimeInitAttrs<T: UserEvent> {
  inner: Option<Box<dyn ErasedRuntimeInitAttrs<T>>>,
}

impl<T: UserEvent> DynRuntimeInitAttrs<T> {
  /// Selects the runtime initialized with the given attributes.
  pub fn new<A: RuntimeSpecificInitAttrs<T>>(attrs: A) -> Self {
    Self {
      inner: Some(Box::new(RuntimeInitAttrs {
        attrs,
        _marker: PhantomData,
      })),
    }
  }

  /// Whether a runtime was selected.
  pub fn is_configured(&self) -> bool {
    self.inner.is_some()
  }
}

impl<T: UserEvent> Default for DynRuntimeInitAttrs<T> {
  fn default() -> Self {
    Self { inner: None }
  }
}

impl<T: UserEvent> fmt::Debug for DynRuntimeInitAttrs<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynRuntimeInitAttrs")
      .field("configured", &self.is_configured())
      .finish()
  }
}

impl<T: UserEvent> RuntimeSpecificInitAttrs<T> for DynRuntimeInitAttrs<T> {
  type Runtime = DynRuntime<T>;

  fn apply_config(&mut self, config: &Config) -> Result<()> {
    match &mut self.inner {
      Some(inner) => inner.apply_config(config),
      None => Ok(()),
    }
  }
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

trait ErasedRuntime<T: UserEvent>: fmt::Debug + Any {
  fn create_proxy(&self) -> DynEventLoopProxy<T>;
  fn handle(&self) -> DynRuntimeHandle<T>;
  fn create_window(
    &self,
    pending: PendingWindow<T, DynRuntime<T>>,
    after_window_creation: Option<AfterWindowCreation>,
  ) -> Result<DetachedWindow<T, DynRuntime<T>>>;
  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, DynRuntime<T>>,
  ) -> Result<DetachedWebview<T, DynRuntime<T>>>;
  fn primary_monitor(&self) -> Option<Monitor>;
  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor>;
  fn available_monitors(&self) -> Vec<Monitor>;
  fn cursor_position(&self) -> Result<PhysicalPosition<f64>>;
  fn set_theme(&self, theme: Option<Theme>);
  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy);
  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&mut self, visible: bool);
  #[cfg(target_os = "macos")]
  fn show(&self);
  #[cfg(target_os = "macos")]
  fn hide(&self);
  fn set_device_event_filter(&mut self, filter: DeviceEventFilter);
  #[cfg(desktop)]
  fn run_iteration(&mut self, callback: RunCallback<T>);
  fn run_return(self: Box<Self>, callback: RunCallback<T>) -> i32;
  fn run(self: Box<Self>, callback: RunCallback<T>);
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: UserEvent, R: Runtime<T>> ErasedRuntime<T> for R {
  fn create_proxy(&self) -> DynEventLoopProxy<T> {
    DynEventLoopProxy::new(Runtime::create_proxy(self))
  }

  fn handle(&self) -> DynRuntimeHandle<T> {
    DynRuntimeHandle::new(Runtime::handle(self))
  }

  fn create_window(
    &self,
    pending: PendingWindow<T, DynRuntime<T>>,
    after_window_creation: Option<AfterWindowCreation>,
  ) -> Result<DetachedWindow<T, DynRuntime<T>>> {
    let pending = pending_window_from_dyn::<T, R>(pending)?;
    Runtime::create_window(self, pending, after_window_creation).map(detached_window_into_dyn)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, DynRuntime<T>>,
  ) -> Result<DetachedWebview<T, DynRuntime<T>>> {
    let pending = pending_webview_from_dyn::<T, R>(pending)?;
    Runtime::create_webview(self, window_id, pending).map(detached_webview_into_dyn)
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    Runtime::primary_monitor(self)
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    Runtime::monitor_from_point(self, x, y)
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    Runtime::available_monitors(self)
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Runtime::cursor_position(self)
  }

  fn set_theme(&self, theme: Option<Theme>) {
    Runtime::set_theme(self, theme)
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy) {
    Runtime::set_activation_policy(self, activation_policy)
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&mut self, visible: bool) {
    Runtime::set_dock_visibility(self, visible)
  }

  #[cfg(target_os = "macos")]
  fn show(&self) {
    Runtime::show(self)
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    Runtime::hide(self)
  }

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    Runtime::set_device_event_filter(self, filter)
  }

  #[cfg(desktop)]
  fn run_iteration(&mut self, callback: RunCallback<T>) {
    Runtime::run_iteration(self, callback)
  }

  fn run_return(self: Box<Self>, callback: RunCallback<T>) -> i32 {
    Runtime::run_return(*self, callback)
  }

  fn run(self: Box<Self>, callback: RunCallback<T>) {
    Runtime::run(*self, callback)
  }

  fn as_any(&self) -> &dyn Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

/// A type-erased [`Runtime`].
///
/// The concrete runtime is selected through its [`RuntimeSpecificInitAttrs`],
/// see [`DynRuntimeInitAttrs::new`].
#[derive(Debug)]
pub struct DynRuntime<T: UserEvent> {
  inner: Box<dyn ErasedRuntime<T>>,
}

impl<T: UserEvent> DynRuntime<T> {
  /// Wraps an already initialized runtime.
  ///
  /// Prefer [`Runtime::new`] with [`DynRuntimeInitAttrs`] to let the runtime be initialized with the
  /// application's [`RuntimeInitArgs`]; this is only useful for runtimes created by other means.
  pub fn from_runtime<R: Runtime<T>>(runtime: R) -> Self {
    Self {
      inner: Box::new(runtime),
    }
  }

  /// Whether the wrapped runtime is of type `R`.
  pub fn is<R: Runtime<T>>(&self) -> bool {
    self.inner.as_any().is::<R>()
  }

  /// Returns a reference to the wrapped runtime if it is of type `R`.
  pub fn downcast_ref<R: Runtime<T>>(&self) -> Option<&R> {
    self.inner.as_any().downcast_ref()
  }

  /// Returns a mutable reference to the wrapped runtime if it is of type `R`.
  pub fn downcast_mut<R: Runtime<T>>(&mut self) -> Option<&mut R> {
    self.inner.as_any_mut().downcast_mut()
  }
}

impl<T: UserEvent> Runtime<T> for DynRuntime<T> {
  type WindowDispatcher = DynWindowDispatcher<T>;
  type WebviewDispatcher = DynWebviewDispatcher<T>;
  type Handle = DynRuntimeHandle<T>;
  type EventLoopProxy = DynEventLoopProxy<T>;
  type PlatformSpecificWebviewAttribute = DynWebviewAttribute;
  type Webview = DynWebview;
  type RuntimeInitAttrs = DynRuntimeInitAttrs<T>;
  type WindowOpener = DynWindowOpener;

  fn new(args: RuntimeInitArgs<Self::RuntimeInitAttrs>) -> Result<Self> {
    let (args, attrs) = args.with_attrs(());
    let attrs = attrs.inner.ok_or(Error::RuntimeNotConfigured)?;
    Ok(Self {
      inner: attrs.build(args)?,
    })
  }

  #[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn new_any_thread(args: RuntimeInitArgs<Self::RuntimeInitAttrs>) -> Result<Self> {
    let (args, attrs) = args.with_attrs(());
    let attrs = attrs.inner.ok_or(Error::RuntimeNotConfigured)?;
    Ok(Self {
      inner: attrs.build_any_thread(args)?,
    })
  }

  fn create_proxy(&self) -> Self::EventLoopProxy {
    self.inner.create_proxy()
  }

  fn handle(&self) -> Self::Handle {
    self.inner.handle()
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    self.inner.create_window(
      pending,
      after_window_creation.map(|f| Box::new(f) as AfterWindowCreation),
    )
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    self.inner.create_webview(window_id, pending)
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    self.inner.primary_monitor()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    self.inner.monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    self.inner.available_monitors()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    self.inner.cursor_position()
  }

  fn set_theme(&self, theme: Option<Theme>) {
    self.inner.set_theme(theme)
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy) {
    self.inner.set_activation_policy(activation_policy)
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&mut self, visible: bool) {
    self.inner.set_dock_visibility(visible)
  }

  #[cfg(target_os = "macos")]
  fn show(&self) {
    self.inner.show()
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    self.inner.hide()
  }

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    self.inner.set_device_event_filter(filter)
  }

  #[cfg(desktop)]
  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, callback: F) {
    self.inner.run_iteration(Box::new(callback))
  }

  fn run_return<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) -> i32 {
    self.inner.run_return(Box::new(callback))
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) {
    self.inner.run(Box::new(callback))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn window_builder_records_theme_and_icon() {
    let builder = DynWindowBuilder::new();
    assert!(!builder.has_icon());
    assert_eq!(builder.get_theme(), None);

    let builder = builder.theme(Some(Theme::Dark)).theme(Some(Theme::Light));
    assert_eq!(builder.get_theme(), Some(Theme::Light));

    let icon = Icon {
      rgba: vec![0; 2 * 2 * 4].into(),
      width: 2,
      height: 2,
    };
    let builder = builder.icon(icon).expect("valid icon");
    assert!(builder.has_icon());
    assert_eq!(builder.ops.len(), 3);
  }

  #[test]
  fn window_builder_rejects_invalid_icon() {
    let icon = Icon {
      rgba: vec![0; 3].into(),
      width: 2,
      height: 2,
    };
    assert!(matches!(
      DynWindowBuilder::new().icon(icon),
      Err(Error::InvalidIcon(_))
    ));
  }

  #[test]
  fn window_builder_theme_falls_back_to_config() {
    let config = WindowConfig {
      theme: Some(Theme::Dark),
      ..Default::default()
    };
    let builder = DynWindowBuilder::with_config(&config);
    assert_eq!(builder.get_theme(), Some(Theme::Dark));
    assert_eq!(builder.theme(None).get_theme(), None);
  }

  #[test]
  fn erased_values_downcast() {
    let webview = DynWebview::new(42u32);
    assert!(webview.is::<u32>());
    assert_eq!(webview.downcast_ref::<u32>(), Some(&42));
    assert!(webview.downcast::<String>().is_err());

    let opener = DynWindowOpener::new("opener".to_string());
    assert!(opener.downcast::<u32>().is_err());
    let opener = DynWindowOpener::new("opener".to_string());
    assert_eq!(opener.downcast::<String>().unwrap(), "opener");

    let attribute = DynWebviewAttribute::new(7u8);
    assert_eq!(attribute.downcast::<u8>().unwrap(), 7);
  }

  #[test]
  fn init_attrs_default_is_unconfigured() {
    let attrs = DynRuntimeInitAttrs::<()>::default();
    assert!(!attrs.is_configured());
    let args = RuntimeInitArgs {
      runtime_init_attrs: attrs,
      ..Default::default()
    };
    assert!(matches!(
      <DynRuntime<()> as Runtime<()>>::new(args),
      Err(Error::RuntimeNotConfigured)
    ));
  }
}
