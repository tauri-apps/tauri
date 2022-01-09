// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The module to process HTML in Tauri.

use html5ever::{interface::QualName, namespace_url, ns, LocalName};
use kuchiki::{Attribute, ExpandedName, NodeRef};

/// The token used on the CSP tag content.
pub const CSP_TOKEN: &str = "__TAURI_CSP__";
/// The token used for script nonces.
pub const SCRIPT_NONCE_TOKEN: &str = "__TAURI_SCRIPT_NONCE__";
/// The token used for style nonces.
pub const STYLE_NONCE_TOKEN: &str = "__TAURI_STYLE_NONCE__";
/// The token used for the invoke key.
pub const INVOKE_KEY_TOKEN: &str = "__TAURI__INVOKE_KEY_TOKEN__";

fn inject_nonce(document: &mut NodeRef, selector: &str, token: &str) {
  if let Ok(scripts) = document.select(selector) {
    for target in scripts {
      let node = target.as_node();
      let element = node.as_element().unwrap();

      let mut attrs = element.attributes.borrow_mut();
      // if the node already has the `nonce` attribute, skip it
      if attrs.get("nonce").is_some() {
        continue;
      }
      attrs.insert("nonce", token.into());
    }
  }
}

/// Inject nonce tokens to all scripts and styles.
pub fn inject_nonce_token(document: &mut NodeRef) {
  inject_nonce(document, "script[src^='http']", SCRIPT_NONCE_TOKEN);
  inject_nonce(document, "style", STYLE_NONCE_TOKEN);
}

/// Injects the invoke key token to each script on the document.
///
/// The invoke key token is replaced at runtime with the actual invoke key value.
pub fn inject_invoke_key_token(document: &mut NodeRef) {
  let mut targets = vec![];
  if let Ok(scripts) = document.select("script") {
    for target in scripts {
      targets.push(target);
    }
    for target in targets {
      let node = target.as_node();
      let element = node.as_element().unwrap();

      let attrs = element.attributes.borrow();
      // if the script is external (has `src`), we won't inject the token
      if attrs.get("src").is_some() {
        continue;
      }

      let replacement_node = match attrs.get("type") {
        Some("module") | Some("application/ecmascript") => {
          let replacement_node = NodeRef::new_element(
            QualName::new(None, ns!(html), "script".into()),
            element
              .attributes
              .borrow()
              .clone()
              .map
              .into_iter()
              .collect::<Vec<_>>(),
          );
          let script = node.text_contents();
          replacement_node.append(NodeRef::new_text(format!(
            r#"
          const __TAURI_INVOKE_KEY__ = {token};
          {script}
        "#,
            token = INVOKE_KEY_TOKEN,
            script = script
          )));
          replacement_node
        }
        Some("application/javascript") | None => {
          let replacement_node = NodeRef::new_element(
            QualName::new(None, ns!(html), "script".into()),
            element
              .attributes
              .borrow()
              .clone()
              .map
              .into_iter()
              .collect::<Vec<_>>(),
          );
          let script = node.text_contents();
          replacement_node.append(NodeRef::new_text(
            script.replace("__TAURI_INVOKE_KEY__", INVOKE_KEY_TOKEN),
          ));
          replacement_node
        }
        _ => {
          continue;
        }
      };

      node.insert_after(replacement_node);
      node.detach();
    }
  }
}

/// Injects a content security policy token to the HTML.
pub fn inject_csp_token(document: &mut NodeRef) {
  if let Ok(ref head) = document.select_first("head") {
    head.as_node().append(create_csp_meta_tag(CSP_TOKEN));
  } else {
    let head = NodeRef::new_element(
      QualName::new(None, ns!(html), LocalName::from("head")),
      None,
    );
    head.append(create_csp_meta_tag(CSP_TOKEN));
    document.prepend(head);
  }
}

fn create_csp_meta_tag(csp: &str) -> NodeRef {
  NodeRef::new_element(
    QualName::new(None, ns!(html), LocalName::from("meta")),
    vec![
      (
        ExpandedName::new(ns!(), LocalName::from("http-equiv")),
        Attribute {
          prefix: None,
          value: "Content-Security-Policy".into(),
        },
      ),
      (
        ExpandedName::new(ns!(), LocalName::from("content")),
        Attribute {
          prefix: None,
          value: csp.into(),
        },
      ),
    ],
  )
}

#[cfg(test)]
mod tests {
  use kuchiki::traits::*;
  #[test]
  fn csp() {
    let htmls = vec![
      "<html><head></head></html>".to_string(),
      "<html></html>".to_string(),
    ];
    for html in htmls {
      let mut document = kuchiki::parse_html().one(html);
      super::inject_csp_token(&mut document);
      assert_eq!(
        document.to_string(),
        format!(
          r#"<html><head><meta content="{}" http-equiv="Content-Security-Policy"></head><body></body></html>"#,
          super::CSP_TOKEN
        )
      );
    }
  }
}
