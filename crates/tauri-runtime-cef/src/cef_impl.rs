use cef::{rc::*, *};
use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex, OnceLock,
  },
};
use tauri_runtime::{
  webview::{InitializationScript, UriSchemeProtocol},
  window::{PendingWindow, WindowId},
  RunEvent, UserEvent,
};

use crate::{AppWindow, CefRuntime, Message};

mod request_handler;

// Message name for process messages containing initialization scripts
const INIT_SCRIPTS_MESSAGE_NAME: &str = "tauri_init_scripts";

// Serializable version of InitializationScript for process messages
#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableInitScript {
  script: String,
  for_main_frame_only: bool,
}

impl From<&InitializationScript> for SerializableInitScript {
  fn from(script: &InitializationScript) -> Self {
    Self {
      script: script.script.clone(),
      for_main_frame_only: script.for_main_frame_only,
    }
  }
}

impl From<SerializableInitScript> for InitializationScript {
  fn from(script: SerializableInitScript) -> Self {
    Self {
      script: script.script,
      for_main_frame_only: script.for_main_frame_only,
    }
  }
}

// Static registry in render process to store initialization scripts received via process messages
// This is populated when the browser process sends scripts via process messages
static RENDER_INIT_SCRIPTS_REGISTRY: OnceLock<Mutex<HashMap<i32, Vec<InitializationScript>>>> =
  OnceLock::new();

fn get_render_init_scripts_registry() -> &'static Mutex<HashMap<i32, Vec<InitializationScript>>> {
  RENDER_INIT_SCRIPTS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
pub struct Context<T: UserEvent> {
  pub windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  pub callback: Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
  pub next_window_id: Arc<AtomicU32>,
  pub next_webview_id: Arc<AtomicU32>,
  pub next_window_event_id: Arc<AtomicU32>,
  pub next_webview_event_id: Arc<AtomicU32>,
  /// Initialization scripts stored per browser ID
  /// Scripts are stored here and then registered to shared registry when browser is available
  pub initialization_scripts: Arc<Mutex<HashMap<i32, Vec<InitializationScript>>>>,
}

impl<T: UserEvent> Context<T> {
  pub fn next_window_id(&self) -> WindowId {
    self.next_window_id.fetch_add(1, Ordering::Relaxed).into()
  }

  pub fn next_webview_id(&self) -> u32 {
    self.next_webview_id.fetch_add(1, Ordering::Relaxed)
  }

  pub fn next_window_event_id(&self) -> u32 {
    self.next_window_event_id.fetch_add(1, Ordering::Relaxed)
  }

  pub fn next_webview_event_id(&self) -> u32 {
    self.next_webview_event_id.fetch_add(1, Ordering::Relaxed)
  }
}

wrap_app! {
  pub struct TauriApp<T: UserEvent> {
    context: Context<T>,
  }

  impl App {
    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
      Some(AppBrowserProcessHandler::new(self.context.clone()))
    }

    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(AppRenderProcessHandler::new(self.context.clone()))
    }

    /// Called before the process starts to register custom schemes.
    /// This is where we mark schemes as fetch-enabled, secure, and CORS-enabled.
    fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
      if let Some(registrar) = registrar {
        // Standard CEF scheme options for custom protocols:
        // - FETCH_ENABLED: Allows Fetch API requests
        // - SECURE: Treats as secure like https (no mixed content warnings)
        // - CORS_ENABLED: Allows CORS requests
        // - STANDARD: Standard URL scheme behavior
        let scheme_options = (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_FETCH_ENABLED as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as i32);

        for scheme in ["ipc", "tauri"] {
          let scheme_name = CefString::from(scheme);
          registrar.add_custom_scheme(Some(&scheme_name), scheme_options);
        }
      }
    }
  }
}

