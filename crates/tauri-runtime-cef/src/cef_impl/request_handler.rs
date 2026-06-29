// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  borrow::Cow,
  io::{Cursor, Read},
  sync::{Arc, Mutex},
};

use cef::{rc::*, *};
use dioxus_debug_cell::RefCell;
use html5ever::{LocalName, interface::QualName, namespace_url, ns};
use http::{
  HeaderMap, HeaderName, HeaderValue,
  header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, ORIGIN},
};
use kuchiki::NodeRef;
use tauri_runtime::{
  UserEvent,
  webview::{NavigationHandler, UriSchemeProtocolHandler},
  window::WindowId,
};
use tauri_utils::{
  config::{Csp, CspDirectiveSources},
  html::{parse as parse_html, serialize_node},
};
use url::Url;

use crate::{
  cef_impl::client::{DragDropEventTarget, DragDropState, WebDragDropResourceRequestHandler},
  runtime::RuntimeContext,
  webview::{CefInitScript, INITIAL_LOAD_URL},
};

type HttpResponse = Arc<RefCell<Option<http::Response<Cursor<Vec<u8>>>>>>;
pub(crate) type SchemeRegistry = Arc<
  Mutex<
    std::collections::HashMap<
      (i32, String),
      (
        String,
        Arc<Box<UriSchemeProtocolHandler>>,
        Arc<Vec<CefInitScript>>,
      ),
    >,
  >,
>;

fn csp_inject_initialization_scripts_hashes(
  existing_csp: String,
  initialization_scripts: &[CefInitScript],
) -> String {
  if initialization_scripts.is_empty() {
    return existing_csp;
  }

  let script_hashes: Vec<String> = initialization_scripts
    .iter()
    .map(|s| s.hash.clone())
    .collect();

  if script_hashes.is_empty() {
    return existing_csp;
  }

  let mut csp_map: std::collections::HashMap<String, CspDirectiveSources> =
    Csp::Policy(existing_csp.to_string()).into();

  let script_src = csp_map
    .entry("script-src".to_string())
    .or_insert_with(|| CspDirectiveSources::List(vec!["'self'".to_string()]));

  script_src.extend(script_hashes);

  Csp::DirectiveMap(csp_map).to_string()
}

fn inject_scripts_into_html_body(
  body: &[u8],
  initialization_scripts: &[CefInitScript],
) -> Option<Vec<u8>> {
  let Ok(body_str) = std::str::from_utf8(body) else {
    return None;
  };

  let document = parse_html(body_str.to_string());

  let head = if let Ok(ref head_node) = document.select_first("head") {
    head_node.as_node().clone()
  } else {
    let head_node = NodeRef::new_element(
      QualName::new(None, ns!(html), LocalName::from("head")),
      None,
    );
    document.prepend(head_node.clone());
    head_node
  };

  for init_script in initialization_scripts.iter().rev() {
    let script_el = NodeRef::new_element(QualName::new(None, ns!(html), "script".into()), None);
    script_el.append(NodeRef::new_text(init_script.script.as_str()));
    head.prepend(script_el);
  }

  Some(serialize_node(&document))
}

