// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::{
    settings::Arch,
    windows::{
      sign::{sign_command, try_sign},
      util::{
        download_webview2_bootstrapper, download_webview2_offline_installer,
        NSIS_OUTPUT_FOLDER_NAME, NSIS_UPDATER_OUTPUT_FOLDER_NAME,
      },
    },
  },
  utils::{
    http_utils::{download_and_verify, verify_file_hash, HashAlgorithm},
    CommandExt,
  },
  Settings,
};
use tauri_utils::display_path;

use anyhow::Context;
use handlebars::{to_json, Handlebars};
use tauri_utils::config::{NSISInstallerMode, NsisCompression, WebviewInstallMode};

use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

// URLS for the NSIS toolchain.
#[cfg(target_os = "windows")]
const NSIS_URL: &str =
  "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3/nsis-3.zip";
#[cfg(target_os = "windows")]
const NSIS_SHA1: &str = "057e83c7d82462ec394af76c87d06733605543d4";
const NSIS_TAURI_UTILS_URL: &str =
  "https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.4.2/nsis_tauri_utils.dll";
const NSIS_TAURI_UTILS_SHA1: &str = "6532DA4545864C6EC95F62F27F2199BFD668560B";

#[cfg(target_os = "windows")]
const NSIS_REQUIRED_FILES: &[&str] = &[
  "makensis.exe",
  "Bin/makensis.exe",
  "Stubs/lzma-x86-unicode",
  "Stubs/lzma_solid-x86-unicode",
  "Plugins/x86-unicode/nsis_tauri_utils.dll",
  "Include/MUI2.nsh",
  "Include/FileFunc.nsh",
  "Include/x64.nsh",
  "Include/nsDialogs.nsh",
  "Include/WinMessages.nsh",
];
#[cfg(not(target_os = "windows"))]
const NSIS_REQUIRED_FILES: &[&str] = &["Plugins/x86-unicode/nsis_tauri_utils.dll"];

const NSIS_REQUIRED_FILES_HASH: &[(&str, &str, &str, HashAlgorithm)] = &[(
  "Plugins/x86-unicode/nsis_tauri_utils.dll",
  NSIS_TAURI_UTILS_URL,
  NSIS_TAURI_UTILS_SHA1,
  HashAlgorithm::Sha1,
)];

/// Runs all of the commands to build the NSIS installer.
/// Returns a vector of PathBuf that shows where the NSIS installer was created.
pub fn bundle_project(settings: &Settings, updater: bool) -> crate::Result<Vec<PathBuf>> {
  let tauri_tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| dirs::cache_dir().unwrap().join("tauri"));

  let nsis_toolset_path = tauri_tools_path.join("NSIS");

  if !nsis_toolset_path.exists() {
    get_and_extract_nsis(&nsis_toolset_path, &tauri_tools_path)?;
  } else if NSIS_REQUIRED_FILES
    .iter()
    .any(|p| !nsis_toolset_path.join(p).exists())
  {
    log::warn!("NSIS directory is missing some files. Recreating it.");
    std::fs::remove_dir_all(&nsis_toolset_path)?;
    get_and_extract_nsis(&nsis_toolset_path, &tauri_tools_path)?;
  } else {
    let mismatched = NSIS_REQUIRED_FILES_HASH
      .iter()
      .filter(|(p, _, hash, hash_algorithm)| {
        verify_file_hash(nsis_toolset_path.join(p), hash, *hash_algorithm).is_err()
      })
      .collect::<Vec<_>>();

    if !mismatched.is_empty() {
      log::warn!("NSIS directory contains mis-hashed files. Redownloading them.");
      for (path, url, hash, hash_algorithm) in mismatched {
        let data = download_and_verify(url, hash, *hash_algorithm)?;
        fs::write(nsis_toolset_path.join(path), data)?;
      }
    }
  }

  build_nsis_app_installer(settings, &nsis_toolset_path, &tauri_tools_path, updater)
}

// Gets NSIS and verifies the download via Sha1
fn get_and_extract_nsis(nsis_toolset_path: &Path, _tauri_tools_path: &Path) -> crate::Result<()> {
  log::info!("Verifying NSIS package");

  #[cfg(target_os = "windows")]
  {
    let data = download_and_verify(NSIS_URL, NSIS_SHA1, HashAlgorithm::Sha1)?;
    log::info!("extracting NSIS");
    crate::utils::http_utils::extract_zip(&data, _tauri_tools_path)?;
    fs::rename(_tauri_tools_path.join("nsis-3.08"), nsis_toolset_path)?;
  }

  let nsis_plugins = nsis_toolset_path.join("Plugins");

  let data = download_and_verify(
    NSIS_TAURI_UTILS_URL,
    NSIS_TAURI_UTILS_SHA1,
    HashAlgorithm::Sha1,
  )?;

  let target_folder = nsis_plugins.join("x86-unicode");
  fs::create_dir_all(&target_folder)?;
  fs::write(target_folder.join("nsis_tauri_utils.dll"), data)?;

  Ok(())
}

