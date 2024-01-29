// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::bundle::{
  common::CommandExt,
  path_utils::{copy_file, FileOpts},
  settings::Settings,
  windows::{
    sign::try_sign,
    util::{
      download_and_verify, download_webview2_bootstrapper, download_webview2_offline_installer,
      extract_zip, HashAlgorithm, WIX_OUTPUT_FOLDER_NAME, WIX_UPDATER_OUTPUT_FOLDER_NAME,
    },
  },
};
use anyhow::{bail, Context};
use handlebars::{to_json, Handlebars};
use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
  collections::{BTreeMap, HashMap, HashSet},
  fs::{create_dir_all, read_to_string, remove_dir_all, rename, write, File},
  io::Write,
  path::{Path, PathBuf},
  process::Command,
};
use tauri_utils::{config::WebviewInstallMode, display_path};
use uuid::Uuid;

// URLS for the WIX toolchain.  Can be used for cross-platform compilation.
pub const WIX_URL: &str =
  "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip";
pub const WIX_SHA256: &str = "2c1888d5d1dba377fc7fa14444cf556963747ff9a0a289a3599cf09da03b9e2e";

// For Cross Platform Compilation.

// const VC_REDIST_X86_URL: &str =
//     "https://download.visualstudio.microsoft.com/download/pr/c8edbb87-c7ec-4500-a461-71e8912d25e9/99ba493d660597490cbb8b3211d2cae4/vc_redist.x86.exe";

// const VC_REDIST_X86_SHA256: &str =
//   "3a43e8a55a3f3e4b73d01872c16d47a19dd825756784f4580187309e7d1fcb74";

// const VC_REDIST_X64_URL: &str =
//     "https://download.visualstudio.microsoft.com/download/pr/9e04d214-5a9d-4515-9960-3d71398d98c3/1e1e62ab57bbb4bf5199e8ce88f040be/vc_redist.x64.exe";

// const VC_REDIST_X64_SHA256: &str =
//   "d6cd2445f68815fe02489fafe0127819e44851e26dfbe702612bc0d223cbbc2b";

// A v4 UUID that was generated specifically for tauri-bundler, to be used as a
// namespace for generating v5 UUIDs from bundle identifier strings.
const UUID_NAMESPACE: [u8; 16] = [
  0xfd, 0x85, 0x95, 0xa8, 0x17, 0xa3, 0x47, 0x4e, 0xa6, 0x16, 0x76, 0x14, 0x8d, 0xfa, 0x0c, 0x7b,
];

/// Mapper between a resource directory name and its ResourceDirectory descriptor.
type ResourceMap = BTreeMap<String, ResourceDirectory>;

#[derive(Debug, Deserialize)]
struct LanguageMetadata {
  #[serde(rename = "asciiCode")]
  ascii_code: usize,
  #[serde(rename = "langId")]
  lang_id: usize,
}

/// A binary to bundle with WIX.
/// External binaries or additional project binaries are represented with this data structure.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize)]
struct Binary {
  /// the GUID to use on the WIX XML.
  guid: String,
  /// the id to use on the WIX XML.
  id: String,
  /// the binary path.
  path: String,
}

/// A Resource file to bundle with WIX.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize, Clone)]
struct ResourceFile {
  /// the GUID to use on the WIX XML.
  guid: String,
  /// the id to use on the WIX XML.
  id: String,
  /// the file path.
  path: PathBuf,
}

/// A resource directory to bundle with WIX.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize)]
struct ResourceDirectory {
  /// the directory path.
  path: String,
  /// the directory name of the described resource.
  name: String,
  /// the files of the described resource directory.
  files: Vec<ResourceFile>,
  /// the directories that are children of the described resource directory.
  directories: Vec<ResourceDirectory>,
}

impl ResourceDirectory {
  /// Adds a file to this directory descriptor.
  fn add_file(&mut self, file: ResourceFile) {
    self.files.push(file);
  }

