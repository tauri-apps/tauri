// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{utils::config::CliConfig, PackageInfo};
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The get CLI matches API.
  CliMatches,
}

impl Cmd {
  #[allow(unused_variables)]
  pub fn run(
    self,
    cli_config: &CliConfig,
    package_info: &PackageInfo,
  ) -> crate::Result<InvokeResponse> {
    match self {
      #[allow(unused_variables)]
      Self::CliMatches => {
        #[cfg(cli)]
        return crate::api::cli::get_matches(cli_config, package_info)
          .map_err(Into::into)
          .map(Into::into);
        #[cfg(not(cli))]
          Err(crate::Error::ApiNotEnabled(
            "CLI definition not set under tauri.conf.json > tauri > cli (https://tauri.studio/docs/api/config#tauri.cli)".to_string(),
          ))
      }
    }
  }
}
