use std::sync::Arc;

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

      println!("{:?}", url.as_ref().map(ToString::to_string));

      if let Some(url) = url {
        // keep the callback around
        let callback = callback.clone();

        let callback = ThreadSafe(callback);
        std::thread::spawn(move || {
          std::thread::sleep(std::time::Duration::from_millis(5));
          let cb = callback.into_owned();
          cb.cont();
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
      callback: Option<&mut Callback>,
    ) -> ::std::os::raw::c_int {
      let Ok(bytes_to_read) = usize::try_from(bytes_to_read) else {
        return 0;
      };
      let data_out = unsafe { std::slice::from_raw_parts_mut(data_out, bytes_to_read) };
      let data = "Hello from Tauri!".as_bytes();
      let count = data_out.len().min(data.len());
      if let Some(bytes_read) = bytes_read {
        let Ok(count) = count.try_into() else {
          return 0;
        };
        *bytes_read = count;
      }
      data_out[..count].copy_from_slice(&data[..count]);
      callback.inspect(|cb| cb.cont());
      1
    }

    fn response_headers(
      &self,
      response: Option<&mut Response>,
      response_length: Option<&mut i64>,
      redirect_url: Option<&mut CefString>,
    ) {
      let Some(response) = response else { return };
      response.set_status(200);
      response.set_header_by_name(Some(&"content-type".into()), Some(&"text/plain".into()), 1);
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
