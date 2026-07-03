// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::SectionItem;
use super::Status;
use colored::Colorize;
use tokio::process::Command;

async fn component_version(component: &str) -> Option<(String, Status)> {
  Command::new(component)
    .arg("-V")
    .output()
    .await
    .map(|o| String::from_utf8_lossy(o.stdout.as_slice()).to_string())
    .map(|v| {
      format!(
        "{component}: {}",
        v.split('\n')
          .next()
          .unwrap()
          .strip_prefix(&format!("{component} "))
          .unwrap_or_default()
      )
    })
    .map(|desc| (desc, Status::Success))
    .ok()
}

pub async fn items() -> Vec<SectionItem> {
  vec![
    SectionItem::new().action_result(component_version("rustc").await.unwrap_or_else(|| {
      (
        format!(
          "rustc: {}\nMaybe you don't have rust installed! Visit {}",
          "not installed!".red(),
          "https://rustup.rs/".cyan()
        ),
        Status::Error,
      )
    })),
    SectionItem::new().action_result(component_version("cargo").await.unwrap_or_else(|| {
      (
        format!(
          "Cargo: {}\nMaybe you don't have rust installed! Visit {}",
          "not installed!".red(),
          "https://rustup.rs/".cyan()
        ),
        Status::Error,
      )
    })),
    SectionItem::new().action_result(component_version("rustup").await.unwrap_or_else(|| {
      (
        format!(
          "rustup: {}\nIf you have rust installed some other way, we recommend uninstalling it\nthen use rustup instead. Visit {}",
          "not installed!".red(),
          "https://rustup.rs/".cyan()
        ),
        Status::Warning,
      )
    })),
    SectionItem::new().action_result(
      Command::new("rustup")
        .args(["show", "active-toolchain"])
        .output()
        .await
        .map(|o| String::from_utf8_lossy(o.stdout.as_slice()).to_string())
        .map(|v| {
          format!(
            "Rust toolchain: {}",
            v.split('\n')
              .next()
              .unwrap()
          )
        })
        .map(|desc| (desc, Status::Success))
        .ok()
        .unwrap_or_else(|| {
          (
            format!(
              "Rust toolchain: couldn't be detected!\nMaybe you don't have rustup installed? if so, Visit {}", "https://rustup.rs/".cyan()
            ),
            Status::Warning,
          )
        }),
    ),
  ]
}
