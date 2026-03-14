// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The module to process HTML in Tauri.

use std::path::{Path, PathBuf};

use dom_query::NodeRef;
use serde::Serialize;

#[cfg(feature = "isolation")]
use serialize_to_javascript::DefaultTemplate;

#[cfg(feature = "isolation")]
use crate::pattern::isolation::IsolationJavascriptCodegen;
use crate::{
  assets::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
  config::{DisabledCspModificationKind, PatternKind},
};

pub use dom_query::Document;

/// Serializes the document to HTML.
pub fn serialize_doc(document: &Document) -> Vec<u8> {
  document.html().as_bytes().to_vec()
}

/// Parses the given HTML string.
pub fn parse_doc(html: String) -> Document {
  Document::from(html.as_str())
}

fn with_head<F: FnOnce(NodeRef<'_>)>(document: &Document, f: F) {
  let head = document.head().unwrap_or_else(|| {
    let html = document.html_root();
    let head = document.tree.new_element("head");
    html.prepend_child(&head);
    head
  });

  f(head)
}

fn inject_nonce(document: &Document, selector: &str, token: &str) {
  let elements = document.select(selector);
  for elem in elements.iter() {
    // if the node already has the `nonce` attribute, skip it
    if elem.attr("nonce").is_some() {
      continue;
    }
    elem.set_attr("nonce", token);
  }
}

/// Inject nonce tokens to all scripts and styles.
pub fn inject_nonce_token(
  document: &Document,
  dangerous_disable_asset_csp_modification: &DisabledCspModificationKind,
) {
  if dangerous_disable_asset_csp_modification.can_modify("script-src") {
    inject_nonce(document, "script[src^='http']", SCRIPT_NONCE_TOKEN);
  }
  if dangerous_disable_asset_csp_modification.can_modify("style-src") {
    inject_nonce(document, "style", STYLE_NONCE_TOKEN);
  }
}

