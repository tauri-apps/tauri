// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// A macOS application bundle package is laid out like:
//
// foobar.app    # Actually a directory
//     Contents      # A further subdirectory
//         Info.plist     # An xml file containing the app's metadata
//         MacOS          # A directory to hold executable binary files
//             foobar          # The main binary executable of the app
//             foobar_helper   # A helper application, possibly providing a CLI
//         Resources      # Data files such as images, sounds, translations and nib files
//             en.lproj        # Folder containing english translation strings/data
//         Frameworks     # A directory containing private frameworks (shared libraries)
//         ...            # Any other optional files the developer wants to place here
//
// See https://developer.apple.com/go/?id=bundle-structure for a full
// explanation.
//
// Currently, cargo-bundle does not support Frameworks, nor does it support placing arbitrary
// files into the `Contents` directory of the bundle.

use super::{
  icon::create_icns_file,
  sign::{notarize, notarize_auth, notarize_without_stapling, sign, SignTarget},
};
use crate::{
  bundle::settings::PlistKind,
  error::{Context, ErrorExt, NotarizeAuthError},
  utils::{fs_utils, CommandExt},
  Error::GenericError,
  Settings,
};

use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

const NESTED_CODE_FOLDER: [&str; 6] = [
  "MacOS",
  "Frameworks",
  "Plugins",
  "Helpers",
  "XPCServices",
  "Libraries",
];

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the .app was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // we should use the bundle name (App name) as a MacOS standard.
  // version or platform shouldn't be included in the App name.
  let app_product_name = format!("{}.app", settings.product_name());

  let app_bundle_path = settings
    .project_out_directory()
    .join("bundle/macos")
    .join(&app_product_name);

  log::info!(action = "Bundling"; "{} ({})", app_product_name, app_bundle_path.display());

  if app_bundle_path.exists() {
    fs::remove_dir_all(&app_bundle_path)
      .fs_context("failed to remove old app bundle", &app_bundle_path)?;
  }
  let bundle_directory = app_bundle_path.join("Contents");
  fs::create_dir_all(&bundle_directory)
    .fs_context("failed to create bundle directory", &bundle_directory)?;

  let resources_dir = bundle_directory.join("Resources");
  let bin_dir = bundle_directory.join("MacOS");
  let mut sign_paths = Vec::new();

  let bundle_icon_file: Option<PathBuf> =
    { create_icns_file(&resources_dir, settings).with_context(|| "Failed to create app icon")? };

  create_info_plist(&bundle_directory, bundle_icon_file, settings)
    .with_context(|| "Failed to create Info.plist")?;

  let framework_paths = copy_frameworks_to_bundle(&bundle_directory, settings)
    .with_context(|| "Failed to bundle frameworks")?;
  sign_paths.extend(framework_paths);

  settings.copy_resources(&resources_dir)?;

  let bin_paths = settings
    .copy_binaries(&bin_dir)
    .with_context(|| "Failed to copy external binaries")?;
  sign_paths.extend(bin_paths.into_iter().map(|path| SignTarget {
    path,
    is_an_executable: true,
  }));

  let bin_paths = copy_binaries_to_bundle(&bundle_directory, settings)?;
  sign_paths.extend(bin_paths.into_iter().map(|path| SignTarget {
    path,
    is_an_executable: true,
  }));

  copy_custom_files_to_bundle(&bundle_directory, settings)?;

  if settings.no_sign() {
    log::warn!("Skipping signing due to --no-sign flag.",);
  } else if let Some(keychain) =
    super::sign::keychain(settings.macos().signing_identity.as_deref())?
  {
    // Sign frameworks and sidecar binaries first, per apple, signing must be done inside out
    // https://developer.apple.com/forums/thread/701514
    sign_paths.push(SignTarget {
      path: app_bundle_path.clone(),
      is_an_executable: true,
    });

    // Remove extra attributes, which could cause codesign to fail
    // https://developer.apple.com/library/archive/qa/qa1940/_index.html
    remove_extra_attr(&app_bundle_path)?;

    // sign application
    sign(&keychain, sign_paths, settings)?;

    // notarization is required for distribution
    match notarize_auth() {
      Ok(auth) => {
        if settings.macos().skip_stapling {
          notarize_without_stapling(&keychain, app_bundle_path.clone(), &auth)?;
        } else {
          notarize(&keychain, app_bundle_path.clone(), &auth)?;
        }
      }
      Err(e) => {
        if matches!(e, NotarizeAuthError::MissingTeamId) {
          return Err(e.into());
        } else {
          log::warn!("skipping app notarization, {}", e.to_string());
        }
      }
    }
  }

  Ok(vec![app_bundle_path])
}

