// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::{
    common::CommandExt,
    windows::util::{
      download, download_and_verify, extract_with_7z, extract_zip, remove_unc_lossy, try_sign,
      validate_version, WEBVIEW2_BOOTSTRAPPER_URL, WEBVIEW2_X64_INSTALLER_GUID,
      WEBVIEW2_X86_INSTALLER_GUID,
    },
  },
  Settings,
};
use anyhow::Context;
use handlebars::{to_json, Handlebars};
use log::{info, warn};
use tauri_utils::{config::WebviewInstallMode, resources::resource_relpath};

use std::{
  collections::BTreeMap,
  fs::{copy, create_dir_all, remove_dir_all, rename, write},
  path::{Path, PathBuf},
  process::Command,
};

// URLS for the NSIS toolchain.
const NSIS_URL: &str =
  "https://sourceforge.net/projects/nsis/files/NSIS%203/3.08/nsis-3.08.zip/download";
const NSIS_SHA1: &str = "057e83c7d82462ec394af76c87d06733605543d4";
const NSIS_NSCURL_URL: &str =
  "https://github.com/negrutiu/nsis-nscurl/releases/download/v1.2022.6.7/NScurl-1.2022.6.7.7z";
const NSIS_APPLICATIONID_URL: &str = "https://github.com/connectiblutz/NSIS-ApplicationID/releases/download/1.1/NSIS-ApplicationID.zip";
const NSIS_NSPROCESS_URL: &str = "https://nsis.sourceforge.io/mediawiki/images/1/18/NsProcess.zip";

const NSIS_REQUIRED_FILES: &[&str] = &[
  "makensis.exe",
  "Bin/makensis.exe",
  "Stubs/lzma-x86-unicode",
  "Stubs/lzma_solid-x86-unicode",
  "Plugins/x86-unicode/NScurl.dll",
  "Plugins/x86-unicode/ApplicationID.dll",
  "Plugins/x86-unicode/nsProcess.dll",
  "Include/MUI2.nsh",
  "Include/FileFunc.nsh",
  "Include/x64.nsh",
];

/// Runs all of the commands to build the NSIS installer.
/// Returns a vector of PathBuf that shows where the NSIS installer was created.
pub fn bundle_project(settings: &Settings, updater: bool) -> crate::Result<Vec<PathBuf>> {
  let tauri_tools_path = dirs_next::cache_dir().unwrap().join("tauri");
  let nsis_toolset_path = tauri_tools_path.join("NSIS");

  if !nsis_toolset_path.exists() {
    get_and_extract_nsis(&nsis_toolset_path, &tauri_tools_path)?;
  } else if NSIS_REQUIRED_FILES
    .iter()
    .any(|p| !nsis_toolset_path.join(p).exists())
  {
    warn!("NSIS directory is missing some files. Recreating it.");
    std::fs::remove_dir_all(&nsis_toolset_path)?;
    get_and_extract_nsis(&nsis_toolset_path, &tauri_tools_path)?;
  }

  build_nsis_app_installer(settings, &nsis_toolset_path, &tauri_tools_path, updater)
}

// Gets NSIS and verifies the download via Sha1
fn get_and_extract_nsis(nsis_toolset_path: &Path, tauri_tools_path: &Path) -> crate::Result<()> {
  info!("Verifying NSIS package");

  let data = download_and_verify(NSIS_URL, NSIS_SHA1, "sha1")?;
  info!("extracting NSIS");
  extract_zip(&data, tauri_tools_path)?;
  rename(tauri_tools_path.join("nsis-3.08"), nsis_toolset_path)?;

  let nsis_plugins = nsis_toolset_path.join("Plugins");

  let data = download(NSIS_NSCURL_URL)?;
  info!("extracting NSIS NScurl plugin");
  extract_with_7z(&data, &nsis_plugins)?;

  let data = download(NSIS_APPLICATIONID_URL)?;
  info!("extracting NSIS ApplicationID plugin");
  extract_zip(&data, &nsis_plugins)?;
  copy(
    nsis_plugins
      .join("ReleaseUnicode")
      .join("ApplicationID.dll"),
    nsis_plugins.join("x86-unicode").join("ApplicationID.dll"),
  )?;

  let data = download(NSIS_NSPROCESS_URL)?;
  info!("extracting NSIS NsProcess plugin");
  extract_with_7z(&data, &nsis_plugins)?;
  copy(
    nsis_plugins.join("Plugin").join("nsProcessW.dll"),
    nsis_plugins.join("x86-unicode").join("nsProcess.dll"),
  )?;
  Ok(())
}

