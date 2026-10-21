// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use tauri_macos_sign::{
  ProvisioningProfileInfo,
  certificate::{self, CertificateInfo},
};
use tauri_utils::display_path;

use super::secrets::{self, Secret, SecretsOptions};
use crate::{
  Result,
  error::{Context, ErrorExt},
};

mod app_store_connect;

const CERTIFICATES_URL: &str = "https://developer.apple.com/account/resources/certificates/add";

#[derive(Parser)]
#[clap(
  about = "Create and export Apple code signing certificates and provisioning profiles for macOS and iOS",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  CreateCertificate(CreateCertificateOptions),
  Csr(CsrOptions),
  ImportCertificate(ImportOptions),
  #[cfg(target_os = "macos")]
  ExportCertificate(ExportOptions),
  ExportProfile(ProfileOptions),
  ExportApiKey(ApiKeyOptions),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::CreateCertificate(options) => create_certificate(options),
    Commands::Csr(options) => csr(options),
    Commands::ImportCertificate(options) => import(options),
    #[cfg(target_os = "macos")]
    Commands::ExportCertificate(options) => export(options),
    Commands::ExportProfile(options) => profile(options),
    Commands::ExportApiKey(options) => api_key(options),
  }
}

/// Platform the certificate is used for, which determines the environment variable names.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Platform {
  /// APPLE_CERTIFICATE and APPLE_CERTIFICATE_PASSWORD
  Macos,
  /// IOS_CERTIFICATE and IOS_CERTIFICATE_PASSWORD
  Ios,
}

impl Platform {
  fn certificate_secrets(self, p12: &[u8], password: &str) -> Vec<Secret> {
    let (certificate, password_name) = match self {
      Self::Macos => ("APPLE_CERTIFICATE", "APPLE_CERTIFICATE_PASSWORD"),
      Self::Ios => ("IOS_CERTIFICATE", "IOS_CERTIFICATE_PASSWORD"),
    };
    vec![
      Secret::new(certificate, secrets::base64(p12)),
      Secret::new(password_name, password),
    ]
  }
}

#[derive(Debug, Parser)]
struct CertificateOutputOptions {
  /// Password of the exported .p12 file. Prompted when not provided, or randomly generated with `--ci`
  #[clap(short, long)]
  password: Option<String>,
  /// Path of the .p12 file to write
  #[clap(short, long)]
  output: Option<PathBuf>,
  /// Platform the certificate is used for, which determines the environment variable names
  #[clap(long, value_enum, default_value_t = Platform::Macos)]
  platform: Platform,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  #[clap(flatten)]
  secrets: SecretsOptions,
}

impl CertificateOutputOptions {
  fn password(&self) -> Result<String> {
    secrets::password(
      self.password.clone(),
      "Password to protect the exported .p12 file",
      true,
      self.ci,
    )
  }
}

fn print_certificate(info: &CertificateInfo) {
  println!();
  println!(
    "Certificate: {}",
    info.common_name.as_deref().unwrap_or("<unnamed>")
  );
  if let Some(team_id) = &info.organizational_unit {
    println!("Team ID: {team_id}");
  }
  println!("SHA-1: {}", info.sha1_thumbprint);
  println!("Expires: {}", info.not_after.format("%Y-%m-%d"));
}

fn write_private(path: &Path, contents: &[u8]) -> Result<()> {
  if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
    std::fs::create_dir_all(parent).fs_context("failed to create directory", parent)?;
  }
  #[cfg(unix)]
  {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    std::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .mode(0o600)
      .open(path)
      .and_then(|mut f| f.write_all(contents))
      .fs_context("failed to write file", path.to_path_buf())
  }
  #[cfg(not(unix))]
  std::fs::write(path, contents).fs_context("failed to write file", path.to_path_buf())
}