fn remove_extra_attr(app_bundle_path: &Path) -> crate::Result<()> {
  Command::new("xattr")
    .arg("-crs")
    .arg(app_bundle_path)
    .output_ok()
    .context("failed to remove extra attributes from app bundle")?;
  Ok(())
}

// Copies the app's binaries to the bundle.
fn copy_binaries_to_bundle(
  bundle_directory: &Path,
  settings: &Settings,
) -> crate::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  let dest_dir = bundle_directory.join("MacOS");
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    let dest_path = dest_dir.join(bin.name());
    fs_utils::copy_file(&bin_path, &dest_path)
      .with_context(|| format!("Failed to copy binary from {bin_path:?}"))?;
    paths.push(dest_path);
  }
  Ok(paths)
}

/// Copies user-defined files to the app under Contents.
fn copy_custom_files_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  for (contents_path, path) in settings.macos().files.iter() {
    if !path.try_exists()? {
      return Err(GenericError(format!(
        "Failed to copy {path:?} to {contents_path:?}. {path:?} does not exist."
      )));
    }

    let contents_path = if contents_path.is_absolute() {
      contents_path.strip_prefix("/").unwrap()
    } else {
      contents_path
    };
    if path.is_file() {
      fs_utils::copy_file(path, &bundle_directory.join(contents_path))
        .with_context(|| format!("Failed to copy file {path:?} to {contents_path:?}"))?;
    } else if path.is_dir() {
      fs_utils::copy_dir(path, &bundle_directory.join(contents_path))
        .with_context(|| format!("Failed to copy directory {path:?} to {contents_path:?}"))?;
    } else {
      return Err(GenericError(format!(
        "{path:?} is not a file or directory."
      )));
    }
  }
  Ok(())
}

