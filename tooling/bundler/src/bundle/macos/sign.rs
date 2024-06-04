// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::{var, var_os},
  ffi::OsString,
  path::{Path, PathBuf},
};

use crate::Settings;

pub struct SignTarget {
  pub path: PathBuf,
  pub is_an_executable: bool,
}

pub fn sign(
  targets: Vec<SignTarget>,
  identity: &str,
  settings: &Settings,
) -> crate::Result<tauri_macos_sign::Keychain> {
  log::info!(action = "Signing"; "with identity \"{}\"", identity);

  let keychain = if let (Some(certificate_encoded), Some(certificate_password)) = (
    var_os("APPLE_CERTIFICATE"),
    var_os("APPLE_CERTIFICATE_PASSWORD"),
  ) {
    // setup keychain allow you to import your certificate
    // for CI build
    tauri_macos_sign::Keychain::with_certificate(&certificate_encoded, &certificate_password)?
  } else {
    tauri_macos_sign::Keychain::with_signing_identity(identity)
  };

  log::info!("Signing app bundle...");

  for target in targets {
    keychain.sign(
      &target.path,
      settings.macos().entitlements.as_ref().map(Path::new),
      target.is_an_executable,
    )?;
  }

  Ok(keychain)
}

pub fn notarize(
  keychain: &tauri_macos_sign::Keychain,
  app_bundle_path: PathBuf,
  credentials: &tauri_macos_sign::AppleNotarizationCredentials,
) -> crate::Result<()> {
  tauri_macos_sign::notarize(keychain, &app_bundle_path, credentials).map_err(Into::into)
}

#[derive(Debug, thiserror::Error)]
pub enum NotarizeAuthError {
  #[error(
    "The team ID is now required for notarization with app-specific password as authentication. Please set the `APPLE_TEAM_ID` environment variable. You can find the team ID in https://developer.apple.com/account#MembershipDetailsCard."
  )]
  MissingTeamId,
  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),
}

pub fn notarize_auth() -> Result<tauri_macos_sign::AppleNotarizationCredentials, NotarizeAuthError>
{
  match (
    var_os("APPLE_ID"),
    var_os("APPLE_PASSWORD"),
    var_os("APPLE_TEAM_ID"),
  ) {
    (Some(apple_id), Some(password), Some(team_id)) => {
      Ok(tauri_macos_sign::AppleNotarizationCredentials::AppleId {
        apple_id,
        password,
        team_id,
      })
    }
    (Some(_apple_id), Some(_password), None) => Err(NotarizeAuthError::MissingTeamId),
    _ => {
      match (var_os("APPLE_API_KEY"), var_os("APPLE_API_ISSUER"), var("APPLE_API_KEY_PATH")) {
        (Some(key_id), Some(issuer), Ok(key_path)) => {
          Ok(tauri_macos_sign::AppleNotarizationCredentials::ApiKey { key_id, key: tauri_macos_sign::ApiKey::Path( key_path.into()), issuer })
        },
        (Some(key_id), Some(issuer), Err(_)) => {
          let mut api_key_file_name = OsString::from("AuthKey_");
          api_key_file_name.push(&key_id);
          api_key_file_name.push(".p8");
          let mut key_path = None;

          let mut search_paths = vec!["./private_keys".into()];
          if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(home_dir.join("private_keys"));
            search_paths.push(home_dir.join(".private_keys"));
            search_paths.push(home_dir.join(".appstoreconnect").join("private_keys"));
          }

          for folder in search_paths {
            if let Some(path) = find_api_key(folder, &api_key_file_name) {
              key_path = Some(path);
              break;
            }
          }

          if let Some(key_path) = key_path {
          Ok(tauri_macos_sign::AppleNotarizationCredentials::ApiKey { key_id, key: tauri_macos_sign::ApiKey::Path(key_path), issuer })
          } else {
            Err(anyhow::anyhow!("could not find API key file. Please set the APPLE_API_KEY_PATH environment variables to the path to the {api_key_file_name:?} file").into())
          }
        }
        _ => Err(anyhow::anyhow!("no APPLE_ID & APPLE_PASSWORD & APPLE_TEAM_ID or APPLE_API_KEY & APPLE_API_ISSUER & APPLE_API_KEY_PATH environment variables found").into())
      }
    }
  }
}

fn find_api_key(folder: PathBuf, file_name: &OsString) -> Option<PathBuf> {
  let path = folder.join(file_name);
  if path.exists() {
    Some(path)
  } else {
    None
  }
}
