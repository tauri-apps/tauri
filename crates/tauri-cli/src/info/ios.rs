// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::mobile::ios::get_xcode_version;

use super::SectionItem;

use colored::Colorize;

pub fn items() -> Vec<SectionItem> {
  vec![
    SectionItem::new().action(|| {
      let teams = cargo_mobile2::apple::teams::find_development_teams().unwrap_or_default();

      if teams.is_empty() {
        "Developer Teams: None".red().to_string().into()
      } else {
        format!(
          "Developer Teams: {}",
          teams
            .iter()
            .map(|t| format!("{} (ID: {})", t.name, t.id))
            .collect::<Vec<String>>()
            .join(", ")
        )
        .into()
      }
    }),
    SectionItem::new().action(|| {
      let xcode_version = get_xcode_version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
      format!("Xcode Version: {}", xcode_version).into()
    }),
  ]
}