/// Writes the .p12 file, optionally installs it in the keychain and returns the CI/CD secrets.
fn finish_certificate(
  options: &CertificateOutputOptions,
  info: &CertificateInfo,
  p12: &[u8],
  password: &str,
  default_output: &str,
  install: bool,
) -> Result<Vec<Secret>> {
  print_certificate(info);

  let output = options
    .output
    .clone()
    .unwrap_or_else(|| PathBuf::from(default_output));
  write_private(&output, p12)?;
  println!(
    "Saved the certificate and its private key to {}",
    display_path(&output)
  );

  if install {
    install_certificate(&output, password)?;
  }

  if let (Platform::Macos, Some(name)) = (options.platform, &info.common_name) {
    println!();
    if install {
      println!(
        "To sign on this machine without environment variables, set `bundle > macOS > signingIdentity` to \"{name}\""
      );
    } else {
      println!(
        "To sign on this machine without environment variables, install the certificate in your keychain (--install) and set `bundle > macOS > signingIdentity` to \"{name}\""
      );
    }
  }

  Ok(options.platform.certificate_secrets(p12, password))
}

#[cfg(target_os = "macos")]
fn install_certificate(path: &Path, password: &str) -> Result<()> {
  tauri_macos_sign::import_into_login_keychain(path, &password.into()).map_err(Box::new)?;
  log::info!(action = "Installed"; "the certificate in the login keychain");
  Ok(())
}

#[cfg(not(target_os = "macos"))]
fn install_certificate(_path: &Path, _password: &str) -> Result<()> {
  log::warn!("Installing certificates in the keychain is only supported on macOS");
  Ok(())
}

fn file_stem(name: &str) -> String {
  name
    .chars()
    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
    .collect::<String>()
    .split('-')
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("-")
}

// ---- csr ----

#[derive(Debug, Parser)]
#[clap(
  about = "Generate a private key and a certificate signing request (CSR) to create a certificate in the Apple Developer portal"
)]
struct CsrOptions {
  /// Common name of the request, usually your name or your company name
  #[clap(long)]
  common_name: Option<String>,
  /// Email address of the request
  #[clap(long)]
  email: Option<String>,
  /// Directory to write the private key and the CSR to
  #[clap(short, long, default_value = ".")]
  out_dir: PathBuf,
  /// Overwrite existing files
  #[clap(short, long)]
  force: bool,
}

fn csr(options: CsrOptions) -> Result<()> {
  let common_name = match options.common_name {
    Some(name) => name,
    None => crate::helpers::prompts::input(
      "Common name (your name or company name)",
      None,
      false,
      false,
    )?
    .context("a common name is required")?,
  };

  let stem = file_stem(&common_name);
  let key_path = options.out_dir.join(format!("{stem}.key"));
  let csr_path = options.out_dir.join(format!("{stem}.certSigningRequest"));
  for path in [&key_path, &csr_path] {
    if path.exists() && !options.force {
      crate::error::bail!(
        "{} already exists, pass --force to overwrite it",
        display_path(path)
      );
    }
  }

  let key = certificate::generate_rsa_private_key().map_err(Box::new)?;
  let csr =
    certificate::create_certificate_signing_request(&key, &common_name, options.email.as_deref())
      .map_err(Box::new)?;

  write_private(&key_path, certificate::private_key_to_pem(&key).as_bytes())?;
  std::fs::write(&csr_path, csr).fs_context("failed to write CSR", csr_path.clone())?;

  println!("Private key: {} (keep it secret!)", display_path(&key_path));
  println!("Certificate signing request: {}", display_path(&csr_path));
  println!();
  println!("Next steps:");
  println!("1. Open {CERTIFICATES_URL}");
  println!(
    "2. Choose the certificate type (`Developer ID Application` to distribute macOS apps outside the App Store, `Apple Distribution` for the App Store) and upload {}",
    display_path(&csr_path)
  );
  println!("3. Download the certificate and run:");
  println!(
    "   tauri signer apple import-certificate --certificate <path/to/downloaded.cer> --key {}",
    display_path(&key_path)
  );

  Ok(())
}

// ---- import ----

