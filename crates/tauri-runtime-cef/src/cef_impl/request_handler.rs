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

// ResponseFilter that injects initialization scripts into HTML responses
wrap_response_filter! {
  pub struct HtmlScriptInjectionFilter {
    initialization_scripts: Vec<CefInitScript>,
    processed_html: RefCell<Option<Vec<u8>>>,
    output_offset: RefCell<usize>,
  }

  impl ResponseFilter {
    fn init_filter(&self) -> ::std::os::raw::c_int {
      // Return 1 to enable buffered mode (RESPONSE_FILTER_NEED_MORE_DATA)
      1
    }

    fn filter(
      &self,
      data_in: Option<&mut Vec<u8>>,
      data_in_read: Option<&mut usize>,
      data_out: Option<&mut Vec<u8>>,
      data_out_written: Option<&mut usize>,
    ) -> ResponseFilterStatus {
      use cef_dll_sys::cef_response_filter_status_t::*;

      if let Some(data_in) = data_in {
        let input_size = data_in.len();

        // Process the HTML once (on first chunk)
        if self.processed_html.borrow().is_none() {
          if let Ok(html_str) = String::from_utf8(data_in.clone()) {
            let document = parse_html(html_str);

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

            // iterate in reverse order since we are prepending each script to the head tag
            for init_script in self.initialization_scripts.iter().rev() {
              let script_el = NodeRef::new_element(
                QualName::new(None, ns!(html), "script".into()),
                None,
              );
              script_el.append(NodeRef::new_text(init_script.script.script.as_str()));
              head.prepend(script_el);
            }

            // Serialize the modified HTML
            let modified_html = serialize_node(&document);
            *self.processed_html.borrow_mut() = Some(modified_html);
          } else {
            // Not valid UTF-8, pass through unchanged
            *self.processed_html.borrow_mut() = Some(data_in.clone());
          }
        }

        // Mark all input as read (CEF requirement: must read all input)
        if let Some(data_in_read) = data_in_read {
          *data_in_read = input_size;
        }
      }

      if let Some(data_out) = data_out {
        if let Some(processed) = self.processed_html.borrow().as_ref() {
          let offset = *self.output_offset.borrow();
          let remaining = processed.len().saturating_sub(offset);

          if remaining > 0 {
            let buffer_size = data_out.capacity();
            let to_write = remaining.min(buffer_size);

            data_out.clear();
            data_out.extend_from_slice(&processed[offset..offset + to_write]);

            if let Some(written) = data_out_written {
              *written = to_write;
            }

            *self.output_offset.borrow_mut() += to_write;

            if *self.output_offset.borrow() >= processed.len() {
              RESPONSE_FILTER_DONE.into()
            } else {
              RESPONSE_FILTER_NEED_MORE_DATA.into()
            }
          } else {
            // No remaining data to write
            RESPONSE_FILTER_DONE.into()
          }
        } else {
          // No processed HTML yet - need more input data
          RESPONSE_FILTER_NEED_MORE_DATA.into()
        }
      } else {
        // No output buffer provided
        // If we have input, we've already processed it, so we're done
        // If we don't have input, this is a final call and we're done
        RESPONSE_FILTER_DONE.into()
      }
    }
  }
}

