use super::common;
use super::settings::Settings;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use sha2::Digest;

use std::collections::BTreeMap;
use std::fs::{create_dir_all, remove_dir_all, write, File};
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;
use zip::ZipArchive;

// URLS for the WIX toolchain.  Can be used for crossplatform compilation.
pub const WIX_URL: &str =
  "https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip";
pub const WIX_SHA256: &str = "37f0a533b0978a454efb5dc3bd3598becf9660aaf4287e55bf68ca6b527d051d";

// For Cross Platform Complilation.

// const VC_REDIST_X86_URL: &str =
//     "https://download.visualstudio.microsoft.com/download/pr/c8edbb87-c7ec-4500-a461-71e8912d25e9/99ba493d660597490cbb8b3211d2cae4/vc_redist.x86.exe";

// const VC_REDIST_X86_SHA256: &str =
//   "3a43e8a55a3f3e4b73d01872c16d47a19dd825756784f4580187309e7d1fcb74";

// const VC_REDIST_X64_URL: &str =
//     "https://download.visualstudio.microsoft.com/download/pr/9e04d214-5a9d-4515-9960-3d71398d98c3/1e1e62ab57bbb4bf5199e8ce88f040be/vc_redist.x64.exe";

// const VC_REDIST_X64_SHA256: &str =
//   "d6cd2445f68815fe02489fafe0127819e44851e26dfbe702612bc0d223cbbc2b";

// A v4 UUID that was generated specifically for cargo-bundle, to be used as a
// namespace for generating v5 UUIDs from bundle identifier strings.
const UUID_NAMESPACE: [u8; 16] = [
  0xfd, 0x85, 0x95, 0xa8, 0x17, 0xa3, 0x47, 0x4e, 0xa6, 0x16, 0x76, 0x14, 0x8d, 0xfa, 0x0c, 0x7b,
];

// setup for the main.wxs template file using handlebars. Dynamically changes the template on compilation based on the application metadata.
lazy_static! {
  static ref HANDLEBARS: Handlebars = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("main.wxs", include_str!("templates/main.wxs"))
      .unwrap();
    handlebars
  };
}

// Function used to download Wix and VC_REDIST. Checks SHA256 to verify the download.
fn download_and_verify(url: &str, hash: &str) -> crate::Result<Vec<u8>> {
  common::print_info(format!("Downloading {}", url).as_str())?;

  let mut response = reqwest::get(url).or_else(|e| Err(e.to_string()))?;

  let mut data: Vec<u8> = Vec::new();

  response
    .read_to_end(&mut data)
    .or_else(|e| Err(e.to_string()))?;

  common::print_info("validating hash")?;

  let mut hasher = sha2::Sha256::new();
  hasher.input(&data);

  let url_hash = hasher.result().to_vec();
  let expected_hash = hex::decode(hash).or_else(|e| Err(e.to_string()))?;

  if expected_hash == url_hash {
    Ok(data)
  } else {
    Err(crate::Error::from("hash mismatch of downloaded file"))
  }
}

fn app_installer_dir(settings: &Settings) -> crate::Result<PathBuf> {
  let arch = match settings.binary_arch() {
    "x86_64" => "x86",
    "x64" => "x64",
    target => {
      return Err(crate::Error::from(format!(
        "Unsupported architecture: {}",
        target
      )))
    }
  };

  Ok(settings.project_out_directory().to_path_buf().join(format!(
    "{}.{}.msi",
    settings.bundle_name(),
    arch
  )))
}

// Extracts the zips from Wix and VC_REDIST into a useable path.
fn extract_zip(data: &Vec<u8>, path: &Path) -> crate::Result<()> {
  let cursor = Cursor::new(data);

  let mut zipa = ZipArchive::new(cursor).or_else(|e| Err(e.to_string()))?;

  for i in 0..zipa.len() {
    let mut file = zipa.by_index(i).or_else(|e| Err(e.to_string()))?;
    let dest_path = path.join(file.name());
    let parent = dest_path.parent().unwrap();

    if !parent.exists() {
      create_dir_all(parent).or_else(|e| Err(e.to_string()))?;
    }

    let mut buff: Vec<u8> = Vec::new();
    file
      .read_to_end(&mut buff)
      .or_else(|e| Err(e.to_string()))?;
    let mut fileout = File::create(dest_path).unwrap();

    fileout.write_all(&buff).or_else(|e| Err(e.to_string()))?;
  }

  Ok(())
}

