use cef::{rc::*, *};
use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};
use tauri_runtime::{
  window::{PendingWindow, WindowId},
  RunEvent, UserEvent,
};

use crate::{AppWindow, CefRuntime, Message};

macro_rules! cef_object {
  ($struct: ident<T: UserEvent>, $inner: ty, $cef_obj: ident, $wrap_trait: ty) => {
    pub struct $struct<T: UserEvent> {
      object: *mut RcImpl<cef_dll_sys::$cef_obj, Self>,
      inner: $inner,
    }

    impl<T: UserEvent> Rc for $struct<T> {
      fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
          let base = &*self.object;
          std::mem::transmute(&base.cef_object)
        }
      }
    }

    impl<T: UserEvent> $wrap_trait for $struct<T> {
      fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::$cef_obj, Self>) {
        self.object = object;
      }
    }

    impl<T: UserEvent> Clone for $struct<T> {
      fn clone(&self) -> Self {
        let object = unsafe {
          let rc_impl = &mut *self.object;
          rc_impl.interface.add_ref();
          self.object
        };

        Self {
          object,
          inner: self.inner.clone(),
        }
      }
    }
  };
  ($struct: ident, $inner: ty, $cef_obj: ident, $wrap_trait: ty) => {
    pub struct $struct {
      object: *mut RcImpl<cef_dll_sys::$cef_obj, Self>,
      inner: $inner,
    }

    impl Rc for $struct {
      fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
          let base = &*self.object;
          std::mem::transmute(&base.cef_object)
        }
      }
    }

    impl $wrap_trait for $struct {
      fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::$cef_obj, Self>) {
        self.object = object;
      }
    }

    impl Clone for $struct {
      fn clone(&self) -> Self {
        let object = unsafe {
          let rc_impl = &mut *self.object;
          rc_impl.interface.add_ref();
          self.object
        };

        Self {
          object,
          inner: self.inner.clone(),
        }
      }
    }
  };
}

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

cef_object!(TauriApp<T: UserEvent>, Context<T>, _cef_app_t, WrapApp);

impl<T: UserEvent> TauriApp<T> {
  pub fn new(context: Context<T>) -> App {
    App::new(Self {
      object: std::ptr::null_mut(),
      inner: context,
    })
  }
}

impl<T: UserEvent> ImplApp for TauriApp<T> {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_app_t {
    self.object.cast()
  }

  fn get_browser_process_handler(&self) -> Option<BrowserProcessHandler> {
    Some(AppBrowserProcessHandler::new(self.inner.clone()))
  }
}

cef_object!(AppBrowserProcessHandler<T: UserEvent>, Context<T>, cef_browser_process_handler_t, WrapBrowserProcessHandler);

impl<T: UserEvent> AppBrowserProcessHandler<T> {
  pub fn new(context: Context<T>) -> BrowserProcessHandler {
    BrowserProcessHandler::new(Self {
      object: std::ptr::null_mut(),
      inner: context,
    })
  }
}

impl<T: UserEvent> ImplBrowserProcessHandler for AppBrowserProcessHandler<T> {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_browser_process_handler_t {
    self.object.cast()
  }

  // The real lifespan of cef starts from `on_context_initialized`, so all the cef objects should be manipulated after that.
  fn on_context_initialized(&self) {
    println!("cef context initialized");
    (self.inner.callback.borrow_mut())(RunEvent::Ready);
  }

  fn get_default_request_context_handler(&self) -> Option<RequestContextHandler> {
    Some(WebRequestContextHandler::new())
  }

  fn get_default_client(&self) -> Option<Client> {
    None
    //Some(BrowserClient::new())
  }
}

cef_object!(
  WebRequestContextHandler,
  (),
  _cef_request_context_handler_t,
  WrapRequestContextHandler
);

impl WebRequestContextHandler {
  fn new() -> RequestContextHandler {
    RequestContextHandler::new(Self {
      object: std::ptr::null_mut(),
      inner: (),
    })
  }
}

impl ImplRequestContextHandler for WebRequestContextHandler {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_request_context_handler_t {
    self.object.cast()
  }

  fn get_resource_request_handler(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
    is_navigation: ::std::os::raw::c_int,
    is_download: ::std::os::raw::c_int,
    request_initiator: Option<&CefStringUtf16>,
    disable_default_handling: Option<&mut ::std::os::raw::c_int>,
  ) -> Option<ResourceRequestHandler> {
    Some(WebResourceRequestHandler::new())
  }
}

cef_object!(
  WebResourceRequestHandler,
  (),
  _cef_resource_request_handler_t,
  WrapResourceRequestHandler
);

impl WebResourceRequestHandler {
  fn new() -> ResourceRequestHandler {
    ResourceRequestHandler::new(Self {
      object: std::ptr::null_mut(),
      inner: (),
    })
  }
}

impl ImplResourceRequestHandler for WebResourceRequestHandler {
  fn get_resource_handler(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
  ) -> Option<ResourceHandler> {
    println!(
      "get_resource_handler {:?}",
      request
        .as_ref()
        .map(|r| r.get_url().map(|s| CefStringUtf8::from(&s).to_string()))
    );
    None
  }

  fn on_before_resource_load(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
    callback: Option<&mut impl ImplCallback>,
  ) -> ReturnValue {
    println!(
      "on_before_resource_load {:?}",
      request
        .as_ref()
        .map(|r| r.get_url().map(|s| CefStringUtf8::from(&s).to_string()))
    );
    Default::default()
  }

  fn get_raw(&self) -> *mut cef_dll_sys::_cef_resource_request_handler_t {
    self.object.cast()
  }
}

cef_object!(BrowserClient, (), _cef_client_t, WrapClient);

