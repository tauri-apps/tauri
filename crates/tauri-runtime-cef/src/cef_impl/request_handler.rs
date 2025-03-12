use std::sync::Arc;

use cef::{rc::*, *};
use tauri_runtime::webview::UriSchemeProtocol;
use url::Url;

use crate::{cef_impl::utils::utf16_string_to_utf8, cef_object};

cef_object!(
  WebResourceRequestHandler,
  (),
  ResourceRequestHandler,
  _cef_resource_request_handler_t,
  WrapResourceRequestHandler
);

impl ImplResourceRequestHandler for WebResourceRequestHandler {
  fn get_resource_handler(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
  ) -> Option<ResourceHandler> {
    None
  }

  fn on_resource_response(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
    response: Option<&mut impl ImplResponse>,
  ) -> ::std::os::raw::c_int {
    Default::default()
  }

  fn on_before_resource_load(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    request: Option<&mut impl ImplRequest>,
    callback: Option<&mut impl ImplCallback>,
  ) -> ReturnValue {
    cef_dll_sys::cef_return_value_t::RV_CONTINUE.into()
  }

  fn get_raw(&self) -> *mut cef_dll_sys::_cef_resource_request_handler_t {
    self.object.cast()
  }
}

cef_object!(
  WebRequestHandler,
  (),
  RequestHandler,
  _cef_request_handler_t,
  WrapRequestHandler
);

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
    Some(WebResourceRequestHandler::new(self.context.clone()))
  }
}

cef_object!(
  WebResourceHandler,
  UriSchemeContext,
  ResourceHandler,
  _cef_resource_handler_t,
  WrapResourceHandler
);

impl ImplResourceHandler for WebResourceHandler {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_resource_handler_t {
    self.object.cast()
  }

  fn process_request(
    &self,
    request: Option<&mut impl ImplRequest>,
    callback: Option<&mut impl ImplCallback>,
  ) -> ::std::os::raw::c_int {
    let Some(request) = request else { return 0 };
    let Some(callback) = callback else { return 0 };

    let url = request
      .get_url()
      .map(utf16_string_to_utf8)
      .map(|url| url.to_string())
      .and_then(|url| Url::parse(&url).ok());

    println!("{:?}", url.as_ref().map(ToString::to_string));

    //callback.cont();
    //return 1;

    if let Some(url) = url {
      // keep the callback around
      unsafe {
        callback.add_ref();
      }

      let callback = ThreadSafe(callback.get_raw());
      std::thread::spawn(move || {
        std::thread::sleep_ms(5);
        let cb = callback.into_owned();
        unsafe {
          (*cb).cont.inspect(|f| {
            f(cb);
          });
          // release after use
          (*cb).release();
        }
      });
      1
    } else {
      0
    }
  }

  fn read_response(
    &self,
    data_out: *mut u8,
    bytes_to_read: ::std::os::raw::c_int,
    bytes_read: Option<&mut ::std::os::raw::c_int>,
    callback: Option<&mut impl ImplCallback>,
  ) -> ::std::os::raw::c_int {
    callback.inspect(|cb| cb.cont());
    bytes_read.map(|read| {
      *read = 5;
    });
    1
  }

  fn get_response_headers(
    &self,
    response: Option<&mut impl ImplResponse>,
    response_length: Option<&mut i64>,
    redirect_url: Option<&mut CefStringUtf16>,
  ) {
    let Some(response) = response else { return };
    response.set_status(200);
    response.set_header_by_name(Some(&"content-type".into()), Some(&"text/plain".into()), 1);
    response_length.map(|length| {
      *length = -1;
    });
  }
}

cef_object!(
  UriSchemeHandlerFactory,
  UriSchemeContext,
  SchemeHandlerFactory,
  cef_scheme_handler_factory_t,
  WrapSchemeHandlerFactory
);

impl ImplSchemeHandlerFactory for UriSchemeHandlerFactory {
  fn get_raw(&self) -> *mut cef_dll_sys::_cef_scheme_handler_factory_t {
    self.object.cast()
  }

  fn create(
    &self,
    browser: Option<&mut impl ImplBrowser>,
    frame: Option<&mut impl ImplFrame>,
    scheme_name: Option<&CefStringUtf16>,
    request: Option<&mut impl ImplRequest>,
  ) -> Option<ResourceHandler> {
    Some(WebResourceHandler::new(self.context.clone()))
  }
}

#[derive(Clone)]
pub struct UriSchemeContext {
  pub handler: Arc<UriSchemeProtocol>,
}

struct ThreadSafe<T>(T);

impl<T> ThreadSafe<T> {
  fn into_owned(self) -> T {
    self.0
  }
}

unsafe impl<T> Send for ThreadSafe<T> {}
unsafe impl<T> Sync for ThreadSafe<T> {}