wrap_resource_request_handler! {
  pub struct WebResourceRequestHandler {
    initialization_scripts: Vec<CefInitScript>,
  }

  impl ResourceRequestHandler {
    // Store CSP header from on_resource_response for use in filter
    fn on_resource_response(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _request: Option<&mut Request>,
      _response: Option<&mut Response>,
    ) -> ::std::os::raw::c_int {
      // Response is read-only here, but we can read the CSP header
      // and it will be available in resource_response_filter
      0
    }

    fn resource_response_filter(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      request: Option<&mut Request>,
      response: Option<&mut Response>,
    ) -> Option<ResponseFilter> {
      let Some(response) = response else {
        return None;
      };

      // Skip DevTools URLs - they use internal resources that shouldn't be modified
      if let Some(request) = request {
        let url = CefString::from(&request.url()).to_string();
        if url.starts_with("devtools://") {
          return None;
        }
      }

      let content_type = response.mime_type();
      let content_type_str = CefString::from(&content_type).to_string().to_lowercase();

      // Only process HTML responses
      if !content_type_str.starts_with("text/html") {
        return None;
      }

      let Some(frame) = frame else {
        return None;
      };

      let is_main_frame = frame.is_main() == 1;

      // Filter scripts based on frame type
      let scripts_to_inject: Vec<_> = if is_main_frame {
        self.initialization_scripts.clone()
      } else {
        self.initialization_scripts
          .iter()
          .filter(|s| !s.script.for_main_frame_only)
          .cloned()
          .collect()
      };

      if scripts_to_inject.is_empty() {
        return None;
      }

      // Get pre-computed hashes for the scripts
      let script_hashes: Vec<String> = if is_main_frame {
        self.initialization_scripts.iter().map(|s| s.hash.clone()).collect()
      } else {
        self.initialization_scripts
          .iter()
          .filter(|s| !s.script.for_main_frame_only)
          .map(|s| s.hash.clone())
          .collect()
      };

      // Return a filter that will inject scripts and update CSP in HTML if header exists
      Some(HtmlScriptInjectionFilter::new(
        scripts_to_inject,
        RefCell::new(None),
        RefCell::new(0),
      ))
    }

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

      if let Some(url) = url {
        let callback = ThreadSafe(callback.clone());
        // TODO: thread safety
        let response_store = ThreadSafe(self.context.response.clone());
        let responder = Box::new(move |response: http::Response<Cow<'static, [u8]>>| {
          let (parts, body) = response.into_parts();
          let response = http::Response::from_parts(parts, Cursor::new(body.into_owned()));
          response_store.into_owned().borrow_mut().replace(response);
          let callback = callback.into_owned();
          callback.cont();
        });

        let label = self.context.label.clone();
        let handler = self.context.handler.clone();

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
      let count = self.context.response.borrow_mut().as_mut().and_then(|response| response.body_mut().read(data_out).ok()).unwrap_or(0);
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
      let (Some(response), Some(response_data)) = (response, &*self.context.response.borrow()) else { return };

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

      // Update CSP header with script hashes for custom schemes
      if let Some(ref initialization_scripts) = self.context.initialization_scripts {
        if !initialization_scripts.is_empty() {
          let scripts_to_include: Vec<_> = initialization_scripts
            .iter()
            .filter(|s| !s.script.for_main_frame_only)
            .cloned()
            .collect();

          if !scripts_to_include.is_empty() {
            let script_hashes: Vec<String> = scripts_to_include
              .iter()
              .map(|s| s.hash.clone())
              .collect();

          let csp_header_name = CefString::from(CONTENT_SECURITY_POLICY.as_str());
          let new_csp = if let Some(existing_csp) = csp_header {
            // Parse CSP using tauri-utils
            let mut csp_map: std::collections::HashMap<String, CspDirectiveSources> =
              Csp::Policy(existing_csp).into();

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

            response.set_header_by_name(
              Some(&csp_header_name),
              Some(&CefString::from(new_csp.as_str())),
              1, // overwrite
            );
          }
        }
      } else if let Some(csp) = csp_header {
        // No scripts to inject, just copy the original CSP header
        response.set_header_by_name(
          Some(&CefString::from(CONTENT_SECURITY_POLICY.as_str())),
          Some(&CefString::from(csp.as_str())),
          0,
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
    context: UriSchemeContext,
  }

  impl SchemeHandlerFactory {
    fn create(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _scheme_name: Option<&CefString>,
      _request: Option<&mut Request>,
    ) -> Option<ResourceHandler> {
      Some(WebResourceHandler::new(self.context.clone()))
    }
  }
}

#[derive(Clone)]
pub struct UriSchemeContext {
  pub label: String,
  pub handler: Arc<UriSchemeProtocol>,
  pub response: Arc<RefCell<Option<http::Response<Cursor<Vec<u8>>>>>>,
  pub initialization_scripts: Option<Vec<CefInitScript>>,
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