wrap_render_process_handler! {
  struct AppRenderProcessHandler<T: UserEvent> {
    context: Context<T>,
  }

  impl RenderProcessHandler {
    /// Called when a process message is received from the browser process.
    /// We use this to receive initialization scripts.
    fn on_process_message_received(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      source_process: ProcessId,
      message: Option<&mut ProcessMessage>,
    ) -> ::std::os::raw::c_int {
      // Only handle messages from browser process
      if source_process.as_ref() != &cef_dll_sys::cef_process_id_t::PID_BROWSER {
        return 0;
      }

      let Some(message) = message else { return 0 };

      // Check if this is our initialization scripts message
      let msg_name = message.name();
      let msg_name_str = {
        let name = CefString::from(&msg_name).to_string();
        name
      };

      if msg_name_str != INIT_SCRIPTS_MESSAGE_NAME {
        return 0;
      }

      // Extract browser_id and scripts from message arguments
      let Some(args) = message.argument_list() else { return 0 };

      // First argument: browser_id (int)
      let browser_id = args.int(0);

      // Second argument: scripts as JSON string
      let scripts_json = {
        let str_free = args.string(1);
        CefString::from(&str_free).to_string()
      };

      // Deserialize scripts
      if let Ok(serializable_scripts) = serde_json::from_str::<Vec<SerializableInitScript>>(&scripts_json) {
        let scripts: Vec<InitializationScript> = serializable_scripts
          .into_iter()
          .map(InitializationScript::from)
          .collect();
        get_render_init_scripts_registry()
          .lock()
          .unwrap()
          .insert(browser_id, scripts);
      }

      1 // Message handled
    }

    /// Called when a new V8 JavaScript context is created.
    /// This happens:
    /// - When the browser first loads a page
    /// - When navigating to a new page (context is recreated)
    /// - When navigating in iframes (each frame gets its own context)
    ///
    /// This ensures initialization scripts run before page scripts on every navigation.
    fn on_context_created(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      context: Option<&mut V8Context>,
    ) {
      let Some(frame) = frame else { return };
      let Some(context) = context else { return };
      let Some(browser) = browser else { return };

      // Get browser ID to look up initialization scripts
      let browser_id = browser.identifier();

      // Check if this is the main frame
      let is_main_frame = frame.is_main() == 1;

      // Get initialization scripts for this browser from render process registry
      // Scripts are received via process messages from the browser process
      let scripts = {
        get_render_init_scripts_registry()
          .lock()
          .unwrap()
          .get(&browser_id)
          .cloned()
          .unwrap_or_default()
      };
      if scripts.is_empty() {
        return;
      }

      // Enter the V8 context so we can execute scripts synchronously
      context.enter();

      // Get the frame URL for proper script execution context
      let frame_url = {
        let url_free = frame.url();
        let url = CefString::from(&url_free).to_string();
        url
      };

      // Inject each initialization script directly into the V8 context
      // Use V8Context::eval for synchronous execution - this runs immediately,
      // before any page scripts can execute
      for script in &scripts {
        // If script is for main frame only, skip if this is not the main frame
        if script.for_main_frame_only && !is_main_frame {
          continue;
        }

        // Execute the script synchronously in the V8 context
        // This runs immediately before any page scripts can execute
        let script_str = CefString::from(script.script.as_str());
        let url_str = CefString::from(frame_url.as_str());
        let mut retval = None;
        let mut exception = None;

        // eval returns 1 on success, 0 on failure
        let result = context.eval(
          Some(&script_str),
          Some(&url_str),
          0,
          Some(&mut retval),
          Some(&mut exception),
        );

        if result == 0 {
          // Script execution failed - log exception if available
          if let Some(exc) = exception {
            let msg = exc.message();
            eprintln!("Failed to execute initialization script: {}", CefString::from(&msg).to_string());
          }
        }
      }

      // Exit the V8 context
      context.exit();
    }
  }
}

wrap_browser_process_handler! {
  struct AppBrowserProcessHandler<T: UserEvent> {
    context: Context<T>,
  }

  impl BrowserProcessHandler {
    // The real lifespan of cef starts from `on_context_initialized`, so all the cef objects should be manipulated after that.
    fn on_context_initialized(&self) {
      (self.context.callback.borrow_mut())(RunEvent::Ready);
    }
  }
}

wrap_client! {
  struct BrowserClient<T: UserEvent> {
    context: Context<T>,
    pending_scripts: Vec<InitializationScript>,
  }

  impl Client {
    fn request_handler(&self) -> Option<RequestHandler> {
      Some(request_handler::WebRequestHandler::new())
    }

    fn load_handler(&self) -> Option<LoadHandler> {
      Some(LoadHandlerImpl::new(
        self.context.clone(),
        self.pending_scripts.clone(),
        Arc::new(Mutex::new(false)),
      ))
    }

    fn life_span_handler(&self) -> Option<LifeSpanHandler> {
      Some(LifeSpanHandlerImpl::new(
        self.context.clone(),
        self.pending_scripts.clone(),
      ))
    }
  }
}

wrap_life_span_handler! {
  struct LifeSpanHandlerImpl<T: UserEvent> {
    context: Context<T>,
    pending_scripts: Vec<InitializationScript>,
  }

  impl LifeSpanHandler {
    /// Called after the browser is created.
    /// Send initialization scripts to render process immediately so they're available
    /// when OnContextCreated is called.
    fn on_after_created(&self, browser: Option<&mut Browser>) {
      if let Some(browser) = browser {
        let browser_id = browser.identifier();

        // Store scripts in Context (browser process)
        if !self.pending_scripts.is_empty() {
          // Store in Context for browser process access
          self.context
            .initialization_scripts
            .lock()
            .unwrap()
            .insert(browser_id, self.pending_scripts.clone());

          // Send scripts to render process via process message
          // Do this as early as possible so scripts are available before OnContextCreated
          if let Some(main_frame) = browser.main_frame() {
            // Convert to serializable format and serialize to JSON
            let serializable_scripts: Vec<SerializableInitScript> = self
              .pending_scripts
              .iter()
              .map(SerializableInitScript::from)
              .collect();

            if let Ok(scripts_json) = serde_json::to_string(&serializable_scripts) {
              // Create process message
              let mut msg = process_message_create(Some(&CefString::from(INIT_SCRIPTS_MESSAGE_NAME)))
                .expect("Failed to create process message");

              // Set arguments: browser_id and scripts JSON
              if let Some(args) = msg.argument_list() {
                args.set_int(0, browser_id);
                args.set_string(1, Some(&CefString::from(scripts_json.as_str())));

                // Send to render process
                main_frame.send_process_message(
                  ProcessId::from(cef_dll_sys::cef_process_id_t::PID_RENDERER),
                  Some(&mut msg),
                );
              }
            }
          }
        }
      }
    }
  }
}

