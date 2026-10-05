// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::{var, var_os},
  ffi::OsString,
  path::PathBuf,
};

use crate::{Entitlements, Settings, bundle::MacOsSettings, error::NotarizeAuthError};

/// Which entitlements a sign target gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignEntitlements {
  /// No entitlements: libraries, frameworks, disk images.
  None,
  /// The app entitlements (`bundle > macOS > entitlements`).
  App,
  /// The sidecar entitlements (`bundle > macOS > sidecarEntitlements`),
  /// falling back to the app entitlements when unset.
  Sidecar,
}

pub struct SignTarget {
  pub path: PathBuf,
  pub is_an_executable: bool,
  pub entitlements: SignEntitlements,
}

fn target_entitlements(kind: SignEntitlements, macos: &MacOsSettings) -> Option<&Entitlements> {
  match kind {
    SignEntitlements::None => None,
    SignEntitlements::App => macos.entitlements.as_ref(),
    SignEntitlements::Sidecar => macos
      .sidecar_entitlements
      .as_ref()
      .or(macos.entitlements.as_ref()),
  }
}

pub fn keychain(identity: Option<&str>) -> crate::Result<Option<tauri_macos_sign::Keychain>> {
  if let (Some(certificate_encoded), Some(certificate_password)) = (
    var_os("APPLE_CERTIFICATE"),
    var_os("APPLE_CERTIFICATE_PASSWORD"),
  ) {
    // import user certificate - useful for CI build
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
    let entitlements = target_entitlements(target.entitlements, settings.macos());
    let (entitlements_path, _temp_file) = match entitlements {
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
  if path.exists() { Some(path) } else { None }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn path(entitlements: Option<&Entitlements>) -> Option<&str> {
    match entitlements {
      Some(Entitlements::Path(path)) => path.to_str(),
      _ => None,
    }
  }

  #[test]
  fn sidecar_falls_back_to_app_entitlements() {
    let macos = MacOsSettings {
      entitlements: Some(Entitlements::Path("app.plist".into())),
      ..Default::default()
    };
    assert_eq!(
      path(target_entitlements(SignEntitlements::App, &macos)),
      Some("app.plist")
    );
    assert_eq!(
      path(target_entitlements(SignEntitlements::Sidecar, &macos)),
      Some("app.plist")
    );
    assert!(target_entitlements(SignEntitlements::None, &macos).is_none());
  }

  #[test]
  fn sidecar_uses_its_own_entitlements_when_set() {
    let macos = MacOsSettings {
      entitlements: Some(Entitlements::Path("app.plist".into())),
      sidecar_entitlements: Some(Entitlements::Path("sidecar.plist".into())),
      ..Default::default()
    };
    assert_eq!(
      path(target_entitlements(SignEntitlements::App, &macos)),
      Some("app.plist")
    );
    assert_eq!(
      path(target_entitlements(SignEntitlements::Sidecar, &macos)),
      Some("sidecar.plist")
    );
    assert!(target_entitlements(SignEntitlements::None, &macos).is_none());
  }
}
