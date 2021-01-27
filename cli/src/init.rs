#![allow(dead_code)]

use std::convert::TryFrom;
use std::path::PathBuf;

pub enum ForceType {
  All,
  Config,
  Template,
}

impl TryFrom<&str> for ForceType {
  type Error = anyhow::Error;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value.to_lowercase().as_str() {
      "all" => Ok(Self::All),
      "conf" => Ok(Self::Config),
      "template" => Ok(Self::Template),
      _ => Err(anyhow::anyhow!("Invalid `force` value.")),
    }
  }
}

pub struct Init {
  force: Option<ForceType>,
  directory: PathBuf,
  tauri_path: Option<PathBuf>,
  app_name: Option<String>,
  window_title: Option<String>,
  dist_dir: Option<String>,
  dev_path: Option<String>,
}

impl Default for Init {
  fn default() -> Self {
    Self {
      force: None,
      directory: std::env::current_dir().expect("failed to read cwd"),
      tauri_path: None,
      app_name: None,
      window_title: None,
      dist_dir: None,
      dev_path: None,
    }
  }
}

impl Init {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn force(mut self, force: ForceType) -> Self {
    self.force = Some(force);
    self
  }

  pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
    self.directory = directory.into();
    self
  }

  pub fn tauri_path(mut self, tauri_path: impl Into<PathBuf>) -> Self {
    self.tauri_path = Some(tauri_path.into());
    self
  }

  pub fn app_name(mut self, app_name: impl Into<String>) -> Self {
    self.app_name = Some(app_name.into());
    self
  }

  pub fn window_title(mut self, window_title: impl Into<String>) -> Self {
    self.window_title = Some(window_title.into());
    self
  }

  pub fn dist_dir(mut self, dist_dir: impl Into<String>) -> Self {
    self.dist_dir = Some(dist_dir.into());
    self
  }

  pub fn dev_path(mut self, dev_path: impl Into<String>) -> Self {
    self.dev_path = Some(dev_path.into());
    self
  }

  pub fn run(self) -> crate::Result<()> {
    unimplemented!()
  }
}
