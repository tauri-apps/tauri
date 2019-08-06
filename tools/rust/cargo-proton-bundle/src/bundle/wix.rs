use super::settings::Settings;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use sha2::Digest;
use slog::info;
use slog::Logger;
use std::collections::BTreeMap;
use std::fs::{create_dir_all, remove_dir_all, write, File};
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;
use zip::ZipArchive;

pub const WIX_URL: &str =
  "https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311-binaries.zip";
pub const WIX_SHA256: &str = "37f0a533b0978a454efb5dc3bd3598becf9660aaf4287e55bf68ca6b527d051d";

const VC_REDIST_X86_URL: &str =
    "https://download.visualstudio.microsoft.com/download/pr/c8edbb87-c7ec-4500-a461-71e8912d25e9/99ba493d660597490cbb8b3211d2cae4/vc_redist.x86.exe";

const VC_REDIST_X86_SHA256: &str =
  "3a43e8a55a3f3e4b73d01872c16d47a19dd825756784f4580187309e7d1fcb74";

const VC_REDIST_X64_URL: &str =
    "https://download.visualstudio.microsoft.com/download/pr/9e04d214-5a9d-4515-9960-3d71398d98c3/1e1e62ab57bbb4bf5199e8ce88f040be/vc_redist.x64.exe";

const VC_REDIST_X64_SHA256: &str =
  "d6cd2445f68815fe02489fafe0127819e44851e26dfbe702612bc0d223cbbc2b";

// A v4 UUID that was generated specifically for cargo-bundle, to be used as a
// namespace for generating v5 UUIDs from bundle identifier strings.
const UUID_NAMESPACE: [u8; 16] = [
  0xfd, 0x85, 0x95, 0xa8, 0x17, 0xa3, 0x47, 0x4e, 0xa6, 0x16, 0x76, 0x14, 0x8d, 0xfa, 0x0c, 0x7b,
];

lazy_static! {
  static ref HANDLEBARS: Handlebars = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("main.wxs", include_str!("templates/main.wxs"))
      .unwrap();
    handlebars
  };
}

fn download_and_verify(logger: &Logger, url: &str, hash: &str) -> Result<Vec<u8>, String> {
  info!(logger, "Downloading {}", url);

  let mut response = reqwest::get(url).or_else(|e| Err(e.to_string()))?;

  let mut data: Vec<u8> = Vec::new();

  response
    .read_to_end(&mut data)
    .or_else(|e| Err(e.to_string()))?;

  info!(logger, "validating hash...");

  let mut hasher = sha2::Sha256::new();
  hasher.input(&data);

  let url_hash = hasher.result().to_vec();
  let expected_hash = hex::decode(hash).or_else(|e| Err(e.to_string()))?;

  if expected_hash == url_hash {
    Ok(data)
  } else {
    Err("hash mismatch of downloaded file".to_string())
  }
}

fn app_installer_dir(settings: &Settings) -> PathBuf {
  let arch = match settings.binary_arch() {
    "i686-pc-windows-msvc" => "x86",
    "x86_64-pc-windows-msvc" => "amd64",
    target => panic!("unsupported target: {}", target),
  };

  settings.project_out_directory().to_path_buf().join(format!(
    "{}.{}.msi",
    settings.bundle_name(),
    arch
  ))
}

fn extract_zip(data: &Vec<u8>, path: &Path) -> Result<(), String> {
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

fn generate_package_guid(settings: &Settings) -> Uuid {
  let namespace = Uuid::from_bytes(&UUID_NAMESPACE).unwrap();
  Uuid::new_v5(&namespace, &settings.bundle_identifier())
}

pub fn get_and_extract_wix(logger: &Logger, path: &Path) -> Result<(), String> {
  info!(logger, "downloading WIX Toolkit...");

  let data = download_and_verify(logger, WIX_URL, WIX_SHA256)?;

  info!(logger, "extracting WIX");

  extract_zip(&data, path)
}

fn run_heat_exe(
  logger: &Logger,
  wix_toolset_path: &Path,
  build_path: &Path,
  harvest_dir: &Path,
  platform: &str,
) -> Result<(), String> {
  let mut args = vec!["dir"];

  let harvest_str = harvest_dir.display().to_string();

  args.push(&harvest_str);
  args.push("-platform");
  args.push(platform);
  args.push("-cg");
  args.push("AppFiles");
  args.push("-dr");
  args.push("APPLICATIONFOLDER");
  args.push("-gg");
  args.push("-srd");
  args.push("-out");
  args.push("appdir.wxs");
  args.push("-var");
  args.push("var.SourceDir");

  let heat_exe = wix_toolset_path.join("head.exe");

  let mut cmd = Command::new(&heat_exe)
    .args(&args)
    .stdout(Stdio::piped())
    .current_dir(build_path)
    .spawn()
    .expect("error running heat.exe");

  {
    let stdout = cmd.stdout.as_mut().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
      info!(logger, "{}", line.unwrap());
    }
  }

  let status = cmd.wait().unwrap();
  if status.success() {
    Ok(())
  } else {
    Err("error running heat.exe".to_string())
  }
}