  /// Generates the wix XML string to bundle this directory resources recursively
  fn get_wix_data(self) -> crate::Result<(String, Vec<String>)> {
    let mut files = String::from("");
    let mut file_ids = Vec::new();
    for file in self.files {
      file_ids.push(file.id.clone());
      files.push_str(
        format!(
          r#"<Component Id="{id}" Guid="{guid}" Win64="$(var.Win64)" KeyPath="yes"><File Id="PathFile_{id}" Source="{path}" /></Component>"#,
          id = file.id,
          guid = file.guid,
          path = file.path.display()
        ).as_str()
      );
    }
    let mut directories = String::from("");
    for directory in self.directories {
      let (wix_string, ids) = directory.get_wix_data()?;
      for id in ids {
        file_ids.push(id)
      }
      directories.push_str(wix_string.as_str());
    }
    let wix_string = if self.name.is_empty() {
      format!("{}{}", files, directories)
    } else {
      format!(
        r#"<Directory Id="I{id}" Name="{name}">{files}{directories}</Directory>"#,
        id = Uuid::new_v4().as_simple(),
        name = self.name,
        files = files,
        directories = directories,
      )
    };

    Ok((wix_string, file_ids))
  }
}

/// Copies the icon to the binary path, under the `resources` folder,
/// and returns the path to the file.
fn copy_icon(settings: &Settings, filename: &str, path: &Path) -> crate::Result<PathBuf> {
  let base_dir = settings.project_out_directory();

  let resource_dir = base_dir.join("resources");
  create_dir_all(&resource_dir)?;
  let icon_target_path = resource_dir.join(filename);

  let icon_path = std::env::current_dir()?.join(path);

  copy_file(
    icon_path,
    &icon_target_path,
    &FileOpts {
      overwrite: true,
      ..Default::default()
    },
  )?;

  Ok(icon_target_path)
}

/// The app installer output path.
fn app_installer_output_path(
  settings: &Settings,
  language: &str,
  version: &str,
  updater: bool,
) -> crate::Result<PathBuf> {
  let arch = match settings.binary_arch() {
    "x86" => "x86",
    "x86_64" => "x64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {}",
        target
      )))
    }
  };

  let package_base_name = format!(
    "{}_{}_{}_{}",
    settings.main_binary_name().replace(".exe", ""),
    version,
    arch,
    language,
  );

  Ok(settings.project_out_directory().to_path_buf().join(format!(
    "bundle/{}/{}.msi",
    if updater {
      WIX_UPDATER_OUTPUT_FOLDER_NAME
    } else {
      WIX_OUTPUT_FOLDER_NAME
    },
    package_base_name
  )))
}

/// Generates the UUID for the Wix template.
fn generate_package_guid(settings: &Settings) -> Uuid {
  generate_guid(settings.bundle_identifier().as_bytes())
}

/// Generates a GUID.
fn generate_guid(key: &[u8]) -> Uuid {
  let namespace = Uuid::from_bytes(UUID_NAMESPACE);
  Uuid::new_v5(&namespace, key)
}

// Specifically goes and gets Wix and verifies the download via Sha256
pub fn get_and_extract_wix(path: &Path) -> crate::Result<()> {
  info!("Verifying wix package");

  let data = download_and_verify(WIX_URL, WIX_SHA256, HashAlgorithm::Sha256)?;

  info!("extracting WIX");

  extract_zip(&data, path)
}

fn clear_env_for_wix(cmd: &mut Command) {
  cmd.env_clear();
  let required_vars: Vec<std::ffi::OsString> =
    vec!["SYSTEMROOT".into(), "TMP".into(), "TEMP".into()];
  for (k, v) in std::env::vars_os() {
    let k = k.to_ascii_uppercase();
    if required_vars.contains(&k) || k.to_string_lossy().starts_with("TAURI") {
      cmd.env(k, v);
    }
  }
}

