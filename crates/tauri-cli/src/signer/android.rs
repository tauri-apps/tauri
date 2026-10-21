// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::{Path, PathBuf},
  process::Command,
};

use clap::{Parser, Subcommand};
use tauri_utils::display_path;

use super::secrets::{self, Secret, SecretsOptions};
use crate::{
  Result,
  error::{Context, ErrorExt},
  mobile::android::signing::{self, GradleSigning, KeystoreProperties},
};

#[derive(Parser)]
#[clap(
  about = "Create and configure the upload key used to sign Android release builds",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  GenerateKeystore(GenerateOptions),
  ConfigureKeystore(ConfigureOptions),
}

#[derive(Debug, Parser)]
#[clap(
  about = "Generate a new upload keystore and configure the Android project to sign release builds with it"
)]
pub struct GenerateOptions {
  /// Path of the keystore file to create.
  /// Defaults to `~/.android/<identifier>-upload.jks`.
  #[clap(short, long)]
  keystore: Option<PathBuf>,
  /// Alias of the upload key in the keystore
  #[clap(short, long, default_value = "upload")]
  alias: String,
  /// Password of the keystore. Prompted when not provided, or randomly generated with `--ci`
  #[clap(short, long, env = "ANDROID_KEYSTORE_PASSWORD")]
  password: Option<String>,
  /// Distinguished name of the certificate, such as `CN=My Name, O=My Company, C=US`.
  /// Defaults to the product name as the common name.
  #[clap(long)]
  dname: Option<String>,
  /// Number of days the key is valid for. Google Play requires keys valid until at least 2033
  #[clap(long, default_value_t = 10000)]
  validity: u32,
  /// Overwrite the keystore if it already exists
  #[clap(short, long)]
  force: bool,
  /// Only generate the keystore, do not configure the Android project
  #[clap(long)]
  skip_project: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  #[clap(flatten)]
  secrets: SecretsOptions,
}

#[derive(Debug, Parser)]
#[clap(about = "Configure the Android project to sign release builds with an existing keystore")]
pub struct ConfigureOptions {
  /// Path of the existing keystore file
  #[clap(short, long)]
  keystore: PathBuf,
  /// Alias of the upload key in the keystore
  #[clap(short, long, default_value = "upload")]
  alias: String,
  /// Password of the keystore. Prompted when not provided
  #[clap(short, long, env = "ANDROID_KEYSTORE_PASSWORD")]
  password: Option<String>,
  /// Password of the upload key, when it differs from the keystore password
  #[clap(long, env = "ANDROID_KEY_PASSWORD")]
  key_password: Option<String>,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  #[clap(flatten)]
  secrets: SecretsOptions,
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::GenerateKeystore(options) => generate(options),
    Commands::ConfigureKeystore(options) => configure(options),
  }
}

struct Project {
  identifier: String,
  product_name: Option<String>,
  /// `gen/android`, when the Android project was initialized.
  android_dir: Option<PathBuf>,
}

fn project() -> Option<Project> {
  let tauri_dir = crate::helpers::app_paths::resolve_tauri_dir()?;
  let config =
    crate::helpers::config::get_config(tauri_utils::platform::Target::Android, &[], &tauri_dir)
      .ok()?;
  let android_dir = tauri_dir.join("gen").join("android");
  Some(Project {
    identifier: config.identifier.clone(),
    product_name: config.product_name.clone(),
    android_dir: android_dir.exists().then_some(android_dir),
  })
}

fn generate(options: GenerateOptions) -> Result<()> {
  let project = project();

  let keystore = match options.keystore {
    Some(path) => path,
    None => {
      let name = project
        .as_ref()
        .map(|p| format!("{}-upload.jks", p.identifier))
        .unwrap_or_else(|| "upload.jks".into());
      dirs::home_dir()
        .context("failed to resolve home directory")?
        .join(".android")
        .join(name)
    }
  };

  if keystore.exists() && !options.force {
    crate::error::bail!(
      "{} already exists; use `tauri signer android configure-keystore --keystore {}` to use it or pass --force to overwrite it",
      display_path(&keystore),
      display_path(&keystore)
    );
  }
  if let Some(parent) = keystore.parent() {
    std::fs::create_dir_all(parent).fs_context("failed to create directory", parent)?;
  }
  let password = secrets::password(options.password, "Keystore password", true, options.ci)?;
  if password.len() < 6 {
    crate::error::bail!("the keystore password must be at least 6 characters long");
  }

  let dname = options.dname.unwrap_or_else(|| {
    let name = project
      .as_ref()
      .and_then(|p| p.product_name.clone())
      .unwrap_or_else(|| "Tauri App".into());
    format!("CN={}", name.replace(',', "\\,"))
  });

  let keytool = keytool()?;
  // only remove the existing keystore (--force) once everything is ready
  if keystore.exists() {
    std::fs::remove_file(&keystore).fs_context("failed to remove keystore", keystore.clone())?;
  }
  log::info!(action = "Generating"; "upload key {}", display_path(&keystore));
  let output = Command::new(&keytool)
    .args(["-genkeypair", "-noprompt", "-storetype", "PKCS12"])
    .args(["-keyalg", "RSA", "-keysize", "2048"])
    .arg("-validity")
    .arg(options.validity.to_string())
    .arg("-alias")
    .arg(&options.alias)
    .arg("-dname")
    .arg(&dname)
    .arg("-keystore")
    .arg(&keystore)
    // read the passwords from the environment so they do not show up in the process list
    .args([
      "-storepass:env",
      "TAURI_KEYTOOL_PASSWORD",
      "-keypass:env",
      "TAURI_KEYTOOL_PASSWORD",
    ])
    .env("TAURI_KEYTOOL_PASSWORD", &password)
    .output()
    .map_err(|error| crate::Error::CommandFailed {
      command: "keytool -genkeypair".into(),
      error,
    })?;
  if !output.status.success() {
    crate::error::bail!(
      "failed to generate keystore: {}",
      String::from_utf8_lossy(&output.stderr).trim()
    );
  }

  // keystore.properties paths are resolved relative to the Android project
  let keystore =
    dunce::canonicalize(&keystore).fs_context("failed to resolve keystore path", keystore)?;

  println!();
  println!("Your upload key was generated: {}", display_path(&keystore));
  println!(
    "Back it up in a safe place (keep it secret!). If you lose it or its password, you won't be able to publish updates unless you ask Google Play to reset your upload key."
  );

  let properties = KeystoreProperties {
    store_file: keystore,
    password,
    key_alias: options.alias,
    key_password: None,
  };

  if !options.skip_project {
    configure_project(project.as_ref(), &properties)?;
  }

  output_secrets(&properties, &options.secrets)
}

