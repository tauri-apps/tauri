// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use html5ever::{
  interface::QualName,
  namespace_url, ns,
  tendril::{fmt::UTF8, NonAtomic, Tendril},
  LocalName,
};
use kuchiki::{traits::*, Attribute, ExpandedName, NodeRef};

/// Injects a content security policy to the HTML.
pub fn inject_csp<H: Into<Tendril<UTF8, NonAtomic>>>(html: H, csp: &str) -> String {
  let document = kuchiki::parse_html().one(html);
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
  document.to_string()
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
  #[test]
  fn csp() {
    let htmls = vec![
      "<html><head></head></html>".to_string(),
      "<html></html>".to_string(),
    ];
    for html in htmls {
      let csp = "default-src 'self'; img-src https://*; child-src 'none';";
      let new = super::inject_csp(html, csp);
      assert_eq!(
        new,
        format!(
          r#"<html><head><meta content="{}" http-equiv="Content-Security-Policy"></head><body></body></html>"#,
          csp
        )
      );
    }
  }
}
