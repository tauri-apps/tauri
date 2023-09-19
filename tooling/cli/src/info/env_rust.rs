// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::SectionItem;
use super::Status;
use colored::Colorize;
use std::process::Command;

pub fn items() -> Vec<SectionItem> {
  vec![
    SectionItem::new(
      || {
        Some(
          Command::new("rustc")
            .arg("-V")
            .output()
            .map(|o| String::from_utf8_lossy(o.stdout.as_slice()).to_string())
            .map(|v| {
              format!(
                "rustc: {}",
                v.split('\n')
                  .next()
                  .unwrap()
                  .strip_prefix("rustc ")
                  .unwrap_or_default()
              )
            })
            .map(|desc| (desc, Status::Success))
            .ok()
            .unwrap_or_else(|| {
              (
                format!(
                  "rustc: {}\nMaybe you don't have rust installed! Visit {}",
                  "not installed!".red(),
                  "https://rustup.rs/".cyan()
                ),
                Status::Error,
              )
            }),
        )
      },
      || None,
      false,
    ),
    SectionItem::new(
      || {
        Some(
          Command::new("cargo")
            .arg("-V")
            .output()
            .map(|o| String::from_utf8_lossy(o.stdout.as_slice()).to_string())
            .map(|v| {
              format!(
                "Cargo: {}",
                v.split('\n')
                  .next()
                  .unwrap()
                  .strip_prefix("cargo ")
                  .unwrap_or_default()
              )
            })
            .map(|desc| (desc, Status::Success))
            .ok()
            .unwrap_or_else(|| {
              (
                format!(
                  "Cargo: {}\nMaybe you don't have rust installed! Visit {}",
                  "not installed!".red(),
                  "https://rustup.rs/".cyan()
                ),
                Status::Error,
              )
            }),
        )
      },
      || None,
      false,
    ),
    SectionItem::new(
      || {
        Some(
          Command::new("rustup")
            .arg("-V")
            .output()
            .map(|o| String::from_utf8_lossy(o.stdout.as_slice()).to_string())
            .map(|v| {
              format!(
                "rustup: {}",
                v.split('\n')
                  .next()
                  .unwrap()
                  .strip_prefix("rustup ")
                  .unwrap_or_default()
              )
            })
            .map(|desc| (desc, Status::Success))
            .ok()
            .unwrap_or_else(|| {
              (
                format!(
                  "rustup: {}\nIf you have rust installed some other way, we recommend uninstalling it\nthen use rustup instead. Visit {}",
                  "not installed!".red(),
                  "https://rustup.rs/".cyan()
                ),
                Status::Warning,
              )
            }),
        )
      },
      || None,
      false,
    ),
    SectionItem::new(
      || {
        Some(
          Command::new("rustup")
            .args(["show", "active-toolchain"])
            .output()
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
        )
      },
      || None,
      false,
    ),
  ]
}
