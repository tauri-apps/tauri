// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::windows::util::{
    download, download_and_verify, extract_7z, extract_zip, remove_unc, try_sign, validate_version,
  },
  Settings,
};
use handlebars::{to_json, Handlebars};
use log::{info, warn};

use std::{
  collections::BTreeMap,
  fs::{create_dir_all, remove_dir_all, rename, write},
  path::{Path, PathBuf},
  process::Command,
};

// URLS for the NSIS toolchain.
const NSIS_URL: &str =
  "https://sourceforge.net/projects/nsis/files/NSIS%203/3.08/nsis-3.08.zip/download";
const NSIS_SHA1: &str = "057e83c7d82462ec394af76c87d06733605543d4";
const NS_CURL_URL: &str =
  "https://github.com/negrutiu/nsis-nscurl/releases/download/v1.2022.6.7/NScurl-1.2022.6.7.7z";

const NSIS_REQUIRED_FILES: &[&str] = &[
  "makensis.exe",
  "Bin/makensis.exe",
  "Stubs/zlib-x86-unicode",
  "Plugins/x86-unicode/NScurl.dll",
  "Include/MUI2.nsh",
  "Include/FileFunc.nsh",
  "Include/x64.nsh",
];

/// Runs all of the commands to build the NSIS installer.
/// Returns a vector of PathBuf that shows where the NSIS installer was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let nsis_path = dirs_next::cache_dir().unwrap().join("tauri").join("NSIS");
  let nsis_toolset_path = nsis_path.join("nsis-3.08");

  if !nsis_path.exists() {
    get_and_extract_nsis(&nsis_path)?;
  } else if NSIS_REQUIRED_FILES
    .iter()
    .any(|p| !nsis_toolset_path.join(p).exists())
  {
    warn!("NSIS directory is missing some files. Recreating it.");
    std::fs::remove_dir_all(&nsis_path)?;
    get_and_extract_nsis(&nsis_path)?;
  }

  build_nsis_app_installer(settings, &nsis_toolset_path)
}

// Gets NSIS and verifies the download via Sha256
fn get_and_extract_nsis(path: &Path) -> crate::Result<()> {
  info!("Verifying NSIS package");

  let data = download_and_verify(NSIS_URL, NSIS_SHA1, "sha1")?;

  info!("extracting NSIS");
  extract_zip(&data, path)?;

  let data = download(NS_CURL_URL)?;

  info!("extracting NScurl");
  extract_7z(&data, &path.join("nsis-3.08").join("Plugins"))?;

  Ok(())
}

fn build_nsis_app_installer(
  settings: &Settings,
  nsis_toolset_path: &Path,
) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    "x86_64" => "x64",
    "x86" => "x86",
    target => {
      return Err(crate::Error::ArchError(format!(
        "unsupported target: {}",
        target
      )))
    }
  };

  validate_version(settings.version_string())?;

  info!("Target: {}", arch);

  let main_binary = settings
    .binaries()
    .iter()
    .find(|bin| bin.main())
    .ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;
  let app_exe_source = settings.binary_path(main_binary);

  try_sign(&app_exe_source, &settings)?;

  let output_path = settings.project_out_directory().join("nsis").join(arch);
  if output_path.exists() {
    remove_dir_all(&output_path)?;
  }
  create_dir_all(&output_path)?;

  let mut data = BTreeMap::new();

  let bundle_id = settings.bundle_identifier();
  let manufacturer = bundle_id.split('.').nth(1).unwrap_or(bundle_id);

  data.insert("bundle_id", to_json(bundle_id));
  data.insert("manufacturer", to_json(manufacturer));
  data.insert("product_name", to_json(settings.product_name()));

  let version = settings.version_string();
  data.insert("version", to_json(&version));

  let mut s = version.split(".");
  data.insert("version_major", to_json(s.next().unwrap()));
  data.insert("version_minor", to_json(s.next().unwrap()));

  if let Some(nsis) = &settings.windows().nsis {
    data.insert(
      "install_mode",
      to_json(if nsis.per_machine {
        "perMachine"
      } else {
        "perUser"
      }),
    );

    if let Some(license) = &nsis.license {
      data.insert("license", to_json(remove_unc(license.canonicalize()?)));
    }
    if let Some(installer_icon) = &nsis.installer_icon {
      data.insert(
        "installer_icon",
        to_json(remove_unc(installer_icon.canonicalize()?)),
      );
    }
    if let Some(header_image) = &nsis.header_image {
      data.insert(
        "header_image",
        to_json(remove_unc(header_image.canonicalize()?)),
      );
    }
    if let Some(sidebar_image) = &nsis.sidebar_image {
      data.insert(
        "sidebar_image",
        to_json(remove_unc(sidebar_image.canonicalize()?)),
      );
    }
  }

  let main_binary = settings
    .binaries()
    .iter()
    .find(|bin| bin.main())
    .ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;
  data.insert(
    "main_binary_name",
    to_json(main_binary.name().replace(".exe", "")),
  );
  data.insert(
    "main_binary_path",
    to_json(settings.binary_path(main_binary)),
  );

  let out_file = "nsis-output.exe";
  data.insert("out_file", to_json(&out_file));

  let mut handlebars = Handlebars::new();
  handlebars
    .register_template_string("installer.nsi", include_str!("./templates/installer.nsi"))
    .map_err(|e| e.to_string())
    .expect("Failed to setup handlebar template");
  let installer_nsi_path = output_path.join("installer.nsi");
  write(
    &installer_nsi_path,
    handlebars.render("installer.nsi", &data)?,
  )?;

  let package_base_name = format!(
    "{}_{}_{}",
    main_binary.name().replace(".exe", ""),
    settings.version_string(),
    arch,
  );

  let nsis_output_path = output_path.join(out_file);
  let nsis_installer_path = settings
    .project_out_directory()
    .to_path_buf()
    .join(format!("bundle/nsis/{}.exe", package_base_name));
  create_dir_all(nsis_installer_path.parent().unwrap())?;

  info!(action = "Running"; "makensis to produce {}", nsis_installer_path.display());
  Command::new(nsis_toolset_path.join("makensis.exe"))
    .args(&[installer_nsi_path])
    .output()?;

  rename(&nsis_output_path, &nsis_installer_path)?;
  try_sign(&nsis_installer_path, &settings)?;

  Ok(vec![nsis_installer_path])
}
