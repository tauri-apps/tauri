use handlebars::Handlebars;
use html5ever::{interface::QualName, namespace_url, ns, LocalName};
use kuchiki::{traits::*, NodeRef};
use once_cell::sync::Lazy;
use tauri_inliner::inline_html_string;

use std::collections::BTreeMap;
use std::path::PathBuf;

fn handlebars() -> &'static Handlebars<'static> {
  static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string(
        "mutation-observer.js",
        include_str!("../templates/mutation-observer.js"),
      )
      .map_err(|e| e.to_string())
      .expect("Failed to setup handlebar template");
    handlebars
  });
  &HANDLEBARS
}

pub struct TauriHtml {
  original: String,
  html_dir: PathBuf,
  inliner_enabled: bool,
  global_tauri: bool,
}

impl TauriHtml {
  pub fn new(html_dir: impl Into<PathBuf>, html: String) -> Self {
    Self {
      original: html,
      html_dir: html_dir.into(),
      inliner_enabled: false,
      global_tauri: false,
    }
  }

  pub fn inliner_enabled(mut self, enabled: bool) -> Self {
    self.inliner_enabled = enabled;
    self
  }

  pub fn global_tauri(mut self, global_tauri: bool) -> Self {
    self.global_tauri = global_tauri;
    self
  }

  pub fn generate(self) -> crate::Result<String> {
    let html = if self.inliner_enabled {
      inline_html_string(&self.original, self.html_dir, Default::default())?
    } else {
      self.original
    };

    let document = kuchiki::parse_html().one(html);
    let body = document.select_first("body");
    let head = document.select_first("head");

    let handlebars = handlebars();

    if let Ok(ref head) = head {
      let mut data = BTreeMap::new();
      data.insert("target".to_string(), "head".to_string());
      let head_mutation_observer_script =
        create_script_element(&handlebars.render("mutation-observer.js", &data)?);
      head.as_node().prepend(head_mutation_observer_script);
    }

    if let Ok(ref body) = body {
      let mut data = BTreeMap::new();
      data.insert("target".to_string(), "body".to_string());
      let body_mutation_observer_script =
        create_script_element(&handlebars.render("mutation-observer.js", &data)?);
      body.as_node().prepend(body_mutation_observer_script);
    }

    if let Ok(target) = if head.is_ok() { head } else { body } {
      let tauri_script = create_script_element(include_str!("../templates/tauri.js"));
      target.as_node().prepend(tauri_script);

      if self.global_tauri {
        let global_api_script = create_script_element(include_str!(
          "../../api-definitions/dist/tauri.bundle.umd.js"
        ));
        target.as_node().prepend(global_api_script);
      }
    }

    let new_html = document.to_string();
    Ok(new_html)
  }
}

fn create_script_element(content: &str) -> NodeRef {
  let script = NodeRef::new_element(
    QualName::new(None, ns!(html), LocalName::from("script")),
    None,
  );
  script.append(NodeRef::new_text(content));
  script
}
