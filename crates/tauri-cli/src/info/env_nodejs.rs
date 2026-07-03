// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ActionResult, SectionItem, VersionMetadata};
use colored::Colorize;

use crate::helpers::{cross_command, npm::manager_version};

async fn manager_version_item(package_manager: &str) -> SectionItem {
  SectionItem::new().action_result(
    manager_version(package_manager)
      .await
      .map(|v| format!("{package_manager}: {v}")),
  )
}

pub async fn items(metadata: &VersionMetadata) -> Vec<SectionItem> {
  let node_target_ver = metadata.js_cli.node.replace(">= ", "");

  let node_item = SectionItem::new().action_result(
    cross_command("node")
      .arg("-v")
      .output()
      .await
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
          ActionResult::Description(format!("node: {}{}", v, {
            let version = semver::Version::parse(v);
            let target_version = semver::Version::parse(node_target_ver.as_str());
            match (version, target_version) {
              (Ok(version), Ok(target_version)) if version < target_version => {
                format!(
                  " ({}, latest: {})",
                  "outdated".red(),
                  target_version.to_string().green()
                )
              }
              _ => "".into(),
            }
          }))
        } else {
          ActionResult::None
        }
      })
      .ok()
      .unwrap_or_default(),
  );

  vec![
    node_item,
    manager_version_item("pnpm").await,
    manager_version_item("yarn").await,
    manager_version_item("npm").await,
    manager_version_item("bun").await,
    manager_version_item("deno").await,
  ]
}
