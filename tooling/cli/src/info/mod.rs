// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::Parser;
use colored::{ColoredString, Colorize};
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde::Deserialize;
use std::{
  fmt::{self, Display, Formatter},
  panic,
  process::Command,
};

mod app;
mod env_nodejs;
mod env_rust;
mod env_system;
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
  serde_json::from_str::<VersionMetadata>(include_str!("../../metadata.json")).map_err(Into::into)
}

pub fn cross_command(bin: &str) -> Command {
  #[cfg(target_os = "windows")]
  let cmd = {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg(bin);
    cmd
  };
  #[cfg(not(target_os = "windows"))]
  let cmd = Command::new(bin);
  cmd
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
  action: Option<Box<dyn FnMut() -> ActionResult>>,
  action_if_err: Option<Box<dyn FnMut() -> ActionResult>>,
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
      action: None,
      action_if_err: None,
      description: None,
      status: Status::Neutral,
    }
  }

  fn action<F: FnMut() -> ActionResult + 'static>(mut self, action: F) -> Self {
    self.action = Some(Box::new(action));
    self
  }

  // fn action_if_err<F: FnMut() -> ActionResult + 'static>(mut self, action: F) -> Self {
  //   self.action_if_err = Some(Box::new(action));
  //   self
  // }

  fn description<S: AsRef<str>>(mut self, description: S) -> Self {
    self.description = Some(description.as_ref().to_string());
    self
  }

  fn run_action(&mut self) {
    let mut res = ActionResult::None;
    if let Some(action) = &mut self.action {
      res = action();
    }
    self.apply_action_result(res);
  }

  fn run_action_if_err(&mut self) {
    let mut res = ActionResult::None;
    if let Some(action) = &mut self.action_if_err {
      res = action();
    }
    self.apply_action_result(res);
  }

  fn apply_action_result(&mut self, result: ActionResult) {
    match result {
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
  }

  fn run(&mut self, interactive: bool) -> Status {
    self.run_action();

    if self.status == Status::Error && interactive && self.action_if_err.is_some() {
      if let Some(description) = &self.description {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt(format!(
            "{}\n  Run the automatic fix?",
            description.replace('\n', "\n  ")
          ))
          .interact()
          .unwrap_or(false);
        if confirmed {
          self.run_action_if_err()
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
  let items = env_nodejs::items(&metadata);
  environment.items.extend(items);

  let mut packages = Section {
    label: "Packages",
    interactive,
    items: Vec::new(),
  };
  packages
    .items
    .extend(packages_rust::items(app_dir, tauri_dir.clone()));
  packages
    .items
    .extend(packages_nodejs::items(app_dir, &metadata));

  let mut app = Section {
    label: "App",
    interactive,
    items: Vec::new(),
  };
  app.items.extend(app::items(app_dir, tauri_dir));

  environment.display();
  packages.display();
  app.display();
  Ok(())
}
