// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs::{create_dir_all, File},
  io::{Cursor, Read, Write},
  path::Path,
};

use log::info;
use sha2::Digest;
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use crate::bundle::windows::sign::{sign, SignParams};
#[cfg(target_os = "windows")]
use crate::Settings;

pub const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
pub const WEBVIEW2_X86_INSTALLER_GUID: &str = "a17bde80-b5ab-47b5-8bbb-1cbe93fc6ec9";
pub const WEBVIEW2_X64_INSTALLER_GUID: &str = "aa5fd9b3-dc11-4cbc-8343-a50f57b311e1";
pub const NSIS_OUTPUT_FOLDER_NAME: &str = "nsis";
pub const NSIS_UPDATER_OUTPUT_FOLDER_NAME: &str = "nsis-updater";
pub const WIX_OUTPUT_FOLDER_NAME: &str = "msi";
pub const WIX_UPDATER_OUTPUT_FOLDER_NAME: &str = "msi-updater";

pub fn download(url: &str) -> crate::Result<Vec<u8>> {
  info!(action = "Downloading"; "{}", url);
  let response = ureq::get(url).call().map_err(Box::new)?;
  let mut bytes = Vec::new();
  response.into_reader().read_to_end(&mut bytes)?;
  Ok(bytes)
}

pub enum HashAlgorithm {
  #[cfg(target_os = "windows")]
  Sha256,
  Sha1,
}

/// Function used to download a file and checks SHA256 to verify the download.
pub fn download_and_verify(
  url: &str,
  hash: &str,
  hash_algorithm: HashAlgorithm,
) -> crate::Result<Vec<u8>> {
  let data = download(url)?;
  info!("validating hash");

  match hash_algorithm {
    #[cfg(target_os = "windows")]
    HashAlgorithm::Sha256 => {
      let hasher = sha2::Sha256::new();
      verify(&data, hash, hasher)?;
    }
    HashAlgorithm::Sha1 => {
      let hasher = sha1::Sha1::new();
      verify(&data, hash, hasher)?;
    }
  }

  Ok(data)
}

fn verify(data: &Vec<u8>, hash: &str, mut hasher: impl Digest) -> crate::Result<()> {
  hasher.update(data);

  let url_hash = hasher.finalize().to_vec();
  let expected_hash = hex::decode(hash)?;
  if expected_hash == url_hash {
    Ok(())
  } else {
    Err(crate::Error::HashError)
  }
}

#[cfg(target_os = "windows")]
pub fn try_sign(file_path: &std::path::PathBuf, settings: &Settings) -> crate::Result<()> {
  use tauri_utils::display_path;

  if let Some(certificate_thumbprint) = settings.windows().certificate_thumbprint.as_ref() {
    info!(action = "Signing"; "{}", display_path(file_path));
    sign(
      file_path,
      &SignParams {
        product_name: settings.product_name().into(),
        digest_algorithm: settings
          .windows()
          .digest_algorithm
          .as_ref()
          .map(|algorithm| algorithm.to_string())
          .unwrap_or_else(|| "sha256".to_string()),
        certificate_thumbprint: certificate_thumbprint.to_string(),
        timestamp_url: settings
          .windows()
          .timestamp_url
          .as_ref()
          .map(|url| url.to_string()),
        tsp: settings.windows().tsp,
      },
    )?;
  }
  Ok(())
}

/// Extracts the zips from memory into a useable path.
pub fn extract_zip(data: &[u8], path: &Path) -> crate::Result<()> {
  let cursor = Cursor::new(data);

  let mut zipa = ZipArchive::new(cursor)?;

  for i in 0..zipa.len() {
    let mut file = zipa.by_index(i)?;

    if let Some(name) = file.enclosed_name() {
      let dest_path = path.join(name);
      if file.is_dir() {
        create_dir_all(&dest_path)?;
        continue;
      }

      let parent = dest_path.parent().expect("Failed to get parent");

      if !parent.exists() {
        create_dir_all(parent)?;
      }

      let mut buff: Vec<u8> = Vec::new();
      file.read_to_end(&mut buff)?;
      let mut fileout = File::create(dest_path).expect("Failed to open file");

      fileout.write_all(&buff)?;
    }
  }

  Ok(())
}
