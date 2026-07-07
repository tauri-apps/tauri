// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This is an internal implementation detail used by the Android backend to inject
//! initialization scripts when `addDocumentStartJavaScript` is not supported.

use base64::{prelude::BASE64_STANDARD, Engine};
use dom_query::Document;
use http::{
  header::{HeaderValue, CONTENT_SECURITY_POLICY, CONTENT_TYPE},
  Response as HttpResponse,
};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

use crate::InitializationScript;

pub fn inject_scripts_into_html(
  mut response: HttpResponse<Cow<'static, [u8]>>,
  scripts: &[InitializationScript],
) -> HttpResponse<Cow<'static, [u8]>> {
  if scripts.is_empty() {
    return response;
  }

  let should_inject_scripts = response
    .headers()
    .get(CONTENT_TYPE)
    // Content-Type must begin with the media type, but is case-insensitive.
    // It may also be followed by any number of semicolon-delimited key value pairs.
    // We don't care about these here.
    // source: https://httpwg.org/specs/rfc9110.html#rfc.section.8.3.1
    .and_then(|content_type| content_type.to_str().ok())
    .map(|content_type_str| content_type_str.to_lowercase().starts_with("text/html"))
    .unwrap_or_default();

  if !should_inject_scripts {
    return response;
  }

  let document = Document::from(String::from_utf8_lossy(response.body()).as_ref());
  let csp = response.headers_mut().get_mut(CONTENT_SECURITY_POLICY);

  // Get or create head element
  let head = document.head().unwrap_or_else(|| {
    let html = document.html_root();
    let head = document.tree.new_element("head");
    html.prepend_child(&head);
    head
  });

  // Iterate in reverse order since we are prepending each script to the head tag
  let mut hashes = Vec::new();
  for script in scripts.iter().rev().map(|s| &s.script) {
    let script_tag = document.tree.new_element("script");
    script_tag.set_text(script.as_str());
    head.prepend_child(&script_tag);
    if csp.is_some() {
      hashes.push(hash_script(script));
    }
  }

  if let Some(csp) = csp {
    let csp_string = csp.to_str().unwrap().to_string();
    let csp_string = if csp_string.contains("script-src") {
      csp_string.replace("script-src", &format!("script-src {}", hashes.join(" ")))
    } else {
      format!("{csp_string} script-src {}", hashes.join(" "))
    };
    *csp = HeaderValue::from_str(&csp_string).unwrap();
  }

  *response.body_mut() = Cow::Owned(document.html().as_bytes().to_vec());
  response
}

fn hash_script(script: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(script);
  let hash = hasher.finalize();
  format!("'sha256-{}'", BASE64_STANDARD.encode(hash))
}

#[cfg(test)]
mod tests {
  use super::*;
  use http::StatusCode;

  #[test]
  fn test_no_scripts_returns_original_response() {
    let body = "<html><head></head><body>Test</body></html>";

    let result = run(body, "text/html", vec![]);

    assert_eq!(result, body);
  }

  #[test]
  fn test_non_html_response_not_modified() {
    let body = r#"{"key": "value"}"#;
    let scripts = vec!["console.log('test');".to_string()];

    let result = run(body, "application/json", scripts);

    assert_eq!(result, body);
  }

  #[test]
  fn test_inject_single_script() {
    let body = "<html><head></head><body>Content</body></html>";
    let scripts = vec!["console.log('injected');".to_string()];

    let result = run(body, "text/html", scripts);

    assert_eq!(
      result,
      "<html><head><script>console.log('injected');</script></head><body>Content</body></html>"
    );
  }

  #[test]
  fn test_inject_multiple_scripts() {
    let body = "<html><head></head><body>Content</body></html>";
    let scripts = vec![
      "var first = 1;".to_owned(),
      "let second = 2;".to_owned(),
      "const third = 3;".to_owned(),
      "window.test = () => console.log('test');".to_owned(),
    ];

    let result = run(body, "text/html", scripts);

    assert_eq!(
      result,
      "<html><head><script>var first = 1;</script><script>let second = 2;</script><script>const third = 3;</script><script>window.test = () => console.log('test');</script></head><body>Content</body></html>"
    );
  }

  #[test]
  fn test_inject_script_creates_head_if_missing() {
    let body = "<html><body>Content</body></html>";
    let scripts = vec!["console.log('test');".to_string()];

    let result = run(body, "text/html", scripts);

    assert_eq!(
      result,
      "<html><head><script>console.log('test');</script></head><body>Content</body></html>"
    );
  }

  #[test]
  fn test_inject_script_creates_html_structure_if_missing() {
    let body = "Just some text";
    let scripts = vec!["console.log('test');".to_string()];

    let result = run(body, "text/html", scripts);

    assert_eq!(
      result,
      "<html><head><script>console.log('test');</script></head><body>Just some text</body></html>"
    );
  }

  #[test]
  fn test_csp_header_updated_with_script_hashes() {
    let body = "<html><head></head><body>Content</body></html>";
    let mut response = create_response(body, "text/html");
    response.headers_mut().insert(
      CONTENT_SECURITY_POLICY,
      HeaderValue::from_static("default-src 'self'"),
    );

    let script_code = "console.log('test');";
    let scripts = vec![script_code.to_string()];

    let scripts: Vec<InitializationScript> = scripts
      .into_iter()
      .map(|script| InitializationScript {
        script,
        for_main_frame_only: true,
      })
      .collect();
    let result = inject_scripts_into_html(response, &scripts);
    let result_body = String::from_utf8_lossy(result.body()).to_string();
    let csp = result.headers().get(CONTENT_SECURITY_POLICY).unwrap();
    let csp_str = csp.to_str().unwrap();

    assert_eq!(
      result_body,
      "<html><head><script>console.log('test');</script></head><body>Content</body></html>"
    );
    assert_eq!(
      csp_str,
      "default-src 'self' script-src 'sha256-3x8DE279hr8o/Aq0dEdH4WApIwn5rbRKhugPzn6Bofw='"
    );
  }

  fn create_response(body: &str, content_type: &'static str) -> HttpResponse<Cow<'static, [u8]>> {
    let mut response = HttpResponse::builder()
      .status(StatusCode::OK)
      .body(Cow::Owned(body.as_bytes().to_vec()))
      .unwrap();
    response
      .headers_mut()
      .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

    response
  }

  /// Helper function to create a response, inject scripts, and return the body as a string
  fn run(body: &str, content_type: &'static str, scripts: Vec<String>) -> String {
    let response = create_response(body, content_type);
    let scripts: Vec<InitializationScript> = scripts
      .into_iter()
      .map(|script| InitializationScript {
        script,
        for_main_frame_only: true,
      })
      .collect();
    let result = inject_scripts_into_html(response, &scripts);
    String::from_utf8_lossy(result.body()).to_string()
  }
}