#[derive(Debug, Parser)]
#[clap(
  about = "Combine a certificate downloaded from the Apple Developer portal with its private key into a .p12 file"
)]
struct ImportOptions {
  /// Certificate file (.cer) downloaded from the Apple Developer portal
  #[clap(short, long)]
  certificate: PathBuf,
  /// Private key used to create the certificate signing request, e.g. generated by `tauri signer apple csr`
  #[clap(short, long)]
  key: PathBuf,
  /// Install the certificate in the login keychain (macOS only)
  #[clap(long)]
  install: bool,
  #[clap(flatten)]
  output: CertificateOutputOptions,
}

fn import(options: ImportOptions) -> Result<()> {
  let cert_data = std::fs::read(&options.certificate)
    .fs_context("failed to read certificate", options.certificate.clone())?;
  let key_data =
    std::fs::read(&options.key).fs_context("failed to read private key", options.key.clone())?;

  let cert = certificate::parse_certificate(&cert_data).map_err(Box::new)?;
  let key = certificate::parse_private_key(&key_data).map_err(Box::new)?;
  let info = CertificateInfo::from_certificate(&cert).map_err(Box::new)?;

  let password = options.output.password()?;
  let name = info
    .common_name
    .clone()
    .unwrap_or_else(|| "code-signing".into());
  let p12 = certificate::create_pkcs12(&cert, &key, &password, &name).map_err(Box::new)?;

  let secrets = finish_certificate(
    &options.output,
    &info,
    &p12,
    &password,
    &format!("{}.p12", file_stem(&name)),
    options.install,
  )?;
  options.output.secrets.output(&secrets)
}

// ---- export ----

#[cfg(target_os = "macos")]
#[derive(Debug, Parser)]
#[clap(about = "Export a signing certificate and its private key from the keychain to a .p12 file")]
struct ExportOptions {
  /// Signing identity to export, either its name (e.g. `Developer ID Application: My Company (TEAMID)`) or SHA-1 hash.
  /// Prompted when not provided.
  #[clap(short, long)]
  identity: Option<String>,
  #[clap(flatten)]
  output: CertificateOutputOptions,
}

#[cfg(target_os = "macos")]
fn export(options: ExportOptions) -> Result<()> {
  let identity = match options.identity.clone() {
    Some(identity) => identity,
    None => {
      let identities = tauri_macos_sign::find_identities().map_err(Box::new)?;
      if identities.is_empty() {
        crate::error::bail!(
          "no code signing identities found in the keychain; use `tauri signer apple create-certificate` or `tauri signer apple csr` to create a certificate"
        );
      }
      if options.output.ci {
        crate::error::bail!("--identity is required in non-interactive mode");
      }
      let items = identities
        .iter()
        .map(|i| {
          format!(
            "{} (expires {})",
            i.common_name.as_deref().unwrap_or(&i.sha1_thumbprint),
            i.not_after.format("%Y-%m-%d")
          )
        })
        .collect::<Vec<_>>();
      let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Signing identity to export")
        .items(&items)
        .default(0)
        .interact()
        .context("failed to prompt signing identity")?;
      identities[selection].sha1_thumbprint.clone()
    }
  };

  let password = options.output.password()?;
  println!("macOS might ask for your login password to allow exporting the private key.");
  let (info, p12) = tauri_macos_sign::export_identity(&identity, &password).map_err(Box::new)?;

  let default_output = format!(
    "{}.p12",
    file_stem(info.common_name.as_deref().unwrap_or("certificate"))
  );
  let secrets = finish_certificate(
    &options.output,
    &info,
    &p12,
    &password,
    &default_output,
    false,
  )?;
  options.output.secrets.output(&secrets)
}

// ---- profile ----

#[derive(Debug, Parser)]
#[clap(about = "Encode a provisioning profile for CI/CD platforms (IOS_MOBILE_PROVISION)")]
struct ProfileOptions {
  /// Path of the provisioning profile. When not provided, the profiles installed by Xcode are searched
  #[clap(short, long)]
  path: Option<PathBuf>,
  /// Bundle identifier the profile must match. Defaults to the app identifier
  #[clap(long)]
  bundle_id: Option<String>,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  #[clap(flatten)]
  secrets: SecretsOptions,
}