// WiX requires versions to be numeric only in a `major.minor.patch.build` format
pub fn convert_version(version_str: &str) -> anyhow::Result<String> {
  let version = semver::Version::parse(version_str).context("invalid app version")?;
  if version.major > 255 {
    bail!("app version major number cannot be greater than 255");
  }
  if version.minor > 255 {
    bail!("app version minor number cannot be greater than 255");
  }
  if version.patch > 65535 {
    bail!("app version patch number cannot be greater than 65535");
  }

  if !version.build.is_empty() {
    let build = version.build.parse::<u64>();
    if build.map(|b| b <= 65535).unwrap_or_default() {
      return Ok(format!(
        "{}.{}.{}.{}",
        version.major, version.minor, version.patch, version.build
      ));
    } else {
      bail!("optional build metadata in app version must be numeric-only and cannot be greater than 65535 for msi target");
    }
  }

  if !version.pre.is_empty() {
    let pre = version.pre.parse::<u64>();
    if pre.is_ok() && pre.unwrap() <= 65535 {
      return Ok(format!(
        "{}.{}.{}.{}",
        version.major, version.minor, version.patch, version.pre
      ));
    } else {
      bail!("optional pre-release identifier in app version must be numeric-only and cannot be greater than 65535 for msi target");
    }
  }

  Ok(version_str.to_string())
}

/// Runs the Candle.exe executable for Wix. Candle parses the wxs file and generates the code for building the installer.
fn run_candle(
  settings: &Settings,
  wix_toolset_path: &Path,
  cwd: &Path,
  wxs_file_path: PathBuf,
  extensions: Vec<PathBuf>,
) -> crate::Result<()> {
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

  let main_binary = settings
    .binaries()
    .iter()
    .find(|bin| bin.main())
    .ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;

  let mut args = vec![
    "-arch".to_string(),
    arch.to_string(),
    wxs_file_path.to_string_lossy().to_string(),
    format!(
      "-dSourceDir={}",
      display_path(settings.binary_path(main_binary))
    ),
  ];

  if settings
    .windows()
    .wix
    .as_ref()
    .map(|w| w.fips_compliant)
    .unwrap_or_default()
  {
    args.push("-fips".into());
  }

  let candle_exe = wix_toolset_path.join("candle.exe");

  info!(action = "Running"; "candle for {:?}", wxs_file_path);
  let mut cmd = Command::new(candle_exe);
  for ext in extensions {
    cmd.arg("-ext");
    cmd.arg(ext);
  }
  clear_env_for_wix(&mut cmd);
  cmd
    .args(&args)
    .current_dir(cwd)
    .output_ok()
    .context("error running candle.exe")?;

  Ok(())
}

/// Runs the Light.exe file. Light takes the generated code from Candle and produces an MSI Installer.
fn run_light(
  wix_toolset_path: &Path,
  build_path: &Path,
  arguments: Vec<String>,
  extensions: &Vec<PathBuf>,
  output_path: &Path,
) -> crate::Result<()> {
  let light_exe = wix_toolset_path.join("light.exe");

  let mut args: Vec<String> = vec!["-o".to_string(), display_path(output_path)];

  args.extend(arguments);

  let mut cmd = Command::new(light_exe);
  for ext in extensions {
    cmd.arg("-ext");
    cmd.arg(ext);
  }
  clear_env_for_wix(&mut cmd);
  cmd
    .args(&args)
    .current_dir(build_path)
    .output_ok()
    .context("error running light.exe")?;

  Ok(())
}

// fn get_icon_data() -> crate::Result<()> {
//   Ok(())
// }

