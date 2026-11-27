// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tauri_macos_sign::certificate::{self, CertificateInfo};
use tauri_utils::display_path;

use super::secrets::{self, Secret, SecretsOptions};
use crate::{
  Result,
  error::{Context, ErrorExt},
};

#[derive(Parser)]
#[clap(
  about = "Prepare Windows code signing certificates",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  ImportCertificate(ImportOptions),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::ImportCertificate(options) => import(options),
  }
}

#[derive(Debug, Parser)]
#[clap(
  about = "Convert a code signing certificate to a .pfx file and configure the app to sign with it",
  long_about = "Convert a code signing certificate and its private key to a .pfx file, print its thumbprint and configure the app to sign with it.\n\nThis only applies to certificates with an exportable private key. Certificates stored in hardware tokens or cloud services (which is required for certificates issued after June 2023) must be configured with `bundle > windows > signCommand` instead."
)]
struct ImportOptions {
  /// Certificate file (.cer, .crt or .pem) issued by your certificate authority
  #[clap(short, long, requires = "key", conflicts_with = "pfx")]
  certificate: Option<PathBuf>,
  /// Private key of the certificate
  #[clap(short, long, requires = "certificate")]
  key: Option<PathBuf>,
  /// Existing .pfx file to read the certificate from
  #[clap(long, required_unless_present = "certificate")]
  pfx: Option<PathBuf>,
  /// Password of the existing .pfx file
  #[clap(long, requires = "pfx")]
  pfx_password: Option<String>,
  /// Password of the generated .pfx file. Prompted when not provided, or randomly generated with `--ci`
  #[clap(short, long)]
  password: Option<String>,
  /// Path of the generated .pfx file
  #[clap(short, long, default_value = "certificate.pfx")]
  output: PathBuf,
  /// Set `bundle > windows > certificateThumbprint` and `digestAlgorithm` in the Tauri configuration file
  #[clap(long)]
  configure: bool,
  /// Timestamp server URL to set in `bundle > windows > timestampUrl`, provided by your certificate authority
  #[clap(long, requires = "configure")]
  timestamp_url: Option<String>,
  /// Install the certificate in the current user's certificate store (Windows only)
  #[clap(long)]
  install: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  #[clap(flatten)]
  secrets: SecretsOptions,
}

fn import(options: ImportOptions) -> Result<()> {
  let (cert, key) = match (&options.certificate, &options.key, &options.pfx) {
    (Some(certificate), Some(key), _) => {
      let cert_data =
        std::fs::read(certificate).fs_context("failed to read certificate", certificate.clone())?;
      let key_data = std::fs::read(key).fs_context("failed to read private key", key.clone())?;
      (
        certificate::parse_certificate(&cert_data).map_err(Box::new)?,
        certificate::parse_private_key(&key_data).map_err(Box::new)?,
      )
    }
    (_, _, Some(pfx)) => {
      let data = std::fs::read(pfx).fs_context("failed to read .pfx file", pfx.clone())?;
      let password = secrets::password(
        options.pfx_password.clone(),
        "Password of the .pfx file",
        false,
        false,
      )?;
      let mut identities = certificate::parse_pkcs12(&data, &password).map_err(Box::new)?;
      if identities.len() > 1 {
        log::warn!("The .pfx file contains several certificates, using the first one");
      }
      let identity = identities.remove(0);
      (identity.certificate, identity.private_key)
    }
    _ => unreachable!("validated by clap"),
  };

  let info = CertificateInfo::from_certificate(&cert).map_err(Box::new)?;
  let password = secrets::password(
    options.password,
    "Password to protect the .pfx file",
    true,
    options.ci,
  )?;
  let name = info
    .common_name
    .clone()
    .unwrap_or_else(|| "code-signing".into());
  let pfx = certificate::create_pkcs12(&cert, &key, &password, &name).map_err(Box::new)?;

  std::fs::write(&options.output, &pfx)
    .fs_context("failed to write .pfx file", options.output.clone())?;

  println!();
  println!("Certificate: {name}");
  println!("Thumbprint: {}", info.sha1_thumbprint);
  println!("Expires: {}", info.not_after.format("%Y-%m-%d"));
  println!(
    "Saved the certificate and its private key to {}",
    display_path(&options.output)
  );

  if options.install {
    install(&options.output, &password)?;
  }

  if options.configure {
    configure(&info.sha1_thumbprint, options.timestamp_url.as_deref())?;
  } else {
    println!();
    println!(
      "To sign with this certificate, set the following in your Tauri configuration file (or pass --configure):"
    );
    println!(
      "{}",
      serde_json::to_string_pretty(&serde_json::json!({
        "bundle": {
          "windows": {
            "certificateThumbprint": info.sha1_thumbprint,
            "digestAlgorithm": "sha256",
            "timestampUrl": options.timestamp_url.as_deref().unwrap_or("<timestamp URL of your certificate authority>"),
          }
        }
      }))
      .unwrap()
    );
  }

  options.secrets.output(&[
    Secret::new("WINDOWS_CERTIFICATE", secrets::base64(&pfx)),
    Secret::new("WINDOWS_CERTIFICATE_PASSWORD", password),
  ])
}

