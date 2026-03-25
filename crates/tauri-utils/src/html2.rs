// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The module to process HTML in Tauri.
//!
//! # Stability
//!
//! This is utility used in Tauri internally and not considered part of the stable API.
//! If you use it, note that it may include breaking changes in the future.

use dom_query::NodeRef;

use crate::{
  assets::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
  config::DisabledCspModificationKind,
};

/// # Stability
///
/// This dependency might receive updates in minor releases.
pub use dom_query::Document;

/// Serializes the document to HTML.
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
pub fn serialize_doc(document: &Document) -> Vec<u8> {
  document.html().as_bytes().to_vec()
}

/// Parses the given HTML string.
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
pub fn parse_doc(html: String) -> Document {
  Document::from(html)
}

fn ensure_head(document: &Document) -> NodeRef<'_> {
  document.head().unwrap_or_else(|| {
    let html = document.html_root();
    let head = document.tree.new_element("head");
    html.prepend_child(&head);
    head
  })
}

fn inject_nonce(document: &Document, selector: &str, token: &str) {
  let elements = document.select(selector);
  for elem in elements.nodes() {
    // if the node already has the `nonce` attribute, skip it
    if elem.attr("nonce").is_some() {
      continue;
    }
    elem.set_attr("nonce", token);
  }
}

/// Inject nonce tokens to all scripts and styles.
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
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
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
pub fn inject_csp(document: &Document, csp: &str) {
  let head = ensure_head(document);
  let meta_tag = document.tree.new_element("meta");
  meta_tag.set_attr("http-equiv", "Content-Security-Policy");
  meta_tag.set_attr("content", csp);
  head.append_child(&meta_tag);
}

/// Injects a content security policy to the HTML.
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
pub fn append_script_to_head(document: &Document, script: &str) {
  let head = ensure_head(document);
  let script_tag = document.tree.new_element("script");
  script_tag.set_text(script);
  head.prepend_child(&script_tag);
}

/// Injects the Isolation JavaScript to a codegen time document.
///
/// Note: This function is not considered part of the stable API.
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
#[cfg(feature = "isolation")]
pub fn inject_codegen_isolation_script(document: &Document) {
  use crate::pattern::isolation::IsolationJavascriptCodegen;
  use serialize_to_javascript::DefaultTemplate;

  let head = ensure_head(document);

  let script_content = IsolationJavascriptCodegen {}
    .render_default(&Default::default())
    .expect("unable to render codegen isolation script template")
    .into_string();

  let script_tag = document.tree.new_element("script");
  script_tag.set_attr("nonce", SCRIPT_NONCE_TOKEN);
  script_tag.set_text(script_content);

  head.prepend_child(&script_tag);
}

/// Temporary workaround for Windows not allowing requests
///
/// Note: this does not prevent path traversal due to the isolation application expectation that it
/// is secure.
///
/// # Stability
///
/// This dependency [`dom_query`] for [`Document`] might receive updates in minor releases.
#[cfg(feature = "isolation")]
pub fn inline_isolation(document: &Document, dir: &std::path::Path) {
  let scripts = document.select("script[src]");

  for script in scripts.nodes() {
    let src = match script.attr("src") {
      Some(s) => s.to_string(),
      None => continue,
    };

    let mut path = std::path::PathBuf::from(src);
    if path.has_root() {
      path = path
        .strip_prefix("/")
        .expect("Tauri \"Isolation\" Pattern only supports relative or absolute (`/`) paths.")
        .into();
    }

    let file = std::fs::read_to_string(dir.join(path)).expect("unable to find isolation file");

    script.set_text(file);
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
  #[cfg(feature = "isolation")]
  fn inline_isolation_replaces_src_with_content() {
    use std::io::Write;

    let temp_dir = tempfile::tempdir().unwrap();
    let mut file = tempfile::NamedTempFile::with_suffix_in(".js", &temp_dir).unwrap();
    file.write_all(b"console.log('test');").unwrap();
    let file_name = file.path().file_name().unwrap().to_str().unwrap();

    let html =
      format!(r#"<html><head><script src="/{file_name}"></script></head><body></body></html>"#);
    let document = parse_doc(html);
    inline_isolation(&document, temp_dir.path());

    assert_eq!(
      String::from_utf8(serialize_doc(&document)).unwrap(),
      r#"<html><head><script>console.log('test');</script></head><body></body></html>"#
    );
  }
}