// Entry point for bundling and creating the MSI installer. For now the only supported platform is Windows x64.
pub fn build_wix_app_installer(
  settings: &Settings,
  wix_toolset_path: &Path,
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

  let app_version = convert_version(settings.version_string())?;

  // target only supports x64.
  info!("Target: {}", arch);

  let main_binary = settings
    .binaries()
    .iter()
    .find(|bin| bin.main())
    .ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;
  let app_exe_source = settings.binary_path(main_binary);

  let output_path = settings.project_out_directory().join("wix").join(arch);

  if output_path.exists() {
    remove_dir_all(&output_path)?;
  }
  create_dir_all(&output_path)?;

  let mut data = BTreeMap::new();

  let silent_webview_install = if let WebviewInstallMode::DownloadBootstrapper { silent }
  | WebviewInstallMode::EmbedBootstrapper { silent }
  | WebviewInstallMode::OfflineInstaller { silent } =
    settings.windows().webview_install_mode
  {
    silent
  } else {
    true
  };

  let webview_install_mode = if updater {
    WebviewInstallMode::DownloadBootstrapper {
      silent: silent_webview_install,
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

  data.insert("install_webview", to_json(true));
  data.insert(
    "webview_installer_args",
    to_json(if silent_webview_install {
      "/silent"
    } else {
      ""
    }),
  );

  match webview_install_mode {
    WebviewInstallMode::Skip | WebviewInstallMode::FixedRuntime { .. } => {
      data.insert("install_webview", to_json(false));
    }
    WebviewInstallMode::DownloadBootstrapper { silent: _ } => {
      data.insert("download_bootstrapper", to_json(true));
      data.insert(
        "webview_installer_args",
        to_json(if silent_webview_install {
          "&apos;/silent&apos;,"
        } else {
          ""
        }),
      );
    }
    WebviewInstallMode::EmbedBootstrapper { silent: _ } => {
      let webview2_bootstrapper_path = download_webview2_bootstrapper(&output_path)?;
      data.insert(
        "webview2_bootstrapper_path",
        to_json(webview2_bootstrapper_path),
      );
    }
    WebviewInstallMode::OfflineInstaller { silent: _ } => {
      let webview2_installer_path =
        download_webview2_offline_installer(&output_path.join(arch), arch)?;
      data.insert("webview2_installer_path", to_json(webview2_installer_path));
    }
  }

  let language_map: HashMap<String, LanguageMetadata> =
    serde_json::from_str(include_str!("./languages.json")).unwrap();

  if let Some(wix) = &settings.windows().wix {
    if let Some(license) = &wix.license {
      if license.ends_with(".rtf") {
        data.insert("license", to_json(license));
      } else {
        let license_contents = read_to_string(license)?;
        let license_rtf = format!(
          r#"{{\rtf1\ansi\ansicpg1252\deff0\nouicompat\deflang1033{{\fonttbl{{\f0\fnil\fcharset0 Calibri;}}}}
{{\*\generator Riched20 10.0.18362}}\viewkind4\uc1
\pard\sa200\sl276\slmult1\f0\fs22\lang9 {}\par
}}
 "#,
          license_contents.replace('\n', "\\par ")
        );
        let rtf_output_path = settings
          .project_out_directory()
          .join("wix")
          .join("LICENSE.rtf");
        std::fs::write(&rtf_output_path, license_rtf)?;
        data.insert("license", to_json(rtf_output_path));
      }
    }
  }

  let configured_languages = settings
    .windows()
    .wix
    .as_ref()
    .map(|w| w.language.clone())
    .unwrap_or_default();

  data.insert("product_name", to_json(settings.product_name()));
  data.insert("version", to_json(app_version));
  let bundle_id = settings.bundle_identifier();
  let manufacturer = settings
    .publisher()
    .unwrap_or_else(|| bundle_id.split('.').nth(1).unwrap_or(bundle_id));
  data.insert("bundle_id", to_json(bundle_id));
  data.insert("manufacturer", to_json(manufacturer));
  let upgrade_code = Uuid::new_v5(
    &Uuid::NAMESPACE_DNS,
    format!("{}.app.x64", &settings.main_binary_name()).as_bytes(),
  )
  .to_string();

  data.insert("upgrade_code", to_json(upgrade_code.as_str()));
  data.insert(
    "allow_downgrades",
    to_json(settings.windows().allow_downgrades),
  );

  let path_guid = generate_package_guid(settings).to_string();
  data.insert("path_component_guid", to_json(path_guid.as_str()));

  let shortcut_guid = generate_package_guid(settings).to_string();
  data.insert("shortcut_guid", to_json(shortcut_guid.as_str()));

  let app_exe_name = settings.main_binary_name().to_string();
  data.insert("app_exe_name", to_json(app_exe_name));

  let binaries = generate_binaries_data(settings)?;

  let binaries_json = to_json(binaries);
  data.insert("binaries", binaries_json);

  let resources = generate_resource_data(settings)?;
  let mut resources_wix_string = String::from("");
  let mut files_ids = Vec::new();
  for (_, dir) in resources {
    let (wix_string, ids) = dir.get_wix_data()?;
    resources_wix_string.push_str(wix_string.as_str());
    for id in ids {
      files_ids.push(id);
    }
  }

  data.insert("resources", to_json(resources_wix_string));
  data.insert("resource_file_ids", to_json(files_ids));

  let merge_modules = get_merge_modules(settings)?;
  data.insert("merge_modules", to_json(merge_modules));

  data.insert("app_exe_source", to_json(app_exe_source));

  // copy icon from `settings.windows().icon_path` folder to resource folder near msi
  let icon_path = copy_icon(settings, "icon.ico", &settings.windows().icon_path)?;

  data.insert("icon_path", to_json(icon_path));

  let mut fragment_paths = Vec::new();
  let mut handlebars = Handlebars::new();
  handlebars.register_escape_fn(handlebars::no_escape);
  let mut custom_template_path = None;
  let mut enable_elevated_update_task = false;

  if let Some(wix) = &settings.windows().wix {
    data.insert("component_group_refs", to_json(&wix.component_group_refs));
    data.insert("component_refs", to_json(&wix.component_refs));
    data.insert("feature_group_refs", to_json(&wix.feature_group_refs));
    data.insert("feature_refs", to_json(&wix.feature_refs));
    data.insert("merge_refs", to_json(&wix.merge_refs));
    fragment_paths = wix.fragment_paths.clone();
    enable_elevated_update_task = wix.enable_elevated_update_task;
    custom_template_path = wix.template.clone();

    if let Some(banner_path) = &wix.banner_path {
      let filename = banner_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
      data.insert(
        "banner_path",
        to_json(copy_icon(settings, &filename, banner_path)?),
      );
    }

    if let Some(dialog_image_path) = &wix.dialog_image_path {
      let filename = dialog_image_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
      data.insert(
        "dialog_image_path",
        to_json(copy_icon(settings, &filename, dialog_image_path)?),
      );
    }
  }

  if let Some(file_associations) = settings.file_associations() {
    data.insert("file_associations", to_json(file_associations));
  }

  if let Some(protocols) = settings.deep_link_protocols() {
    let schemes = protocols
      .iter()
      .flat_map(|p| &p.schemes)
      .collect::<Vec<_>>();
    data.insert("deep_link_protocols", to_json(schemes));
  }

  if let Some(path) = custom_template_path {
    handlebars
      .register_template_string("main.wxs", read_to_string(path)?)
      .map_err(|e| e.to_string())
      .expect("Failed to setup custom handlebar template");
  } else {
    handlebars
      .register_template_string("main.wxs", include_str!("../templates/main.wxs"))
      .map_err(|e| e.to_string())
      .expect("Failed to setup handlebar template");
  }

  if enable_elevated_update_task {
    data.insert(
      "msiexec_args",
      to_json(
        settings
          .updater()
          .and_then(|updater| updater.msiexec_args)
          .map(|args| args.join(" "))
          .unwrap_or_else(|| "/passive".to_string()),
      ),
    );

    // Create the update task XML
    let mut skip_uac_task = Handlebars::new();
    let xml = include_str!("../templates/update-task.xml");
    skip_uac_task
      .register_template_string("update.xml", xml)
      .map_err(|e| e.to_string())
      .expect("Failed to setup Update Task handlebars");
    let temp_xml_path = output_path.join("update.xml");
    let update_content = skip_uac_task.render("update.xml", &data)?;
    write(temp_xml_path, update_content)?;

    // Create the Powershell script to install the task
    let mut skip_uac_task_installer = Handlebars::new();
    skip_uac_task_installer.register_escape_fn(handlebars::no_escape);
    let xml = include_str!("../templates/install-task.ps1");
    skip_uac_task_installer
      .register_template_string("install-task.ps1", xml)
      .map_err(|e| e.to_string())
      .expect("Failed to setup Update Task Installer handlebars");
    let temp_ps1_path = output_path.join("install-task.ps1");
    let install_script_content = skip_uac_task_installer.render("install-task.ps1", &data)?;
    write(temp_ps1_path, install_script_content)?;

    // Create the Powershell script to uninstall the task
    let mut skip_uac_task_uninstaller = Handlebars::new();
    skip_uac_task_uninstaller.register_escape_fn(handlebars::no_escape);
    let xml = include_str!("../templates/uninstall-task.ps1");
    skip_uac_task_uninstaller
      .register_template_string("uninstall-task.ps1", xml)
      .map_err(|e| e.to_string())
      .expect("Failed to setup Update Task Uninstaller handlebars");
    let temp_ps1_path = output_path.join("uninstall-task.ps1");
    let install_script_content = skip_uac_task_uninstaller.render("uninstall-task.ps1", &data)?;
    write(temp_ps1_path, install_script_content)?;

    data.insert("enable_elevated_update_task", to_json(true));
  }

  let main_wxs_path = output_path.join("main.wxs");
  write(main_wxs_path, handlebars.render("main.wxs", &data)?)?;

  let mut candle_inputs = vec![("main.wxs".into(), Vec::new())];

  let current_dir = std::env::current_dir()?;
  let extension_regex = Regex::new("\"http://schemas.microsoft.com/wix/(\\w+)\"")?;
  for fragment_path in fragment_paths {
    let fragment_path = current_dir.join(fragment_path);
    let fragment = read_to_string(&fragment_path)?;
    let mut extensions = Vec::new();
    for cap in extension_regex.captures_iter(&fragment) {
      extensions.push(wix_toolset_path.join(format!("Wix{}.dll", &cap[1])));
    }
    candle_inputs.push((fragment_path, extensions));
  }

  let mut fragment_extensions = HashSet::new();
  //Default extensions
  fragment_extensions.insert(wix_toolset_path.join("WixUIExtension.dll"));
  fragment_extensions.insert(wix_toolset_path.join("WixUtilExtension.dll"));

  for (path, extensions) in candle_inputs {
    for ext in &extensions {
      fragment_extensions.insert(ext.clone());
    }
    run_candle(settings, wix_toolset_path, &output_path, path, extensions)?;
  }

  let mut output_paths = Vec::new();

  for (language, language_config) in configured_languages.0 {
    let language_metadata = language_map.get(&language).unwrap_or_else(|| {
      panic!(
        "Language {} not found. It must be one of {}",
        language,
        language_map
          .keys()
          .cloned()
          .collect::<Vec<String>>()
          .join(", ")
      )
    });

    let locale_contents = match language_config.locale_path {
      Some(p) => read_to_string(p)?,
      None => format!(
        r#"<WixLocalization Culture="{}" xmlns="http://schemas.microsoft.com/wix/2006/localization"></WixLocalization>"#,
        language.to_lowercase(),
      ),
    };

    let locale_strings = include_str!("./default-locale-strings.xml")
      .replace("__language__", &language_metadata.lang_id.to_string())
      .replace("__codepage__", &language_metadata.ascii_code.to_string())
      .replace("__productName__", settings.product_name());

    let mut unset_locale_strings = String::new();
    let prefix_len = "<String ".len();
    for locale_string in locale_strings.split('\n').filter(|s| !s.is_empty()) {
      // strip `<String ` prefix and `>{value}</String` suffix.
      let id = locale_string
        .chars()
        .skip(prefix_len)
        .take(locale_string.find('>').unwrap() - prefix_len)
        .collect::<String>();
      if !locale_contents.contains(&id) {
        unset_locale_strings.push_str(locale_string);
      }
    }

    let locale_contents = locale_contents.replace(
      "</WixLocalization>",
      &format!("{}</WixLocalization>", unset_locale_strings),
    );
    let locale_path = output_path.join("locale.wxl");
    {
      let mut fileout = File::create(&locale_path).expect("Failed to create locale file");
      fileout.write_all(locale_contents.as_bytes())?;
    }

    let arguments = vec![
      format!(
        "-cultures:{}",
        if language == "en-US" {
          language.to_lowercase()
        } else {
          format!("{};en-US", language.to_lowercase())
        }
      ),
      "-loc".into(),
      display_path(&locale_path),
      "*.wixobj".into(),
    ];
    let msi_output_path = output_path.join("output.msi");
    let msi_path =
      app_installer_output_path(settings, &language, settings.version_string(), updater)?;
    create_dir_all(msi_path.parent().unwrap())?;

    info!(action = "Running"; "light to produce {}", display_path(&msi_path));

    run_light(
      wix_toolset_path,
      &output_path,
      arguments,
      &(fragment_extensions.clone().into_iter().collect()),
      &msi_output_path,
    )?;
    rename(&msi_output_path, &msi_path)?;
    try_sign(&msi_path, settings)?;
    output_paths.push(msi_path);
  }

  Ok(output_paths)
}

