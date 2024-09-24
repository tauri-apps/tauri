// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs::{create_dir_all, File},
  io::{Cursor, Read, Write},
  path::{Path, PathBuf},
};

use regex::Regex;
use sha2::Digest;
use url::Url;
use zip::ZipArchive;

pub const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
pub const WEBVIEW2_OFFLINE_INSTALLER_X86_URL: &str =
  "https://go.microsoft.com/fwlink/?linkid=2099617";
pub const WEBVIEW2_OFFLINE_INSTALLER_X64_URL: &str =
  "https://go.microsoft.com/fwlink/?linkid=2124701";
pub const WEBVIEW2_URL_PREFIX: &str =
  "https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/";
pub const NSIS_OUTPUT_FOLDER_NAME: &str = "nsis";
pub const NSIS_UPDATER_OUTPUT_FOLDER_NAME: &str = "nsis-updater";
pub const WIX_OUTPUT_FOLDER_NAME: &str = "msi";
pub const WIX_UPDATER_OUTPUT_FOLDER_NAME: &str = "msi-updater";

pub fn webview2_guid_path(url: &str) -> crate::Result<(String, String)> {
  let agent = ureq::AgentBuilder::new().try_proxy_from_env(true).build();
  let response = agent.head(url).call().map_err(Box::new)?;
  let final_url = response.get_url();
  let remaining_url = final_url.strip_prefix(WEBVIEW2_URL_PREFIX).ok_or_else(|| {
    anyhow::anyhow!(
      "WebView2 URL prefix mismatch. Expected `{}`, found `{}`.",
      WEBVIEW2_URL_PREFIX,
      final_url
    )
  })?;
  let (guid, filename) = remaining_url.split_once('/').ok_or_else(|| {
    anyhow::anyhow!(
      "WebView2 URL format mismatch. Expected `<GUID>/<FILENAME>`, found `{}`.",
      remaining_url
    )
  })?;
  Ok((guid.into(), filename.into()))
}

pub fn download_webview2_bootstrapper(base_path: &Path) -> crate::Result<PathBuf> {
  let file_path = base_path.join("MicrosoftEdgeWebview2Setup.exe");
  if !file_path.exists() {
    std::fs::write(&file_path, download(WEBVIEW2_BOOTSTRAPPER_URL)?)?;
  }
  Ok(file_path)
}

pub fn download_webview2_offline_installer(base_path: &Path, arch: &str) -> crate::Result<PathBuf> {
  let url = if arch == "x64" {
    WEBVIEW2_OFFLINE_INSTALLER_X64_URL
  } else {
    WEBVIEW2_OFFLINE_INSTALLER_X86_URL
  };
  let (guid, filename) = webview2_guid_path(url)?;
  let dir_path = base_path.join(guid);
  let file_path = dir_path.join(filename);
  if !file_path.exists() {
    create_dir_all(dir_path)?;
    std::fs::write(&file_path, download(url)?)?;
  }
  Ok(file_path)
}

fn generate_github_mirror_url_from_template(github_url: &str) -> Option<String> {
  std::env::var("TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE")
    .ok()
    .and_then(|template| {
      let re =
        Regex::new(r"https://github.com/([^/]+)/([^/]+)/releases/download/([^/]+)/(.*)").unwrap();
      re.captures(github_url).map(|caps| {
        template
          .replace("<owner>", &caps[1])
          .replace("<repo>", &caps[2])
          .replace("<version>", &caps[3])
          .replace("<asset>", &caps[4])
      })
    })
}

fn generate_github_mirror_url_from_base(github_url: &str) -> Option<String> {
  std::env::var("TAURI_BUNDLER_TOOLS_GITHUB_MIRROR")
    .ok()
    .and_then(|cdn| Url::parse(&cdn).ok())
    .map(|mut cdn| {
      cdn.set_path(github_url);
      cdn.to_string()
    })
}

fn generate_github_alternative_url(url: &str) -> Option<(ureq::Agent, String)> {
  if !url.starts_with("https://github.com/") {
    return None;
  }

  generate_github_mirror_url_from_template(url)
    .or_else(|| generate_github_mirror_url_from_base(url))
    .map(|alt_url| (ureq::AgentBuilder::new().build(), alt_url))
}

