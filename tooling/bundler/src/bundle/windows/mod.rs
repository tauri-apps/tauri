// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "windows")]
pub mod msi;
pub mod nsis;
#[cfg(target_os = "windows")]
pub mod sign;

#[cfg(target_os = "windows")]
pub mod azuresign;

mod util;
use log::info;
pub use util::{
  NSIS_OUTPUT_FOLDER_NAME, NSIS_UPDATER_OUTPUT_FOLDER_NAME, WIX_OUTPUT_FOLDER_NAME,
  WIX_UPDATER_OUTPUT_FOLDER_NAME,
};

use crate::Settings;

/// Attempts to sign the windows file via one of two methods, Signtool (OV Certs) or AzureSignTool (EV Certs)
/// If using AzureSignTool, the following environment variables will be used:
///
/// #### Required:
/// * AZURE_KEYVAULT_URI
/// * AZURE_CLIENT_ID
/// * AZURE_TENANT_ID
/// * AZURE_CLIENT_SECRET
/// * AZURE_CERTIFICATE_NAME
///
/// #### Optional:
/// * AZURE_DESCRIPTION_URL
/// * AZURE_TIMESTAMP_URL (must be an RFC 3161 timestamp server. If not speficied, this will default to the timestampUrl in the Tauri config)
///
pub fn try_sign(file_path: &std::path::PathBuf, settings: &Settings) -> crate::Result<()> {
  match azuresign::AzureSignToolArgs::generate(settings) {
    Some(args) => {
      info!("Attempting to sign with AzureSignTool");
      azuresign::sign(file_path, &args)?;
    }
    None => {
      sign::try_sign(file_path, settings)?;
    }
  }
  Ok(())
}