// Creates the Info.plist file.
fn create_info_plist(
  bundle_dir: &Path,
  bundle_icon_file: Option<PathBuf>,
  settings: &Settings,
) -> crate::Result<()> {
  let mut plist = plist::Dictionary::new();
  plist.insert("CFBundleDevelopmentRegion".into(), "English".into());
  plist.insert("CFBundleDisplayName".into(), settings.product_name().into());
  plist.insert(
    "CFBundleExecutable".into(),
    settings.main_binary_name()?.into(),
  );
  if let Some(path) = bundle_icon_file {
    plist.insert(
      "CFBundleIconFile".into(),
      path
        .file_name()
        .expect("No file name")
        .to_string_lossy()
        .into_owned()
        .into(),
    );
  }
  plist.insert(
    "CFBundleIdentifier".into(),
    settings.bundle_identifier().into(),
  );
  plist.insert("CFBundleInfoDictionaryVersion".into(), "6.0".into());
  if let Some(bundle_name) = settings
    .macos()
    .bundle_name
    .as_deref()
    .unwrap_or_else(|| settings.product_name())
    .into()
  {
    plist.insert("CFBundleName".into(), bundle_name.into());
  }
  plist.insert("CFBundlePackageType".into(), "APPL".into());
  plist.insert(
    "CFBundleShortVersionString".into(),
    settings.version_string().into(),
  );
  plist.insert(
    "CFBundleVersion".into(),
    settings
      .macos()
      .bundle_version
      .as_deref()
      .unwrap_or_else(|| settings.version_string())
      .into(),
  );
  plist.insert("CSResourcesFileMapped".into(), true.into());
  if let Some(category) = settings.app_category() {
    plist.insert(
      "LSApplicationCategoryType".into(),
      category.macos_application_category_type().into(),
    );
  }
  if let Some(version) = settings.macos().minimum_system_version.clone() {
    plist.insert("LSMinimumSystemVersion".into(), version.into());
  }

  if let Some(associations) = settings.file_associations() {
    let exported_associations = associations
      .iter()
      .filter_map(|association| {
        association.exported_type.as_ref().map(|exported_type| {
          let mut dict = plist::Dictionary::new();

          dict.insert(
            "UTTypeIdentifier".into(),
            exported_type.identifier.clone().into(),
          );
          if let Some(description) = &association.description {
            dict.insert("UTTypeDescription".into(), description.clone().into());
          }
          if let Some(conforms_to) = &exported_type.conforms_to {
            dict.insert(
              "UTTypeConformsTo".into(),
              plist::Value::Array(conforms_to.iter().map(|s| s.clone().into()).collect()),
            );
          }

          let mut specification = plist::Dictionary::new();
          specification.insert(
            "public.filename-extension".into(),
            plist::Value::Array(
              association
                .ext
                .iter()
                .map(|s| s.to_string().into())
                .collect(),
            ),
          );
          if let Some(mime_type) = &association.mime_type {
            specification.insert("public.mime-type".into(), mime_type.clone().into());
          }

          dict.insert("UTTypeTagSpecification".into(), specification.into());

          plist::Value::Dictionary(dict)
        })
      })
      .collect::<Vec<_>>();

    if !exported_associations.is_empty() {
      plist.insert(
        "UTExportedTypeDeclarations".into(),
        plist::Value::Array(exported_associations),
      );
    }

    plist.insert(
      "CFBundleDocumentTypes".into(),
      plist::Value::Array(
        associations
          .iter()
          .map(|association| {
            let mut dict = plist::Dictionary::new();

            if !association.ext.is_empty() {
              dict.insert(
                "CFBundleTypeExtensions".into(),
                plist::Value::Array(
                  association
                    .ext
                    .iter()
                    .map(|ext| ext.to_string().into())
                    .collect(),
                ),
              );
            }

            if let Some(content_types) = &association.content_types {
              dict.insert(
                "LSItemContentTypes".into(),
                plist::Value::Array(content_types.iter().map(|s| s.to_string().into()).collect()),
              );
            }

            dict.insert(
              "CFBundleTypeName".into(),
              association
                .name
                .as_ref()
                .unwrap_or(&association.ext[0].0)
                .to_string()
                .into(),
            );
            dict.insert(
              "CFBundleTypeRole".into(),
              association.role.to_string().into(),
            );
            dict.insert("LSHandlerRank".into(), association.rank.to_string().into());
            plist::Value::Dictionary(dict)
          })
          .collect(),
      ),
    );
  }

  if let Some(protocols) = settings.deep_link_protocols() {
    plist.insert(
      "CFBundleURLTypes".into(),
      plist::Value::Array(
        protocols
          .iter()
          .filter(|p| !p.schemes.is_empty())
          .map(|protocol| {
            let mut dict = plist::Dictionary::new();
            dict.insert(
              "CFBundleURLSchemes".into(),
              plist::Value::Array(
                protocol
                  .schemes
                  .iter()
                  .map(|s| s.to_string().into())
                  .collect(),
              ),
            );
            dict.insert(
              "CFBundleURLName".into(),
              protocol
                .name
                .clone()
                .unwrap_or(format!(
                  "{} {}",
                  settings.bundle_identifier(),
                  protocol.schemes[0]
                ))
                .into(),
            );
            dict.insert("CFBundleTypeRole".into(), protocol.role.to_string().into());
            plist::Value::Dictionary(dict)
          })
          .collect(),
      ),
    );
  }

  plist.insert("LSRequiresCarbon".into(), true.into());
  plist.insert("NSHighResolutionCapable".into(), true.into());
  if let Some(copyright) = settings.copyright_string() {
    plist.insert("NSHumanReadableCopyright".into(), copyright.into());
  }

  if let Some(exception_domain) = settings.macos().exception_domain.clone() {
    let mut security = plist::Dictionary::new();
    let mut domain = plist::Dictionary::new();
    domain.insert("NSExceptionAllowsInsecureHTTPLoads".into(), true.into());
    domain.insert("NSIncludesSubdomains".into(), true.into());

    let mut exception_domains = plist::Dictionary::new();
    exception_domains.insert(exception_domain, domain.into());
    security.insert("NSExceptionDomains".into(), exception_domains.into());
    plist.insert("NSAppTransportSecurity".into(), security.into());
  }

  if let Some(user_plist) = &settings.macos().info_plist {
    let user_plist = match user_plist {
      PlistKind::Path(path) => plist::Value::from_file(path)?,
      PlistKind::Plist(value) => value.clone(),
    };
    if let Some(dict) = user_plist.into_dictionary() {
      for (key, value) in dict {
        plist.insert(key, value);
      }
    }
  }

  plist::Value::Dictionary(plist).to_file_xml(bundle_dir.join("Info.plist"))?;

  Ok(())
}

// Copies the framework under `{src_dir}/{framework}.framework` to `{dest_dir}/{framework}.framework`.
fn copy_framework_from(dest_dir: &Path, framework: &str, src_dir: &Path) -> crate::Result<bool> {
  let src_name = format!("{framework}.framework");
  let src_path = src_dir.join(&src_name);
  if src_path.exists() {
    fs_utils::copy_dir(&src_path, &dest_dir.join(&src_name))?;
    Ok(true)
  } else {
    Ok(false)
  }
}