fn installed_profiles() -> Vec<(PathBuf, ProvisioningProfileInfo)> {
  let Some(home) = dirs::home_dir() else {
    return Vec::new();
  };
  let mut profiles = Vec::new();
  for dir in [
    home.join("Library/MobileDevice/Provisioning Profiles"),
    home.join("Library/Developer/Xcode/UserData/Provisioning Profiles"),
  ] {
    let Ok(entries) = std::fs::read_dir(dir) else {
      continue;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if path
        .extension()
        .is_some_and(|ext| ext == "mobileprovision" || ext == "provisionprofile")
        && let Ok(data) = std::fs::read(&path)
        && let Ok(info) = ProvisioningProfileInfo::parse(&data)
        && !profiles
          .iter()
          .any(|(_, p): &(PathBuf, ProvisioningProfileInfo)| p.uuid == info.uuid)
      {
        profiles.push((path, info));
      }
    }
  }
  profiles
}

fn app_identifier() -> Option<String> {
  let tauri_dir = crate::helpers::app_paths::resolve_tauri_dir()?;
  crate::helpers::config::get_config(tauri_utils::platform::Target::Ios, &[], &tauri_dir)
    .ok()
    .map(|c| c.identifier.clone())
}

fn profile(options: ProfileOptions) -> Result<()> {
  let path = match options.path {
    Some(path) => path,
    None => {
      let bundle_id = options
        .bundle_id
        .or_else(app_identifier)
        .context("--bundle-id is required outside of a Tauri project")?;
      let mut profiles = installed_profiles()
        .into_iter()
        .filter(|(_, p)| !p.is_expired() && p.matches_bundle_identifier(&bundle_id))
        .collect::<Vec<_>>();
      // exact matches first, then distribution profiles, then the latest to expire
      profiles.sort_by_key(|(_, p)| {
        (
          p.bundle_identifier() != Some(bundle_id.as_str()),
          match p.kind() {
            "App Store" => 0,
            "ad hoc" => 1,
            "enterprise" => 2,
            _ => 3,
          },
          std::cmp::Reverse(p.expiration_date),
        )
      });

      match profiles.len() {
        0 => crate::error::bail!(
          "no installed provisioning profile matches {bundle_id}; download it from https://developer.apple.com/account/resources/profiles/list and pass it with --path, or create one with `tauri signer apple create-certificate --profile app-store`"
        ),
        1 => profiles.remove(0).0,
        _ if options.ci => profiles.remove(0).0,
        _ => {
          let items = profiles
            .iter()
            .map(|(_, p)| format!("{} ({}, {})", p.name, p.kind(), p.uuid))
            .collect::<Vec<_>>();
          let selection =
            dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
              .with_prompt("Provisioning profile")
              .items(&items)
              .default(0)
              .interact()
              .context("failed to prompt provisioning profile")?;
          profiles.remove(selection).0
        }
      }
    }
  };

  let data = std::fs::read(&path).fs_context("failed to read provisioning profile", path)?;
  let info = ProvisioningProfileInfo::parse(&data).map_err(Box::new)?;

  println!();
  println!("Provisioning profile: {} ({})", info.name, info.kind());
  if let Some(app_id) = &info.application_identifier {
    println!("App ID: {app_id}");
  }
  println!("UUID: {}", info.uuid);
  if info.is_expired() {
    log::warn!("This provisioning profile is expired");
  }
  if info.kind() == "development" {
    log::warn!(
      "This is a development provisioning profile, it cannot be used to distribute the app on the App Store"
    );
  }

  options
    .secrets
    .output(&[Secret::new("IOS_MOBILE_PROVISION", secrets::base64(&data))])
}

// ---- api-key ----

#[derive(Debug, Parser)]
struct ApiKeyArgs {
  /// App Store Connect API key ID
  #[clap(long, env = "APPLE_API_KEY")]
  api_key: Option<String>,
  /// App Store Connect API issuer ID
  #[clap(long, env = "APPLE_API_ISSUER")]
  api_issuer: Option<String>,
  /// Path of the App Store Connect API private key (AuthKey_<KEY_ID>.p8)
  #[clap(long, env = "APPLE_API_KEY_PATH")]
  api_key_path: Option<PathBuf>,
}

