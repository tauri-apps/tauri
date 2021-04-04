use crate::{
  api::{assets::Assets, config::WindowUrl},
  hooks::{InvokeHandler, InvokeMessage, OnPageLoad, PageLoadPayload, SetupHook},
  plugin::{Plugin, PluginStore},
  runtime::{
    flavor::wry::Wry,
    manager::WindowManager,
    sealed::ManagerPrivate,
    tag::Tag,
    updater,
    webview::{Attributes, WindowConfig},
    window::{PendingWindow, Window},
    Context, Dispatch, Manager, Params, Runtime, RuntimeOrDispatch,
  },
};

/// A handle to the currently running application.
pub struct App<M: Params> {
  runtime: M::Runtime,
  manager: M,
}

#[cfg(feature = "updater")]
impl<M: Params> App<M> {
  /// Runs the updater hook with built-in dialog.
  fn run_updater_dialog(&self, window: Window<M>) {
    let updater_config = self.manager.config().tauri.updater.clone();
    let package_info = self.manager.package_info().clone();
    crate::async_runtime::spawn(async move {
      updater::check_update_with_dialog(updater_config, package_info, window).await
    });
  }

  /// Listen updater events when dialog are disabled.
  fn listen_updater_events(&self, window: Window<M>) {
    let updater_config = self.manager.config().tauri.updater.clone();
    updater::listener(updater_config, self.manager.package_info().clone(), &window);
  }

  fn run_updater(&self, main_window: Option<Window<M>>) {
    if let Some(main_window) = main_window {
      let event_window = main_window.clone();
      let updater_config = self.manager.config().tauri.updater.clone();
      // check if updater is active or not
      if updater_config.dialog && updater_config.active {
        // if updater dialog is enabled spawn a new task
        self.run_updater_dialog(main_window.clone());
        let config = self.manager.config().tauri.updater.clone();
        let package_info = self.manager.package_info().clone();
        // When dialog is enabled, if user want to recheck
        // if an update is available after first start
        // invoke the Event `tauri://update` from JS or rust side.
        main_window.listen(
          updater::EVENT_CHECK_UPDATE
            .parse()
            .unwrap_or_else(|_| panic!("bad label")),
          move |_msg| {
            let window = event_window.clone();
            let package_info = package_info.clone();
            let config = config.clone();
            // re-spawn task inside tokyo to launch the download
            // we don't need to emit anything as everything is handled
            // by the process (user is asked to restart at the end)
            // and it's handled by the updater
            crate::async_runtime::spawn(async move {
              updater::check_update_with_dialog(config, package_info, window).await
            });
          },
        );
      } else if updater_config.active {
        // we only listen for `tauri://update`
        // once we receive the call, we check if an update is available or not
        // if there is a new update we emit `tauri://update-available` with details
        // this is the user responsabilities to display dialog and ask if user want to install
        // to install the update you need to invoke the Event `tauri://update-install`
        self.listen_updater_events(main_window);
      }
    }
  }
}

impl<M: Params> Manager<M> for App<M> {}
impl<M: Params> ManagerPrivate<M> for App<M> {
  fn manager(&self) -> &M {
    &self.manager
  }

  fn runtime(&mut self) -> RuntimeOrDispatch<'_, M> {
    RuntimeOrDispatch::Runtime(&mut self.runtime)
  }
}

#[allow(missing_docs)]
pub struct Runner<M: Params> {
  pending_windows: Vec<PendingWindow<M>>,
  manager: M,
  setup: SetupHook<M>,
}

impl<M: Params> Runner<M> {
  /// Consume and run the [`Application`] until it is finished.
  pub fn run(mut self) -> crate::Result<()> {
    // set up all the windows defined in the config
    for config in self.manager.config().tauri.windows.clone() {
      let url = config.url.clone();
      let label = config
        .label
        .parse()
        .unwrap_or_else(|_| panic!("bad label: {}", config.label));

      self
        .pending_windows
        .push(PendingWindow::new(WindowConfig(config), label, url));
    }

    self.manager.initialize_plugins()?;
    let labels = self
      .pending_windows
      .iter()
      .map(|p| p.label.clone())
      .collect::<Vec<_>>();

    let mut app = App {
      runtime: M::Runtime::new()?,
      manager: self.manager,
    };

    let pending_windows = self.pending_windows;
    #[cfg(feature = "updater")]
    let mut main_window = None;

    for pending in pending_windows {
      let pending = app.manager.prepare_window(pending, &labels)?;
      let detached = app.runtime.create_window(pending)?;
      let window = app.manager.attach_window(detached);
      #[cfg(feature = "updater")]
      if main_window.is_none() {
        main_window = Some(window);
      }
    }

    #[cfg(feature = "updater")]
    app.run_updater(main_window);

    (self.setup)(&mut app)?;
    app.runtime.run();
    Ok(())
  }
}

/// The App builder.
pub struct AppBuilder<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets,
  R: Runtime,
{
  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<WindowManager<E, L, A, R>>>,

  /// The setup hook.
  setup: SetupHook<WindowManager<E, L, A, R>>,

  /// Page load hook.
  on_page_load: Box<OnPageLoad<WindowManager<E, L, A, R>>>,

  /// windows to create when starting up.
  pending_windows: Vec<PendingWindow<WindowManager<E, L, A, R>>>,

  /// All passed plugins
  plugins: PluginStore<WindowManager<E, L, A, R>>,
}

impl<E, L, A, R> AppBuilder<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets,
  R: Runtime,
{
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      setup: Box::new(|_| Ok(())),
      invoke_handler: Box::new(|_| ()),
      on_page_load: Box::new(|_, _| ()),
      pending_windows: Default::default(),
      plugins: PluginStore::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(InvokeMessage<WindowManager<E, L, A, R>>) + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// Defines the setup hook.
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: Fn(&mut App<WindowManager<E, L, A, R>>) -> Result<(), Box<dyn std::error::Error>>
      + Send
      + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<WindowManager<E, L, A, R>>, PageLoadPayload) + Send + Sync + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin<P: Plugin<WindowManager<E, L, A, R>> + 'static>(mut self, plugin: P) -> Self {
    self.plugins.register(plugin);
    self
  }

  /// Creates a new webview.
  pub fn create_window<F>(mut self, label: L, url: WindowUrl, setup: F) -> Self
  where
    F: FnOnce(<R::Dispatcher as Dispatch>::Attributes) -> <R::Dispatcher as Dispatch>::Attributes,
  {
    let attributes = setup(<R::Dispatcher as Dispatch>::Attributes::new());
    self
      .pending_windows
      .push(PendingWindow::new(attributes, label, url));
    self
  }

  /// Builds the [`App`] and the underlying [`Runtime`].
  pub fn build(self, context: Context<A>) -> Runner<WindowManager<E, L, A, R>> {
    Runner {
      pending_windows: self.pending_windows,
      setup: self.setup,
      manager: WindowManager::with_handlers(context, self.invoke_handler, self.on_page_load),
    }
  }
}

/// Make `Wry` the default `ApplicationExt` for `AppBuilder`
impl<A: Assets> Default for AppBuilder<String, String, A, Wry> {
  fn default() -> Self {
    Self::new()
  }
}
