// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::bundle::settings::CustomSignCommandSettings;
#[cfg(windows)]
use crate::bundle::windows::util;
use crate::{utils::CommandExt, Settings};
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::OnceLock;
use std::{path::Path, process::Command};

impl Settings {
  pub(crate) fn can_sign(&self) -> bool {
    self.windows().sign_command.is_some() || self.windows().certificate_thumbprint.is_some()
  }

  pub(crate) fn sign_params(&self) -> SignParams {
    SignParams {
      product_name: self.product_name().into(),
      digest_algorithm: self
        .windows()
        .digest_algorithm
        .as_ref()
        .map(|algorithm| algorithm.to_string())
        .unwrap_or_else(|| "sha256".to_string()),
      certificate_thumbprint: self
        .windows()
        .certificate_thumbprint
        .clone()
        .unwrap_or_default(),
      timestamp_url: self
        .windows()
        .timestamp_url
        .as_ref()
        .map(|url| url.to_string()),
      tsp: self.windows().tsp,
      sign_command: self.windows().sign_command.clone(),
    }
  }
}

#[cfg_attr(not(windows), allow(dead_code))]
pub struct SignParams {
  pub product_name: String,
  pub digest_algorithm: String,
  pub certificate_thumbprint: String,
  pub timestamp_url: Option<String>,
  pub tsp: bool,
  pub sign_command: Option<CustomSignCommandSettings>,
}

#[cfg(windows)]
fn signtool() -> Option<PathBuf> {
  // sign code forked from https://github.com/forbjok/rust-codesign
  static SIGN_TOOL: OnceLock<crate::Result<PathBuf>> = OnceLock::new();
  SIGN_TOOL
    .get_or_init(|| {
      if let Some(signtool) = std::env::var_os("TAURI_WINDOWS_SIGNTOOL_PATH") {
        return Ok(PathBuf::from(signtool));
      }

      const INSTALLED_ROOTS_REGKEY_PATH: &str = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
      const KITS_ROOT_REGVALUE_NAME: &str = r"KitsRoot10";

      // Open 32-bit HKLM "Installed Roots" key
      let installed_roots_key = windows_registry::LOCAL_MACHINE
        .open(INSTALLED_ROOTS_REGKEY_PATH)
        .map_err(|_| crate::Error::OpenRegistry(INSTALLED_ROOTS_REGKEY_PATH.to_string()))?;

      // Get the Windows SDK root path
      let kits_root_10_path: String = installed_roots_key
        .get_string(KITS_ROOT_REGVALUE_NAME)
        .map_err(|_| crate::Error::GetRegistryValue(KITS_ROOT_REGVALUE_NAME.to_string()))?;

      // Construct Windows SDK bin path
      let kits_root_10_bin_path = Path::new(&kits_root_10_path).join("bin");

      let mut installed_kits: Vec<String> = installed_roots_key
        .keys()
        .map_err(|_| crate::Error::FailedToEnumerateRegKeys)?
        .collect();

      // Sort installed kits
      installed_kits.sort();

      /* Iterate through installed kit version keys in reverse (from newest to oldest),
      adding their bin paths to the list.
      Windows SDK 10 v10.0.15063.468 and later will have their signtools located there. */
      let mut kit_bin_paths: Vec<PathBuf> = installed_kits
        .iter()
        .rev()
        .map(|kit| kits_root_10_bin_path.join(kit))
        .collect();

      /* Add kits root bin path.
      For Windows SDK 10 versions earlier than v10.0.15063.468, signtool will be located there. */
      kit_bin_paths.push(kits_root_10_bin_path);

      // Choose which version of SignTool to use based on OS bitness
      let arch_dir = util::os_bitness().ok_or(crate::Error::UnsupportedBitness)?;

      /* Iterate through all bin paths, checking for existence of a SignTool executable. */
      for kit_bin_path in &kit_bin_paths {
        /* Construct SignTool path. */
        let signtool_path = kit_bin_path.join(arch_dir).join("signtool.exe");

        /* Check if SignTool exists at this location. */
        if signtool_path.exists() {
          // SignTool found. Return it.
          return Ok(signtool_path);
        }
      }

      Err(crate::Error::SignToolNotFound)
    })
    .as_ref()
    .ok()
    .cloned()
}

