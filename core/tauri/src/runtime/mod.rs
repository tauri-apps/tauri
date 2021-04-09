// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
  api::{assets::Assets, config::Config},
  event::{Event, EventHandler},
  runtime::{
    webview::{Attributes, AttributesPrivate, Icon, WindowConfig},
    window::{DetachedWindow, PendingWindow, Window},
  },
};
use serde::Serialize;
use std::convert::TryFrom;

pub(crate) mod app;
pub mod flavor;
pub(crate) mod manager;
pub(crate) mod tag;
#[cfg(feature = "updater")]
pub(crate) mod updater;
pub(crate) mod webview;
pub(crate) mod window;

pub use self::tag::Tag;
use std::collections::HashMap;

/// Important configurable items required by Tauri.
pub struct Context<A: Assets> {
  /// The config the application was prepared with.
  pub config: Config,

  /// The assets to be served directly by Tauri.
  pub assets: A,

  /// The default window icon Tauri should use when creating windows.
  pub default_window_icon: Option<Vec<u8>>,

  /// Package information.
  pub package_info: tauri_api::PackageInfo,
}

/// The webview runtime interface.
pub trait Runtime: Sized + 'static {
  /// The message dispatcher.
  type Dispatcher: Dispatch<Runtime = Self>;

  /// Creates a new webview runtime.
  fn new() -> crate::Result<Self>;

  /// Creates a new webview window.
  fn create_window<M: Params<Runtime = Self>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>>;

  /// Run the webview runtime.
  fn run(self);
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait Dispatch: Clone + Send + Sized + 'static {
  /// The runtime this [`Dispatch`] runs under.
  type Runtime: Runtime;

  /// Representation of a window icon.
  type Icon: TryFrom<Icon, Error = crate::Error>;

  /// The webview builder type.
  type Attributes: Attributes<Icon = Self::Icon>
    + AttributesPrivate
    + From<WindowConfig>
    + Clone
    + Send;

  /// Creates a new webview window.
  fn create_window<M: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>>;

  /// Updates the window resizable flag.
  fn set_resizable(&self, resizable: bool) -> crate::Result<()>;

  /// Updates the window title.
  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()>;

  /// Maximizes the window.
  fn maximize(&self) -> crate::Result<()>;

  /// Unmaximizes the window.
  fn unmaximize(&self) -> crate::Result<()>;

  /// Minimizes the window.
  fn minimize(&self) -> crate::Result<()>;

  /// Unminimizes the window.
  fn unminimize(&self) -> crate::Result<()>;

  /// Shows the window.
  fn show(&self) -> crate::Result<()>;

  /// Hides the window.
  fn hide(&self) -> crate::Result<()>;

  /// Closes the window.
  fn close(&self) -> crate::Result<()>;

  /// Updates the hasDecorations flag.
  fn set_decorations(&self, decorations: bool) -> crate::Result<()>;

  /// Updates the window alwaysOnTop flag.
  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()>;

  /// Updates the window width.
  fn set_width(&self, width: f64) -> crate::Result<()>;

  /// Updates the window height.
  fn set_height(&self, height: f64) -> crate::Result<()>;

  /// Resizes the window.
  fn resize(&self, width: f64, height: f64) -> crate::Result<()>;

  /// Updates the window min size.
  fn set_min_size(&self, min_width: f64, min_height: f64) -> crate::Result<()>;

  /// Updates the window max size.
  fn set_max_size(&self, max_width: f64, max_height: f64) -> crate::Result<()>;

  /// Updates the X position.
  fn set_x(&self, x: f64) -> crate::Result<()>;

  /// Updates the Y position.
  fn set_y(&self, y: f64) -> crate::Result<()>;

  /// Updates the window position.
  fn set_position(&self, x: f64, y: f64) -> crate::Result<()>;

  /// Updates the window fullscreen state.
  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Updates the window icon.
  fn set_icon(&self, icon: Self::Icon) -> crate::Result<()>;

  /// Executes javascript on the window this [`Dispatch`] represents.
  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()>;
}

/// Prevent implementation details from leaking out of the [`Manager`] and [`Managed`] traits.
pub(crate) mod sealed {
  use super::Params;
  use crate::{
    api::{config::Config, PackageInfo},
    event::{Event, EventHandler},
    hooks::{InvokeMessage, PageLoadPayload},
    runtime::{
      window::{DetachedWindow, PendingWindow, Window},
      RuntimeOrDispatch,
    },
  };
  use serde::Serialize;
  use std::collections::{HashMap, HashSet};
  use uuid::Uuid;

  /// private manager api
  pub trait ParamsPrivate<M: Params>: Clone + Send + Sized + 'static {
    /// Pass messages not handled by modules or plugins to the running application
    fn run_invoke_handler(&self, message: InvokeMessage<M>);

    /// Ran once for every window when the page is loaded.
    fn run_on_page_load(&self, window: Window<M>, payload: PageLoadPayload);

    /// Pass a message to be handled by a plugin that expects the command.
    fn extend_api(&self, command: String, message: InvokeMessage<M>);

    /// Initialize all the plugins attached to the [`Manager`].
    fn initialize_plugins(&self) -> crate::Result<()>;

    /// Prepare a [`PendingWindow`] to be created by the [`Runtime`].
    ///
    /// The passed labels should represent either all the windows in the manager. If the application
    /// has not yet been started, the passed labels should represent all windows that will be
    /// created before starting.
    fn prepare_window(
      &self,
      pending: PendingWindow<M>,
      labels: &[M::Label],
    ) -> crate::Result<PendingWindow<M>>;

