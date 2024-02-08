// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::{common::CommandExt, windows::util},
  Settings,
};
use log::{debug, info};
use std::{
  env::var_os,
  ffi::OsStr,
  path::{Path, PathBuf},
  process::Command,
};
use winreg::{
  enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY},
  RegKey,
};

/// Enum to hold the different signing params depending on the current setup. (Signtool or AzureSignTool)
pub enum SignParams {
  SignTool(SignToolParams),
  Azure(AzureSignToolParams),
}

impl SignParams {
  /// Check if binary is already signed.
  /// Used to skip sidecar binaries that are already signed.
  /// If we're using AzureSignTool, we'll always return false because we can't check if it's already signed.
  /// AzureSignTool will skip signing already signed binaries anyway.
  pub fn verify(&self, path: &Path) -> crate::Result<bool> {
    match self {
      SignParams::SignTool(_) => {
        // Construct SignTool command
        let signtool = locate_signtool()?;

        let mut cmd = Command::new(&signtool);
        cmd.arg("verify");
        cmd.arg("/pa");
        cmd.arg(path);

        Ok(cmd.status()?.success())
      }
      SignParams::Azure(_) => Ok(false),
    }
  }

  /// Retrieves the proper signing command depending on the current setup.
  pub fn sign_command<P: AsRef<OsStr>>(&self, path: P) -> crate::Result<(Command, PathBuf)> {
    match self {
      SignParams::SignTool(params) => params.sign_command(path),
      SignParams::Azure(params) => params.sign_command(path),
    }
  }

  pub fn sign<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    info!(action = "Signing"; "{}", tauri_utils::display_path(path.as_ref()));
    match self {
      SignParams::SignTool(params) => params.sign(path),
      SignParams::Azure(params) => params.sign(path),
    }
  }
}

/// This contains params needed for the windows signtool.exe
pub struct SignToolParams {
  pub product_name: String,
  pub digest_algorithm: String,
  pub certificate_thumbprint: String,
  pub timestamp_url: Option<String>,
  pub tsp: bool,
}

impl SignToolParams {
  pub fn sign_command<P: AsRef<OsStr>>(&self, path: P) -> crate::Result<(Command, PathBuf)> {
    // Construct SignTool command
    let signtool = locate_signtool()?;

    let mut cmd = Command::new(&signtool);
    cmd.arg("sign");
    cmd.args(["/fd", &self.digest_algorithm]);
    cmd.args(["/sha1", &self.certificate_thumbprint]);
    cmd.args(["/d", &self.product_name]);

    if let Some(ref timestamp_url) = self.timestamp_url {
      if self.tsp {
        cmd.args(["/tr", timestamp_url]);
        cmd.args(["/td", &self.digest_algorithm]);
      } else {
        cmd.args(["/t", timestamp_url]);
      }
    }

    cmd.arg(path);

    Ok((cmd, signtool))
  }

  pub fn sign<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    info!(action = "Signing"; "{} with identity \"{}\"", tauri_utils::display_path(path.as_ref()), self.certificate_thumbprint);

    let (mut cmd, signtool) = self.sign_command(path.as_ref())?;
    debug!("Running signtool {:?}", signtool);

    // Execute SignTool command
    let output = cmd.output_ok()?;

    let stdout = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    info!("{:?}", stdout);

    Ok(())
  }
}
/// This contains params needed for the AzureSignTool.exe
/// These are the environment variables that need to be set:
pub struct AzureSignToolParams {
  keyvault_url: String,
  client_id: String,
  tenant_id: String,
  secret: String,
  certificate_name: String,
  product_name: String,
  description_url: Option<String>,
  timestamp_url: Option<String>,
}

impl AzureSignToolParams {
  pub fn sign_command<P: AsRef<OsStr>>(&self, path: P) -> crate::Result<(Command, PathBuf)> {
    let azuresigntool = locate_azuresigntool()?;

    let mut cmd = Command::new(&azuresigntool);
    cmd.arg("sign");
    cmd.args(["-kvu", &self.keyvault_url]);
    cmd.args(["-kvi", &self.client_id]);
    cmd.args(["-kvt", &self.tenant_id]);
    cmd.args(["-kvs", &self.secret]);
    cmd.args(["-kvc", &self.certificate_name]);
    cmd.args(["-d", &self.product_name]);

    if let Some(ref description_url) = self.description_url {
      cmd.args(["-du", description_url]);
    }

    if let Some(ref timestamp_url) = self.timestamp_url {
      cmd.args(["-tr", timestamp_url]);
    }

    // Ignore already signed files
    cmd.arg("-s");

    cmd.arg(path);

    Ok((cmd, azuresigntool))
  }

  pub fn sign<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
    info!(action = "Signing"; "{} with Azure Key Vault certificate: {}", tauri_utils::display_path(path.as_ref()), self.certificate_name);

