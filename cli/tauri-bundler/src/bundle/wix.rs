use super::common;
use super::path_utils::{copy, Options};
use super::settings::Settings;

use handlebars::{to_json, Handlebars};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use sha2::Digest;
use uuid::Uuid;
use zip::ZipArchive;

use std::collections::BTreeMap;
use std::fs::{create_dir_all, remove_dir_all, write, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// URLS for the WIX toolchain.  Can be used for crossplatform compilation.
pub const WIX_URL: &str =
  "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip";
pub const WIX_SHA256: &str = "2c1888d5d1dba377fc7fa14444cf556963747ff9a0a289a3599cf09da03b9e2e";

// For Cross Platform Complilation.

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

// setup for the main.wxs template file using handlebars. Dynamically changes the template on compilation based on the application metadata.
lazy_static! {
  static ref HANDLEBARS: Handlebars<'static> = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("main.wxs", include_str!("templates/main.wxs"))
      .or_else(|e| Err(e.to_string()))
      .expect("Failed to setup handlebar template");
    handlebars
  };
}

/// Mapper between a resource directory name and its ResourceDirectory descriptor.
type ResourceMap = BTreeMap<String, ResourceDirectory>;

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
  path: String,
}

/// A resource directory to bundle with WIX.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize)]
struct ResourceDirectory {
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
          path = file.path
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
    let wix_string = if self.name == "" {
      format!("{}{}", files, directories)
    } else {
      format!(
        r#"<Directory Id="{name}" Name="{name}">{contents}</Directory>"#,
        name = self.name,
        contents = format!("{}{}", files, directories)
      )
    };

    Ok((wix_string, file_ids))
  }
}

/// Copies the icons to the binary path, under the `resources` folder,
/// and returns the path to that directory.
fn copy_icons(settings: &Settings) -> crate::Result<PathBuf> {
  let base_dir = settings.project_out_directory();

  let resource_dir = base_dir.join("resources");

  let mut image_path = PathBuf::from(settings.project_out_directory());

  // pop off till in tauri_src dir
  image_path.pop();
  image_path.pop();

  // get icon dir and icon file.
  let image_path = image_path.join("icons");
  let opts = super::path_utils::Options::default();

  copy(
    image_path,
    &resource_dir,
    &Options {
      copy_files: true,
      overwrite: true,
      ..opts
    },
  )?;

  Ok(resource_dir)
}

/// Function used to download Wix and VC_REDIST. Checks SHA256 to verify the download.
fn download_and_verify(url: &str, hash: &str) -> crate::Result<Vec<u8>> {
  common::print_info(format!("Downloading {}", url).as_str())?;

  let response = attohttpc::get(url).send()?;

  let data: Vec<u8> = response.bytes()?;

  common::print_info("validating hash")?;

  let mut hasher = sha2::Sha256::new();
  hasher.update(&data);

  let url_hash = hasher.finalize().to_vec();
  let expected_hash = hex::decode(hash)?;

  if expected_hash == url_hash {
    Ok(data)
  } else {
    Err(crate::Error::HashError)
  }
}

/// The installer directory of the app.
fn app_installer_dir(settings: &Settings) -> crate::Result<PathBuf> {
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
    "{}_{}_{}",
    settings.main_binary_name().replace(".exe", ""),
    settings.version_string(),
    arch
  );

  Ok(
    settings
      .project_out_directory()
      .to_path_buf()
      .join(format!("bundle/msi/{}.msi", package_base_name)),
  )
}

/// Extracts the zips from Wix and VC_REDIST into a useable path.
fn extract_zip(data: &[u8], path: &Path) -> crate::Result<()> {
  let cursor = Cursor::new(data);

  let mut zipa = ZipArchive::new(cursor)?;

  for i in 0..zipa.len() {
    let mut file = zipa.by_index(i)?;
    let dest_path = path.join(file.name());
    let parent = dest_path.parent().expect("Failed to get parent");

    if !parent.exists() {
      create_dir_all(parent)?;
    }

    let mut buff: Vec<u8> = Vec::new();
    file.read_to_end(&mut buff)?;
    let mut fileout = File::create(dest_path).expect("Failed to open file");

    fileout.write_all(&buff)?;
  }

  Ok(())
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
  common::print_info("Verifying wix package")?;

  let data = download_and_verify(WIX_URL, WIX_SHA256)?;

  common::print_info("extracting WIX")?;

  extract_zip(&data, path)
}

// For if bundler needs DLL files.

// fn run_heat_exe(
//   wix_toolset_path: &Path,
//   build_path: &Path,
//   harvest_dir: &Path,
//   platform: &str,
// ) -> Result<(), String> {
//   let mut args = vec!["dir"];

//   let harvest_str = harvest_dir.display().to_string();

//   args.push(&harvest_str);
//   args.push("-platform");
//   args.push(platform);
//   args.push("-cg");
//   args.push("AppFiles");
//   args.push("-dr");
//   args.push("APPLICATIONFOLDER");
//   args.push("-gg");
//   args.push("-srd");
//   args.push("-out");
//   args.push("appdir.wxs");
//   args.push("-var");
//   args.push("var.SourceDir");