/// Generates the data required for the external binaries and extra binaries bundling.
fn generate_binaries_data(settings: &Settings) -> crate::Result<Vec<Binary>> {
  let mut binaries = Vec::new();
  let cwd = std::env::current_dir()?;
  let tmp_dir = std::env::temp_dir();
  let regex = Regex::new(r"[^\w\d\.]")?;
  for src in settings.external_binaries() {
    let src = src?;
    let binary_path = cwd.join(&src);
    let dest_filename = src
      .file_name()
      .expect("failed to extract external binary filename")
      .to_string_lossy()
      .replace(&format!("-{}", settings.target()), "");
    let dest = tmp_dir.join(&dest_filename);
    std::fs::copy(binary_path, &dest)?;

    binaries.push(Binary {
      guid: Uuid::new_v4().to_string(),
      path: dest
        .into_os_string()
        .into_string()
        .expect("failed to read external binary path"),
      id: regex
        .replace_all(&dest_filename.replace('-', "_"), "")
        .to_string(),
    });
  }

  for bin in settings.binaries() {
    if !bin.main() {
      binaries.push(Binary {
        guid: Uuid::new_v4().to_string(),
        path: settings
          .binary_path(bin)
          .into_os_string()
          .into_string()
          .expect("failed to read binary path"),
        id: regex
          .replace_all(&bin.name().replace('-', "_"), "")
          .to_string(),
      })
    }
  }

  Ok(binaries)
}

