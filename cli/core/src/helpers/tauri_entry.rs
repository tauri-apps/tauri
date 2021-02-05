use kuchiki::traits::*;
use tauri_inliner::inline_html_string;

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

  pub fn get(self) -> crate::Result<String> {
    let html = if self.inliner_enabled {
      inline_html_string(&self.original, self.html_dir, Default::default())?
    } else {
      self.original
    };

    let new_html = kuchiki::parse_html().one(html).to_string();

    Ok(new_html)
  }
}

#[derive(Default)]
pub struct TauriScript {
  global_tauri: bool,
}

impl TauriScript {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn global_tauri(mut self, global_tauri: bool) -> Self {
    self.global_tauri = global_tauri;
    self
  }

  pub fn get(self) -> String {
    let mut scripts = Vec::new();
    scripts.push(include_str!("../templates/tauri.js"));
    scripts.push(include_str!("../templates/mutation-observer.js"));

    if self.global_tauri {
      scripts.insert(
        0,
        include_str!(concat!(env!("OUT_DIR"), "/tauri.bundle.umd.js")),
      );
    }

    scripts.join("\n\n")
  }
}