/// Check if binary is already signed.
/// Used to skip sidecar binaries that are already signed.
#[cfg(windows)]
pub fn verify(path: &Path) -> crate::Result<bool> {
  let signtool = signtool().ok_or(crate::Error::SignToolNotFound)?;

  let mut cmd = Command::new(signtool);
  cmd.arg("verify");
  cmd.arg("/pa");
  cmd.arg(path);

  Ok(cmd.status()?.success())
}

pub fn sign_command_custom<P: AsRef<Path>>(
  path: P,
  command: &CustomSignCommandSettings,
) -> crate::Result<Command> {
  let path = path.as_ref();

  let mut cmd = Command::new(&command.cmd);
  for arg in &command.args {
    if arg == "%1" {
      cmd.arg(path);
    } else {
      cmd.arg(arg);
    }
  }
  Ok(cmd)
}

#[cfg(windows)]
pub fn sign_command_default<P: AsRef<Path>>(
  path: P,
  params: &SignParams,
) -> crate::Result<Command> {
  let signtool = signtool().ok_or(crate::Error::SignToolNotFound)?;

  let mut cmd = Command::new(signtool);
  cmd.arg("sign");
  cmd.args(["/fd", &params.digest_algorithm]);
  cmd.args(["/sha1", &params.certificate_thumbprint]);
  cmd.args(["/d", &params.product_name]);

  if let Some(ref timestamp_url) = params.timestamp_url {
    if params.tsp {
      cmd.args(["/tr", timestamp_url]);
      cmd.args(["/td", &params.digest_algorithm]);
    } else {
      cmd.args(["/t", timestamp_url]);
    }
  }

  cmd.arg(path.as_ref());

  Ok(cmd)
}

pub fn sign_command<P: AsRef<Path>>(path: P, params: &SignParams) -> crate::Result<Command> {
  match &params.sign_command {
    Some(custom_command) => sign_command_custom(path, custom_command),
    #[cfg(windows)]
    None => sign_command_default(path, params),

    // should not be reachable
    #[cfg(not(windows))]
    None => Ok(Command::new("")),
  }
}

pub fn sign_custom<P: AsRef<Path>>(
  path: P,
  custom_command: &CustomSignCommandSettings,
) -> crate::Result<()> {
  let path = path.as_ref();

  log::info!(action = "Signing";"{} with a custom signing command", tauri_utils::display_path(path));

  let mut cmd = sign_command_custom(path, custom_command)?;

  let output = cmd.output_ok()?;

  let stdout = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
  log::info!("{:?}", stdout);

  Ok(())
}

#[cfg(windows)]
pub fn sign_default<P: AsRef<Path>>(path: P, params: &SignParams) -> crate::Result<()> {
  let signtool = signtool().ok_or(crate::Error::SignToolNotFound)?;
  let path = path.as_ref();

  log::info!(action = "Signing"; "{} with identity \"{}\"", tauri_utils::display_path(path), params.certificate_thumbprint);

  let mut cmd = sign_command_default(path, params)?;
  log::debug!("Running signtool {:?}", signtool);

  // Execute SignTool command
  let output = cmd.output_ok()?;

  let stdout = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
  log::info!("{:?}", stdout);

  Ok(())
}

pub fn sign<P: AsRef<Path>>(path: P, params: &SignParams) -> crate::Result<()> {
  match &params.sign_command {
    Some(custom_command) => sign_custom(path, custom_command),
    #[cfg(windows)]
    None => sign_default(path, params),
    // should not be reachable, as user should either use Windows
    // or specify a custom sign_command but we succeed anyways
    #[cfg(not(windows))]
    None => Ok(()),
  }
}

pub fn try_sign(file_path: &std::path::PathBuf, settings: &Settings) -> crate::Result<()> {
  if settings.can_sign() {
    log::info!(action = "Signing"; "{}", tauri_utils::display_path(file_path));
    sign(file_path, &settings.sign_params())?;
  }
  Ok(())
}
