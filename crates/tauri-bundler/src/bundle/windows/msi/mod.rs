// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::{
    settings::{Arch, Settings},
    windows::{
      sign::{should_sign, try_sign},
      util::{
        WIX_OUTPUT_FOLDER_NAME, WIX_UPDATER_OUTPUT_FOLDER_NAME, download_webview2_bootstrapper,
        download_webview2_offline_installer, vc_runtime_dlls,
      },
    },
  },
  error::{Context, ErrorExt},
  utils::{
    CommandExt,
    fs_utils::copy_file,
    http_utils::{HashAlgorithm, download_and_verify, extract_zip},
  },
};
use handlebars::{Handlebars, html_escape, to_json};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
  collections::{BTreeMap, HashMap, HashSet},
  ffi::OsStr,
  fs::{self, File},
  io::Write,
  path::{Path, PathBuf},
  process::Command,
};
use tauri_utils::{config::WebviewInstallMode, display_path};
use uuid::Uuid;

// URLs for the WIX toolchain. Can be used for cross-platform compilation.
pub const WIX_URL: &str =
  "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip";
pub const WIX_SHA256: &str = "6ac824e1642d6f7277d0ed7ea09411a508f6116ba6fae0aa5f2c7daa2ff43d31";

const WIX_REQUIRED_FILES: &[&str] = &[
  "candle.exe",
  "candle.exe.config",
  "darice.cub",
  "light.exe",
  "light.exe.config",
  "wconsole.dll",
  "winterop.dll",
  "wix.dll",
  "WixUIExtension.dll",
  "WixUtilExtension.dll",
];

/// Runs all of the commands to build the MSI installer.
/// Returns a vector of PathBuf that shows where the MSI was created.
pub fn bundle_project(settings: &Settings, updater: bool) -> crate::Result<Vec<PathBuf>> {
  let tauri_tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| dirs::cache_dir().unwrap().join("tauri"));

  let wix_path = tauri_tools_path.join("WixTools314");

  if !wix_path.exists() {
    get_and_extract_wix(&wix_path)?;
  } else if WIX_REQUIRED_FILES
    .iter()
    .any(|p| !wix_path.join(p).exists())
  {
    log::warn!("WixTools directory is missing some files. Recreating it.");
    std::fs::remove_dir_all(&wix_path)?;
    get_and_extract_wix(&wix_path)?;
  }

  build_wix_app_installer(settings, &wix_path, updater)
}

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
struct ResourceFile {
  /// the GUID to use on the WIX XML.
  guid: String,
  /// the id to use on the WIX XML.
  id: String,
  /// the source file path.
  source_path: PathBuf,
  /// file name override, defaulting to the file name of [`Self::source_path`].
  target_name_override: Option<String>,
}

impl ResourceFile {
  fn new(source_path: PathBuf, target_name_override: Option<String>) -> Self {
    Self {
      id: format!("I{}", Uuid::new_v4().as_simple()),
      guid: Uuid::new_v4().to_string(),
      source_path,
      target_name_override,
    }
  }
}

/// A resource directory to bundle with WIX.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Default)]
struct ResourceDirectory {
  /// the files of the described resource directory.
  files: Vec<ResourceFile>,
  /// the directories that are children of the described resource directory.
  directories: HashMap<String, ResourceDirectory>,
}

impl ResourceDirectory {
  /// Adds a file to this directory descriptor.
  fn add_file(&mut self, file: ResourceFile) {
    self.files.push(file);
  }

