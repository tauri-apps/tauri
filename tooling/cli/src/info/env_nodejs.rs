// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::VersionMetadata;
use super::{SectionItem, Status};
use colored::Colorize;

use crate::helpers::cross_command;

pub fn items(metadata: &VersionMetadata) -> (Vec<SectionItem>, Option<String>) {
  let yarn_version = cross_command("yarn")
    .arg("-v")
    .output()
    .map(|o| {
      if o.status.success() {
        let v = String::from_utf8_lossy(o.stdout.as_slice()).to_string();
        Some(v.split('\n').next().unwrap().to_string())
      } else {
        None
      }
    })
    .ok()
    .unwrap_or_default();
  let yarn_version_c = yarn_version.clone();
  let node_target_ver = metadata.js_cli.node.replace(">= ", "");

  (
    vec![
      SectionItem::new(
        move || {
          cross_command("node")
            .arg("-v")
            .output()
            .map(|o| {
              if o.status.success() {
                let v = String::from_utf8_lossy(o.stdout.as_slice()).to_string();
                let v = v
                  .split('\n')
                  .next()
                  .unwrap()
                  .strip_prefix('v')
                  .unwrap_or_default()
                  .trim();
                Some((
                  format!("node: {}{}", v, {
                    let version = semver::Version::parse(v).unwrap();
                    let target_version = semver::Version::parse(node_target_ver.as_str()).unwrap();
                    if version < target_version {
                      format!(
                        " ({}, latest: {})",
                        "outdated".red(),
                        target_version.to_string().green()
                      )
                    } else {
                      "".into()
                    }
                  }),
                  Status::Neutral,
                ))
              } else {
                None
              }
            })
            .ok()
            .unwrap_or_default()
        },
        || None,
        false,
      ),
      SectionItem::new(
        || {
          cross_command("pnpm")
            .arg("-v")
            .output()
            .map(|o| {
              if o.status.success() {
                let v = String::from_utf8_lossy(o.stdout.as_slice()).to_string();
                Some((
                  format!("pnpm: {}", v.split('\n').next().unwrap()),
                  Status::Neutral,
                ))
              } else {
                None
              }
            })
            .ok()
            .unwrap_or_default()
        },
        || None,
        false,
      ),
      SectionItem::new(
        move || {
          yarn_version_c
            .as_ref()
            .map(|v| (format!("yarn: {v}"), Status::Neutral))
        },
        || None,
        false,
      ),
      SectionItem::new(
        || {
          cross_command("npm")
            .arg("-v")
            .output()
            .map(|o| {
              if o.status.success() {
                let v = String::from_utf8_lossy(o.stdout.as_slice()).to_string();
                Some((
                  format!("npm: {}", v.split('\n').next().unwrap()),
                  Status::Neutral,
                ))
              } else {
                None
              }
            })
            .ok()
            .unwrap_or_default()
        },
        || None,
        false,
      ),
    ],
    yarn_version,
  )
}
