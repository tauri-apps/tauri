use cef::{rc::*, *};
use std::{
  cell::RefCell,
  collections::HashMap,
  io::Cursor,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};
use tauri_runtime::{
  webview::UriSchemeProtocol,
  window::{PendingWindow, WindowId},
  RunEvent, UserEvent,
};

use crate::{AppWindow, CefRuntime, Message};

mod request_handler;

#[derive(Clone)]
pub struct Context<T: UserEvent> {
  pub windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  pub callback: Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
  pub next_window_id: Arc<AtomicU32>,
  pub next_webview_id: Arc<AtomicU32>,
  pub next_window_event_id: Arc<AtomicU32>,
  pub next_webview_event_id: Arc<AtomicU32>,
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
  }
}

wrap_browser_process_handler! {
  struct AppBrowserProcessHandler<T: UserEvent> {
    context: Context<T>,
  }

  impl BrowserProcessHandler {
    // The real lifespan of cef starts from `on_context_initialized`, so all the cef objects should be manipulated after that.
    fn on_context_initialized(&self) {
      println!("cef context initialized");
      (self.context.callback.borrow_mut())(RunEvent::Ready);
    }
  }
}

wrap_client! {
  struct BrowserClient;

  impl Client {
    fn request_handler(&self) -> Option<RequestHandler> {
      Some(request_handler::WebRequestHandler::new())
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
      quit_message_loop();
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

  let mut client = BrowserClient::new();
  let url = CefString::from(webview.url.as_str());

  let mut request_context = request_context_create_context(
    Some(&RequestContextSettings::default()),
    Option::<&mut RequestContextHandler>::None,
  );
  if let Some(request_context) = &request_context {
    for (scheme, handler) in webview.uri_scheme_protocols {
      request_context.register_scheme_handler_factory(
        Some(&scheme.as_str().into()),
        None,
        Some(&mut request_handler::UriSchemeHandlerFactory::new(
          request_handler::UriSchemeContext {
            handler: Arc::new(handler) as Arc<UriSchemeProtocol>,
            resource: Arc::new(RefCell::new(Cursor::new(
              "Hello from Tauri!".as_bytes().to_vec(),
            ))),
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