/// Injects a content security policy to the HTML.
pub fn inject_csp(document: &Document, csp: &str) {
  with_head(document, |head| {
    let meta_tag = format!(r#"<meta http-equiv="Content-Security-Policy" content="{csp}">"#);

    head.prepend_html(meta_tag.as_str());
  });
}

/// Injects a content security policy to the HTML.
pub fn append_script_to_head(document: &Document, script: &str) {
  with_head(document, |head| {
    let script_tag = format!(r#"<script>{script}</script>"#);

    head.prepend_html(script_tag.as_str());
  });
}

/// The shape of the JavaScript Pattern config
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase", tag = "pattern")]
pub enum PatternObject {
  /// Brownfield pattern.
  Brownfield,
  /// Isolation pattern. Recommended for security purposes.
  Isolation {
    /// Which `IsolationSide` this `PatternObject` is getting injected into
    side: IsolationSide,
  },
}

impl From<&PatternKind> for PatternObject {
  fn from(pattern_kind: &PatternKind) -> Self {
    match pattern_kind {
      PatternKind::Brownfield => Self::Brownfield,
      PatternKind::Isolation { .. } => Self::Isolation {
        side: IsolationSide::default(),
      },
    }
  }
}

/// Where the JavaScript is injected to
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IsolationSide {
  /// Original frame, the Brownfield application
  #[default]
  Original,
  /// Secure frame, the isolation security application
  Secure,
}

/// Injects the Isolation JavaScript to a codegen time document.
///
/// Note: This function is not considered part of the stable API.
#[cfg(feature = "isolation")]
pub fn inject_codegen_isolation_script(document: &Document) {
  with_head(document, |head| {
    let script_content = IsolationJavascriptCodegen {}
      .render_default(&Default::default())
      .expect("unable to render codegen isolation script template")
      .into_string();

    let script_tag = format!(
      r#"<script nonce="{}">{}</script>"#,
      SCRIPT_NONCE_TOKEN, script_content
    );

    head.prepend_html(script_tag.as_str());
  });
}

/// Temporary workaround for Windows not allowing requests
///
/// Note: this does not prevent path traversal due to the isolation application expectation that it
/// is secure.
pub fn inline_isolation(document: &Document, dir: &Path) {
  let scripts = document.select("script[src]");

  for script in scripts.iter() {
    let src = match script.attr("src") {
      Some(s) => s.to_string(),
      None => continue,
    };

    let mut path = PathBuf::from(src);
    if path.has_root() {
      path = path
        .strip_prefix("/")
        .expect("Tauri \"Isolation\" Pattern only supports relative or absolute (`/`) paths.")
        .into();
    }

    let file = std::fs::read_to_string(dir.join(path)).expect("unable to find isolation file");

    script.set_html(file.as_str());
    script.remove_attr("src");
  }
}

// TODO: Verify this, this is not found in the HTML spec, see https://github.com/tauri-apps/tauri/pull/14265#discussion_r2415396842
/// Normalize line endings in script content to match what the browser uses for CSP hashing.
///
/// According to the HTML spec, browsers normalize:
/// - `\r\n` → `\n`
/// - `\r`   → `\n`
pub fn normalize_script_for_csp(input: &[u8]) -> Vec<u8> {
  let mut output = Vec::with_capacity(input.len());

  let mut i = 0;
  while i < input.len() {
    match input[i] {
      b'\r' => {
        if i + 1 < input.len() && input[i + 1] == b'\n' {
          // CRLF → LF
          output.push(b'\n');
          i += 2;
        } else {
          // Lone CR → LF
          output.push(b'\n');
          i += 1;
        }
      }
      _ => {
        output.push(input[i]);
        i += 1;
      }
    }
  }

  output
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use super::*;
  use crate::{
    assets::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
    config,
  };

  #[test]
  fn csp() {
    let htmls = vec![
      "<html><head></head></html>".to_string(),
      "<html></html>".to_string(),
    ];

    for html in htmls {
      let document = parse_doc(html);
      let csp = "csp-string";
      inject_csp(&document, csp);

      assert_eq!(
        String::from_utf8(serialize_doc(&document)).unwrap(),
        format!(
          r#"<html><head><meta http-equiv="Content-Security-Policy" content="{csp}"></head><body></body></html>"#
        )
      );
    }
  }

  #[test]
  fn normalize_script_for_csp_test() {
    let js = "// Copyright 2019-2024 Tauri Programme within The Commons Conservancy\r// SPDX-License-Identifier: Apache-2.0\n// SPDX-License-Identifier: MIT\r\n\r\nwindow.__TAURI_ISOLATION_HOOK__ = (payload, options) => {\r\n  return payload\r\n}\r\n";
    let expected = "// Copyright 2019-2024 Tauri Programme within The Commons Conservancy\n// SPDX-License-Identifier: Apache-2.0\n// SPDX-License-Identifier: MIT\n\nwindow.__TAURI_ISOLATION_HOOK__ = (payload, options) => {\n  return payload\n}\n";

    assert_eq!(normalize_script_for_csp(js.as_bytes()), expected.as_bytes())
  }

  #[test]
  fn parse_and_serialize_roundtrips() {
    let htmls = [
      "<html><head><title>Test</title></head><body><h1>Hello</h1></body></html>",
      "<!DOCTYPE html><html><head></head><body></body></html>",
    ];

    for html in htmls {
      let parsed = parse_doc(html.to_string());
      let serialized = serialize_doc(&parsed);
      let result = String::from_utf8(serialized).unwrap();

      assert_eq!(result, html);
    }
  }

  #[test]
  fn inject_nonce_to_scripts() {
    let html = r#"<html><head><script src="http://example.com/script.js"></script></head><body></body></html>"#;

    let document = parse_doc(html.to_string());
    inject_nonce_token(&document, &config::DisabledCspModificationKind::Flag(false));

    assert_eq!(
      String::from_utf8(serialize_doc(&document)).unwrap(),
      format!(
        r#"<html><head><script src="http://example.com/script.js" nonce="{SCRIPT_NONCE_TOKEN}"></script></head><body></body></html>"#
      )
    );
  }

  #[test]
  fn inject_nonce_to_styles() {
    let html = r#"<html><head><style>body { color: red; }</style></head><body></body></html>"#;

    let document = parse_doc(html.to_string());
    inject_nonce_token(&document, &config::DisabledCspModificationKind::Flag(false));

    assert_eq!(
      String::from_utf8(serialize_doc(&document)).unwrap(),
      format!(
        r#"<html><head><style nonce="{STYLE_NONCE_TOKEN}">body {{ color: red; }}</style></head><body></body></html>"#
      )
    );
  }

  #[test]
  fn append_script_to_head_test() {
    let html = r#"<html><head></head><body></body></html>"#;

    let document = parse_doc(html.to_string());
    append_script_to_head(&document, r#"console.log('Test')"#);

    assert_eq!(
      String::from_utf8(serialize_doc(&document)).unwrap(),
      format!(r#"<html><head><script>console.log('Test')</script></head><body></body></html>"#)
    );
  }

  #[test]
  fn inject_nonce_skips_existing() {
    let html = r#"<html><head><script src="http://example.com/script.js" nonce="existing"></script></head><body></body></html>"#;

    let document = parse_doc(html.to_string());
    inject_nonce_token(&document, &config::DisabledCspModificationKind::Flag(false));

    assert_eq!(String::from_utf8(serialize_doc(&document)).unwrap(), html);
  }

  #[test]
  fn inject_nonce_respects_disabled_modification() {
    let html = r#"<html><head><script src="http://example.com/script.js"></script></head><body></body></html>"#;

    let document = parse_doc(html.to_string());
    inject_nonce_token(&document, &config::DisabledCspModificationKind::Flag(true));

    assert_eq!(
      String::from_utf8(serialize_doc(&document)).unwrap(),
      r#"<html><head><script src="http://example.com/script.js"></script></head><body></body></html>"#
    );
  }

  #[test]
  fn inline_isolation_replaces_src_with_content() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut file = tempfile::tempfile_in(&temp_dir).unwrap();
    file.write_all(b"console.log('test');").unwrap();

    let html = r#"<html><head><script src="/test_script.js"></script></head><body></body></html>"#;
    let document = parse_doc(html.to_string());
    inline_isolation(&document, temp_dir.path());

    assert_eq!(
      String::from_utf8(serialize_doc(&document)).unwrap(),
      r#"<html><head><script>console.log('test');</script></head><body></body></html>"#
    );
  }
}