fn build_nsis_app_installer(
  settings: &Settings,
  nsis_toolset_path: &Path,
  tauri_tools_path: &Path,
  updater: bool,
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

  data.insert("arch", to_json(arch));
  data.insert("bundle_id", to_json(bundle_id));
  data.insert("manufacturer", to_json(manufacturer));
  data.insert("product_name", to_json(settings.product_name()));

  let version = settings.version_string();
  data.insert("version", to_json(&version));

  let mut s = version.split(".");
  data.insert("version_major", to_json(s.next().unwrap()));
  data.insert("version_minor", to_json(s.next().unwrap()));
  data.insert("version_build", to_json(s.next().unwrap()));

  data.insert(
    "allow_downgrades",
    to_json(settings.windows().allow_downgrades),
  );

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
      data.insert(
        "license",
        to_json(remove_unc_lossy(license.canonicalize()?)),
      );
    }
    if let Some(installer_icon) = &nsis.installer_icon {
      data.insert(
        "installer_icon",
        to_json(remove_unc_lossy(installer_icon.canonicalize()?)),
      );
    }
    if let Some(header_image) = &nsis.header_image {
      data.insert(
        "header_image",
        to_json(remove_unc_lossy(header_image.canonicalize()?)),
      );
    }
    if let Some(sidebar_image) = &nsis.sidebar_image {
      data.insert(
        "sidebar_image",
        to_json(remove_unc_lossy(sidebar_image.canonicalize()?)),
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

  let resources = generate_resource_data(&settings)?;
  data.insert("resources", to_json(resources));

  let binaries = generate_binaries_data(settings)?;
  data.insert("binaries", to_json(binaries));

  let silent_webview2_install = if let WebviewInstallMode::DownloadBootstrapper { silent }
  | WebviewInstallMode::EmbedBootstrapper { silent }
  | WebviewInstallMode::OfflineInstaller { silent } =
    settings.windows().webview_install_mode
  {
    silent
  } else {
    true
  };

  let webview2_install_mode = if updater {
    WebviewInstallMode::DownloadBootstrapper {
      silent: silent_webview2_install,
    }
  } else {
    let mut webview_install_mode = settings.windows().webview_install_mode.clone();
    if let Some(fixed_runtime_path) = settings.windows().webview_fixed_runtime_path.clone() {
      webview_install_mode = WebviewInstallMode::FixedRuntime {
        path: fixed_runtime_path,
      };
    } else if let Some(wix) = &settings.windows().wix {
      if wix.skip_webview_install {
        webview_install_mode = WebviewInstallMode::Skip;
      }
    }
    webview_install_mode
  };

  let webview2_installer_args = to_json(if silent_webview2_install {
    "/silent"
  } else {
    ""
  });

  data.insert("webview2_installer_args", to_json(webview2_installer_args));
  data.insert(
    "install_webview2_mode",
    to_json(match webview2_install_mode {
      WebviewInstallMode::DownloadBootstrapper { silent: _ } => "downloadBootstrapper",
      WebviewInstallMode::EmbedBootstrapper { silent: _ } => "embedBootstrapper",
      WebviewInstallMode::OfflineInstaller { silent: _ } => "offlineInstaller",
      _ => "",
    }),
  );

  match webview2_install_mode {
    WebviewInstallMode::EmbedBootstrapper { silent: _ } => {
      let webview2_bootstrapper_path = tauri_tools_path.join("MicrosoftEdgeWebview2Setup.exe");
      std::fs::write(
        &webview2_bootstrapper_path,
        download(WEBVIEW2_BOOTSTRAPPER_URL)?,
      )?;
      data.insert(
        "webview2_bootstrapper_path",
        to_json(webview2_bootstrapper_path),
      );
    }
    WebviewInstallMode::OfflineInstaller { silent: _ } => {
      let guid = if arch == "x64" {
        WEBVIEW2_X64_INSTALLER_GUID
      } else {
        WEBVIEW2_X86_INSTALLER_GUID
      };
      let offline_installer_path = tauri_tools_path
        .join("Webview2OfflineInstaller")
        .join(guid)
        .join(arch);
      create_dir_all(&offline_installer_path)?;
      let webview2_installer_path =
        offline_installer_path.join("MicrosoftEdgeWebView2RuntimeInstaller.exe");
      if !webview2_installer_path.exists() {
        std::fs::write(
          &webview2_installer_path,
          download(
            &format!("https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/{}/MicrosoftEdgeWebView2RuntimeInstaller{}.exe",
              guid,
              arch.to_uppercase(),
            ),
          )?,
        )?;
      }
      data.insert("webview2_installer_path", to_json(webview2_installer_path));
    }
    _ => {}
  }

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
    "{}_{}_{}-setup",
    main_binary.name().replace(".exe", ""),
    settings.version_string(),
    arch,
  );

  let nsis_output_path = output_path.join(out_file);
  let nsis_installer_path = settings.project_out_directory().to_path_buf().join(format!(
    "bundle/{}/{}.exe",
    if updater { "nsis-updater" } else { "nsis" },
    package_base_name
  ));
  create_dir_all(nsis_installer_path.parent().unwrap())?;

  info!(action = "Running"; "makensis.exe to produce {}", nsis_installer_path.display());
  Command::new(nsis_toolset_path.join("makensis.exe"))
    .args(&[installer_nsi_path])
    .current_dir(output_path)
    .output_ok()
    .context("error running makensis.exe")?;

  rename(&nsis_output_path, &nsis_installer_path)?;
  try_sign(&nsis_installer_path, settings)?;

  Ok(vec![nsis_installer_path])
}