//   let heat_exe = wix_toolset_path.join("heat.exe");

//   let mut cmd = Command::new(&heat_exe)
//     .args(&args)
//     .stdout(Stdio::piped())
//     .current_dir(build_path)
//     .spawn()
//     .expect("error running heat.exe");

//   {
//     let stdout = cmd.stdout.as_mut().unwrap();
//     let reader = BufReader::new(stdout);

//     for line in reader.lines() {
//       info!(logger, "{}", line.unwrap());
//     }
//   }

//   let status = cmd.wait().unwrap();
//   if status.success() {
//     Ok(())
//   } else {
//     Err("error running heat.exe".to_string())
//   }
// }

/// Runs the Candle.exe executable for Wix. Candle parses the wxs file and generates the code for building the installer.
fn run_candle(
  settings: &Settings,
  wix_toolset_path: &Path,
  build_path: &Path,
  wxs_file_name: &str,
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

  let args = vec![
    "-arch".to_string(),
    arch.to_string(),
    wxs_file_name.to_string(),
    format!(
      "-dSourceDir={}",
      settings.binary_path(main_binary).display()
    ),
  ];

  let candle_exe = wix_toolset_path.join("candle.exe");
  common::print_info(format!("running candle for {}", wxs_file_name).as_str())?;

  let mut cmd = Command::new(&candle_exe);
  cmd
    .args(&args)
    .stdout(Stdio::piped())
    .current_dir(build_path);

  common::print_info("running candle.exe")?;
  common::execute_with_verbosity(&mut cmd, &settings).map_err(|_| crate::Error::CandleError)
}

/// Runs the Light.exe file. Light takes the generated code from Candle and produces an MSI Installer.
fn run_light(
  wix_toolset_path: &Path,
  build_path: &Path,
  wixobjs: &[&str],
  output_path: &Path,
) -> crate::Result<PathBuf> {
  let light_exe = wix_toolset_path.join("light.exe");

  let mut args: Vec<String> = vec![
    "-ext".to_string(),
    "WixUIExtension".to_string(),
    "-o".to_string(),
    output_path.display().to_string(),
  ];

  for p in wixobjs {
    args.push((*p).to_string());
  }

  let mut cmd = Command::new(&light_exe);
  cmd
    .args(&args)
    .stdout(Stdio::piped())
    .current_dir(build_path);

  common::print_info(format!("running light to produce {}", output_path.display()).as_str())?;
  common::execute_with_verbosity(&mut cmd, &settings)
    .map(|_| output_path.to_path_buf())
    .map_err(|_| crate::Error::LightError)
}

// fn get_icon_data() -> crate::Result<()> {
//   Ok(())
// }

// Entry point for bundling and creating the MSI installer. For now the only supported platform is Windows x64.
pub fn build_wix_app_installer(
  settings: &Settings,
  wix_toolset_path: &Path,
) -> crate::Result<PathBuf> {
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

  // target only supports x64.
  common::print_info(format!("Target: {}", arch).as_str())?;

  let output_path = settings
    .project_out_directory()
    .join("bundle/msi")
    .join(arch);

  let mut data = BTreeMap::new();

  if let Ok(tauri_config) = crate::bundle::tauri_config::get() {
    data.insert(
      "embedded_server",
      to_json(tauri_config.tauri.embedded_server.active),
    );
  }

  data.insert("product_name", to_json(settings.bundle_name()));
  data.insert("version", to_json(settings.version_string()));
  let manufacturer = settings.bundle_identifier().to_string();
  data.insert("manufacturer", to_json(manufacturer.as_str()));
  let upgrade_code = Uuid::new_v5(
    &Uuid::NAMESPACE_DNS,
    format!("{}.app.x64", &settings.main_binary_name()).as_bytes(),
  )
  .to_string();

  data.insert("upgrade_code", to_json(&upgrade_code.as_str()));

  let path_guid = generate_package_guid(settings).to_string();
  data.insert("path_component_guid", to_json(&path_guid.as_str()));

  let shortcut_guid = generate_package_guid(settings).to_string();
  data.insert("shortcut_guid", to_json(&shortcut_guid.as_str()));

  let app_exe_name = settings.main_binary_name().to_string();
  data.insert("app_exe_name", to_json(&app_exe_name));

  let binaries = generate_binaries_data(&settings)?;

  let binaries_json = to_json(&binaries);
  data.insert("binaries", binaries_json);

  let resources = generate_resource_data(&settings)?;
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

  let main_binary = settings
    .binaries()
    .iter()
    .find(|bin| bin.main())
    .ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;
  let app_exe_source = settings.binary_path(main_binary).display().to_string();

  data.insert("app_exe_source", to_json(&app_exe_source));

  // copy icons from icons folder to resource folder near msi
  let image_path = copy_icons(&settings)?;

  let path = image_path.join("icon.ico").display().to_string();

  data.insert("icon_path", to_json(path.as_str()));

  let temp = HANDLEBARS.render("main.wxs", &data)?;

  if output_path.exists() {
    remove_dir_all(&output_path)?;
  }

  create_dir_all(&output_path)?;

  let main_wxs_path = output_path.join("main.wxs");
  write(&main_wxs_path, temp)?;

  let input_basenames = vec!["main"];

  for basename in &input_basenames {
    let wxs = format!("{}.wxs", basename);
    run_candle(settings, &wix_toolset_path, &output_path, &wxs)?;
  }

  let wixobjs = vec!["main.wixobj"];
  let target = run_light(
    &wix_toolset_path,
    &output_path,
    &wixobjs,
    &app_installer_dir(settings)?,
  )?;

  Ok(target)
}