fn run_candle(
  settings: &Settings,
  logger: &Logger,
  wix_toolset_path: &Path,
  build_path: &Path,
  wxs_file_name: &str,
) -> Result<(), String> {
  let arch = "x64";

  let args = vec![
    "-ext".to_string(),
    "WixBalExtension".to_string(),
    "-ext".to_string(),
    "WixUtilExtension".to_string(),
    "-arch".to_string(),
    arch.to_string(),
    wxs_file_name.to_string(),
    format!("-dSourceDir={}", settings.binary_path().display()),
  ];

  let candle_exe = wix_toolset_path.join("candle.exe");
  info!(logger, "running candle for {}", wxs_file_name);

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
      info!(logger, "{}", line.unwrap());
    }
  }

  let status = cmd.wait().unwrap();
  if status.success() {
    Ok(())
  } else {
    Err("error running candle.exe".to_string())
  }
}

fn run_light(
  logger: &Logger,
  wix_toolset_path: &Path,
  build_path: &Path,
  wixobjs: &[&str],
  output_path: &Path,
) -> Result<(), String> {
  let light_exe = wix_toolset_path.join("light.exe");

  let mut args: Vec<String> = vec!["-o".to_string(), output_path.display().to_string()];

  for p in wixobjs {
    args.push(p.to_string());
  }

  info!(logger, "running light to produce {}", output_path.display());

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
      info!(logger, "{}", line.unwrap());
    }
  }

  let status = cmd.wait().unwrap();
  if status.success() {
    Ok(())
  } else {
    Err("error running light.exe".to_string())
  }
}

pub fn build_wix_app_installer(
  logger: &Logger,
  settings: &Settings,
  wix_toolset_path: &Path,
  current_dir: PathBuf,
) -> Result<(), String> {
  let arch = match settings.binary_arch() {
    "i686-pc-windows-msvc" => "x86",
    "x86_64-pc-windows-msvc" => "amd64",
    target => return Err(format!("unsupported target: {}", target)),
  };

  let output_path = settings.project_out_directory().join("wix").join(arch);

  let mut data = BTreeMap::new();

  data.insert("product_name", settings.bundle_name());
  data.insert("version", settings.version_string());
  let upgrade_code = if arch == "x86" {
    Uuid::new_v5(
      &uuid::NAMESPACE_DNS,
      format!("{}.app.x64", &settings.bundle_name()).as_str(),
    )
    .to_string()
  } else if arch == "x64" {
    Uuid::new_v5(
      &uuid::NAMESPACE_DNS,
      format!("{}.app.x64", &settings.bundle_name()).as_str(),
    )
    .to_string()
  } else {
    panic!("unsupported target: {}");
  };

  data.insert("upgrade_code", &upgrade_code);

  let path_guid = generate_package_guid(settings).to_string();
  data.insert("path_component_guid", &path_guid.as_str());

  let app_exe_name = settings
    .binary_path()
    .file_name()
    .unwrap()
    .to_string_lossy()
    .to_string();
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

  run_heat_exe(
    logger,
    &wix_toolset_path,
    &output_path,
    &Settings::get_workspace_dir(&current_dir),
    arch,
  )?;

  let input_basenames = vec!["main", "appdir"];

  for basename in &input_basenames {
    let wxs = format!("{}.wxs", basename);
    run_candle(settings, logger, &wix_toolset_path, &output_path, &wxs)?;
  }

  let wixobjs = vec!["main.wixobj", "appdir.wixobj"];
  run_light(
    logger,
    &wix_toolset_path,
    &output_path,
    &wixobjs,
    &app_installer_dir(settings),
  )?;

  Ok(())
}
