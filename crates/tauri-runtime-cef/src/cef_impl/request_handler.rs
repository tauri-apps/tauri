use std::{
  borrow::Cow,
  cell::RefCell,
  io::{Cursor, Read},
  sync::Arc,
};

use cef::{rc::*, *};
use tauri_runtime::webview::UriSchemeProtocol;
use url::Url;

wrap_resource_request_handler! {
  pub struct WebResourceRequestHandler;

  impl ResourceRequestHandler {
    fn resource_handler(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      request: Option<&mut Request>,
    ) -> Option<ResourceHandler> {
      None
    }

    fn on_resource_response(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      request: Option<&mut Request>,
      response: Option<&mut Response>,
    ) -> ::std::os::raw::c_int {
      Default::default()
    }

    fn on_before_resource_load(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      request: Option<&mut Request>,
      callback: Option<&mut Callback>,
    ) -> ReturnValue {
      sys::cef_return_value_t::RV_CONTINUE.into()
    }
  }
}

wrap_request_handler! {
  pub struct WebRequestHandler;

  impl RequestHandler {
    fn resource_request_handler(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      request: Option<&mut Request>,
      is_navigation: ::std::os::raw::c_int,
      is_download: ::std::os::raw::c_int,
      request_initiator: Option<&CefString>,
      disable_default_handling: Option<&mut ::std::os::raw::c_int>,
    ) -> Option<ResourceRequestHandler> {
      Some(WebResourceRequestHandler::new())
    }
  }
}

wrap_resource_handler! {
  pub struct WebResourceHandler {
    context: UriSchemeContext,
  }

  impl ResourceHandler {
    fn process_request(
      &self,
      request: Option<&mut Request>,
      callback: Option<&mut Callback>,
    ) -> ::std::os::raw::c_int {
      let Some(request) = request else { return 0 };
      let Some(callback) = callback else { return 0 };

      let url = CefString::from(&request.url()).to_string();
      let url = Url::parse(&url).ok();

      let data = read_request_body(request);

      if let Some(url) = url {
        let callback = ThreadSafe(callback.clone());
        // TODO: thread safety
        let resource = ThreadSafe(self.context.resource.clone());
        let responder = Box::new(move |response: http::Response<Cow<'static, [u8]>>| {
          // TODO: handle multiple concurrent requests
          resource.into_owned().borrow_mut().replace(Cursor::new(response.into_body().into_owned()));
          let callback = callback.into_owned();
          callback.cont();
        });

        // TODO: headers
        let http_request = http::Request::builder().uri(url.as_str()).body(data).unwrap();
        (self.context.handler)(&self.context.label, http_request, responder);
        1
      } else {
        0
      }
    }

    fn read(
      &self,
      data_out: *mut u8,
      bytes_to_read: ::std::os::raw::c_int,
      bytes_read: Option<&mut ::std::os::raw::c_int>,
      callback: Option<&mut ResourceReadCallback>,
    ) -> ::std::os::raw::c_int {
      let Ok(bytes_to_read) = usize::try_from(bytes_to_read) else {
        return 0;
      };
      let data_out = unsafe { std::slice::from_raw_parts_mut(data_out, bytes_to_read) };
      let count = self.context.resource.borrow_mut().as_mut().and_then(|response| response.read(data_out).ok()).unwrap_or(0);
      if let Some(bytes_read) = bytes_read {
        let Ok(count) = count.try_into() else {
          return 0;
        };
        *bytes_read = count;
        if count > 0 {
          return 1;
        }
      }
      0
    }

    fn response_headers(
      &self,
      response: Option<&mut Response>,
      response_length: Option<&mut i64>,
      redirect_url: Option<&mut CefString>,
    ) {
      let Some(response) = response else { return };
      response.set_status(200);
      response.set_mime_type(Some(&"text/html".into()));
      response.set_header_by_name(Some(&"content-type".into()), Some(&"text/html".into()), 1);
      response_length.map(|length| {
        *length = -1;
      });

      if let Some(redirect_url) = redirect_url {
        let _ = std::mem::take(redirect_url);
      }
    }
  }
}

wrap_scheme_handler_factory! {
  pub struct UriSchemeHandlerFactory {
    context: UriSchemeContext,
  }

  impl SchemeHandlerFactory {
    fn create(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      scheme_name: Option<&CefString>,
      request: Option<&mut Request>,
    ) -> Option<ResourceHandler> {
      Some(WebResourceHandler::new(self.context.clone()))
    }
  }
}

#[derive(Clone)]
pub struct UriSchemeContext {
  pub label: String,
  pub handler: Arc<UriSchemeProtocol>,
  pub resource: Arc<RefCell<Option<Cursor<Vec<u8>>>>>,
}

struct ThreadSafe<T>(T);

impl<T> ThreadSafe<T> {
  fn into_owned(self) -> T {
    self.0
  }
}

unsafe impl<T> Send for ThreadSafe<T> {}
unsafe impl<T> Sync for ThreadSafe<T> {}

fn read_request_body(request: &mut Request) -> Vec<u8> {
  let mut body = Vec::new();

  if let Some(post_data) = request.post_data() {
    let mut elements = Vec::new();
    post_data.elements(Some(&mut elements));
    for element in elements.into_iter().filter_map(|v| v) {
      match element.get_type().as_ref() {
        cef_dll_sys::cef_postdataelement_type_t::PDE_TYPE_BYTES => {
          let size = element.bytes_count();
          if size > 0 {
            let mut buf = vec![0u8; size];
            // Copy bytes into our buffer
            let copied = element.bytes(size, buf.as_mut_ptr());
            // Safety: CEF promises it wrote `copied` bytes into buf
            unsafe {
              buf.set_len(copied);
            }
            body.extend(buf);
          }
        }
        cef_dll_sys::cef_postdataelement_type_t::PDE_TYPE_FILE => {
          // Read file from disk
          let file_path = CefString::from(&element.file()).to_string();
          if let Ok(mut file) = std::fs::File::open(&file_path) {
            use std::io::Read;
            let mut buf = Vec::new();
            if let Ok(_) = file.read_to_end(&mut buf) {
              body.extend(buf);
            }
          }
        }
        _ => {}
      }
    }
  }

  body
}