impl ApiKeyArgs {
  fn resolve(&self, ci: bool) -> Result<app_store_connect::ApiKey> {
    let prompt = |value: &Option<String>, name: &str, flag: &str| -> Result<String> {
      match value {
        Some(value) => Ok(value.clone()),
        None if ci => crate::error::bail!("{flag} is required in non-interactive mode"),
        None => crate::helpers::prompts::input(name, None, false, false)?
          .with_context(|| format!("{flag} is required")),
      }
    };

    let key_id = prompt(&self.api_key, "App Store Connect API key ID", "--api-key")?;
    let issuer_id = prompt(
      &self.api_issuer,
      "App Store Connect API issuer ID",
      "--api-issuer",
    )?;

    let private_key = if let Some(path) = &self.api_key_path {
      std::fs::read_to_string(path).fs_context("failed to read API key", path.clone())?
    } else if let Some(content) = std::env::var_os("APPLE_API_KEY_CONTENT") {
      let tauri_macos_sign::ApiKey::Raw(key) =
        tauri_macos_sign::ApiKey::from_content(&content).map_err(Box::new)?
      else {
        unreachable!()
      };
      String::from_utf8(key).context("invalid APPLE_API_KEY_CONTENT")?
    } else {
      let path = find_api_key_file(&key_id).with_context(|| {
        format!(
          "could not find AuthKey_{key_id}.p8; pass its path with --api-key-path or set APPLE_API_KEY_PATH"
        )
      })?;
      std::fs::read_to_string(&path).fs_context("failed to read API key", path)?
    };

    Ok(app_store_connect::ApiKey {
      key_id,
      issuer_id,
      private_key,
    })
  }
}

/// Looks for the API key in the same directories as `xcrun altool` and `notarytool`.
fn find_api_key_file(key_id: &str) -> Option<PathBuf> {
  let file_name = format!("AuthKey_{key_id}.p8");
  let mut dirs = vec![PathBuf::from("private_keys")];
  if let Some(home) = dirs::home_dir() {
    dirs.push(home.join("private_keys"));
    dirs.push(home.join(".private_keys"));
    dirs.push(home.join(".appstoreconnect").join("private_keys"));
  }
  dirs
    .into_iter()
    .map(|dir| dir.join(&file_name))
    .find(|path| path.exists())
}

#[derive(Debug, Parser)]
#[clap(
  about = "Encode an App Store Connect API key for CI/CD platforms, used for notarization and iOS automatic signing"
)]
struct ApiKeyOptions {
  #[clap(flatten)]
  key: ApiKeyArgs,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  #[clap(flatten)]
  secrets: SecretsOptions,
}

fn api_key(options: ApiKeyOptions) -> Result<()> {
  let key = options.key.resolve(options.ci)?;
  // validates the key
  app_store_connect::Client::new(app_store_connect::ApiKey {
    key_id: key.key_id.clone(),
    issuer_id: key.issuer_id.clone(),
    private_key: key.private_key.clone(),
  })?;

  options.secrets.output(&[
    Secret::new("APPLE_API_KEY", key.key_id),
    Secret::new("APPLE_API_ISSUER", key.issuer_id),
    Secret::new(
      "APPLE_API_KEY_CONTENT",
      secrets::base64(key.private_key.trim().as_bytes()),
    ),
  ])
}

// ---- create ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CertificateType {
  /// Developer ID Application: distribute macOS apps outside the App Store (requires the Account Holder role)
  DeveloperId,
  /// Apple Distribution: distribute iOS and macOS apps on the App Store, and ad hoc iOS builds
  Distribution,
  /// Apple Development: run iOS apps on development devices
  Development,
}