#[cfg(windows)]
fn install(path: &std::path::Path, password: &str) -> Result<()> {
  let status = std::process::Command::new("powershell")
    .args([
      "-NoProfile",
      "-NonInteractive",
      "-Command",
      "Import-PfxCertificate -FilePath $env:TAURI_PFX_PATH -CertStoreLocation Cert:\\CurrentUser\\My -Password (ConvertTo-SecureString -String $env:TAURI_PFX_PASSWORD -Force -AsPlainText) | Out-Null",
    ])
    .env("TAURI_PFX_PATH", path)
    .env("TAURI_PFX_PASSWORD", password)
    .status()
    .map_err(|error| crate::Error::CommandFailed {
      command: "Import-PfxCertificate".into(),
      error,
    })?;
  if !status.success() {
    crate::error::bail!("failed to install the certificate in the certificate store");
  }
  log::info!(action = "Installed"; "the certificate in Cert:\\CurrentUser\\My");
  Ok(())
}

#[cfg(not(windows))]
fn install(_path: &std::path::Path, _password: &str) -> Result<()> {
  log::warn!("Installing certificates in the certificate store is only supported on Windows");
  Ok(())
}

fn configure(thumbprint: &str, timestamp_url: Option<&str>) -> Result<()> {
  let tauri_dir = crate::helpers::app_paths::resolve_tauri_dir()
    .context("could not find a Tauri project to configure")?;
  let (mut config, config_path) = tauri_utils::config::parse::parse_value(
    tauri_utils::platform::Target::Windows,
    tauri_dir.join("tauri.conf.json"),
  )
  .context("failed to parse config")?;

  let windows = config
    .as_object_mut()
    .context("invalid configuration")?
    .entry("bundle")
    .or_insert_with(|| serde_json::json!({}))
    .as_object_mut()
    .context("invalid `bundle` configuration")?
    .entry("windows")
    .or_insert_with(|| serde_json::json!({}))
    .as_object_mut()
    .context("invalid `bundle > windows` configuration")?;
  // the kebab-case aliases (common in Tauri.toml) would conflict with the camelCase keys
  windows.remove("certificate-thumbprint");
  windows.insert("certificateThumbprint".into(), thumbprint.into());
  if !windows.contains_key("digestAlgorithm") && !windows.contains_key("digest-algorithm") {
    windows.insert("digestAlgorithm".into(), "sha256".into());
  }
  if let Some(timestamp_url) = timestamp_url {
    windows.remove("timestamp-url");
    windows.insert("timestampUrl".into(), timestamp_url.into());
  }

  let serialized = if config_path.extension().is_some_and(|ext| ext == "toml") {
    toml::to_string_pretty(&config).context("failed to serialize config")?
  } else {
    serde_json::to_string_pretty(&config).context("failed to serialize config")?
  };
  std::fs::write(&config_path, serialized)
    .fs_context("failed to write config", config_path.clone())?;

  log::info!(action = "Updated"; "{} to sign Windows binaries with this certificate", display_path(&config_path));
  Ok(())
}
