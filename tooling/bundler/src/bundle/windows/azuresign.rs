use std::{env::var_os, process::Command};

use log::info;

use crate::{bundle::common::CommandExt, Settings};

pub struct AzureSignToolArgs {
  keyvault_uri: String,
  client_id: String,
  tenant_id: String,
  secret: String,
  certificate_name: String,
  product_name: String,
  description_url: Option<String>,
  timestamp_url: Option<String>,
}

impl AzureSignToolArgs {
  /// Attempts to generate a [AzureSignToolArgs] struct from the environment variables and settings object.
  /// If the required environment variables are not set, this will return None.
  pub fn generate(settings: &Settings) -> Option<Self> {
    match (
      var_os("AZURE_KEYVAULT_URI"),
      var_os("AZURE_CLIENT_ID"),
      var_os("AZURE_TENANT_ID"),
      var_os("AZURE_CLIENT_SECRET"),
      var_os("AZURE_CERTIFICATE_NAME"),
    ) {
      (
        Some(keyvault_uri),
        Some(client_id),
        Some(tenant_id),
        Some(secret),
        Some(certificate_name),
      ) => {
        let description_url =
          var_os("AZURE_DESCRIPTION_URL").map(|s| s.to_string_lossy().to_string());
        let timestamp_url = match var_os("AZURE_TIMESTAMP_URL") {
          Some(timestamp_url) => Some(timestamp_url.to_string_lossy().to_string()),
          None => settings.windows().timestamp_url.clone(),
        };

        Some(Self {
          keyvault_uri: keyvault_uri.to_string_lossy().to_string(),
          client_id: client_id.to_string_lossy().to_string(),
          tenant_id: tenant_id.to_string_lossy().to_string(),
          secret: secret.to_string_lossy().to_string(),
          product_name: settings.product_name().into(),
          certificate_name: certificate_name.to_string_lossy().to_string(),
          description_url,
          timestamp_url,
        })
      }
      _ => None,
    }
  }
}

pub fn sign(file_path: &std::path::PathBuf, args: &AzureSignToolArgs) -> crate::Result<()> {
  let mut cmd = Command::new("azuresigntool");
  cmd.arg("sign");
  cmd.args(["-kvu", &args.keyvault_uri]);
  cmd.args(["-kvi", &args.client_id]);
  cmd.args(["-kvt", &args.tenant_id]);
  cmd.args(["-kvs", &args.secret]);
  cmd.args(["-kvc", &args.certificate_name]);
  cmd.args(["-d", &args.product_name]);

  if let Some(description_url) = &args.description_url {
    cmd.args(["-du", description_url]);
  }

  if let Some(timestamp_url) = &args.timestamp_url {
    cmd.args(["-tr", timestamp_url]);
  }

  // Ignore already signed files
  cmd.arg("-s");

  cmd.arg(&file_path);

  let output = cmd.output_ok()?;

  info!("Signed with AzureSignTool: {:?}", file_path);

  Ok(())
}

/// Checks if azure sign tool is able to be used
pub fn can_azure_sign(settings: &Settings) -> bool {
  match AzureSignToolArgs::generate(settings) {
    Some(_) => true,
    None => false,
  }
}