// Copies the macOS application bundle frameworks to the .app
fn copy_frameworks_to_bundle(
  bundle_directory: &Path,
  settings: &Settings,
) -> crate::Result<Vec<SignTarget>> {
  let mut paths = Vec::new();

  let frameworks = settings.macos().frameworks.clone().unwrap_or_default();
  if frameworks.is_empty() {
    return Ok(paths);
  }
  let dest_dir = bundle_directory.join("Frameworks");
  fs::create_dir_all(&dest_dir).fs_context("failed to create Frameworks directory", &dest_dir)?;
  for framework in frameworks.iter() {
    if framework.ends_with(".framework") {
      let src_path = PathBuf::from(framework);
      let src_name = src_path
        .file_name()
        .expect("Couldn't get framework filename");
      let dest_path = dest_dir.join(src_name);
      fs_utils::copy_dir(&src_path, &dest_path)?;
      add_framework_sign_path(&src_path, &dest_path, &mut paths);
      continue;
    } else if framework.ends_with(".dylib") {
      let src_path = PathBuf::from(framework);
      if !src_path.exists() {
        return Err(GenericError(format!("Library not found: {framework}")));
      }
      let src_name = src_path.file_name().expect("Couldn't get library filename");
      let dest_path = dest_dir.join(src_name);
      fs_utils::copy_file(&src_path, &dest_path)?;
      paths.push(SignTarget {
        path: dest_path,
        is_an_executable: false,
      });
      continue;
    } else if framework.contains('/') {
      return Err(GenericError(format!(
        "Framework path should have .framework extension: {framework}"
      )));
    }
    if let Some(home_dir) = dirs::home_dir() {
      if copy_framework_from(&dest_dir, framework, &home_dir.join("Library/Frameworks/"))? {
        continue;
      }
    }
    if copy_framework_from(&dest_dir, framework, &PathBuf::from("/Library/Frameworks/"))?
      || copy_framework_from(
        &dest_dir,
        framework,
        &PathBuf::from("/Network/Library/Frameworks/"),
      )?
    {
      continue;
    }
    return Err(GenericError(format!(
      "Could not locate framework: {framework}"
    )));
  }
  Ok(paths)
}

/// Recursively add framework's sign paths.
/// If the framework has multiple versions, it will sign "Current" version by default.
fn add_framework_sign_path(
  framework_root: &Path,
  dest_path: &Path,
  sign_paths: &mut Vec<SignTarget>,
) {
  if framework_root.join("Versions/Current").exists() {
    add_nested_code_sign_path(
      &framework_root.join("Versions/Current"),
      &dest_path.join("Versions/Current"),
      sign_paths,
    );
  } else {
    add_nested_code_sign_path(framework_root, dest_path, sign_paths);
  }
  sign_paths.push(SignTarget {
    path: dest_path.into(),
    is_an_executable: false,
  });
}

/// Recursively add executable bundle's sign path (.xpc, .app).
fn add_executable_bundle_sign_path(
  bundle_root: &Path,
  dest_path: &Path,
  sign_paths: &mut Vec<SignTarget>,
) {
  if bundle_root.join("Contents").exists() {
    add_nested_code_sign_path(
      &bundle_root.join("Contents"),
      &dest_path.join("Contents"),
      sign_paths,
    );
  } else {
    add_nested_code_sign_path(bundle_root, dest_path, sign_paths);
  }
  sign_paths.push(SignTarget {
    path: dest_path.into(),
    is_an_executable: true,
  });
}