/// Generates the data required for the external binaries and extra binaries bundling.
fn generate_binaries_data(settings: &Settings) -> crate::Result<Vec<Binary>> {
  let mut binaries = Vec::new();
  let regex = Regex::new(r"[^\w\d\.]")?;
  let cwd = std::env::current_dir()?;
  for src in settings.external_binaries() {
    let src = src?;
    let filename = src
      .file_name()
      .expect("failed to extract external binary filename")
      .to_os_string()
      .into_string()
      .expect("failed to convert external binary filename to string");

    let guid = generate_guid(filename.as_bytes()).to_string();

    binaries.push(Binary {
      guid,
      path: cwd
        .join(src)
        .into_os_string()
        .into_string()
        .expect("failed to read external binary path"),
      id: regex.replace_all(&filename, "").to_string(),
    });
  }

  for bin in settings.binaries() {
    let filename = bin.name();
    let guid = generate_guid(filename.as_bytes()).to_string();
    if !bin.main() {
      binaries.push(Binary {
        guid,
        path: settings
          .binary_path(bin)
          .into_os_string()
          .into_string()
          .expect("failed to read binary path"),
        id: regex.replace_all(&filename, "").to_string(),
      })
    }
  }

  Ok(binaries)
}

/// Generates the data required for the resource bundling on wix
fn generate_resource_data(settings: &Settings) -> crate::Result<ResourceMap> {
  let mut resources = ResourceMap::new();
  let regex = Regex::new(r"[^\w\d\.]")?;
  let cwd = std::env::current_dir()?;

  let mut dlls = vec![];
  for dll in glob::glob(
    settings
      .project_out_directory()
      .join("*.dll")
      .to_string_lossy()
      .to_string()
      .as_str(),
  )? {
    let path = dll?;
    let filename = path
      .file_name()
      .expect("failed to extract resource filename")
      .to_os_string()
      .into_string()
      .expect("failed to convert resource filename to string");
    dlls.push(ResourceFile {
      guid: generate_guid(filename.as_bytes()).to_string(),
      path: path.to_string_lossy().to_string(),
      id: regex.replace_all(&filename, "").to_string(),
    });
  }
  if !dlls.is_empty() {
    resources.insert(
      "".to_string(),
      ResourceDirectory {
        name: "".to_string(),
        directories: vec![],
        files: dlls,
      },
    );
  }

  for src in settings.resource_files() {
    let src = src?;

    let filename = src
      .file_name()
      .expect("failed to extract resource filename")
      .to_os_string()
      .into_string()
      .expect("failed to convert resource filename to string");

    let resource_path = cwd
      .join(src.clone())
      .into_os_string()
      .into_string()
      .expect("failed to read resource path");

    let resource_entry = ResourceFile {
      guid: generate_guid(filename.as_bytes()).to_string(),
      path: resource_path,
      id: regex.replace_all(&filename, "").to_string(),
    };

    // split the resource path directories
    let mut directories = src
      .components()
      .filter(|component| {
        let comp = component.as_os_str();
        comp != "." && comp != ".."
      })
      .collect::<Vec<_>>();
    directories.truncate(directories.len() - 1);
    // transform the directory structure to a chained vec structure
    for directory in directories {
      let directory_name = directory
        .as_os_str()
        .to_os_string()
        .into_string()
        .expect("failed to read resource folder name");

      // if the directory is already on the map
      if resources.contains_key(&directory_name) {
        let directory_entry = &mut resources
          .get_mut(&directory_name)
          .expect("Unable to handle resources");
        if directory_entry.name == directory_name {
          // the directory entry is the root of the chain
          directory_entry.add_file(resource_entry.clone());
        } else {
          let index = directory_entry
            .directories
            .iter()
            .position(|f| f.name == directory_name);
          if index.is_some() {
            // the directory entry is already a part of the chain
            let dir = directory_entry
              .directories
              .get_mut(index.expect("Unable to get index"))
              .expect("Unable to get directory");
            dir.add_file(resource_entry.clone());
          } else {
            // push it to the chain
            directory_entry.directories.push(ResourceDirectory {
              name: directory_name.clone(),
              directories: vec![],
              files: vec![resource_entry.clone()],
            });
          }
        }
      } else {
        resources.insert(
          directory_name.clone(),
          ResourceDirectory {
            name: directory_name.clone(),
            directories: vec![],
            files: vec![resource_entry.clone()],
          },
        );
      }
    }
  }

  Ok(resources)
}