impl CertificateType {
  fn api_name(self) -> &'static str {
    match self {
      Self::DeveloperId => "DEVELOPER_ID_APPLICATION",
      Self::Distribution => "DISTRIBUTION",
      Self::Development => "DEVELOPMENT",
    }
  }

  fn file_name(self) -> &'static str {
    match self {
      Self::DeveloperId => "developer-id-application",
      Self::Distribution => "apple-distribution",
      Self::Development => "apple-development",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ProfileType {
  /// App Store Connect distribution (TestFlight and the App Store)
  AppStore,
  /// Ad hoc distribution to registered devices
  AdHoc,
  /// Development builds on registered devices
  Development,
}

impl ProfileType {
  fn api_name(self) -> &'static str {
    match self {
      Self::AppStore => "IOS_APP_STORE",
      Self::AdHoc => "IOS_APP_ADHOC",
      Self::Development => "IOS_APP_DEVELOPMENT",
    }
  }

  fn needs_devices(self) -> bool {
    !matches!(self, Self::AppStore)
  }

  fn certificate_type(self) -> CertificateType {
    match self {
      Self::AppStore | Self::AdHoc => CertificateType::Distribution,
      Self::Development => CertificateType::Development,
    }
  }
}

#[derive(Debug, Parser)]
#[clap(
  about = "Create a signing certificate (and optionally an iOS provisioning profile) with the App Store Connect API",
  long_about = "Create a signing certificate (and optionally an iOS provisioning profile) with the App Store Connect API.\n\nRequires an App Store Connect API team key with the Admin role, created in https://appstoreconnect.apple.com/access/integrations/api. The private key is generated locally and never leaves this machine."
)]
struct CreateCertificateOptions {
  /// Type of the certificate to create.
  /// Defaults to `developer-id`, or the type required by `--profile`
  #[clap(long = "type", value_enum)]
  certificate_type: Option<CertificateType>,
  /// Also create an iOS provisioning profile of this type for the app,
  /// registering its bundle identifier if needed
  #[clap(long, value_enum)]
  profile: Option<ProfileType>,
  /// Bundle identifier of the provisioning profile. Defaults to the app identifier
  #[clap(long, requires = "profile")]
  bundle_id: Option<String>,
  /// Install the certificate in the login keychain and the provisioning profile for Xcode (macOS only)
  #[clap(long)]
  install: bool,
  #[clap(flatten)]
  key: ApiKeyArgs,
  #[clap(flatten)]
  output: CertificateOutputOptions,
}

fn create_certificate(mut options: CreateCertificateOptions) -> Result<()> {
  let certificate_type = match (options.certificate_type, options.profile) {
    (Some(certificate_type), Some(profile)) if certificate_type != profile.certificate_type() => {
      crate::error::bail!(
        "{} provisioning profiles require a `{}` certificate",
        profile.api_name(),
        profile
          .certificate_type()
          .to_possible_value()
          .unwrap()
          .get_name()
      )
    }
    (Some(certificate_type), _) => certificate_type,
    (None, Some(profile)) => profile.certificate_type(),
    (None, None) => CertificateType::DeveloperId,
  };
  if options.profile.is_some() {
    options.output.platform = Platform::Ios;
  }

  let bundle_id = match options.profile {
    Some(_) => Some(
      options
        .bundle_id
        .clone()
        .or_else(app_identifier)
        .context("--bundle-id is required outside of a Tauri project")?,
    ),
    None => None,
  };

  let client = app_store_connect::Client::new(options.key.resolve(options.output.ci)?)?;
  let password = options.output.password()?;

  log::info!(action = "Creating"; "{} certificate", certificate_type.api_name());
  let key = certificate::generate_rsa_private_key().map_err(Box::new)?;
  let csr =
    certificate::create_certificate_signing_request(&key, "Tauri", None).map_err(Box::new)?;
  let created = client.create_certificate(certificate_type.api_name(), &csr)?;
  let content = created
    .attributes
    .certificate_content
    .context("App Store Connect did not return the certificate content")?;
  let cert_der = {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
      .decode(content)
      .context("failed to decode certificate")?
  };
  let cert = certificate::parse_certificate(&cert_der).map_err(Box::new)?;
  let info = CertificateInfo::from_certificate(&cert).map_err(Box::new)?;
  let name = info
    .common_name
    .clone()
    .unwrap_or_else(|| certificate_type.file_name().into());
  let p12 = certificate::create_pkcs12(&cert, &key, &password, &name).map_err(Box::new)?;

  // save the certificate first so it is not lost if creating the profile fails
  let mut secrets = finish_certificate(
    &options.output,
    &info,
    &p12,
    &password,
    &format!("{}.p12", certificate_type.file_name()),
    options.install,
  )?;

  if let (Some(profile_type), Some(bundle_id)) = (options.profile, bundle_id) {
    let profile = create_profile(&client, profile_type, &bundle_id, &created.id)?;
    let profile_path = PathBuf::from(format!(
      "{}-{}.mobileprovision",
      file_stem(&bundle_id),
      file_stem(profile_type.api_name()).to_lowercase()
    ));
    std::fs::write(&profile_path, &profile)
      .fs_context("failed to write provisioning profile", profile_path.clone())?;
    println!(
      "Saved the provisioning profile to {}",
      display_path(&profile_path)
    );
    if options.install {
      install_profile(&profile)?;
    }
    secrets.push(Secret::new(
      "IOS_MOBILE_PROVISION",
      secrets::base64(&profile),
    ));
  }

  options.output.secrets.output(&secrets)
}

