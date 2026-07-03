// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::Context,
  helpers::app_paths::{resolve_frontend_dir, resolve_tauri_dir},
  Result,
};
use clap::Parser;
use colored::{ColoredString, Colorize};
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use tauri_utils::platform::Target;

mod app;
mod env_nodejs;
mod env_rust;
pub mod env_system;
#[cfg(target_os = "macos")]
mod ios;
mod packages_nodejs;
mod packages_rust;
pub mod plugins;

#[derive(Deserialize)]
struct JsCliVersionMetadata {
  version: String,
  node: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMetadata {
  #[serde(rename = "cli.js")]
  js_cli: JsCliVersionMetadata,
}

fn version_metadata() -> Result<VersionMetadata> {
  serde_json::from_str::<VersionMetadata>(include_str!("../../metadata-v2.json"))
    .context("failed to parse version metadata")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Status {
  Neutral = 0,
  #[default]
  Success,
  Warning,
  Error,
}

impl Status {
  fn color<S: AsRef<str>>(&self, s: S) -> ColoredString {
    let s = s.as_ref();
    match self {
      Status::Neutral => s.normal(),
      Status::Success => s.green(),
      Status::Warning => s.yellow(),
      Status::Error => s.red(),
    }
  }
}

impl Display for Status {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Status::Neutral => "-".cyan(),
        Status::Success => "✔".green(),
        Status::Warning => "⚠".yellow(),
        Status::Error => "✘".red(),
      }
    )
  }
}

#[derive(Default)]
pub enum ActionResult {
  Full {
    description: String,
    status: Status,
  },
  Description(String),
  #[default]
  None,
}

impl From<String> for ActionResult {
  fn from(value: String) -> Self {
    ActionResult::Description(value)
  }
}

impl From<(String, Status)> for ActionResult {
  fn from(value: (String, Status)) -> Self {
    ActionResult::Full {
      description: value.0,
      status: value.1,
    }
  }
}

impl From<Option<String>> for ActionResult {
  fn from(value: Option<String>) -> Self {
    value.map(ActionResult::Description).unwrap_or_default()
  }
}

impl From<Option<(String, Status)>> for ActionResult {
  fn from(value: Option<(String, Status)>) -> Self {
    value
      .map(|v| ActionResult::Full {
        description: v.0,
        status: v.1,
      })
      .unwrap_or_default()
  }
}

pub struct SectionItem {
  /// If description is none, the item is skipped
  description: Option<String>,
  status: Status,
}

impl Display for SectionItem {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let desc = self
      .description
      .as_ref()
      .map(|s| s.replace('\n', "\n      "))
      .unwrap_or_default();
    let (first, second) = desc.split_once(':').unwrap();
    write!(f, "{} {}:{}", self.status, first.bold(), second)
  }
}

impl SectionItem {
  fn new() -> Self {
    Self {
      description: None,
      status: Status::Neutral,
    }
  }

  /// Applies the result of a (now eagerly evaluated) action to this item.
  fn action_result<R: Into<ActionResult>>(mut self, result: R) -> Self {
    match result.into() {
      ActionResult::Full {
        description,
        status,
      } => {
        self.description = Some(description);
        self.status = status;
      }
      ActionResult::Description(description) => {
        self.description = Some(description);
      }
      ActionResult::None => {}
    }
    self
  }

  fn description<S: AsRef<str>>(mut self, description: S) -> Self {
    self.description = Some(description.as_ref().to_string());
    self
  }
}

struct Section<'a> {
  label: &'a str,
  items: Vec<SectionItem>,
}

impl Section<'_> {
  fn display(&mut self) {
    let mut status = Status::Neutral;

    for item in &self.items {
      if item.status > status {
        status = item.status;
      }
    }

    let status_str = format!("[{status}]");
    let status = status.color(status_str);

    println!();
    println!("{} {}", status, self.label.bold().yellow());
    for item in &self.items {
      if item.description.is_some() {
        println!("    {item}");
      }
    }
  }
}

#[derive(Debug, Parser)]
#[clap(
  about = "Show a concise list of information about the environment, Rust, Node.js and their versions as well as a few relevant project configurations"
)]
pub struct Options {
  /// Interactive mode to apply automatic fixes.
  #[clap(long)]
  pub interactive: bool,
}

pub async fn command(options: Options) -> Result<()> {
  let Options { interactive: _ } = options;

  let frontend_dir = resolve_frontend_dir();
  let tauri_dir = resolve_tauri_dir();

  let package_manager = match frontend_dir.as_ref() {
    Some(frontend_dir) => packages_nodejs::package_manager(frontend_dir).await,
    None => crate::helpers::npm::PackageManager::Npm,
  };

  let metadata = version_metadata()?;

  let mut environment = Section {
    label: "Environment",
    items: Vec::new(),
  };
  environment.items.extend(env_system::items().await);
  environment.items.extend(env_rust::items().await);
  let items = env_nodejs::items(&metadata).await;
  environment.items.extend(items);

  let mut packages = Section {
    label: "Packages",
    items: Vec::new(),
  };
  packages
    .items
    .extend(packages_rust::items(frontend_dir.as_ref(), tauri_dir.as_deref()).await);
  packages
    .items
    .extend(packages_nodejs::items(frontend_dir.as_ref(), package_manager, &metadata).await);

  let mut plugins = Section {
    label: "Plugins",
    items: plugins::items(frontend_dir.as_ref(), tauri_dir.as_deref(), package_manager).await,
  };

  let mut app = Section {
    label: "App",
    items: Vec::new(),
  };
  if let Some(tauri_dir) = &tauri_dir {
    if let Ok(config) = crate::helpers::config::get_config(Target::current(), &[], tauri_dir) {
      app
        .items
        .extend(app::items(&config, frontend_dir.as_ref()).await);
    };
  }

  environment.display();

  packages.display();

  plugins.display();

  if let (Some(frontend_dir), Some(tauri_dir)) = (&frontend_dir, &tauri_dir) {
    if let Err(error) = plugins::check_mismatched_packages(frontend_dir, tauri_dir).await {
      println!("\n{}: {error}", "Error".bright_red().bold());
    }
  }

  app.display();

  // iOS
  #[cfg(target_os = "macos")]
  {
    if let Some(p) = &tauri_dir {
      if p.join("gen/apple").exists() {
        let mut ios = Section {
          label: "iOS",
          items: Vec::new(),
        };
        ios.items.extend(ios::items().await);
        ios.display();
      }
    }
  }

  Ok(())
}
