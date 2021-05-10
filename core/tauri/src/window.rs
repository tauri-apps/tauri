// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "menu")]
use crate::runtime::MenuId;
use crate::{
  api::config::WindowUrl,
  command::{CommandArg, CommandItem},
  event::{Event, EventHandler},
  manager::WindowManager,
  runtime::{
    monitor::Monitor as RuntimeMonitor,
    tag::{TagRef, ToJsString},
    webview::{InvokePayload, WebviewAttributes, WindowBuilder},
    window::{
      dpi::{PhysicalPosition, PhysicalSize, Position, Size},
      DetachedWindow, PendingWindow, WindowEvent,
    },
    Dispatch, Icon, Params, Runtime,
  },
  sealed::ManagerBase,
  sealed::RuntimeOrDispatch,
  Invoke, InvokeError, InvokeMessage, InvokeResolver, Manager, PageLoadPayload,
};

use serde::Serialize;

use std::{
  borrow::Borrow,
  hash::{Hash, Hasher},
};

/// The window menu event.
#[cfg(feature = "menu")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
#[derive(Debug, Clone)]
pub struct MenuEvent<I: MenuId> {
  pub(crate) menu_item_id: I,
}

#[cfg(feature = "menu")]
impl<I: MenuId> MenuEvent<I> {
  /// The menu item id.
  pub fn menu_item_id(&self) -> &I {
    &self.menu_item_id
  }
}

/// Monitor descriptor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
  pub(crate) name: Option<String>,
  pub(crate) size: PhysicalSize<u32>,
  pub(crate) position: PhysicalPosition<i32>,
  pub(crate) scale_factor: f64,
}

impl From<RuntimeMonitor> for Monitor {
  fn from(monitor: RuntimeMonitor) -> Self {
    Self {
      name: monitor.name,
      size: monitor.size,
      position: monitor.position,
      scale_factor: monitor.scale_factor,
    }
  }
}

impl Monitor {
  /// Returns a human-readable name of the monitor.
  /// Returns None if the monitor doesn't exist anymore.
  pub fn name(&self) -> Option<&String> {
    self.name.as_ref()
  }

  /// Returns the monitor's resolution.
  pub fn size(&self) -> &PhysicalSize<u32> {
    &self.size
  }

  /// Returns the top-left corner position of the monitor relative to the larger full screen area.
  pub fn position(&self) -> &PhysicalPosition<i32> {
    &self.position
  }

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub fn scale_factor(&self) -> f64 {
    self.scale_factor
  }
}

/// A webview window managed by Tauri.
///
/// This type also implements [`Manager`] which allows you to manage other windows attached to
/// the same application.
///
/// TODO: expand these docs since this is a pretty important type
pub struct Window<P: Params> {
  /// The webview window created by the runtime.
  window: DetachedWindow<P>,

  /// The manager to associate this webview window with.
  manager: WindowManager<P>,
}

impl<M: Params> Clone for Window<M> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      manager: self.manager.clone(),
    }
  }
}

impl<P: Params> Hash for Window<P> {
  /// Only use the [`Window`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.window.label.hash(state)
  }
}

impl<P: Params> Eq for Window<P> {}
impl<P: Params> PartialEq for Window<P> {
  /// Only use the [`Window`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.window.label.eq(&other.window.label)
  }
}

impl<P: Params> Manager<P> for Window<P> {}
impl<P: Params> ManagerBase<P> for Window<P> {
  fn manager(&self) -> &WindowManager<P> {
    &self.manager
  }
}

impl<'de, P: Params> CommandArg<'de, P> for Window<P> {
  /// Grabs the [`Window`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, P>) -> Result<Self, InvokeError> {
    Ok(command.message.window())
  }
}

impl<P: Params> Window<P> {
  /// Create a new window that is attached to the manager.
  pub(crate) fn new(manager: WindowManager<P>, window: DetachedWindow<P>) -> Self {
    Self { window, manager }
  }