fn add_nested_code_sign_path(src_path: &Path, dest_path: &Path, sign_paths: &mut Vec<SignTarget>) {
  for folder_name in NESTED_CODE_FOLDER.iter() {
    let src_folder_path = src_path.join(folder_name);
    let dest_folder_path = dest_path.join(folder_name);

    if src_folder_path.exists() {
      for entry in walkdir::WalkDir::new(src_folder_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
      {
        if entry.path_is_symlink() || entry.file_name().to_string_lossy().starts_with('.') {
          continue;
        }

        let dest_path = dest_folder_path.join(entry.file_name());
        let ext = entry.path().extension();
        if entry.path().is_dir() {
          // Bundles, like .app, .framework, .xpc
          if ext == Some(OsStr::new("framework")) {
            add_framework_sign_path(&entry.clone().into_path(), &dest_path, sign_paths);
          } else if ext == Some(OsStr::new("xpc")) || ext == Some(OsStr::new("app")) {
            add_executable_bundle_sign_path(&entry.clone().into_path(), &dest_path, sign_paths);
          }
        } else if entry.path().is_file() {
          // Binaries, like .dylib, Mach-O executables
          if ext == Some(OsStr::new("dylib")) {
            sign_paths.push(SignTarget {
              path: dest_path,
              is_an_executable: false,
            });
          } else if ext.is_none() {
            sign_paths.push(SignTarget {
              path: dest_path,
              is_an_executable: true,
            });
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bundle::{BundleSettings, MacOsSettings, PackageSettings, SettingsBuilder};
  use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
  };

  /// Helper that builds a `Settings` instance and bundle directory for tests.
  /// It receives a mapping of bundle-relative paths to source paths and
  /// returns the generated bundle directory and settings.
  fn create_test_bundle(
    project_dir: &Path,
    files: HashMap<PathBuf, PathBuf>,
  ) -> (PathBuf, crate::bundle::Settings) {
    let macos_settings = MacOsSettings {
      files,
      ..Default::default()
    };

    let settings = SettingsBuilder::new()
      .project_out_directory(project_dir)
      .package_settings(PackageSettings {
        product_name: "TestApp".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: macos_settings,
        ..Default::default()
      })
      .target("x86_64-apple-darwin".into())
      .build()
      .expect("failed to build settings");

    let bundle_dir = project_dir.join("TestApp.app/Contents");
    fs::create_dir_all(&bundle_dir).expect("failed to create bundle dir");

    (bundle_dir, settings)
  }

  #[test]
  fn test_copy_custom_file_to_bundle_file() {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Prepare a single file to copy.
    let src_file = tmp_dir.path().join("sample.txt");
    fs::write(&src_file, b"hello tauri").expect("failed to write sample file");

    let files_map = HashMap::from([(PathBuf::from("Resources/sample.txt"), src_file.clone())]);

    let (bundle_dir, settings) = create_test_bundle(tmp_dir.path(), files_map);

    copy_custom_files_to_bundle(&bundle_dir, &settings)
      .expect("copy_custom_files_to_bundle failed");

    let dest_file = bundle_dir.join("Resources/sample.txt");
    assert!(dest_file.exists() && dest_file.is_file());
    assert_eq!(fs::read_to_string(dest_file).unwrap(), "hello tauri");
  }

  #[test]
  fn test_copy_custom_file_to_bundle_dir() {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Create a source directory with a nested file.
    let src_dir = tmp_dir.path().join("assets");
    fs::create_dir_all(&src_dir).expect("failed to create assets directory");
    let nested_file = src_dir.join("nested.txt");
    fs::write(&nested_file, b"nested").expect("failed to write nested file");

    let files_map = HashMap::from([(PathBuf::from("MyAssets"), src_dir.clone())]);

    let (bundle_dir, settings) = create_test_bundle(tmp_dir.path(), files_map);

    copy_custom_files_to_bundle(&bundle_dir, &settings)
      .expect("copy_custom_files_to_bundle failed");

    let dest_nested_file = bundle_dir.join("MyAssets/nested.txt");
    assert!(
      dest_nested_file.exists(),
      "{dest_nested_file:?} does not exist"
    );
    assert!(
      dest_nested_file.is_file(),
      "{dest_nested_file:?} is not a file"
    );
    assert_eq!(
      fs::read_to_string(dest_nested_file).unwrap().trim(),
      "nested"
    );
  }

  #[test]
  fn test_copy_custom_files_to_bundle_missing_source() {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");

    // Intentionally reference a non-existent path.
    let missing_path = tmp_dir.path().join("does_not_exist.txt");

    let files_map = HashMap::from([(PathBuf::from("Missing.txt"), missing_path)]);

    let (bundle_dir, settings) = create_test_bundle(tmp_dir.path(), files_map);

    let result = copy_custom_files_to_bundle(&bundle_dir, &settings);

    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("does not exist"));
  }

  #[test]
  fn test_copy_custom_files_to_bundle_invalid_source() {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");

    let files_map = HashMap::from([(PathBuf::from("Invalid.txt"), PathBuf::from("///"))]);

    let (bundle_dir, settings) = create_test_bundle(tmp_dir.path(), files_map);

    let result = copy_custom_files_to_bundle(&bundle_dir, &settings);
    assert!(result.is_err());
    assert!(result
      .err()
      .unwrap()
      .to_string()
      .contains("Failed to copy directory"));
  }

  #[test]
  fn test_copy_custom_files_to_bundle_dev_null() {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");

    let files_map = HashMap::from([(PathBuf::from("Invalid.txt"), PathBuf::from("/dev/null"))]);

    let (bundle_dir, settings) = create_test_bundle(tmp_dir.path(), files_map);

    let result = copy_custom_files_to_bundle(&bundle_dir, &settings);
    assert!(result.is_err());
    assert!(result
      .err()
      .unwrap()
      .to_string()
      .contains("is not a file or directory."));
  }
}