fn try_add_numeric_build_number(version_str: &str) -> anyhow::Result<String> {
  let version = semver::Version::parse(version_str).context("invalid app version")?;
  if !version.build.is_empty() {
    let build = version.build.parse::<u64>();
    if build.is_ok() {
      return Ok(format!(
        "{}.{}.{}.{}",
        version.major, version.minor, version.patch, version.build
      ));
    } else {
      log::warn!(
        "Unable to parse version build metadata. Numeric value expected, received: `{}`. This will be replaced with `0` in `VIProductVersion` because Windows requires this field to be numeric.",
        version.build
      );
    }
  }

  Ok(format!(
    "{}.{}.{}.0",
    version.major, version.minor, version.patch,
  ))
}

fn build_nsis_app_installer(
  settings: &Settings,
  _nsis_toolset_path: &Path,
  tauri_tools_path: &Path,
  updater: bool,
) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    Arch::X86_64 => "x64",
    Arch::X86 => "x86",
    Arch::AArch64 => "arm64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "unsupported architecture: {:?}",
        target
      )))
    }
  };

  log::info!("Target: {}", arch);

  let output_path = settings.project_out_directory().join("nsis").join(arch);
  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }
  fs::create_dir_all(&output_path)?;

  let mut data = BTreeMap::new();

  let bundle_id = settings.bundle_identifier();
  let manufacturer = settings
    .publisher()
    .unwrap_or_else(|| bundle_id.split('.').nth(1).unwrap_or(bundle_id));

  #[cfg(not(target_os = "windows"))]
  {
    let mut dir = dirs::cache_dir().unwrap();
    dir.extend(["tauri", "NSIS", "Plugins", "x86-unicode"]);
    data.insert("additional_plugins_path", to_json(dir));
  }

  data.insert("arch", to_json(arch));
  data.insert("bundle_id", to_json(bundle_id));
  data.insert("manufacturer", to_json(manufacturer));
  data.insert("product_name", to_json(settings.product_name()));
  data.insert("short_description", to_json(settings.short_description()));
  data.insert(
    "homepage",
    to_json(settings.homepage_url().unwrap_or_default()),
  );
  data.insert(
    "long_description",
    to_json(settings.long_description().unwrap_or_default()),
  );
  data.insert("copyright", to_json(settings.copyright_string()));

  if settings.can_sign() {
    let sign_cmd = format!("{:?}", sign_command("%1", &settings.sign_params())?);
    data.insert("uninstaller_sign_cmd", to_json(sign_cmd));
  }

  let version = settings.version_string();
  data.insert("version", to_json(version));
  data.insert(
    "version_with_build",
    to_json(try_add_numeric_build_number(version)?),
  );

  data.insert(
    "allow_downgrades",
    to_json(settings.windows().allow_downgrades),
  );

  if let Some(license_file) = settings.license_file() {
    let license_file = dunce::canonicalize(license_file)?;
    let license_file_with_bom = output_path.join("license_file");
    let content = std::fs::read(license_file)?;
    write_utf8_with_bom(&license_file_with_bom, content)?;
    data.insert("license", to_json(license_file_with_bom));
  }

  let nsis = settings.windows().nsis.as_ref();

  let custom_template_path = nsis.as_ref().and_then(|n| n.template.clone());

  let install_mode = nsis
    .as_ref()
    .map(|n| n.install_mode)
    .unwrap_or(NSISInstallerMode::CurrentUser);

  if let Some(nsis) = nsis {
    if let Some(installer_icon) = &nsis.installer_icon {
      data.insert(
        "installer_icon",
        to_json(dunce::canonicalize(installer_icon)?),
      );
    }

    if let Some(header_image) = &nsis.header_image {
      data.insert("header_image", to_json(dunce::canonicalize(header_image)?));
    }

    if let Some(sidebar_image) = &nsis.sidebar_image {
      data.insert(
        "sidebar_image",
        to_json(dunce::canonicalize(sidebar_image)?),
      );
    }

    if let Some(installer_hooks) = &nsis.installer_hooks {
      let installer_hooks = dunce::canonicalize(installer_hooks)?;
      data.insert("installer_hooks", to_json(installer_hooks));
    }

    if let Some(start_menu_folder) = &nsis.start_menu_folder {
      data.insert("start_menu_folder", to_json(start_menu_folder));
    }
    if let Some(minimum_webview2_version) = &nsis.minimum_webview2_version {
      data.insert(
        "minimum_webview2_version",
        to_json(minimum_webview2_version),
      );
    }
  }

  let compression = settings
    .windows()
    .nsis
    .as_ref()
    .map(|n| n.compression)
    .unwrap_or_default();
  data.insert(
    "compression",
    to_json(match compression {
      NsisCompression::Zlib => "zlib",
      NsisCompression::Bzip2 => "bzip2",
      NsisCompression::Lzma => "lzma",
      NsisCompression::None => "none",
    }),
  );

  data.insert(
    "install_mode",
    to_json(match install_mode {
      NSISInstallerMode::CurrentUser => "currentUser",
      NSISInstallerMode::PerMachine => "perMachine",
      NSISInstallerMode::Both => "both",
    }),
  );

  let languages = nsis
    .and_then(|nsis| nsis.languages.clone())
    .unwrap_or_else(|| vec!["English".into()]);
  data.insert("languages", to_json(languages.clone()));

  data.insert(
    "display_language_selector",
    to_json(
      nsis
        .map(|nsis| nsis.display_language_selector && languages.len() > 1)
        .unwrap_or(false),
    ),
  );

  let custom_language_files = nsis.and_then(|nsis| nsis.custom_language_files.clone());

  let mut language_files_paths = Vec::new();
  for lang in &languages {
    // if user provided a custom lang file, we rewrite it with BOM
    if let Some(path) = custom_language_files.as_ref().and_then(|h| h.get(lang)) {
      let path = dunce::canonicalize(path)?;
      let path_with_bom = path
        .file_name()
        .map(|f| output_path.join(f))
        .unwrap_or_else(|| output_path.join(format!("{lang}_custom.nsh")));
      let content = std::fs::read(path)?;
      write_utf8_with_bom(&path_with_bom, content)?;
      language_files_paths.push(path_with_bom);
    } else {
      // if user has not provided a custom lang file,
      // we check our translated languages
      if let Some((file_name, content)) = get_lang_data(lang) {
        let path = output_path.join(file_name);
        write_utf8_with_bom(&path, content)?;
        language_files_paths.push(path);
      } else {
        log::warn!("Custom tauri messages for {lang} are not translated.\nIf it is a valid language listed on <https://github.com/kichik/nsis/tree/9465c08046f00ccb6eda985abbdbf52c275c6c4d/Contrib/Language%20files>, please open a Tauri feature request\n or you can provide a custom language file for it in `tauri.conf.json > bundle > windows > nsis > custom_language_files`");
      }
    }
  }
  data.insert("language_files", to_json(language_files_paths));

  let main_binary = settings.main_binary()?;
  let main_binary_path = settings.binary_path(main_binary);
  data.insert("main_binary_name", to_json(main_binary.name()));
  data.insert("main_binary_path", to_json(&main_binary_path));

  let out_file = "nsis-output.exe";
  data.insert("out_file", to_json(out_file));

  let resources = generate_resource_data(settings)?;
  let resources_dirs =
    std::collections::HashSet::<PathBuf>::from_iter(resources.values().map(|r| r.0.to_owned()));

  let mut resources_ancestors = resources_dirs
    .iter()
    .flat_map(|p| p.ancestors())
    .collect::<Vec<_>>();
  resources_ancestors.sort_unstable();
  resources_ancestors.dedup();
  resources_ancestors.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
  resources_ancestors.pop(); // Last one is always ""

  // We need to convert / to \ for nsis to move the files into the correct dirs
  #[cfg(not(target_os = "windows"))]
  let resources: ResourcesMap = resources
    .into_iter()
    .map(|(r, p)| {
      (
        r,
        (
          p.0.display().to_string().replace('/', "\\").into(),
          p.1.display().to_string().replace('/', "\\").into(),
        ),
      )
    })
    .collect();
  #[cfg(not(target_os = "windows"))]
  let resources_ancestors: Vec<PathBuf> = resources_ancestors
    .into_iter()
    .map(|p| p.display().to_string().replace('/', "\\").into())
    .collect();
  #[cfg(not(target_os = "windows"))]
  let resources_dirs: Vec<PathBuf> = resources_dirs
    .into_iter()
    .map(|p| p.display().to_string().replace('/', "\\").into())
    .collect();

  data.insert("resources_ancestors", to_json(resources_ancestors));
  data.insert("resources_dirs", to_json(resources_dirs));
  data.insert("resources", to_json(&resources));

  let binaries = generate_binaries_data(settings)?;
  data.insert("binaries", to_json(&binaries));

  let estimated_size = generate_estimated_size(&main_binary_path, &binaries, &resources)?;
  data.insert("estimated_size", to_json(estimated_size));

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
    settings.windows().webview_install_mode.clone()
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
      let webview2_bootstrapper_path = download_webview2_bootstrapper(tauri_tools_path)?;
      data.insert(
        "webview2_bootstrapper_path",
        to_json(webview2_bootstrapper_path),
      );
    }
    WebviewInstallMode::OfflineInstaller { silent: _ } => {
      let webview2_installer_path =
        download_webview2_offline_installer(&tauri_tools_path.join(arch), arch)?;
      data.insert("webview2_installer_path", to_json(webview2_installer_path));
    }
    _ => {}
  }

  let mut handlebars = Handlebars::new();
  handlebars.register_helper("or", Box::new(handlebars_or));
  handlebars.register_helper("association-description", Box::new(association_description));
  handlebars.register_escape_fn(|s| {
    let mut output = String::new();
    for c in s.chars() {
      match c {
        '\"' => output.push_str("$\\\""),
        '$' => output.push_str("$$"),
        '`' => output.push_str("$\\`"),
        '\n' => output.push_str("$\\n"),
        '\t' => output.push_str("$\\t"),
        '\r' => output.push_str("$\\r"),
        _ => output.push(c),
      }
    }
    output
  });
  if let Some(path) = custom_template_path {
    handlebars
      .register_template_string("installer.nsi", std::fs::read_to_string(path)?)
      .map_err(|e| e.to_string())
      .expect("Failed to setup custom handlebar template");
  } else {
    handlebars
      .register_template_string("installer.nsi", include_str!("./installer.nsi"))
      .map_err(|e| e.to_string())
      .expect("Failed to setup handlebar template");
  }

  write_utf8_with_bom(
    output_path.join("FileAssociation.nsh"),
    include_bytes!("./FileAssociation.nsh"),
  )?;
  write_utf8_with_bom(output_path.join("utils.nsh"), include_bytes!("./utils.nsh"))?;

  let installer_nsi_path = output_path.join("installer.nsi");
  write_utf8_with_bom(
    &installer_nsi_path,
    handlebars.render("installer.nsi", &data)?,
  )?;

  let package_base_name = format!(
    "{}_{}_{}-setup",
    settings.product_name(),
    settings.version_string(),
    arch,
  );

  let nsis_output_path = output_path.join(out_file);
  let nsis_installer_path = settings.project_out_directory().to_path_buf().join(format!(
    "bundle/{}/{}.exe",
    if updater {
      NSIS_UPDATER_OUTPUT_FOLDER_NAME
    } else {
      NSIS_OUTPUT_FOLDER_NAME
    },
    package_base_name
  ));
  fs::create_dir_all(nsis_installer_path.parent().unwrap())?;

  log::info!(action = "Running"; "makensis.exe to produce {}", display_path(&nsis_installer_path));

  #[cfg(target_os = "windows")]
  let mut nsis_cmd = Command::new(_nsis_toolset_path.join("makensis.exe"));
  #[cfg(not(target_os = "windows"))]
  let mut nsis_cmd = Command::new("makensis");

  nsis_cmd
    .args(["-INPUTCHARSET", "UTF8", "-OUTPUTCHARSET", "UTF8"])
    .arg(match settings.log_level() {
      log::Level::Error => "-V1",
      log::Level::Warn => "-V2",
      log::Level::Info => "-V3",
      _ => "-V4",
    })
    .arg(installer_nsi_path)
    .env_remove("NSISDIR")
    .env_remove("NSISCONFDIR")
    .current_dir(output_path)
    .piped()
    .context("error running makensis.exe")?;

  fs::rename(nsis_output_path, &nsis_installer_path)?;

  if settings.can_sign() {
    try_sign(&nsis_installer_path, settings)?;
  } else {
    #[cfg(not(target_os = "windows"))]
    log::warn!("Signing, by default, is only supported on Windows hosts, but you can specify a custom signing command in `bundler > windows > sign_command`, for now, skipping signing the installer...");
  }

  Ok(vec![nsis_installer_path])
}