  /// Creates a new webview window.
  pub fn create_window<F>(
    &mut self,
    label: P::Label,
    url: WindowUrl,
    setup: F,
  ) -> crate::Result<Window<P>>
  where
    F: FnOnce(
      <<P::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder,
      WebviewAttributes,
    ) -> (
      <<P::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder,
      WebviewAttributes,
    ),
  {
    let (window_builder, webview_attributes) = setup(
      <<P::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder::new(),
      WebviewAttributes::new(url),
    );
    self.create_new_window(
      RuntimeOrDispatch::Dispatch(self.dispatcher()),
      PendingWindow::new(window_builder, webview_attributes, label),
    )
  }

  /// The current window's dispatcher.
  pub(crate) fn dispatcher(&self) -> <P::Runtime as Runtime>::Dispatcher {
    self.window.dispatcher.clone()
  }

  #[allow(dead_code)]
  pub(crate) fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .run_on_main_thread(f)
      .map_err(Into::into)
  }

  /// How to handle this window receiving an [`InvokeMessage`].
  pub(crate) fn on_message(self, command: String, payload: InvokePayload) -> crate::Result<()> {
    let manager = self.manager.clone();
    match command.as_str() {
      "__initialized" => {
        let payload: PageLoadPayload = serde_json::from_value(payload.inner)?;
        manager.run_on_page_load(self, payload);
      }
      _ => {
        let message = InvokeMessage::new(
          self.clone(),
          manager.state(),
          command.to_string(),
          payload.inner,
        );
        let resolver = InvokeResolver::new(self, payload.callback, payload.error);
        let invoke = Invoke { message, resolver };
        if let Some(module) = &payload.tauri_module {
          let module = module.to_string();
          crate::endpoints::handle(module, invoke, manager.config(), manager.package_info());
        } else if command.starts_with("plugin:") {
          manager.extend_api(invoke);
        } else {
          manager.run_invoke_handler(invoke);
        }
      }
    }

    Ok(())
  }

  /// The label of this window.
  pub fn label(&self) -> &P::Label {
    &self.window.label
  }

  /// Emits an event to the current window.
  pub fn emit<E: ?Sized, S>(&self, event: &E, payload: S) -> crate::Result<()>
  where
    P::Event: Borrow<E>,
    E: TagRef<P::Event>,
    S: Serialize,
  {
    self.eval(&format!(
      "window['{}']({{event: {}, payload: {}}}, '{}')",
      self.manager.event_emit_function_name(),
      event.to_js_string()?,
      serde_json::to_value(payload)?,
      self.manager.generate_salt(),
    ))?;

    Ok(())
  }

  /// Emits an event on all windows except this one.
  pub fn emit_others<E: ?Sized, S>(&self, event: &E, payload: S) -> crate::Result<()>
  where
    P::Event: Borrow<E>,
    E: TagRef<P::Event>,
    S: Serialize + Clone,
  {
    self.manager.emit_filter(event, payload, |w| w != self)
  }

  /// Listen to an event on this window.
  pub fn listen<E: Into<P::Event>, F>(&self, event: E, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.listen(event.into(), Some(label), handler)
  }

  /// Listen to a an event on this window a single time.
  pub fn once<E: Into<P::Event>, F>(&self, event: E, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.once(event.into(), Some(label), handler)
  }

  /// Triggers an event on this window.
  pub fn trigger<E: ?Sized>(&self, event: &E, data: Option<String>)
  where
    P::Event: Borrow<E>,
    E: TagRef<P::Event>,
  {
    let label = self.window.label.clone();
    self.manager.trigger(event, Some(label), data)
  }

  /// Evaluates JavaScript on this window.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.window.dispatcher.eval_script(js).map_err(Into::into)
  }

  /// Registers a window event listener.
  pub fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) {
    self.window.dispatcher.on_window_event(f);
  }

  /// Registers a menu event listener.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub fn on_menu_event<F: Fn(MenuEvent<P::MenuId>) + Send + 'static>(&self, f: F) {
    let menu_ids = self.manager.menu_ids();
    self.window.dispatcher.on_menu_event(move |event| {
      f(MenuEvent {
        menu_item_id: menu_ids.get(&event.menu_item_id).unwrap().clone(),
      })
    });
  }

  // Getters

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub fn scale_factor(&self) -> crate::Result<f64> {
    self.window.dispatcher.scale_factor().map_err(Into::into)
  }

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  pub fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.dispatcher.inner_position().map_err(Into::into)
  }

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  pub fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.dispatcher.outer_position().map_err(Into::into)
  }

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  pub fn inner_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.dispatcher.inner_size().map_err(Into::into)
  }

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  pub fn outer_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.dispatcher.outer_size().map_err(Into::into)
  }

  /// Gets the window's current fullscreen state.
  pub fn is_fullscreen(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_fullscreen().map_err(Into::into)
  }

  /// Gets the window's current maximized state.
  pub fn is_maximized(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_maximized().map_err(Into::into)
  }

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  pub fn current_monitor(&self) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .current_monitor()
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  pub fn primary_monitor(&self) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .primary_monitor()
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the list of all the monitors available on the system.
  pub fn available_monitors(&self) -> crate::Result<Vec<Monitor>> {
    self
      .window
      .dispatcher
      .available_monitors()
      .map(|m| m.into_iter().map(Into::into).collect())
      .map_err(Into::into)
  }

  // Setters

  /// Opens the dialog to prints the contents of the webview.
  /// Currently only supported on macOS on `wry`.
  /// `window.print()` works on all platforms.
  pub fn print(&self) -> crate::Result<()> {
    self.window.dispatcher.print().map_err(Into::into)
  }

  /// Determines if this window should be resizable.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_resizable(resizable)
      .map_err(Into::into)
  }

  /// Set this window's title.
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_title(title.to_string())
      .map_err(Into::into)
  }

  /// Maximizes this window.
  pub fn maximize(&self) -> crate::Result<()> {
    self.window.dispatcher.maximize().map_err(Into::into)
  }

  /// Un-maximizes this window.
  pub fn unmaximize(&self) -> crate::Result<()> {
    self.window.dispatcher.unmaximize().map_err(Into::into)
  }

  /// Minimizes this window.
  pub fn minimize(&self) -> crate::Result<()> {
    self.window.dispatcher.minimize().map_err(Into::into)
  }

  /// Un-minimizes this window.
  pub fn unminimize(&self) -> crate::Result<()> {
    self.window.dispatcher.unminimize().map_err(Into::into)
  }

  /// Show this window.
  pub fn show(&self) -> crate::Result<()> {
    self.window.dispatcher.show().map_err(Into::into)
  }

  /// Hide this window.
  pub fn hide(&self) -> crate::Result<()> {
    self.window.dispatcher.hide().map_err(Into::into)
  }

  /// Closes this window.
  pub fn close(&self) -> crate::Result<()> {
    self.window.dispatcher.close().map_err(Into::into)
  }

  /// Determines if this window should be [decorated].
  ///
  /// [decorated]: https://en.wikipedia.org/wiki/Window_(computing)#Window_decoration
  pub fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_decorations(decorations)
      .map_err(Into::into)
  }

  /// Determines if this window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_always_on_top(always_on_top)
      .map_err(Into::into)
  }

  /// Resizes this window.
  pub fn set_size<S: Into<Size>>(&self, size: S) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_size(size.into())
      .map_err(Into::into)
  }

  /// Sets this window's minimum size.
  pub fn set_min_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_min_size(size.map(|s| s.into()))
      .map_err(Into::into)
  }

  /// Sets this window's maximum size.
  pub fn set_max_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_max_size(size.map(|s| s.into()))
      .map_err(Into::into)
  }

  /// Sets this window's position.
  pub fn set_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_position(position.into())
      .map_err(Into::into)
  }

  /// Determines if this window should be fullscreen.
  pub fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_fullscreen(fullscreen)
      .map_err(Into::into)
  }

  /// Sets this window' icon.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self.window.dispatcher.set_icon(icon).map_err(Into::into)
  }

  /// Starts dragging the window.
  pub fn start_dragging(&self) -> crate::Result<()> {
    self.window.dispatcher.start_dragging().map_err(Into::into)
  }

  pub(crate) fn verify_salt(&self, salt: String) -> bool {
    self.manager.verify_salt(salt)
  }
}