  /// Generates the wix XML string to bundle this directory resources recursively
  fn render_wix(self, directory_name: Option<String>) -> crate::Result<(String, Vec<String>)> {
    let mut files = String::from("");
    let mut file_ids = Vec::new();
    for file in self.files {
      let ResourceFile {
        id,
        guid,
        source_path,
        target_name_override,
      } = file;
      let name_attribute = target_name_override
        .map(|name| format!(r#"Name="{}" "#, html_escape(&name)))
        .unwrap_or_default();
      let source_path = html_escape(&source_path.to_string_lossy());
      files.push_str(
        &format!(
          r#"<Component Id="{id}" Guid="{guid}" Win64="$(var.Win64)" KeyPath="yes"><File Id="PathFile_{id}" Source="{source_path}" {name_attribute}/></Component>"#,
        )
      );
      file_ids.push(id);
    }
    let mut directories = String::from("");
    for (directory_name, directory) in self.directories {
      let (wix_string, ids) = directory.render_wix(Some(directory_name))?;
      for id in ids {
        file_ids.push(id)
      }
      directories.push_str(wix_string.as_str());
    }
    let wix_string = if let Some(directory_name) = directory_name {
      format!(
        r#"<Directory Id="I{id}" Name="{name}">{files}{directories}</Directory>"#,
        id = Uuid::new_v4().as_simple(),
        name = html_escape(&directory_name),
        files = files,
        directories = directories,
      )
    } else {
      format!("{files}{directories}")
    };

    Ok((wix_string, file_ids))
  }
}

/// Copies the icon to the binary path, under the `resources` folder,
/// and returns the path to the file.
fn copy_icon(settings: &Settings, filename: &str, path: &Path) -> crate::Result<PathBuf> {
  let base_dir = settings.project_out_directory();

  let resource_dir = base_dir.join("resources");
  fs::create_dir_all(&resource_dir)?;
  let icon_target_path = resource_dir.join(filename);

  let icon_path = std::env::current_dir()?.join(path);

  copy_file(&icon_path, &icon_target_path)?;

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
    Arch::X86_64 => "x64",
    Arch::X86 => "x86",
    Arch::AArch64 => "arm64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };

  let package_base_name = format!(
    "{}_{}_{}_{}",
    settings.product_name(),
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

fn wix_identifier(id: &str) -> String {
  let mut identifier: String = id
    .replace('-', "_")
    .chars()
    .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
    .collect();

  if !identifier
    .chars()
    .next()
    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
  {
    identifier.insert(0, '_');
  }

  identifier
}

// Specifically goes and gets Wix and verifies the download via Sha256
pub fn get_and_extract_wix(path: &Path) -> crate::Result<()> {
  log::info!("Verifying wix package");

  let data = download_and_verify(WIX_URL, WIX_SHA256, HashAlgorithm::Sha256)?;

  log::info!("extracting WIX");

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

fn validate_wix_version(version_str: &str) -> crate::Result<()> {
  let components = version_str
    .split('.')
    .flat_map(|c| c.parse::<u64>().ok())
    .collect::<Vec<_>>();

  if components.len() < 3 {
    crate::error::bail!(
      "app wix version should be in the format major.minor.patch.build (build is optional)"
    );
  }

  if components[0] > 255 {
    crate::error::bail!("app version major number cannot be greater than 255");
  }
  if components[1] > 255 {
    crate::error::bail!("app version minor number cannot be greater than 255");
  }
  if components[2] > 65535 {
    crate::error::bail!("app version patch number cannot be greater than 65535");
  }

  if components.len() == 4 && components[3] > 65535 {
    crate::error::bail!("app version build number cannot be greater than 65535");
  }

  Ok(())
}

// WiX requires versions to be numeric only in a `major.minor.patch.build` format
fn convert_version(version_str: &str) -> crate::Result<String> {
  let version = semver::Version::parse(version_str)
    .map_err(Into::into)
    .context("invalid app version")?;
  if !version.build.is_empty() {
    let build = version.build.parse::<u64>();
    if build.map(|b| b <= 65535).unwrap_or_default() {
      return Ok(format!(
        "{}.{}.{}.{}",
        version.major, version.minor, version.patch, version.build
      ));
    } else {
      crate::error::bail!(
        "optional build metadata in app version must be numeric-only and cannot be greater than 65535 for msi target"
      );
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
      crate::error::bail!(
        "optional pre-release identifier in app version must be numeric-only and cannot be greater than 65535 for msi target"
      );
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
    Arch::X86_64 => "x64",
    Arch::X86 => "x86",
    Arch::AArch64 => "arm64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "unsupported architecture: {target:?}"
      )));
    }
  };

  let main_binary = settings.main_binary()?;

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

  log::info!(action = "Running"; "candle for {:?}", wxs_file_path);
  let mut cmd = Command::new(candle_exe);
  for ext in extensions {
    cmd.arg("-ext");
    cmd.arg(ext);
  }
  clear_env_for_wix(&mut cmd);
  cmd.args(&args).current_dir(cwd).output_ok()?;

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
  cmd.args(&args).current_dir(build_path).output_ok()?;

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
    Arch::X86_64 => "x64",
    Arch::X86 => "x86",
    Arch::AArch64 => "arm64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "unsupported architecture: {target:?}"
      )));
    }
  };

  let app_version = if let Some(version) = settings
    .windows()
    .wix
    .as_ref()
    .and_then(|wix| wix.version.clone())
  {
    version
  } else {
    convert_version(settings.version_string())?
  };

  validate_wix_version(&app_version)?;

  // target only supports x64.
  log::info!("Target: {}", arch);

  let output_path = settings.project_out_directory().join("wix").join(arch);

  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }
  fs::create_dir_all(&output_path)?;

  // when we're performing code signing, we'll sign some WiX DLLs, so we make a local copy
  let wix_toolset_path = if settings.windows().can_sign() {
    let wix_path = output_path.join("wix");
    crate::utils::fs_utils::copy_dir(wix_toolset_path, &wix_path)?;
    wix_path
  } else {
    wix_toolset_path.to_path_buf()
  };

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
    settings.windows().webview_install_mode.clone()
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

  if let Some(minimum_webview2_version) = &settings.windows().minimum_webview2_version {
    data.insert(
      "minimum_webview2_version",
      to_json(minimum_webview2_version),
    );
  }

  if let Some(license) = settings.license_file() {
    if license.ends_with(".rtf") {
      data.insert("license", to_json(license));
    } else {
      let license_contents = fs::read_to_string(license)?;
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

  let language_map: HashMap<String, LanguageMetadata> =
    serde_json::from_str(include_str!("./languages.json")).unwrap();

  let configured_languages = settings
    .windows()
    .wix
    .as_ref()
    .map(|w| w.language.clone())
    .unwrap_or_default();

  data.insert("product_name", to_json(settings.product_name()));
  data.insert("version", to_json(app_version));
  data.insert(
    "long_description",
    to_json(settings.long_description().unwrap_or_default()),
  );
  data.insert("homepage", to_json(settings.homepage_url()));
  let bundle_id = settings.bundle_identifier();
  let manufacturer = settings
    .publisher()
    .unwrap_or_else(|| bundle_id.split('.').nth(1).unwrap_or(bundle_id));
  data.insert("bundle_id", to_json(bundle_id));
  data.insert("manufacturer", to_json(manufacturer));

  // NOTE: if this is ever changed, make sure to also update `tauri inspect wix-upgrade-code` subcommand
  let upgrade_code = settings
    .windows()
    .wix
    .as_ref()
    .and_then(|w| w.upgrade_code)
    .unwrap_or_else(|| {
      Uuid::new_v5(
        &Uuid::NAMESPACE_DNS,
        format!("{}.exe.app.x64", settings.product_name()).as_bytes(),
      )
    });
  data.insert("upgrade_code", to_json(upgrade_code.to_string()));
  data.insert(
    "allow_downgrades",
    to_json(settings.windows().allow_downgrades),
  );

  let path_guid = generate_package_guid(settings).to_string();
  data.insert("path_component_guid", to_json(path_guid.as_str()));

  let shortcut_guid = generate_package_guid(settings).to_string();
  data.insert("shortcut_guid", to_json(shortcut_guid.as_str()));

  let binaries = generate_binaries_data(settings)?;

  let binaries_json = to_json(binaries);
  data.insert("binaries", binaries_json);

  let resources = generate_resource_data(settings)?;
  let (resources_wix_string, files_ids) = resources.render_wix(None)?;

  data.insert("resources", to_json(resources_wix_string));
  data.insert("resource_file_ids", to_json(files_ids));

  let merge_modules = get_merge_modules(settings)?;
  data.insert("merge_modules", to_json(merge_modules));

  // Note: `main_binary_name` is not used in our template but we keep it as it is potentially useful for custom templates
  let main_binary_name = settings.main_binary_name()?;
  data.insert("main_binary_name", to_json(main_binary_name));

  let main_binary = settings.main_binary()?;
  let main_binary_path = settings.binary_path(main_binary);
  data.insert("main_binary_path", to_json(main_binary_path));

  // copy icon from `settings.windows().icon_path` folder to resource folder near msi
  #[allow(deprecated)]
  let icon_path = if !settings.windows().icon_path.as_os_str().is_empty() {
    settings.windows().icon_path.clone()
  } else {
    settings
      .icon_files()
      .flatten()
      .find(|i| i.extension() == Some(OsStr::new("ico")))
      .context("Couldn't find a .ico icon")?
  };
  let icon_path = copy_icon(settings, "icon.ico", &icon_path)?;

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
    fragment_paths.clone_from(&wix.fragment_paths);
    enable_elevated_update_task = wix.enable_elevated_update_task;
    custom_template_path.clone_from(&wix.template);

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
    if !schemes.is_empty() {
      data.insert("deep_link_protocols", to_json(schemes));
    }
  }

  if let Some(path) = custom_template_path {
    handlebars
      .register_template_string("main.wxs", fs::read_to_string(path)?)
      .map_err(|e| e.to_string())
      .expect("Failed to setup custom handlebar template");
  } else {
    handlebars
      .register_template_string("main.wxs", include_str!("./main.wxs"))
      .map_err(|e| e.to_string())
      .expect("Failed to setup handlebar template");
  }

  if enable_elevated_update_task {
    data.insert(
      "msiexec_args",
      to_json(
        settings
          .updater()
          .map(|updater| updater.msiexec_args)
          .map(|args| args.join(" "))
          .unwrap_or_else(|| "/passive".to_string()),
      ),
    );

    // Create the update task XML
    let skip_uac_task = Handlebars::new();
    let xml = include_str!("./update-task.xml");
    let update_content = skip_uac_task.render_template(xml, &data)?;
    let temp_xml_path = output_path.join("update.xml");
    fs::write(temp_xml_path, update_content)?;

    // Create the Powershell script to install the task
    let mut skip_uac_task_installer = Handlebars::new();
    skip_uac_task_installer.register_escape_fn(handlebars::no_escape);
    let xml = include_str!("./install-task.ps1");
    let install_script_content = skip_uac_task_installer.render_template(xml, &data)?;
    let temp_ps1_path = output_path.join("install-task.ps1");
    fs::write(temp_ps1_path, install_script_content)?;

    // Create the Powershell script to uninstall the task
    let mut skip_uac_task_uninstaller = Handlebars::new();
    skip_uac_task_uninstaller.register_escape_fn(handlebars::no_escape);
    let xml = include_str!("./uninstall-task.ps1");
    let install_script_content = skip_uac_task_uninstaller.render_template(xml, &data)?;
    let temp_ps1_path = output_path.join("uninstall-task.ps1");
    fs::write(temp_ps1_path, install_script_content)?;

    data.insert("enable_elevated_update_task", to_json(true));
  }

  let main_wxs_path = output_path.join("main.wxs");
  fs::write(&main_wxs_path, handlebars.render("main.wxs", &data)?)?;

  let mut candle_inputs = vec![];

  let current_dir = std::env::current_dir()?;
  let extension_regex = Regex::new("\"http://schemas.microsoft.com/wix/(\\w+)\"")?;
  let input_paths =
    std::iter::once(main_wxs_path).chain(fragment_paths.iter().map(|p| current_dir.join(p)));

  for input_path in input_paths {
    let input_content = fs::read_to_string(&input_path)?;
    let input_handlebars = Handlebars::new();
    let input = input_handlebars.render_template(&input_content, &data)?;
    let mut extensions = Vec::new();
    for cap in extension_regex.captures_iter(&input) {
      let path = wix_toolset_path.join(format!("Wix{}.dll", &cap[1]));
      if settings.windows().can_sign() {
        try_sign(&path, settings)?;
      }
      extensions.push(path);
    }
    candle_inputs.push((input_path, extensions));
  }

  let mut fragment_extensions = HashSet::new();
  // Default extensions
  fragment_extensions.insert(wix_toolset_path.join("WixUIExtension.dll"));
  fragment_extensions.insert(wix_toolset_path.join("WixUtilExtension.dll"));

  // sign default extensions
  if settings.windows().can_sign() {
    for path in &fragment_extensions {
      try_sign(path, settings)?;
    }
  }

  for (path, extensions) in candle_inputs {
    for ext in &extensions {
      fragment_extensions.insert(ext.clone());
    }
    run_candle(settings, &wix_toolset_path, &output_path, path, extensions)?;
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
      Some(p) => fs::read_to_string(p)?,
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
      &format!("{unset_locale_strings}</WixLocalization>"),
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
    fs::create_dir_all(msi_path.parent().unwrap())?;

    log::info!(action = "Running"; "light to produce {}", display_path(&msi_path));

    run_light(
      &wix_toolset_path,
      &output_path,
      arguments,
      &(fragment_extensions.clone().into_iter().collect()),
      &msi_output_path,
    )?;
    fs::rename(&msi_output_path, &msi_path)?;

    if settings.windows().can_sign() {
      try_sign(&msi_path, settings)?;
    }

    output_paths.push(msi_path);
  }

  Ok(output_paths)
}