wrap_request_handler! {
  pub struct WebRequestHandler<T: UserEvent> {
    navigation_handler: Option<Arc<NavigationHandler>>,
    context: RuntimeContext<T>,
    window_id: WindowId,
    webview_id: u32,
    drag_drop_event_target: DragDropEventTarget,
    drag_drop_handler_enabled: bool,
    drag_drop_state: Arc<Mutex<DragDropState>>,
    web_content_process_terminate_handler: Option<Arc<dyn Fn() + Send>>,
  }

  impl RequestHandler {
    fn on_render_process_terminated(
      &self,
      _browser: Option<&mut Browser>,
      _status: TerminationStatus,
      _error_code: ::std::os::raw::c_int,
      _error_string: Option<&CefString>,
    ) {
      if let Some(handler) = &self.web_content_process_terminate_handler {
        handler();
      }
    }

    fn on_before_browse(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      request: Option<&mut Request>,
      _user_gesture: ::std::os::raw::c_int,
      _is_redirect: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
      let _ = (&self.context, self.window_id, self.webview_id);

      let Some(frame) = frame else {
        return 0;
      };
      // we only fire main frame navigation events to match the behavior of the wry runtime
      if frame.is_main() == 0 {
        return 0;
      }
      let Some(request) = request else {
        return 0;
      };

      let url_str = CefString::from(&request.url()).to_string();

      if url_str == INITIAL_LOAD_URL {
        return 0;
      }

      let Ok(url) = url::Url::parse(&url_str) else {
        return 0;
      };

      let Some(handler) = &self.navigation_handler else {
        return 0;
      };

      let should_navigate = handler(&url);
      if should_navigate { 0 } else { 1 }
    }

    fn resource_request_handler(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _request: Option<&mut Request>,
      _is_navigation: ::std::os::raw::c_int,
      _is_download: ::std::os::raw::c_int,
      _request_initiator: Option<&CefString>,
      _disable_default_handling: Option<&mut ::std::os::raw::c_int>,
    ) -> Option<ResourceRequestHandler> {
      // The handler only intercepts the drag-drop bridge requests; when the
      // bridge is disabled it would pass every request straight through, so skip
      // building (and cloning the context + state into) a handler that CEF calls
      // for every subresource/fetch/XHR the page makes.
      if !self.drag_drop_handler_enabled {
        return None;
      }

      Some(WebDragDropResourceRequestHandler::new(
        self.context.clone(),
        self.window_id,
        self.webview_id,
        self.drag_drop_event_target,
        self.drag_drop_handler_enabled,
        self.drag_drop_state.clone(),
      ))
    }
  }
}

wrap_resource_handler! {
  pub struct WebResourceHandler {
    webview_label: String,
    handler: Arc<Box<UriSchemeProtocolHandler>>,
    initialization_scripts: Arc<Vec<CefInitScript>>,
    // Serialized origin of the main frame that initiated this request, captured
    // browser-side in the scheme handler factory. The renderer can issue an IPC
    // request before its execution context is fully wired to the loader; in
    // that window Chromium tags the request with `Origin: null` even though the
    // document already has a proper origin. We use this to repair the `Origin`
    // header in that case. `None` when the initiator is not the (non-opaque)
    // main frame, so sandboxed/subframe `Origin: null` requests are left as-is.
    initiator_origin: Option<String>,
    // we clone response to send it to the handler thread
    response: HttpResponse,
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

      if let Some(url) = url {
        let callback = ThreadSafe(callback.clone());
        let response_store = ThreadSafe(self.response.clone());
        let initialization_scripts = self.initialization_scripts.clone();
        let responder = Box::new(move |response: http::Response<Cow<'static, [u8]>>| {
          let is_html = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .map(|ct| ct.to_lowercase().starts_with("text/html"))
            .unwrap_or(false);

          let (parts, body) = response.into_parts();
          let body_bytes = body.into_owned();
          let body_bytes = if is_html {
            inject_scripts_into_html_body(&body_bytes, &initialization_scripts)
              .unwrap_or(body_bytes)
          } else {
            body_bytes
          };

          let mut response = http::Response::from_parts(parts, Cursor::new(body_bytes));

          if let Some(csp) = response.headers_mut().get_mut(CONTENT_SECURITY_POLICY) {
            let csp_string = csp.to_str().unwrap_or_default().to_string();
            let new_csp =
              csp_inject_initialization_scripts_hashes(csp_string, &initialization_scripts);
            if let Ok(new_csp) = HeaderValue::from_str(&new_csp) {
              *csp = new_csp;
            }
          }

          response_store.into_owned().borrow_mut().replace(response);

          let callback = callback.into_owned();
          callback.cont();
        });

        let label = self.webview_label.clone();
        let handler = self.handler.clone();

        let data = read_request_body(request);
        let mut headers = get_request_headers(request);

        // The renderer can issue an IPC request before its execution context is
        // fully wired to the loader; in that window Chromium sends the request
        // with `Origin: null` even though the document already has a real
        // origin. Repair it from the initiating main frame's URL, which the
        // browser process tracks reliably. Only done when the renderer sent no
        // origin or a literal `null`, so a correct renderer-sent origin always
        // wins.
        if let Some(initiator_origin) = &self.initiator_origin {
          let origin_missing_or_null = headers
            .get(ORIGIN)
            .map(|value| value.as_bytes() == b"null")
            .unwrap_or(true);
          if origin_missing_or_null && let Ok(value) = HeaderValue::from_str(initiator_origin) {
            headers.insert(ORIGIN, value);
          }
        }

        let method_str = CefString::from(&request.method()).to_string();
        let method = http::Method::from_bytes(method_str.as_bytes()).unwrap_or(http::Method::GET);

        std::thread::spawn(move || {
          let mut http_request = http::Request::builder()
            .method(method)
            .uri(url.as_str())
            .body(data)
            .unwrap();
          *http_request.headers_mut() = headers;
          // handler is Arc<Box<UriSchemeProtocol>>, so we need to dereference to call it
          (**handler)(&label, http_request, responder);
        });
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
      _callback: Option<&mut ResourceReadCallback>,
    ) -> ::std::os::raw::c_int {
      let Ok(bytes_to_read) = usize::try_from(bytes_to_read) else {
        return 0;
      };
      let data_out = unsafe { std::slice::from_raw_parts_mut(data_out, bytes_to_read) };
      let count = self
        .response
        .borrow_mut()
        .as_mut()
        .and_then(|response| response.body_mut().read(data_out).ok())
        .unwrap_or(0);
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
      let (Some(response), Some(response_data)) = (response, &*self.response.borrow()) else {
        return;
      };

      response.set_status(response_data.status().as_u16() as i32);
      let mut content_type = None;

      // Set response headers and remember the MIME type for CEF.
      for (name, value) in response_data.headers() {
        let Ok(value) = value.to_str() else {
          continue;
        };

        response.set_header_by_name(Some(&name.as_str().into()), Some(&value.into()), 0);

        if name == CONTENT_TYPE {
          content_type.replace(value.to_string());
        }
      }

      response.set_header_by_name(Some(&"Cache-Control".into()), Some(&"no-store".into()), 1);

      let mime_type = content_type
        .as_ref()
        .and_then(|t| t.split(';').next())
        .map(str::trim)
        .unwrap_or("text/plain");
      response.set_mime_type(Some(&mime_type.into()));

      if let Some(length) = response_length {
        *length = -1;
      }

      if let Some(redirect_url) = redirect_url {
        let _ = std::mem::take(redirect_url);
      }
    }
  }
}