fn handlebars_or(
  h: &handlebars::Helper<'_>,
  _: &Handlebars<'_>,
  _: &handlebars::Context,
  _: &mut handlebars::RenderContext<'_, '_>,
  out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
  let param1 = h.param(0).unwrap().render();
  let param2 = h.param(1).unwrap();

  out.write(&if param1.is_empty() {
    param2.render()
  } else {
    param1
  })?;
  Ok(())
}

fn association_description(
  h: &handlebars::Helper<'_>,
  _: &Handlebars<'_>,
  _: &handlebars::Context,
  _: &mut handlebars::RenderContext<'_, '_>,
  out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
  let description = h.param(0).unwrap().render();
  let ext = h.param(1).unwrap();

  out.write(&if description.is_empty() {
    format!("{} File", ext.render().to_uppercase())
  } else {
    description
  })?;
  Ok(())
}

/// BTreeMap<OriginalPath, (ParentOfTargetPath, TargetPath)>
type ResourcesMap = BTreeMap<PathBuf, (PathBuf, PathBuf)>;
fn generate_resource_data(settings: &Settings) -> crate::Result<ResourcesMap> {
  let mut resources = ResourcesMap::new();

  let cwd = std::env::current_dir()?;

  let mut added_resources = Vec::new();

  // Adding WebViewer2Loader.dll in case windows-gnu toolchain is used
  if settings.target().ends_with("-gnu") {
    let loader_path =
      dunce::simplified(&settings.project_out_directory().join("WebView2Loader.dll")).to_path_buf();
    if loader_path.exists() {
      added_resources.push(loader_path.clone());
      resources.insert(
        loader_path,
        (PathBuf::new(), PathBuf::from("WebView2Loader.dll")),
      );
    }
  }

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

    let target_path = resource.target();
    resources.insert(
      resource_path,
      (
        target_path
          .parent()
          .expect("Couldn't get parent of target path")
          .to_path_buf(),
        target_path.to_path_buf(),
      ),
    );
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
    let binary_path = dunce::canonicalize(cwd.join(&src))?;
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

fn generate_estimated_size(
  main: &PathBuf,
  binaries: &BinariesMap,
  resources: &ResourcesMap,
) -> crate::Result<u64> {
  let mut size = 0;
  for k in std::iter::once(main)
    .chain(binaries.keys())
    .chain(resources.keys())
  {
    size += std::fs::metadata(k)
      .with_context(|| format!("when getting size of {}", k.display()))?
      .len();
  }
  Ok(size / 1024)
}

fn get_lang_data(lang: &str) -> Option<(String, &[u8])> {
  let path = format!("{lang}.nsh");
  let content: &[u8] = match lang.to_lowercase().as_str() {
    "arabic" => include_bytes!("./languages/Arabic.nsh"),
    "bulgarian" => include_bytes!("./languages/Bulgarian.nsh"),
    "dutch" => include_bytes!("./languages/Dutch.nsh"),
    "english" => include_bytes!("./languages/English.nsh"),
    "german" => include_bytes!("./languages/German.nsh"),
    "italian" => include_bytes!("./languages/Italian.nsh"),
    "japanese" => include_bytes!("./languages/Japanese.nsh"),
    "korean" => include_bytes!("./languages/Korean.nsh"),
    "portuguesebr" => include_bytes!("./languages/PortugueseBR.nsh"),
    "russian" => include_bytes!("./languages/Russian.nsh"),
    "tradchinese" => include_bytes!("./languages/TradChinese.nsh"),
    "simpchinese" => include_bytes!("./languages/SimpChinese.nsh"),
    "french" => include_bytes!("./languages/French.nsh"),
    "spanish" => include_bytes!("./languages/Spanish.nsh"),
    "spanishinternational" => include_bytes!("./languages/SpanishInternational.nsh"),
    "persian" => include_bytes!("./languages/Persian.nsh"),
    "turkish" => include_bytes!("./languages/Turkish.nsh"),
    "swedish" => include_bytes!("./languages/Swedish.nsh"),
    "portuguese" => include_bytes!("./languages/Portuguese.nsh"),
    "ukrainian" => include_bytes!("./languages/Ukrainian.nsh"),
    _ => return None,
  };
  Some((path, content))
}

fn write_utf8_with_bom<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) -> crate::Result<()> {
  use std::fs::File;
  use std::io::{BufWriter, Write};

  let file = File::create(path)?;
  let mut output = BufWriter::new(file);
  output.write_all(&[0xEF, 0xBB, 0xBF])?; // UTF-8 BOM
  output.write_all(content.as_ref())?;
  Ok(())
}