// Generates the UUID for the Wix template.
fn generate_package_guid(settings: &Settings) -> Uuid {
  let namespace = Uuid::from_bytes(&UUID_NAMESPACE).unwrap();
  Uuid::new_v5(&namespace, &settings.bundle_identifier())
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

// Runs the Candle.exe executable for Wix.  Candle parses the wxs file and generates the code for building the installer.
fn run_candle(
  settings: &Settings,
  wix_toolset_path: &Path,
  build_path: &Path,
  wxs_file_name: &str,
) -> crate::Result<()> {
  let arch = "x64";

  let args = vec![
    "-arch".to_string(),
    arch.to_string(),
    wxs_file_name.to_string(),
    format!("-dSourceDir={}", settings.binary_path().display()),
  ];

  let candle_exe = wix_toolset_path.join("candle.exe");
  common::print_info(format!("running candle for {}", wxs_file_name).as_str())?;

  let mut cmd = Command::new(&candle_exe)
    .args(&args)
    .stdout(Stdio::piped())
    .current_dir(build_path)
    .spawn()
    .expect("error running candle.exe");
  {
    let stdout = cmd.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      common::print_info(line.unwrap().as_str())?;
    }
  }

  let status = cmd.wait().unwrap();
  if status.success() {
    Ok(())
  } else {
    Err(crate::Error::from("error running candle.exe"))
  }
}

// Runs the Light.exe file.  Light takes the generated code from Candle and produces an MSI Installer.
fn run_light(
  wix_toolset_path: &Path,
  build_path: &Path,
  wixobjs: &[&str],
  output_path: &Path,
) -> crate::Result<PathBuf> {
  let light_exe = wix_toolset_path.join("light.exe");

  let mut args: Vec<String> = vec!["-o".to_string(), output_path.display().to_string()];

  for p in wixobjs {
    args.push(p.to_string());
  }

  common::print_info(format!("running light to produce {}", output_path.display()).as_str())?;

  let mut cmd = Command::new(&light_exe)
    .args(&args)
    .stdout(Stdio::piped())
    .current_dir(build_path)
    .spawn()
    .expect("error running light.exe");
  {
    let stdout = cmd.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      common::print_info(line.unwrap().as_str())?;
    }
  }

  let status = cmd.wait().unwrap();
  if status.success() {
    Ok(output_path.to_path_buf())
  } else {
    Err(crate::Error::from("error running light.exe"))
  }
}

// Entry point for bundling and creating the MSI installer.  For now the only supported platform is Windows x64.
pub fn build_wix_app_installer(
  settings: &Settings,
  wix_toolset_path: &Path,
) -> crate::Result<PathBuf> {
  let arch = match settings.binary_arch() {
    "x86_64" => "x64",
    "x86" => "x86",
    target => {
      return Err(crate::Error::from(format!(
        "unsupported target: {}",
        target
      )))
    }
  };

  // common::print_warning("Only x64 supported")?;
  // target only supports x64.
  common::print_info(format!("Target: {}", arch).as_str())?;

  let output_path = settings.project_out_directory().join("wix").join(arch);

  let mut data = BTreeMap::new();

  data.insert("product_name", settings.bundle_name());
  data.insert("version", settings.version_string());
  let manufacturer = settings.bundle_identifier().to_string();
  data.insert("manufacturer", manufacturer.as_str());
  let upgrade_code = Uuid::new_v5(
    &uuid::NAMESPACE_DNS,
    format!("{}.app.x64", &settings.binary_name()).as_str(),
  )
  .to_string();

  data.insert("upgrade_code", &upgrade_code.as_str());

  let path_guid = generate_package_guid(settings).to_string();
  data.insert("path_component_guid", &path_guid.as_str());

  let app_exe_name = settings.binary_name().to_string();
  data.insert("app_exe_name", &app_exe_name);

  let app_exe_source = settings.binary_path().display().to_string();

  data.insert("app_exe_source", &app_exe_source);

  let temp = HANDLEBARS
    .render("main.wxs", &data)
    .or_else(|e| Err(e.to_string()))?;

  if output_path.exists() {
    remove_dir_all(&output_path).or_else(|e| Err(e.to_string()))?;
  }

  create_dir_all(&output_path).or_else(|e| Err(e.to_string()))?;

  let main_wxs_path = output_path.join("main.wxs");
  write(&main_wxs_path, temp).or_else(|e| Err(e.to_string()))?;

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