impl BrowserClient {
  pub fn new() -> Client {
    Client::new(Self {
      object: std::ptr::null_mut(),
      inner: (),
    })
  }
}

impl ImplClient for BrowserClient {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_client_t {
    self.object.cast()
  }

  fn get_request_handler(&self) -> Option<RequestHandler> {
    Some(WebRequestHandler::new())
  }
}

cef_object!(
  WebRequestHandler,
  (),
  _cef_request_handler_t,
  WrapRequestHandler
);

impl WebRequestHandler {
  fn new() -> RequestHandler {
    RequestHandler::new(Self {
      object: std::ptr::null_mut(),
      inner: (),
    })
  }
}

impl ImplRequestHandler for WebRequestHandler {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_request_handler_t {
    self.object.cast()
  }

  fn get_resource_request_handler(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
    is_navigation: ::std::os::raw::c_int,
    is_download: ::std::os::raw::c_int,
    request_initiator: Option<&CefStringUtf16>,
    disable_default_handling: Option<&mut ::std::os::raw::c_int>,
  ) -> Option<ResourceRequestHandler> {
    Some(WebResourceRequestHandler::new())
  }
}

struct AppWindowDelegate {
  base: *mut RcImpl<cef_dll_sys::_cef_window_delegate_t, Self>,
  browser_view: BrowserView,
}

impl AppWindowDelegate {
  pub fn new(browser_view: BrowserView) -> WindowDelegate {
    WindowDelegate::new(Self {
      base: std::ptr::null_mut(),
      browser_view,
    })
  }
}

impl WrapWindowDelegate for AppWindowDelegate {
  fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_window_delegate_t, Self>) {
    self.base = object;
  }
}

impl Clone for AppWindowDelegate {
  fn clone(&self) -> Self {
    unsafe {
      let rc_impl = &mut *self.base;
      rc_impl.interface.add_ref();
    }

    Self {
      base: self.base,
      browser_view: self.browser_view.clone(),
    }
  }
}

impl Rc for AppWindowDelegate {
  fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
    unsafe {
      let base = &*self.base;
      std::mem::transmute(&base.cef_object)
    }
  }
}

impl ImplViewDelegate for AppWindowDelegate {
  fn on_child_view_changed(
    &self,
    _view: Option<&mut impl ImplView>,
    _added: ::std::os::raw::c_int,
    _child: Option<&mut impl ImplView>,
  ) {
    // view.as_panel().map(|x| x.as_window().map(|w| w.close()));
  }

  fn get_raw(&self) -> *mut cef_dll_sys::_cef_view_delegate_t {
    self.base.cast()
  }
}

impl ImplPanelDelegate for AppWindowDelegate {}

impl ImplWindowDelegate for AppWindowDelegate {
  fn on_window_created(&self, window: Option<&mut impl ImplWindow>) {
    if let Some(window) = window {
      let mut view = self.browser_view.clone();
      window.add_child_view(Some(&mut view));
      window.show();
    }
  }

  fn on_window_destroyed(&self, _window: Option<&mut impl ImplWindow>) {
    quit_message_loop();
  }

  fn with_standard_window_buttons(
    &self,
    _window: Option<&mut impl ImplWindow>,
  ) -> ::std::os::raw::c_int {
    1
  }

  fn can_resize(&self, _window: Option<&mut impl ImplWindow>) -> ::std::os::raw::c_int {
    1
  }

  fn can_maximize(&self, _window: Option<&mut impl ImplWindow>) -> ::std::os::raw::c_int {
    1
  }

  fn can_minimize(&self, _window: Option<&mut impl ImplWindow>) -> ::std::os::raw::c_int {
    1
  }

  fn can_close(&self, _window: Option<&mut impl ImplWindow>) -> ::std::os::raw::c_int {
    1
  }
}

pub struct SendMessageTask<T: UserEvent> {
  context: Context<T>,
  message: Arc<RefCell<Message<T>>>,
  object: *mut RcImpl<cef_dll_sys::_cef_task_t, Self>,
}

impl<T: UserEvent> SendMessageTask<T> {
  pub fn new(context: Context<T>, message: Message<T>) -> Task {
    Task::new(Self {
      context,
      message: Arc::new(RefCell::new(message)),
      object: std::ptr::null_mut(),
    })
  }
}

impl<T: UserEvent> Rc for SendMessageTask<T> {
  fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
    unsafe {
      let base = &*self.object;
      std::mem::transmute(&base.cef_object)
    }
  }
}

impl<T: UserEvent> Clone for SendMessageTask<T> {
  fn clone(&self) -> Self {
    let object = unsafe {
      let rc_impl = &mut *self.object;
      rc_impl.interface.add_ref();
      self.object
    };
    Self {
      context: self.context.clone(),
      message: self.message.clone(),
      object,
    }
  }
}

impl<T: UserEvent> WrapTask for SendMessageTask<T> {
  fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_task_t, Self>) {
    self.object = object;
  }
}

impl<T: UserEvent> ImplTask for SendMessageTask<T> {
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

  fn get_raw(&self) -> *mut cef_dll_sys::_cef_task_t {
    unsafe { &mut (&mut *self.object).cef_object }
  }
}

fn create_window<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  pending: PendingWindow<T, CefRuntime<T>>,
) {
  let label = pending.label.clone();

  let mut client = BrowserClient::new();
  let url = pending
    .webview
    .as_ref()
    .map(|w| w.url.as_str())
    .map(|url| CefString::from(&CefStringUtf8::from(url)));

  let browser_view = browser_view_create(
    Some(&mut client),
    url.as_ref(),
    Some(&Default::default()),
    Option::<&mut DictionaryValue>::None,
    Option::<&mut RequestContext>::None,
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