/// Generates the data required for the external binaries and extra binaries bundling.
fn generate_binaries_data(settings: &Settings) -> crate::Result<Vec<Binary>> {
  let mut binaries = Vec::new();
  let cwd = std::env::current_dir()?;
  let tmp_dir = std::env::temp_dir();
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
      id: wix_identifier(&dest_filename),
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
        id: wix_identifier(bin.name()),
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
    &PathBuf::from(glob::Pattern::escape(
      &settings.project_out_directory().to_string_lossy(),
    ))
    .join("*.msm")
    .to_string_lossy(),
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
fn generate_resource_data(settings: &Settings) -> crate::Result<ResourceDirectory> {
  let cwd = std::env::current_dir()?;

  let mut root_resource_directory = ResourceDirectory::default();
  let mut added_resources = HashSet::new();

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
    added_resources.insert(resource_path.clone());

    if settings.windows().can_sign() && should_sign(&resource_path)? {
      try_sign(&resource_path, settings)?;
    }

    let resource_entry = ResourceFile::new(
      resource_path,
      Some(
        resource
          .target()
          .file_name()
          .expect("failed to read resource file name")
          .to_string_lossy()
          .into_owned(),
      ),
    );

    let target_path = resource.target();
    let components_count = target_path.components().count();
    let directories = target_path
      .components()
      .take(components_count - 1) // the last component is the file
      .collect::<Vec<_>>();

    let mut directory_entry = &mut root_resource_directory;

    for directory in directories {
      let directory_name = directory
        .as_os_str()
        .to_os_string()
        .into_string()
        .expect("failed to read resource folder name");

      directory_entry = directory_entry
        .directories
        .entry(directory_name)
        .or_default();
    }
    directory_entry.add_file(resource_entry);
  }

  // Adding WebViewer2Loader.dll in case windows-gnu toolchain is used
  if settings.target().ends_with("-gnu") {
    let loader_path =
      dunce::simplified(&settings.project_out_directory().join("WebView2Loader.dll")).to_path_buf();

    if loader_path.exists() {
      if settings.windows().can_sign() {
        try_sign(&loader_path, settings)?;
      }
      added_resources.insert(loader_path.clone());
      let loader_resource = ResourceFile::new(loader_path, None);
      root_resource_directory.files.push(loader_resource);
    }
  }

  let mut dlls = Vec::new();

  if settings.windows().bundle_vc_runtime {
    for dll in vc_runtime_dlls(settings.binary_arch())? {
      let resource_path = dunce::simplified(&dll).to_path_buf();
      if added_resources.contains(&resource_path) {
        continue;
      }
      added_resources.insert(resource_path.to_path_buf());
      dlls.push(ResourceFile::new(resource_path.to_path_buf(), None));
    }
  }

  root_resource_directory.files.extend(dlls);

  // Handle CEF support if cef_path is set,
  // using https://github.com/chromiumembedded/cef/blob/master/tools/distrib/win/README.redistrib.txt as a reference
  if let Some(cef_path) = settings.bundle_settings().cef_path.as_ref() {
    let project_out = settings.project_out_directory();
    let cef_filenames = [
      // required
      "libcef.dll",
      "chrome_elf.dll",
      "icudtl.dat",
      "v8_context_snapshot.bin",
      // required end
      // "optional" - but not really since we want support for all of this
      "chrome_100_percent.pak",
      "chrome_200_percent.pak",
      "resources.pak",
      // Direct3D support
      "d3dcompiler_47.dll",
      // DirectX compiler support
      // TODO: check if x64 means no arm64
      "dxil.dll",
      "dxcompiler.dll",
      // ANGEL support
      "libEGL.dll",
      "libGLESv2.dll",
      // SwANGLE support
      "vk_swiftshader.dll",
      "vk_swiftshader_icd.json",
      "vulkan-1.dll",
      // sandbox - may need to be behind a setting?
      "bootstrap.exe",
      "bootstrapc.exe",
    ];

    let mut cef_files = Vec::with_capacity(cef_filenames.len());
    for f in cef_filenames {
      let from = cef_path.join(f);
      let path = dunce::simplified(&project_out.join(f)).to_path_buf();
      fs::copy(&from, &path).fs_context("failed to copy CEF file for MSI bundle", from)?;
      if settings.windows().can_sign() && should_sign(&path)? {
        try_sign(&path, settings)?;
      }
      cef_files.push(ResourceFile::new(path, None));
    }

    for f in &cef_files {
      added_resources.insert(f.source_path.clone());
    }

    root_resource_directory.files.append(&mut cef_files);

    // TODO: locales?
    // crash without at least en
    let locale_names = [
      "en-US.pak",
      "en-US_FEMININE.pak",
      "en-US_MASCULINE.pak",
      "en-US_NEUTER.pak",
    ];

    let locales_out = dunce::simplified(&project_out.join("locales")).to_path_buf();
    fs::create_dir_all(&locales_out).fs_context(
      "failed to create locales directory for CEF",
      locales_out.clone(),
    )?;

    let mut locales = Vec::with_capacity(locale_names.len());
    for f in locale_names {
      let from = cef_path.join("locales").join(f);
      let path = dunce::simplified(&locales_out.join(f)).to_path_buf();
      fs::copy(&from, &path).fs_context("failed to copy CEF locale for MSI bundle", from)?;
      locales.push(ResourceFile::new(path, None));
    }

    for f in &locales {
      added_resources.insert(f.source_path.clone());
    }

    root_resource_directory
      .directories
      .entry("locales".to_string())
      .or_default()
      .files
      .append(&mut locales);
  }

  Ok(root_resource_directory)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validates_wix_version() {
    assert!(validate_wix_version("1.1.1").is_ok());
    assert!(validate_wix_version("1.1.1.1").is_ok());
    assert!(validate_wix_version("255.1.1.1").is_ok());
    assert!(validate_wix_version("1.255.1.1").is_ok());
    assert!(validate_wix_version("1.1.65535.1").is_ok());
    assert!(validate_wix_version("1.1.1.65535").is_ok());

    assert!(validate_wix_version("256.1.1.1").is_err());
    assert!(validate_wix_version("1.256.1.1").is_err());
    assert!(validate_wix_version("1.1.65536.1").is_err());
    assert!(validate_wix_version("1.1.1.65536").is_err());
  }

  #[test]
  fn converts_version_to_wix() {
    assert_eq!(convert_version("1.1.2").unwrap(), "1.1.2");
    assert_eq!(convert_version("1.1.2-4").unwrap(), "1.1.2.4");
    assert_eq!(convert_version("1.1.2-65535").unwrap(), "1.1.2.65535");
    assert_eq!(convert_version("1.1.2+2").unwrap(), "1.1.2.2");

    assert!(convert_version("1.1.2-alpha").is_err());
    assert!(convert_version("1.1.2-alpha.4").is_err());
    assert!(convert_version("1.1.2+asd.3").is_err());
  }

  #[test]
  fn sanitizes_wix_identifiers() {
    assert_eq!(wix_identifier("7za.exe"), "_7za.exe");
    assert_eq!(wix_identifier("my-app.exe"), "my_app.exe");
    assert_eq!(wix_identifier("bad name!.exe"), "badname.exe");
    assert_eq!(wix_identifier(".bin"), "_.bin");
    assert_eq!(wix_identifier(""), "_");
    assert_eq!(wix_identifier("app_1.2"), "app_1.2");
  }

  #[test]
  fn includes_mapped_resource_file_name_in_wix_data() {
    let resource = ResourceFile::new("MyFile".into(), Some("myFileRenamed".into()));
    let resource_id = resource.id.clone();
    let directory = ResourceDirectory {
      files: vec![resource],
      directories: HashMap::new(),
    };

    let (wix_data, file_ids) = directory.render_wix(None).unwrap();

    assert_eq!(file_ids, vec![resource_id]);
    assert!(wix_data.contains(r#"Name="myFileRenamed""#));
    assert!(wix_data.contains(r#"Source="MyFile""#));
  }
}