/// BTreeMap<OriginalPath, TargetPath>
type ResourcesMap = BTreeMap<PathBuf, PathBuf>;
fn generate_resource_data(settings: &Settings) -> crate::Result<ResourcesMap> {
  let mut resources = ResourcesMap::new();
  let cwd = std::env::current_dir()?;

  let mut added_resources = Vec::new();

  for src in settings.resource_files() {
    let src = src?;
    let resource_path = remove_unc_lossy(cwd.join(&src).canonicalize()?);

    // In some glob resource paths like `assets/**/*` a file might appear twice
    // because the `tauri_utils::resources::ResourcePaths` iterator also reads a directory
    // when it finds one. So we must check it before processing the file.
    if added_resources.contains(&resource_path) {
      continue;
    }
    added_resources.push(resource_path.clone());

    let target_path = resource_relpath(&src);
    resources.insert(resource_path, target_path);
  }

  Ok(resources)
}

/// BTreeMap<OriginalPath, TargetFileName>
type BinariesMap = BTreeMap<PathBuf, String>;
fn generate_binaries_data(settings: &Settings) -> crate::Result<BinariesMap> {
  let mut binaries = BinariesMap::new();
  let cwd = std::env::current_dir()?;

  for src in settings.external_binaries() {
    let src = src?;
    let binary_path = remove_unc_lossy(cwd.join(&src).canonicalize()?);
    let dest_filename = src
      .file_name()
      .expect("failed to extract external binary filename")
      .to_string_lossy()
      .replace(&format!("-{}", settings.target()), "");
    binaries.insert(binary_path, dest_filename);
  }

  for bin in settings.binaries() {
    if !bin.main() {
      let bin_path = settings.binary_path(bin);
      binaries.insert(
        bin_path.clone(),
        bin_path
          .file_name()
          .expect("failed to extract external binary filename")
          .to_string_lossy()
          .to_string(),
      );
    }
  }

  Ok(binaries)
}