wrap_scheme_handler_factory! {
  pub struct UriSchemeHandlerFactory {
    registry: SchemeRegistry,
    scheme: String,
  }

  impl SchemeHandlerFactory {
    fn create(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      _scheme_name: Option<&CefString>,
      _request: Option<&mut Request>,
    ) -> Option<ResourceHandler> {
      let browser = browser?;
      let id = browser.identifier();

      // get handler from our regsitry based on browser ID and scheme
      let (webview_label, handler, initialization_scripts) = self
        .registry
        .lock()
        .unwrap()
        .get(&(id, self.scheme.clone()))
        .cloned()?;

      // Capture the initiating main frame's origin so `process_request` can
      // repair a racy `Origin: null` header. Restricted to the main frame: it
      // is never an opaque-origin (sandboxed) document in a Tauri webview, so
      // upgrading its origin is safe; subframes are intentionally left alone.
      let initiator_origin = frame
        .filter(|frame| frame.is_main() == 1)
        .map(|frame| CefString::from(&frame.url()).to_string())
        .and_then(|url| Url::parse(&url).ok())
        .map(|url| url.origin().ascii_serialization())
        .filter(|origin| origin != "null");

      Some(WebResourceHandler::new(
        webview_label,
        handler,
        initialization_scripts,
        initiator_origin,
        Arc::new(RefCell::new(None)),
      ))
    }
  }
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
    let mut elements = vec![None; post_data.element_count()];
    post_data.elements(Some(&mut elements));
    for element in elements.into_iter().flatten() {
      match element.get_type().as_ref() {
        sys::cef_postdataelement_type_t::PDE_TYPE_BYTES => {
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
        sys::cef_postdataelement_type_t::PDE_TYPE_FILE => {
          // Read file from disk
          let file_path = CefString::from(&element.file()).to_string();
          if let Ok(mut file) = std::fs::File::open(&file_path) {
            use std::io::Read;
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_ok() {
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

fn get_request_headers(request: &mut Request) -> HeaderMap {
  let mut headers = HeaderMap::new();

  let mut map = CefStringMultimap::new();

  request.header_map(Some(&mut map));

  // Iterate through all entries
  for (name, value) in map {
    for v in value {
      headers.append(
        HeaderName::from_bytes(name.as_bytes()).unwrap(),
        HeaderValue::from_str(&v).unwrap(),
      );
    }
  }

  headers
}
