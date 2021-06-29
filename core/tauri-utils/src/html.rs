// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use html5ever::{interface::QualName, namespace_url, ns, LocalName};
use kuchiki::{Attribute, ExpandedName, NodeRef};

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
      // if the script is external (has `src`) or its type is not "module", we won't inject the token
      if attrs.get("src").is_some() || attrs.get("type") != Some("module") {
        continue;
      }

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
          const __TAURI_INVOKE_KEY__ = __TAURI__INVOKE_KEY_TOKEN__;
          {}
        "#,
        script
      )));

      node.insert_after(replacement_node);
      node.detach();
    }
  }
}

/// Injects a content security policy to the HTML.
pub fn inject_csp(document: &mut NodeRef, csp: &str) {
  if let Ok(ref head) = document.select_first("head") {
    head.as_node().append(create_csp_meta_tag(csp));
  } else {
    let head = NodeRef::new_element(
      QualName::new(None, ns!(html), LocalName::from("head")),
      None,
    );
    head.append(create_csp_meta_tag(csp));
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
      let csp = "default-src 'self'; img-src https://*; child-src 'none';";
      super::inject_csp(&mut document, csp);
      assert_eq!(
        document.to_string(),
        format!(
          r#"<html><head><meta content="{}" http-equiv="Content-Security-Policy"></head><body></body></html>"#,
          csp
        )
      );
    }
  }
}