fn create_profile(
  client: &app_store_connect::Client,
  profile_type: ProfileType,
  bundle_id: &str,
  certificate_id: &str,
) -> Result<Vec<u8>> {
  let bundle_id_resource = match client.find_bundle_id(bundle_id)? {
    Some(resource) => resource,
    None => {
      log::info!(action = "Registering"; "bundle identifier {bundle_id}");
      let name = bundle_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>();
      client.create_bundle_id(bundle_id, name.trim(), "IOS")?
    }
  };

  let device_ids = if profile_type.needs_devices() {
    let devices = client.list_devices("IOS")?;
    if devices.is_empty() {
      crate::error::bail!(
        "{} provisioning profiles require at least one registered device; register your devices in https://developer.apple.com/account/resources/devices/list",
        profile_type.api_name()
      );
    }
    devices.into_iter().map(|d| d.id).collect()
  } else {
    Vec::new()
  };

  let name = format!(
    "{bundle_id} {} {}",
    profile_type.api_name(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_secs())
      .unwrap_or_default()
  );
  log::info!(action = "Creating"; "provisioning profile {name}");
  let profile = client.create_profile(
    &name,
    profile_type.api_name(),
    &bundle_id_resource.id,
    certificate_id,
    &device_ids,
  )?;

  use base64::Engine;
  base64::engine::general_purpose::STANDARD
    .decode(
      profile
        .attributes
        .profile_content
        .context("App Store Connect did not return the profile content")?,
    )
    .context("failed to decode provisioning profile")
}

#[cfg(target_os = "macos")]
fn install_profile(profile: &[u8]) -> Result<()> {
  let info = ProvisioningProfileInfo::parse(profile).map_err(Box::new)?;
  let dir = dirs::home_dir()
    .context("failed to resolve home directory")?
    .join("Library/MobileDevice/Provisioning Profiles");
  std::fs::create_dir_all(&dir).fs_context("failed to create directory", dir.clone())?;
  let path = dir.join(format!("{}.mobileprovision", info.uuid));
  std::fs::write(&path, profile).fs_context("failed to install provisioning profile", path)?;
  log::info!(action = "Installed"; "the provisioning profile");
  Ok(())
}

#[cfg(not(target_os = "macos"))]
fn install_profile(_profile: &[u8]) -> Result<()> {
  log::warn!("Installing provisioning profiles is only supported on macOS");
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn file_stems() {
    assert_eq!(
      file_stem("Developer ID Application: My Company, Inc. (TEAM1234)"),
      "Developer-ID-Application-My-Company-Inc-TEAM1234"
    );
    assert_eq!(file_stem("IOS_APP_STORE"), "IOS-APP-STORE");
  }
}
