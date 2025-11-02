// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  borrow::Cow,
  cell::RefCell,
  io::{Cursor, Read},
  sync::Arc,
};

use cef::{rc::*, *};
use html5ever::{interface::QualName, namespace_url, ns, LocalName};
use http::{
  header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE},
  HeaderMap, HeaderName, HeaderValue,
};
use kuchiki::NodeRef;
use tauri_runtime::webview::UriSchemeProtocol;
use tauri_utils::{
  config::{Csp, CspDirectiveSources},
  html::{parse as parse_html, serialize_node},
};
use url::Url;

use super::CefInitScript;

fn csp_inject_initialization_scripts_hashes(
  existing_csp: Option<&str>,
  initialization_scripts: &[CefInitScript],
) -> Option<String> {
  if initialization_scripts.is_empty() {
    return existing_csp.map(|s| s.to_string());
  }

  // For custom schemes, include ALL script hashes (we inject all scripts into HTML)
  // This matches the HTML injection behavior in inject_scripts_into_html_body
  let script_hashes: Vec<String> = initialization_scripts
    .iter()
    .map(|s| s.hash.clone())
    .collect();

  if script_hashes.is_empty() {
    return existing_csp.map(|s| s.to_string());
  }

  let new_csp = if let Some(existing_csp) = existing_csp {
    // Parse CSP using tauri-utils
    let mut csp_map: std::collections::HashMap<String, CspDirectiveSources> =
      Csp::Policy(existing_csp.to_string()).into();

    // Update or create script-src directive with script hashes
    let script_src = csp_map
      .entry("script-src".to_string())
      .or_insert_with(|| CspDirectiveSources::List(vec!["'self'".to_string()]));

    // Extend with script hashes
    script_src.extend(script_hashes);

    // Convert back to CSP string
    Csp::DirectiveMap(csp_map).to_string()
  } else {
    // No existing CSP, create new one with just script-src
    let mut csp_map = std::collections::HashMap::new();
    let mut script_src = CspDirectiveSources::List(vec!["'self'".to_string()]);
    script_src.extend(script_hashes);
    csp_map.insert("script-src".to_string(), script_src);
    Csp::DirectiveMap(csp_map).to_string()
  };

  Some(new_csp)
}

/// Helper function to inject initialization scripts into HTML body
fn inject_scripts_into_html_body(
  body: &[u8],
  initialization_scripts: &[CefInitScript],
) -> Option<Vec<u8>> {
  // Check if body is valid UTF-8 HTML
  let Ok(body_str) = std::str::from_utf8(body) else {
    return None;
  };

  // Parse HTML and inject scripts
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

  // Inject initialization scripts (for custom schemes, inject all scripts)
  for init_script in initialization_scripts.iter().rev() {
    let script_el = NodeRef::new_element(QualName::new(None, ns!(html), "script".into()), None);
    script_el.append(NodeRef::new_text(init_script.script.script.as_str()));
    head.prepend(script_el);
  }

  // Serialize the modified HTML
  Some(serialize_node(&document))
}

wrap_resource_request_handler! {
  pub struct WebResourceRequestHandler {
    initialization_scripts: Vec<CefInitScript>,
  }

  impl ResourceRequestHandler {


    fn on_before_resource_load(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _request: Option<&mut Request>,
      _callback: Option<&mut Callback>,
    ) -> ReturnValue {
      sys::cef_return_value_t::RV_CONTINUE.into()
    }
  }
}

wrap_request_handler! {
  pub struct WebRequestHandler {
    initialization_scripts: Vec<CefInitScript>,
  }

  impl RequestHandler {
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
      Some(WebResourceRequestHandler::new(
        self.initialization_scripts.clone(),
      ))
    }
  }
}

wrap_resource_handler! {
  pub struct WebResourceHandler {
    webview_label: String,
    handler: Arc<UriSchemeProtocol>,
    initialization_scripts: Vec<CefInitScript>,
    // we clone response to send it to the handler thread
    response: Arc<RefCell<Option<http::Response<Cursor<Vec<u8>>>>>>,
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
        // TODO: thread safety
        let response_store = ThreadSafe(self.response.clone());
        let initialization_scripts = self.initialization_scripts.clone();
        let responder = Box::new(move |response: http::Response<Cow<'static, [u8]>>| {
          // Check if this is an HTML response that needs script injection
          let content_type = response.headers().get(CONTENT_TYPE);
          let is_html = content_type
            .and_then(|ct| ct.to_str().ok())
            .map(|ct| ct.to_lowercase().starts_with("text/html"))
            .unwrap_or(false);

          let (parts, body) = response.into_parts();
          let body_bytes = body.into_owned();

          let modified_body = if is_html {
            inject_scripts_into_html_body(&body_bytes, &initialization_scripts)
              .unwrap_or(body_bytes)
          } else {
            body_bytes
          };

          let response = http::Response::from_parts(parts, Cursor::new(modified_body));
          response_store.into_owned().borrow_mut().replace(response);

          let callback = callback.into_owned();
          callback.cont();
        });

        let label = self.webview_label.clone();
        let handler = self.handler.clone();

        let data = read_request_body(request);
        let headers = get_request_headers(request);
        let method_str = CefString::from(&request.method()).to_string();
        let method = http::Method::from_bytes(method_str.as_bytes())
          .unwrap_or(http::Method::GET);

        std::thread::spawn(move || {
          let mut http_request = http::Request::builder().method(method).uri(url.as_str()).body(data).unwrap();
          *http_request.headers_mut() = headers;
          (handler)(&label, http_request, responder);
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
      let count = self.response.borrow_mut().as_mut().and_then(|response| response.body_mut().read(data_out).ok()).unwrap_or(0);
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
      let (Some(response), Some(response_data)) = (response, &*self.response.borrow()) else { return };

      response.set_status(response_data.status().as_u16() as i32);
      let mut content_type = None;
      let mut csp_header: Option<String> = None;

      // First pass: collect CSP header and set other headers
      for (name, value) in response_data.headers() {
        let Ok(value) = value.to_str() else { continue; };

        if name == CONTENT_SECURITY_POLICY {
          csp_header = Some(value.to_string());
        } else {
          response.set_header_by_name(Some(&name.as_str().into()), Some(&value.into()), 0);
        }

        if name == CONTENT_TYPE {
          content_type.replace(value.into());
        }
      }

      let new_csp = csp_inject_initialization_scripts_hashes(
        csp_header.as_deref(),
        &self.initialization_scripts,
      );

      if let Some(new_csp) = new_csp {
        let csp_header_name = CefString::from(CONTENT_SECURITY_POLICY.as_str());
        response.set_header_by_name(
          Some(&csp_header_name),
          Some(&CefString::from(new_csp.as_str())),
          0, // overwrite
        );
      }

      response.set_mime_type(Some(&content_type.unwrap_or_else(|| "text/plain".into())));

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
    webview_label: String,
    handler: Arc<UriSchemeProtocol>,
    initialization_scripts: Vec<CefInitScript>,
  }

  impl SchemeHandlerFactory {
    fn create(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _scheme_name: Option<&CefString>,
      _request: Option<&mut Request>,
    ) -> Option<ResourceHandler> {
      Some(WebResourceHandler::new(self.webview_label.clone(), self.handler.clone(), self.initialization_scripts.clone(), Arc::new(RefCell::new(None))))
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
