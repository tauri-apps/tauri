// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The module to process HTML in Tauri.

use std::path::{Path, PathBuf};

use html5ever::{
  interface::QualName,
  namespace_url, ns,
  serialize::{HtmlSerializer, SerializeOpts, Serializer, TraversalScope},
  tendril::TendrilSink,
  LocalName,
};
pub use kuchiki::NodeRef;
use kuchiki::{Attribute, ExpandedName, NodeData};
use serde::Serialize;
#[cfg(feature = "isolation")]
use serialize_to_javascript::DefaultTemplate;

use crate::config::{DisabledCspModificationKind, PatternKind};
#[cfg(feature = "isolation")]
use crate::pattern::isolation::IsolationJavascriptCodegen;

/// The token used on the CSP tag content.
pub const CSP_TOKEN: &str = "__TAURI_CSP__";
/// The token used for script nonces.
pub const SCRIPT_NONCE_TOKEN: &str = "__TAURI_SCRIPT_NONCE__";
/// The token used for style nonces.
pub const STYLE_NONCE_TOKEN: &str = "__TAURI_STYLE_NONCE__";

// taken from https://github.com/kuchiki-rs/kuchiki/blob/57ee6920d835315a498e748ba4b07a851ae5e498/src/serializer.rs#L12
fn serialize_node_ref_internal<S: Serializer>(
  node: &NodeRef,
  serializer: &mut S,
  traversal_scope: TraversalScope,
) -> crate::Result<()> {
  match (traversal_scope, node.data()) {
    (ref scope, NodeData::Element(element)) => {
      if *scope == TraversalScope::IncludeNode {
        let attrs = element.attributes.borrow();

        // Unfortunately we need to allocate something to hold these &'a QualName
        let attrs = attrs
          .map
          .iter()
          .map(|(name, attr)| {
            (
              QualName::new(attr.prefix.clone(), name.ns.clone(), name.local.clone()),
              &attr.value,
            )
          })
          .collect::<Vec<_>>();

        serializer.start_elem(
          element.name.clone(),
          attrs.iter().map(|&(ref name, value)| (name, &**value)),
        )?
      }

      let children = match element.template_contents.as_ref() {
        Some(template_root) => template_root.children(),
        None => node.children(),
      };
      for child in children {
        serialize_node_ref_internal(&child, serializer, TraversalScope::IncludeNode)?
      }

      if *scope == TraversalScope::IncludeNode {
        serializer.end_elem(element.name.clone())?
      }
      Ok(())
    }

    (_, &NodeData::DocumentFragment) | (_, &NodeData::Document(_)) => {
      for child in node.children() {
        serialize_node_ref_internal(&child, serializer, TraversalScope::IncludeNode)?
      }
      Ok(())
    }

    (TraversalScope::ChildrenOnly(_), _) => Ok(()),

    (TraversalScope::IncludeNode, NodeData::Doctype(doctype)) => {
      serializer.write_doctype(&doctype.name).map_err(Into::into)
    }
    (TraversalScope::IncludeNode, NodeData::Text(text)) => {
      serializer.write_text(&text.borrow()).map_err(Into::into)
    }
    (TraversalScope::IncludeNode, NodeData::Comment(text)) => {
      serializer.write_comment(&text.borrow()).map_err(Into::into)
    }
    (TraversalScope::IncludeNode, NodeData::ProcessingInstruction(contents)) => {
      let contents = contents.borrow();
      serializer
        .write_processing_instruction(&contents.0, &contents.1)
        .map_err(Into::into)
    }
  }
}

/// Serializes the node to HTML.
pub fn serialize_node(node: &NodeRef) -> Vec<u8> {
  let mut u8_vec = Vec::new();
  let mut ser = HtmlSerializer::new(
    &mut u8_vec,
    SerializeOpts {
      traversal_scope: TraversalScope::IncludeNode,
      ..Default::default()
    },
  );
  serialize_node_ref_internal(node, &mut ser, TraversalScope::IncludeNode).unwrap();
  u8_vec
}

/// Parses the given HTML string.
pub fn parse(html: String) -> NodeRef {
  kuchiki::parse_html().one(html)
}

fn with_head<F: FnOnce(&NodeRef)>(document: &NodeRef, f: F) {
  if let Ok(ref node) = document.select_first("head") {
    f(node.as_node())
  } else {
    let node = NodeRef::new_element(
      QualName::new(None, ns!(html), LocalName::from("head")),
      None,
    );
    f(&node);
    document.prepend(node)
  }
}

fn inject_nonce(document: &NodeRef, selector: &str, token: &str) {
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
pub fn inject_nonce_token(
  document: &NodeRef,
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
pub fn inject_csp(document: &NodeRef, csp: &str) {
  with_head(document, |head| {
    head.append(create_csp_meta_tag(csp));
  });
}

/// Injects a content security policy token to the HTML.
pub fn inject_csp_token(document: &NodeRef) {
  inject_csp(document, CSP_TOKEN)
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IsolationSide {
  /// Original frame, the Brownfield application
  Original,
  /// Secure frame, the isolation security application
  Secure,
}

impl Default for IsolationSide {
  fn default() -> Self {
    Self::Original
  }
}

/// Injects the Isolation JavaScript to a codegen time document.
///
/// Note: This function is not considered part of the stable API.
#[cfg(feature = "isolation")]
pub fn inject_codegen_isolation_script(document: &NodeRef) {
  with_head(document, |head| {
    let script = NodeRef::new_element(QualName::new(None, ns!(html), "script".into()), None);
    script.append(NodeRef::new_text(
      IsolationJavascriptCodegen {}
        .render_default(&Default::default())
        .expect("unable to render codegen isolation script template")
        .into_string(),
    ));

    head.prepend(script);
  });
}

/// Temporary workaround for Windows not allowing requests
///
/// Note: this does not prevent path traversal due to the isolation application expectation that it
/// is secure.
pub fn inline_isolation(document: &NodeRef, dir: &Path) {
  for script in document
    .select("script[src]")
    .expect("unable to parse document for scripts")
  {
    let src = {
      let attributes = script.attributes.borrow();
      attributes
        .get(LocalName::from("src"))
        .expect("script with src attribute has no src value")
        .to_string()
    };

    let mut path = PathBuf::from(src);
    if path.has_root() {
      path = path
        .strip_prefix("/")
        .expect("Tauri \"Isolation\" Pattern only supports relative or absolute (`/`) paths.")
        .into();
    }

    let file = std::fs::read_to_string(dir.join(path)).expect("unable to find isolation file");
    script.as_node().append(NodeRef::new_text(file));

    let mut attributes = script.attributes.borrow_mut();
    attributes.remove(LocalName::from("src"));
  }
}

/// Temporary naive method to check if a string is a html
pub fn is_html(data_string: &str) -> bool {
  data_string.contains('<') && data_string.contains('>')
}

/// Temporary naive method to extract data from html data string
pub fn extract_html_content(input: &str) -> Option<&str> {
  if input.starts_with("data:text/html,") {
      Some(&input[15..])
  } else {
      None
  }
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
      let document = kuchiki::parse_html().one(html);
      super::inject_csp_token(&document);
      assert_eq!(
        document.to_string(),
        format!(
          r#"<html><head><meta http-equiv="Content-Security-Policy" content="{}"></head><body></body></html>"#,
          super::CSP_TOKEN
        )
      );
    }
  }
}