wrap_load_handler! {
  struct LoadHandlerImpl<T: UserEvent> {
    context: Context<T>,
    pending_scripts: Vec<InitializationScript>,
    scripts_registered: Arc<Mutex<bool>>,
  }

  impl LoadHandler {
    fn on_loading_state_change(
      &self,
      browser: Option<&mut Browser>,
      _is_loading: ::std::os::raw::c_int,
      _can_go_back: ::std::os::raw::c_int,
      _can_go_forward: ::std::os::raw::c_int,
    ) {
      // Scripts are now sent in LifeSpanHandler::on_after_created (earlier),
      // so we don't need to send them here anymore
    }
  }
}

wrap_window_delegate! {
  struct AppWindowDelegate {
    browser_view: BrowserView,
  }

  impl ViewDelegate {
    fn on_child_view_changed(
      &self,
      _view: Option<&mut View>,
      _added: ::std::os::raw::c_int,
      _child: Option<&mut View>,
    ) {
      // view.as_panel().map(|x| x.as_window().map(|w| w.close()));
    }
  }

  impl PanelDelegate {}

  impl WindowDelegate {
    fn on_window_created(&self, window: Option<&mut Window>) {
      if let Some(window) = window {
        let mut view = View::from(&self.browser_view);
        window.add_child_view(Some(&mut view));
        window.show();
      }
    }

    fn on_window_destroyed(&self, _window: Option<&mut Window>) {
      // TODO: send destroyed event
    }

    fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }
  }
}

wrap_task! {
  pub struct SendMessageTask<T: UserEvent>  {
    context: Context<T>,
    message: Arc<RefCell<Message<T>>>,
  }

  impl Task {
    fn execute(&self) {
      match self.message.replace(Message::Noop) {
        Message::CreateWindow {
          window_id,
          webview_id,
          pending,
          after_window_creation: _todo,
        } => create_window(&self.context, window_id, webview_id, pending),
        Message::Task(t) => t(),
        Message::UserEvent(evt) => {
          (self.context.callback.borrow_mut())(RunEvent::UserEvent(evt));
        }
        Message::Noop => {}
      }
    }
  }
}

fn create_window<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  pending: PendingWindow<T, CefRuntime<T>>,
) {
  let label = pending.label.clone();

  let webview = pending.webview.unwrap();

  // Get initialization scripts from webview attributes
  let initialization_scripts = webview.webview_attributes.initialization_scripts.clone();

  let mut client = BrowserClient::new(context.clone(), initialization_scripts.clone());
  let url = CefString::from(webview.url.as_str());

  let global_context =
    request_context_get_global_context().expect("Failed to get global request context");
  let global_cache_path: CefStringUtf16 = (&global_context.cache_path()).into();

  let mut request_context = request_context_create_context(
    Some(&RequestContextSettings {
      cache_path: global_cache_path,
      ..Default::default()
    }),
    Option::<&mut RequestContextHandler>::None,
  );
  if let Some(request_context) = &request_context {
    // Ensure schemes are registered with proper flags (fetch-enabled, secure, etc.)
    // This is done in App::on_register_custom_schemes, but we also track
    // which schemes we've seen

    for (scheme, handler) in webview.uri_scheme_protocols {
      let label = label.clone();
      request_context.register_scheme_handler_factory(
        Some(&scheme.as_str().into()),
        None,
        Some(&mut request_handler::UriSchemeHandlerFactory::new(
          request_handler::UriSchemeContext {
            label,
            handler: Arc::new(handler) as Arc<UriSchemeProtocol>,
            response: Arc::new(RefCell::new(None)),
          },
        )),
      );
    }
  } else {
    eprintln!("failed to create context");
  }

  let browser_view = browser_view_create(
    Some(&mut client),
    Some(&url),
    Some(&Default::default()),
    Option::<&mut DictionaryValue>::None,
    request_context.as_mut(),
    Option::<&mut BrowserViewDelegate>::None,
  )
  .expect("Failed to create browser view");

  let mut delegate = AppWindowDelegate::new(browser_view);

  let window = window_create_top_level(Some(&mut delegate)).expect("Failed to create window");

  context
    .windows
    .borrow_mut()
    .insert(window_id, AppWindow { label, window });
}