fn configure(options: ConfigureOptions) -> Result<()> {
  let keystore = dunce::canonicalize(&options.keystore)
    .fs_context("failed to read keystore", options.keystore.clone())?;
  if options.ci && options.password.is_none() {
    crate::error::bail!(
      "--password (or ANDROID_KEYSTORE_PASSWORD) is required in non-interactive mode"
    );
  }
  let password = secrets::password(options.password, "Keystore password", false, false)?;

  if let Ok(keytool) = keytool() {
    verify_keystore(&keytool, &keystore, &password, &options.alias)?;
  } else {
    log::warn!("keytool not found, skipping keystore verification");
  }

  let properties = KeystoreProperties {
    store_file: keystore,
    password,
    key_alias: options.alias,
    key_password: options.key_password,
  };

  configure_project(project().as_ref(), &properties)?;
  output_secrets(&properties, &options.secrets)
}

fn verify_keystore(keytool: &Path, keystore: &Path, password: &str, alias: &str) -> Result<()> {
  let output = Command::new(keytool)
    .args(["-list", "-keystore"])
    .arg(keystore)
    .arg("-alias")
    .arg(alias)
    .args(["-storepass:env", "TAURI_KEYTOOL_PASSWORD"])
    .env("TAURI_KEYTOOL_PASSWORD", password)
    .output()
    .map_err(|error| crate::Error::CommandFailed {
      command: "keytool -list".into(),
      error,
    })?;
  if !output.status.success() {
    let message = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    crate::error::bail!(
      "failed to read key `{alias}` from {}: {}",
      display_path(keystore),
      format!("{} {}", message.trim(), stderr.trim()).trim()
    );
  }
  Ok(())
}

fn configure_project(project: Option<&Project>, properties: &KeystoreProperties) -> Result<()> {
  let Some(android_dir) = project.and_then(|p| p.android_dir.as_deref()) else {
    log::warn!(
      "Android project not found; run `tauri android init` and then `tauri signer android configure-keystore --keystore {}` to configure it",
      display_path(&properties.store_file)
    );
    return Ok(());
  };

  let properties_path = properties.write(android_dir)?;
  log::info!(action = "Writing"; "{}", display_path(&properties_path));

  match signing::ensure_release_signing_config(android_dir)? {
    GradleSigning::Added => log::info!(
      action = "Updating"; "{} to sign release builds",
      display_path(signing::app_build_gradle_path(android_dir))
    ),
    GradleSigning::Configured => {}
    GradleSigning::Custom => log::warn!(
      "{} already defines a release signing config; update it to read {} or remove it to use the generated configuration",
      display_path(signing::app_build_gradle_path(android_dir)),
      signing::KEYSTORE_PROPERTIES
    ),
  }

  println!("Release builds are now signed with this key.");
  Ok(())
}

fn output_secrets(properties: &KeystoreProperties, options: &SecretsOptions) -> Result<()> {
  let keystore = std::fs::read(&properties.store_file)
    .fs_context("failed to read keystore", properties.store_file.clone())?;
  let mut values = vec![
    Secret::new("ANDROID_KEYSTORE", secrets::base64(&keystore)),
    Secret::new("ANDROID_KEYSTORE_PASSWORD", properties.password.clone()),
    Secret::new("ANDROID_KEY_ALIAS", properties.key_alias.clone()),
  ];
  if let Some(key_password) = &properties.key_password {
    values.push(Secret::new("ANDROID_KEY_PASSWORD", key_password.clone()));
  }
  options.output(&values)
}

/// Finds the `keytool` executable from the JDK, preferring the one bundled with Android Studio.
fn keytool() -> Result<PathBuf> {
  let exe = if cfg!(windows) {
    "keytool.exe"
  } else {
    "keytool"
  };

  let mut java_homes = Vec::new();
  if let Some(java_home) = std::env::var_os("JAVA_HOME") {
    java_homes.push(PathBuf::from(java_home));
  }
  #[cfg(windows)]
  java_homes.push(PathBuf::from(
    "C:\\Program Files\\Android\\Android Studio\\jbr",
  ));
  #[cfg(target_os = "macos")]
  java_homes.push(PathBuf::from(
    "/Applications/Android Studio.app/Contents/jbr/Contents/Home",
  ));
  #[cfg(target_os = "linux")]
  java_homes.push(PathBuf::from("/opt/android-studio/jbr"));

  java_homes
    .into_iter()
    .map(|home| home.join("bin").join(exe))
    .find(|path| path.exists())
    .or_else(|| which::which("keytool").ok())
    .context(
      "keytool not found; install a JDK (Android Studio bundles one) or set the JAVA_HOME environment variable",
    )
}