#[derive(Serialize)]
struct MergeModule {
  name: String,
  path: String,
}

fn get_merge_modules(settings: &Settings) -> crate::Result<Vec<MergeModule>> {
  let mut merge_modules = Vec::new();
  let regex = Regex::new(r"[^\w\d\.]")?;
  for msm in glob::glob(
    settings
      .project_out_directory()
      .join("*.msm")
      .to_string_lossy()
      .to_string()
      .as_str(),
  )? {
    let path = msm?;
    let filename = path
      .file_name()
      .expect("failed to extract merge module filename")
      .to_os_string()
      .into_string()
      .expect("failed to convert merge module filename to string");
    merge_modules.push(MergeModule {
      name: regex.replace_all(&filename, "").to_string(),
      path: path.to_string_lossy().to_string(),
    });
  }
  Ok(merge_modules)
}

/// Generates the data required for the resource bundling on wix
fn generate_resource_data(settings: &Settings) -> crate::Result<ResourceMap> {
  let mut resources = ResourceMap::new();
  let cwd = std::env::current_dir()?;

  let mut added_resources = Vec::new();

  for resource in settings.resource_files().iter() {
    let resource = resource?;

    let src = cwd.join(resource.path());
    let resource_path = dunce::simplified(&src).to_path_buf();
    // In some glob resource paths like `assets/**/*` a file might appear twice
    // because the `tauri_utils::resources::ResourcePaths` iterator also reads a directory
    // when it finds one. So we must check it before processing the file.
    if added_resources.contains(&resource_path) {
      continue;
    }

    added_resources.push(resource_path.clone());

    let resource_entry = ResourceFile {
      id: format!("I{}", Uuid::new_v4().as_simple()),
      guid: Uuid::new_v4().to_string(),
      path: resource_path.clone(),
    };

    // split the resource path directories
    let target_path = resource.target();
    let components_count = target_path.components().count();
    let directories = target_path
      .components()
      .take(components_count - 1) // the last component is the file
      .collect::<Vec<_>>();

    // transform the directory structure to a chained vec structure
    let first_directory = directories
      .first()
      .map(|d| d.as_os_str().to_string_lossy().into_owned())
      .unwrap_or_else(String::new);

    if !resources.contains_key(&first_directory) {
      resources.insert(
        first_directory.clone(),
        ResourceDirectory {
          path: first_directory.clone(),
          name: first_directory.clone(),
          directories: vec![],
          files: vec![],
        },
      );
    }

    let mut directory_entry = resources
      .get_mut(&first_directory)
      .expect("Unable to handle resources");

    let mut path = String::new();
    // the first component is already parsed on `first_directory` so we skip(1)
    for directory in directories.into_iter().skip(1) {
      let directory_name = directory
        .as_os_str()
        .to_os_string()
        .into_string()
        .expect("failed to read resource folder name");
      path.push_str(directory_name.as_str());
      path.push(std::path::MAIN_SEPARATOR);

      let index = directory_entry
        .directories
        .iter()
        .position(|f| f.path == path);
      match index {
        Some(i) => directory_entry = directory_entry.directories.get_mut(i).unwrap(),
        None => {
          directory_entry.directories.push(ResourceDirectory {
            path: path.clone(),
            name: directory_name,
            directories: vec![],
            files: vec![],
          });
          directory_entry = directory_entry.directories.iter_mut().last().unwrap();
        }
      }
    }
    directory_entry.add_file(resource_entry);
  }

  let mut dlls = Vec::new();

  let out_dir = settings.project_out_directory();
  for dll in glob::glob(out_dir.join("*.dll").to_string_lossy().to_string().as_str())? {
    let path = dll?;
    let resource_path = dunce::simplified(&path);
    let relative_path = path
      .strip_prefix(out_dir)
      .unwrap()
      .to_string_lossy()
      .into_owned();
    if !added_resources.iter().any(|r| r.ends_with(&relative_path)) {
      dlls.push(ResourceFile {
        id: format!("I{}", Uuid::new_v4().as_simple()),
        guid: Uuid::new_v4().to_string(),
        path: resource_path.to_path_buf(),
      });
    }
  }

  if !dlls.is_empty() {
    resources.insert(
      "".to_string(),
      ResourceDirectory {
        path: "".to_string(),
        name: "".to_string(),
        directories: vec![],
        files: dlls,
      },
    );
  }

  Ok(resources)
}