    let (mut cmd, signtool) = self.sign_command(path.as_ref())?;
    debug!("Running AzureSignTool {:?}", signtool);

    // Execute SignTool command
    let _ = cmd.output_ok()?;
    info!("AzureSignTool completed successfully.");

    Ok(())
  }
}

// sign code forked from https://github.com/forbjok/rust-codesign
fn locate_signtool() -> crate::Result<PathBuf> {
  const INSTALLED_ROOTS_REGKEY_PATH: &str = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
  const KITS_ROOT_REGVALUE_NAME: &str = r"KitsRoot10";

  let installed_roots_key_path = Path::new(INSTALLED_ROOTS_REGKEY_PATH);

  // Open 32-bit HKLM "Installed Roots" key
  let installed_roots_key = RegKey::predef(HKEY_LOCAL_MACHINE)
    .open_subkey_with_flags(installed_roots_key_path, KEY_READ | KEY_WOW64_32KEY)
    .map_err(|_| crate::Error::OpenRegistry(INSTALLED_ROOTS_REGKEY_PATH.to_string()))?;

  // Get the Windows SDK root path
  let kits_root_10_path: String = installed_roots_key
    .get_value(KITS_ROOT_REGVALUE_NAME)
    .map_err(|_| crate::Error::GetRegistryValue(KITS_ROOT_REGVALUE_NAME.to_string()))?;

  // Construct Windows SDK bin path
  let kits_root_10_bin_path = Path::new(&kits_root_10_path).join("bin");

  let mut installed_kits: Vec<String> = installed_roots_key
    .enum_keys()
    /* Report and ignore errors, pass on values. */
    .filter_map(|res| match res {
      Ok(v) => Some(v),
      Err(_) => None,
    })
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
}

/// This is check if azuresigntool is available on the users system.
fn locate_azuresigntool() -> crate::Result<PathBuf> {
  let mut cmd = Command::new("azuresigntool.exe");
  cmd.arg("--help");
  let _ = cmd.output_ok()?;
  Ok(PathBuf::from("azuresigntool.exe"))
}

impl Settings {
  /// Attempts to create signing params from the environment variables and settings object.
  /// If neither AzureSignTool or SignTool can be used because of missing environment variables, this will return `None`.
  pub(crate) fn sign_params(&self) -> Option<SignParams> {
    let product_name = self.product_name().to_string();

    // We'll attempt to create a SignParams struct from the environment variables and the settings
    // First we'll start with AzureSignTool, if we have all the required environment variables set, then we'll use that.
    // If not, we'll fallback to Signtool.
    // But if we don't have a certificate thumbprint set, we can't sign at all and we'll return None.
    match (
      var_os("AZURE_KEYVAULT_URL"),
      var_os("AZURE_CLIENT_ID"),
      var_os("AZURE_TENANT_ID"),
      var_os("AZURE_CLIENT_SECRET"),
      var_os("AZURE_CERTIFICATE_NAME"),
    ) {
      (
        Some(keyvault_url),
        Some(client_id),
        Some(tenant_id),
        Some(secret),
        Some(certificate_name),
      ) => {
        let description_url =
          var_os("AZURE_DESCRIPTION_URL").map(|s| s.to_string_lossy().to_string());
        let timestamp_url = match var_os("AZURE_TIMESTAMP_URL") {
          Some(timestamp_url) => Some(timestamp_url.to_string_lossy().to_string()),
          None => self.windows().timestamp_url.clone(),
        };

        Some(SignParams::Azure(AzureSignToolParams {
          keyvault_url: keyvault_url.to_string_lossy().to_string(),
          client_id: client_id.to_string_lossy().to_string(),
          tenant_id: tenant_id.to_string_lossy().to_string(),
          secret: secret.to_string_lossy().to_string(),
          product_name,
          certificate_name: certificate_name.to_string_lossy().to_string(),
          description_url,
          timestamp_url,
        }))
      }
      _ => {
        // If there isn't a configured certificate thumbprint, we can't sign.
        if self.windows().certificate_thumbprint.is_none() {
          return None;
        }

        let certificate_thumbprint = self
          .windows()
          .certificate_thumbprint
          .clone()
          .unwrap_or_default();
        let timestamp_url = self.windows().timestamp_url.clone();
        let tsp = self.windows().tsp;
        let digest_algorithm = self
          .windows()
          .digest_algorithm
          .as_ref()
          .map(|algorithm| algorithm.to_string())
          .unwrap_or_else(|| "sha256".to_string());

        Some(SignParams::SignTool(SignToolParams {
          product_name,
          digest_algorithm,
          certificate_thumbprint,
          timestamp_url,
          tsp,
        }))
      }
    }
  }
}
