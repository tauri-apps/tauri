// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::Parser;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde::Deserialize;
use std::{
  fmt::{self, Display, Formatter},
  panic,
};

mod app;
mod env_nodejs;
mod env_rust;
mod env_system;
#[cfg(target_os = "macos")]
mod ios;
mod packages_nodejs;
mod packages_rust;

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
    .map_err(Into::into)
}

#[cfg(not(debug_assertions))]
pub(crate) fn cli_current_version() -> Result<String> {
  version_metadata().map(|meta| meta.js_cli.version)
}

#[cfg(not(debug_assertions))]
pub(crate) fn cli_upstream_version() -> Result<String> {
  let upstream_metadata = match ureq::get(
    "https://raw.githubusercontent.com/tauri-apps/tauri/dev/tooling/cli/metadata-v2.json",
  )
  .timeout(std::time::Duration::from_secs(3))
  .call()
  {
    Ok(r) => r,
    Err(ureq::Error::Status(code, _response)) => {
      let message = format!("Unable to find updates at the moment. Code: {}", code);
      return Err(anyhow::Error::msg(message));
    }
    Err(ureq::Error::Transport(transport)) => {
      let message = format!(
        "Unable to find updates at the moment. Error: {:?}",
        transport.kind()
      );
      return Err(anyhow::Error::msg(message));
    }
  };

  upstream_metadata
    .into_string()
    .and_then(|meta_str| Ok(serde_json::from_str::<VersionMetadata>(&meta_str)))
    .and_then(|json| Ok(json.unwrap().js_cli.version))
    .map_err(|e| anyhow::Error::new(e))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Status {
  Neutral = 0,
  #[default]
  Success,
  Warning,
  Error,
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

pub struct SectionItem {
  /// If description is none, the item is skipped
  description: Option<String>,
  status: Status,
  /// This closure return will be assigned to status and description
  action: Box<dyn FnMut() -> Option<(String, Status)>>,
  /// This closure return will be assigned to status and description
  action_if_err: Box<dyn FnMut() -> Option<(String, Status)>>,
  has_action_if_err: bool,
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
  fn new<
    F1: FnMut() -> Option<(String, Status)> + 'static,
    F2: FnMut() -> Option<(String, Status)> + 'static,
  >(
    action: F1,
    action_if_err: F2,
    has_action_if_err: bool,
  ) -> Self {
    Self {
      action: Box::new(action),
      action_if_err: Box::new(action_if_err),
      has_action_if_err,
      description: None,
      status: Status::Neutral,
    }
  }
  fn run(&mut self, interactive: bool) -> Status {
    if let Some(ret) = (self.action)() {
      self.description = Some(ret.0);
      self.status = ret.1;
    }

    if self.status == Status::Error && interactive && self.has_action_if_err {
      if let Some(description) = &self.description {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt(format!(
            "{}\n  Run the automatic fix?",
            description.replace('\n', "\n  ")
          ))
          .interact()
          .unwrap_or(false);
        if confirmed {
          if let Some(ret) = (self.action_if_err)() {
            self.description = Some(ret.0);
            self.status = ret.1;
          }
        }
      }
    }
    self.status
  }
}

struct Section<'a> {
  label: &'a str,
  interactive: bool,
  items: Vec<SectionItem>,
}

impl Section<'_> {
  fn display(&mut self) {
    let mut status = Status::Neutral;

    for item in &mut self.items {
      let s = item.run(self.interactive);
      if s > status {
        status = s;
      }
    }

    let status_str = format!("[{status}]");
    let status = match status {
      Status::Neutral => status_str.normal(),
      Status::Success => status_str.green(),
      Status::Warning => status_str.yellow(),
      Status::Error => status_str.red(),
    };

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
#[clap(about = "Shows information about Tauri dependencies and project configuration")]
pub struct Options {
  /// Interactive mode to apply automatic fixes.
  #[clap(long)]
  pub interactive: bool,
}

pub fn command(options: Options) -> Result<()> {
  let Options { interactive } = options;
  let hook = panic::take_hook();
  panic::set_hook(Box::new(|_info| {
    // do nothing
  }));
  let app_dir = panic::catch_unwind(crate::helpers::app_paths::app_dir)
    .map(Some)
    .unwrap_or_default();
  let tauri_dir = panic::catch_unwind(crate::helpers::app_paths::tauri_dir)
    .map(Some)
    .unwrap_or_default();
  panic::set_hook(hook);
  let metadata = version_metadata()?;

  let mut environment = Section {
    label: "Environment",
    interactive,
    items: Vec::new(),
  };
  environment.items.extend(env_system::items());
  environment.items.extend(env_rust::items());
  let (items, yarn_version) = env_nodejs::items(&metadata);
  environment.items.extend(items);

  let mut packages = Section {
    label: "Packages",
    interactive,
    items: Vec::new(),
  };
  packages
    .items
    .extend(packages_rust::items(app_dir, tauri_dir.as_deref()));
  packages
    .items
    .extend(packages_nodejs::items(app_dir, &metadata, yarn_version));

  let mut app = Section {
    label: "App",
    interactive,
    items: Vec::new(),
  };
  app.items.extend(app::items(app_dir, tauri_dir.as_deref()));

  environment.display();
  packages.display();
  app.display();

  // iOS
  #[cfg(target_os = "macos")]
  {
    if let Some(p) = &tauri_dir {
      if p.join("gen/apple").exists() {
        let mut ios = Section {
          label: "iOS",
          interactive,
          items: Vec::new(),
        };
        ios.items.extend(ios::items());
        ios.display();
      }
    }
  }

  Ok(())
}