    /// Attach a detached window to the manager.
    fn attach_window(&self, window: DetachedWindow<M>) -> Window<M>;

    /// Emit an event to javascript windows that pass the predicate.
    fn emit_filter_internal<S: Serialize + Clone, F: Fn(&Window<Self>) -> bool>(
      &self,
      event: String,
      payload: Option<S>,
      filter: F,
    ) -> crate::Result<()>;

    /// Emit an event to javascript windows that pass the predicate.
    fn emit_filter<S: Serialize + Clone, F: Fn(&Window<M>) -> bool>(
      &self,
      event: M::Event,
      payload: Option<S>,
      predicate: F,
    ) -> crate::Result<()>;

    /// All current window labels existing.
    fn labels(&self) -> HashSet<M::Label>;

    /// The configuration the [`Manager`] was built with.
    fn config(&self) -> &Config;

    /// App package information.
    fn package_info(&self) -> &PackageInfo;

    /// Remove the specified event handler.
    fn unlisten(&self, handler_id: EventHandler);

    /// Trigger an event.
    fn trigger(&self, event: M::Event, window: Option<M::Label>, data: Option<String>);

    /// Set up a listener to an event.
    fn listen<F: Fn(Event) + Send + 'static>(
      &self,
      event: M::Event,
      window: Option<M::Label>,
      handler: F,
    ) -> EventHandler;

    /// Set up a listener to and event that is automatically removed after called once.
    fn once<F: Fn(Event) + Send + 'static>(
      &self,
      event: M::Event,
      window: Option<M::Label>,
      handler: F,
    );

    fn event_listeners_object_name(&self) -> String;
    fn event_queue_object_name(&self) -> String;
    fn event_emit_function_name(&self) -> String;

    /// Generate a random salt and store it in the manager
    fn generate_salt(&self) -> Uuid;

    /// Verify that the passed salt is a valid salt in the manager.
    fn verify_salt(&self, salt: String) -> bool;

    /// Get a single managed window.
    fn get_window(&self, label: &M::Label) -> Option<Window<M>>;

    /// Get all managed windows.
    fn windows(&self) -> HashMap<M::Label, Window<M>>;
  }

  /// Represents a managed handle to the application runner.
  pub trait ManagerPrivate<M: Params> {
    /// The manager behind the [`Managed`] item.
    fn manager(&self) -> &M;

    /// The runtime or runtime dispatcher of the [`Managed`] item.
    fn runtime(&mut self) -> RuntimeOrDispatch<'_, M>;
  }
}

/// Represents either a [`Runtime`] or its dispatcher.
pub enum RuntimeOrDispatch<'m, M: Params> {
  /// Mutable reference to the [`Runtime`].
  Runtime(&'m mut M::Runtime),

  /// Copy of the [`Runtime`]'s dispatcher.
  Dispatch(<M::Runtime as Runtime>::Dispatcher),
}

/// Represents a managed handle to the application runner
pub trait Manager<M: Params>: sealed::ManagerPrivate<M> {
  /// The [`Config`] the manager was created with.
  fn config(&self) -> &Config {
    self.manager().config()
  }

  /// Emits a event to all windows.
  fn emit_all<S: Serialize + Clone>(
    &self,
    event: M::Event,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self.manager().emit_filter(event, payload, |_| true)
  }

  /// Emits an event to a window with the specified label.
  fn emit_to<S: Serialize + Clone>(
    &self,
    label: &M::Label,
    event: M::Event,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self
      .manager()
      .emit_filter(event, payload, |w| w.label() == label)
  }

  /// Creates a new [`Window`] on the [`Runtime`] and attaches it to the [`Manager`].
  fn create_window(&mut self, pending: PendingWindow<M>) -> crate::Result<Window<M>> {
    let labels = self.manager().labels().into_iter().collect::<Vec<_>>();
    let pending = self.manager().prepare_window(pending, &labels)?;
    match self.runtime() {
      RuntimeOrDispatch::Runtime(runtime) => runtime.create_window(pending),
      RuntimeOrDispatch::Dispatch(mut dispatcher) => dispatcher.create_window(pending),
    }
    .map(|window| self.manager().attach_window(window))
  }

  /// Listen to a global event.
  fn listen_global<F>(&self, event: M::Event, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    self.manager().listen(event, None, handler)
  }

  /// Listen to a global event only once.
  fn once_global<F>(&self, event: M::Event, handler: F)
  where
    F: Fn(Event) + Send + 'static,
  {
    self.manager().once(event, None, handler)
  }

  /// Trigger a global event.
  fn trigger_global(&self, event: M::Event, data: Option<String>) {
    self.manager().trigger(event, None, data)
  }

  /// Remove an event listener.
  fn unlisten(&self, handler_id: EventHandler) {
    self.manager().unlisten(handler_id)
  }

  /// Fetch a single window from the manager.
  fn get_window(&self, label: &M::Label) -> Option<Window<M>> {
    self.manager().get_window(label)
  }

  /// Fetch all managed windows.
  fn windows(&self) -> HashMap<M::Label, Window<M>> {
    self.manager().windows()
  }
}

/// Types that the manager needs to have passed in by the application.
pub trait Params: sealed::ParamsPrivate<Self> {
  /// The event type used to create and listen to events.
  type Event: Tag;

  /// The type used to determine the name of windows.
  type Label: Tag;

  /// Assets that Tauri should serve from itself.
  type Assets: Assets;

  /// The underlying webview runtime used by the Tauri application.
  type Runtime: Runtime;
}