fn create_agent_and_url(url: &str) -> (ureq::Agent, String) {
  generate_github_alternative_url(url).unwrap_or((
    ureq::AgentBuilder::new().try_proxy_from_env(true).build(),
    url.to_owned(),
  ))
}

pub fn download(url: &str) -> crate::Result<Vec<u8>> {
  let (agent, final_url) = create_agent_and_url(url);

  log::info!(action = "Downloading"; "{}", final_url);

  let response = agent.get(&final_url).call().map_err(Box::new)?;
  let mut bytes = Vec::new();
  response.into_reader().read_to_end(&mut bytes)?;
  Ok(bytes)
}

#[derive(Clone, Copy)]
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
  log::info!("validating hash");
  verify_hash(&data, hash, hash_algorithm)?;
  Ok(data)
}

pub fn verify_hash(data: &[u8], hash: &str, hash_algorithm: HashAlgorithm) -> crate::Result<()> {
  match hash_algorithm {
    #[cfg(target_os = "windows")]
    HashAlgorithm::Sha256 => {
      let hasher = sha2::Sha256::new();
      verify_data_with_hasher(data, hash, hasher)
    }
    HashAlgorithm::Sha1 => {
      let hasher = sha1::Sha1::new();
      verify_data_with_hasher(data, hash, hasher)
    }
  }
}

fn verify_data_with_hasher(data: &[u8], hash: &str, mut hasher: impl Digest) -> crate::Result<()> {
  hasher.update(data);

  let url_hash = hasher.finalize().to_vec();
  let expected_hash = hex::decode(hash)?;
  if expected_hash == url_hash {
    Ok(())
  } else {
    Err(crate::Error::HashError)
  }
}

pub fn verify_file_hash<P: AsRef<Path>>(
  path: P,
  hash: &str,
  hash_algorithm: HashAlgorithm,
) -> crate::Result<()> {
  let data = std::fs::read(path)?;
  verify_hash(&data, hash, hash_algorithm)
}

/// Extracts the zips from memory into a usable path.
#[allow(dead_code)]
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
  use windows_sys::Win32::System::SystemInformation::{
    GetNativeSystemInfo, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
  };

  let mut system_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
  unsafe { GetNativeSystemInfo(&mut system_info) };
  match unsafe { system_info.Anonymous.Anonymous.wProcessorArchitecture } {
    PROCESSOR_ARCHITECTURE_INTEL => Some("x86"),
    PROCESSOR_ARCHITECTURE_AMD64 => Some("x64"),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::generate_github_mirror_url_from_template;
  use std::env;

  const GITHUB_ASSET_URL: &str =
    "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip";
  const NON_GITHUB_ASSET_URL: &str = "https://someotherwebsite.com/somefile.zip";

  #[test]
  fn test_generate_mirror_url_no_env_var() {
    env::remove_var("TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE");

    assert!(generate_github_mirror_url_from_template(GITHUB_ASSET_URL).is_none());
  }

  #[test]
  fn test_generate_mirror_url_non_github_url() {
    env::set_var(
      "TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE",
      "https://mirror.example.com/<owner>/<repo>/releases/download/<version>/<asset>",
    );

    assert!(generate_github_mirror_url_from_template(NON_GITHUB_ASSET_URL).is_none());
  }

  struct TestCase {
    template: &'static str,
    expected_url: &'static str,
  }

  #[test]
  fn test_generate_mirror_url_correctly() {
    let test_cases = vec![
            TestCase {
                template: "https://mirror.example.com/<owner>/<repo>/releases/download/<version>/<asset>",
                expected_url: "https://mirror.example.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip",
            },
            TestCase {
                template: "https://mirror.example.com/<asset>",
                expected_url: "https://mirror.example.com/wix311-binaries.zip",
            },
        ];

    for case in test_cases {
      env::set_var("TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE", case.template);
      assert_eq!(
        generate_github_mirror_url_from_template(GITHUB_ASSET_URL),
        Some(case.expected_url.to_string())
      );
    }
  }
}
