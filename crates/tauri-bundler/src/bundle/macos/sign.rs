// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::{var, var_os},
  ffi::OsString,
  path::PathBuf,
};

use crate::{error::NotarizeAuthError, Entitlements, Settings};

pub struct SignTarget {
  pub path: PathBuf,
  pub is_an_executable: bool,
}

pub fn keychain(identity: Option<&str>) -> crate::Result<Option<tauri_macos_sign::Keychain>> {
  if let (Some(certificate_encoded), Some(certificate_password)) = (
    var_os("APPLE_CERTIFICATE"),
    var_os("APPLE_CERTIFICATE_PASSWORD"),
  ) {
    // import user certificate - useful for for CI build
    let keychain =
      tauri_macos_sign::Keychain::with_certificate(&certificate_encoded, &certificate_password)
        .map_err(Box::new)?;
    if let Some(identity) = identity {
      let certificate_identity = keychain.signing_identity();
      if !certificate_identity.contains(identity) {
        return Err(crate::Error::GenericError(format!(
          "certificate from APPLE_CERTIFICATE \"{certificate_identity}\" environment variable does not match provided identity \"{identity}\""
        )));
      }
    }
    Ok(Some(keychain))
  } else if let Some(identity) = identity {
    Ok(Some(tauri_macos_sign::Keychain::with_signing_identity(
      identity,
    )))
  } else {
    Ok(None)
  }
}

pub fn sign(
  keychain: &tauri_macos_sign::Keychain,
  targets: Vec<SignTarget>,
  settings: &Settings,
) -> crate::Result<()> {
  log::info!(action = "Signing"; "with identity \"{}\"", keychain.signing_identity());

  for target in targets {
    let (entitlements_path, _temp_file) = match settings.macos().entitlements.as_ref() {
      Some(Entitlements::Path(path)) => (Some(path.to_owned()), None),
      Some(Entitlements::Plist(plist)) => {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        plist::to_writer_xml(temp_file.as_file_mut(), &plist)?;
        (Some(temp_file.path().to_path_buf()), Some(temp_file))
      }
      None => (None, None),
    };

    keychain
      .sign(
        &target.path,
        entitlements_path.as_deref(),
        target.is_an_executable && settings.macos().hardened_runtime,
      )
      .map_err(Box::new)?;
  }

  Ok(())
}

pub fn notarize(
  keychain: &tauri_macos_sign::Keychain,
  app_bundle_path: PathBuf,
  credentials: &tauri_macos_sign::AppleNotarizationCredentials,
) -> crate::Result<()> {
  tauri_macos_sign::notarize(keychain, &app_bundle_path, credentials)
    .map_err(Box::new)
    .map_err(Into::into)
}

pub fn notarize_without_stapling(
  keychain: &tauri_macos_sign::Keychain,
  app_bundle_path: PathBuf,
  credentials: &tauri_macos_sign::AppleNotarizationCredentials,
) -> crate::Result<()> {
  tauri_macos_sign::notarize_without_stapling(keychain, &app_bundle_path, credentials)
    .map_err(Box::new)
    .map_err(Into::into)
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
      match (
        var_os("APPLE_API_KEY"),
        var_os("APPLE_API_ISSUER"),
        var("APPLE_API_KEY_PATH"),
      ) {
        (Some(key_id), Some(issuer), Ok(key_path)) => {
          Ok(tauri_macos_sign::AppleNotarizationCredentials::ApiKey {
            key_id,
            key: tauri_macos_sign::ApiKey::Path(key_path.into()),
            issuer,
          })
        }
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
            Ok(tauri_macos_sign::AppleNotarizationCredentials::ApiKey {
              key_id,
              key: tauri_macos_sign::ApiKey::Path(key_path),
              issuer,
            })
          } else {
            Err(NotarizeAuthError::MissingApiKey {
              file_name: api_key_file_name.to_string_lossy().into_owned(),
            })
          }
        }
        _ => Err(NotarizeAuthError::MissingCredentials),
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
