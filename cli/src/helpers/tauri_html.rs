use html5ever::{interface::QualName, namespace_url, ns, LocalName};
use inline_assets::inline_html_string;
use kuchiki::{traits::*, NodeRef};

use std::path::PathBuf;

pub struct TauriHtml {
  original: String,
  html_dir: PathBuf,
  inliner_enabled: bool,
}

impl TauriHtml {
  pub fn new(html_dir: impl Into<PathBuf>, html: String) -> Self {
    Self {
      original: html,
      html_dir: html_dir.into(),
      inliner_enabled: false,
    }
  }

  pub fn inliner_enabled(mut self, enabled: bool) -> Self {
    self.inliner_enabled = enabled;
    self
  }

  pub fn generate(self) -> crate::Result<String> {
    let html = if self.inliner_enabled {
      inline_html_string(&self.original, self.html_dir, Default::default())?
    } else {
      self.original
    };

    let document = kuchiki::parse_html().one(html);
    let target = document.select_first("head").unwrap_or_else(|_| {
      document
        .select_first("body")
        .expect("html must contain head or body")
    });

    let tauri_script = NodeRef::new_element(
      QualName::new(None, ns!(html), LocalName::from("script")),
      None,
    );
    tauri_script.append(NodeRef::new_text(include_str!("../templates/tauri.js")));
    target.as_node().prepend(tauri_script);

    let new_html = document.to_string();
    Ok(new_html)
  }
}
