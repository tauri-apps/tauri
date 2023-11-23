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

pub const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
pub const WEBVIEW2_X86_OFFLINE_INSTALLER_GUID: &str = "102e2c62-f0d7-4f1c-bb31-c7f7f80d95a9";
pub const WEBVIEW2_X64_OFFLINE_INSTALLER_GUID: &str = "e95be380-76c6-4632-b1b2-2a5d424b61aa";
pub const NSIS_OUTPUT_FOLDER_NAME: &str = "nsis";
pub const NSIS_UPDATER_OUTPUT_FOLDER_NAME: &str = "nsis-updater";
pub const WIX_OUTPUT_FOLDER_NAME: &str = "msi";
pub const WIX_UPDATER_OUTPUT_FOLDER_NAME: &str = "msi-updater";

pub fn download(url: &str) -> crate::Result<Vec<u8>> {
  info!(action = "Downloading"; "{}", url);

  let agent = ureq::AgentBuilder::new().try_proxy_from_env(true).build();
  let response = agent.get(url).call().map_err(Box::new)?;
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

#[cfg(target_os = "windows")]
pub fn os_bitness<'a>() -> Option<&'a str> {
  use windows_sys::Win32::System::{
    Diagnostics::Debug::{PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_INTEL},
    SystemInformation::{GetNativeSystemInfo, SYSTEM_INFO},
  };

  let mut system_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
  unsafe { GetNativeSystemInfo(&mut system_info) };
  match unsafe { system_info.Anonymous.Anonymous.wProcessorArchitecture } {
    PROCESSOR_ARCHITECTURE_INTEL => Some("x86"),
    PROCESSOR_ARCHITECTURE_AMD64 => Some("x64"),
    _ => None,
  }
}
