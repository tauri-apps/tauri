// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  path::{Path, PathBuf},
};
#[cfg(windows)]
use std::{io::Write, process::Command};
use ureq::ResponseExt;

use crate::bundle::settings::Arch;
use crate::utils::http_utils::{base_ureq_agent, download};

pub const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
pub const WEBVIEW2_OFFLINE_INSTALLER_X86_URL: &str =
  "https://go.microsoft.com/fwlink/?linkid=2099617";
pub const WEBVIEW2_OFFLINE_INSTALLER_X64_URL: &str =
  "https://go.microsoft.com/fwlink/?linkid=2124701";
pub const WEBVIEW2_URL_PREFIX: &str =
  "https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/";
pub const NSIS_OUTPUT_FOLDER_NAME: &str = "nsis";
pub const NSIS_UPDATER_OUTPUT_FOLDER_NAME: &str = "nsis-updater";
pub const WIX_OUTPUT_FOLDER_NAME: &str = "msi";
pub const WIX_UPDATER_OUTPUT_FOLDER_NAME: &str = "msi-updater";

#[cfg(windows)]
const VSWHERE: &[u8] = include_bytes!("vswhere.exe");
const VCTOOLS_REDIST_DIR_ENV_VAR: &str = "VCTOOLS_REDIST_DIR";
#[cfg(windows)]
const VC_REDIST_COMPONENT: &str = "Microsoft.VisualStudio.Component.VC.Redist.14.Latest";

pub fn webview2_guid_path(url: &str) -> crate::Result<(String, String)> {
  let agent = base_ureq_agent();
  let response = agent.head(url).call().map_err(Box::new)?;
  let final_url = response.get_uri().to_string();
  let remaining_url = final_url.strip_prefix(WEBVIEW2_URL_PREFIX).ok_or_else(|| {
    crate::Error::GenericError(format!(
      "WebView2 URL prefix mismatch. Expected `{WEBVIEW2_URL_PREFIX}`, found `{final_url}`."
    ))
  })?;
  let (guid, filename) = remaining_url.split_once('/').ok_or_else(|| {
    crate::Error::GenericError(format!(
      "WebView2 URL format mismatch. Expected `<GUID>/<FILENAME>`, found `{remaining_url}`."
    ))
  })?;
  Ok((guid.into(), filename.into()))
}

pub fn download_webview2_bootstrapper(base_path: &Path) -> crate::Result<PathBuf> {
  let file_path = base_path.join("MicrosoftEdgeWebview2Setup.exe");
  if !file_path.exists() {
    std::fs::write(&file_path, download(WEBVIEW2_BOOTSTRAPPER_URL)?)?;
  }
  Ok(file_path)
}

pub fn download_webview2_offline_installer(base_path: &Path, arch: &str) -> crate::Result<PathBuf> {
  let url = if arch == "x64" {
    WEBVIEW2_OFFLINE_INSTALLER_X64_URL
  } else {
    WEBVIEW2_OFFLINE_INSTALLER_X86_URL
  };
  let (guid, filename) = webview2_guid_path(url)?;
  let dir_path = base_path.join(guid);
  let file_path = dir_path.join(filename);
  if !file_path.exists() {
    fs::create_dir_all(dir_path)?;
    std::fs::write(&file_path, download(url)?)?;
  }
  Ok(file_path)
}

/// Finds the Visual C++ runtime DLLs for the given architecture.
pub fn vc_runtime_dlls(arch: Arch) -> crate::Result<Vec<PathBuf>> {
  let arch = vc_runtime_arch(arch)?;
  let redist_dir = vc_redist_dir()?;
  let runtime_dir = vc_runtime_dir(&redist_dir, arch)?;

  let dlls = glob::glob(&glob_path(&runtime_dir, "*.dll"))?.collect::<Result<Vec<_>, _>>()?;
  if dlls.is_empty() {
    return Err(crate::Error::GenericError(format!(
      "no Visual C++ runtime DLLs found in `{}`",
      runtime_dir.display()
    )));
  }

  Ok(dlls)
}

#[inline(always)]
fn vc_runtime_arch(arch: Arch) -> crate::Result<&'static str> {
  match arch {
    Arch::X86_64 => Ok("x64"),
    Arch::X86 => Ok("x86"),
    Arch::AArch64 => Ok("arm64"),
    _ => Err(crate::Error::GenericError(
      "bundling the Visual C++ runtime is only supported for Windows x86, x64 and arm64 targets"
        .into(),
    )),
  }
}

#[cfg(windows)]
fn visual_studio_dir() -> crate::Result<PathBuf> {
  let Some(vswhere) = vswhere_path() else {
    return Err(crate::Error::GenericError(
      "failed to prepare bundled vswhere.exe".into(),
    ));
  };

  let output = Command::new(vswhere)
    .args([
      "-latest",
      "-prerelease",
      "-products",
      "*",
      "-requires",
      VC_REDIST_COMPONENT,
      "-property",
      "installationPath",
      "-format",
      "value",
      "-utf8",
    ])
    .output()?;

  if !output.status.success() {
    return Err(crate::Error::GenericError(format!(
      "failed to locate Visual Studio with the {VC_REDIST_COMPONENT} component"
    )));
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  let Some(vs_dir) = stdout.lines().map(str::trim).find(|line| !line.is_empty()) else {
    return Err(crate::Error::GenericError(format!(
      "failed to locate Visual Studio with the {VC_REDIST_COMPONENT} component"
    )));
  };

  Ok(PathBuf::from(vs_dir))
}

fn vc_redist_dir() -> crate::Result<PathBuf> {
  if let Ok(redist_dir) = std::env::var(VCTOOLS_REDIST_DIR_ENV_VAR) {
    return Ok(PathBuf::from(redist_dir));
  }

  #[cfg(windows)]
  {
    let vs_dir = visual_studio_dir()?;
    Ok(vs_dir.join("VC/Redist/MSVC"))
  }

  #[cfg(not(windows))]
  {
    Err(crate::Error::GenericError(format!(
      "failed to find Visual C++ runtime redist directory; set {VCTOOLS_REDIST_DIR_ENV_VAR} when bundling the Visual C++ runtime from non-Windows hosts"
    )))
  }
}

fn vc_runtime_dir(redist_dir: &Path, arch: &str) -> crate::Result<PathBuf> {
  let Some(latest_version_dir) = latest_vc_redist_version_dir(redist_dir)? else {
    return Err(crate::Error::GenericError(format!(
      "failed to find Visual C++ runtime versions in `{}`",
      redist_dir.display()
    )));
  };

  let arch_dir = latest_version_dir.join(arch);
  let Some(runtime_dir) = glob::glob(&glob_path(&arch_dir, "Microsoft.VC*.CRT"))?
    .filter_map(Result::ok)
    .find(|path| path.is_dir())
  else {
    return Err(crate::Error::GenericError(format!(
      "failed to find Visual C++ runtime directory for `{arch}` in `{}`",
      arch_dir.display()
    )));
  };

  Ok(runtime_dir)
}

fn latest_vc_redist_version_dir(redist_dir: &Path) -> crate::Result<Option<PathBuf>> {
  let dir = fs::read_dir(redist_dir)?
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.is_dir())
    .filter_map(|path| {
      let version = path
        .file_name()?
        .to_str()?
        .parse::<semver::Version>()
        .ok()?;
      Some((version, path))
    })
    .max_by(|(a, _), (b, _)| a.cmp(b))
    .map(|(_, path)| path);
  Ok(dir)
}

/// Builds a glob pattern from a literal base path and an unescaped glob suffix.
///
/// The base path is escaped so Visual Studio paths containing glob metacharacters are treated as
/// literal directories, while `pattern` remains active glob syntax.
fn glob_path(path: &Path, pattern: &str) -> String {
  PathBuf::from(glob::Pattern::escape(&path.to_string_lossy()))
    .join(pattern)
    .to_string_lossy()
    .into_owned()
}

/// Returns the bundled `vswhere.exe` path.
///
/// The executable is written to a temporary file so callers do not depend on a system-installed
/// `vswhere.exe`.
#[cfg(windows)]
pub fn vswhere_path() -> Option<PathBuf> {
  let mut vswhere = std::env::temp_dir();
  vswhere.push("vswhere.exe");

  if !vswhere.exists() {
    let mut file = std::fs::File::create(&vswhere).ok()?;
    file.write_all(VSWHERE).ok()?;
  }

  Some(vswhere)
}

#[cfg(target_os = "windows")]
pub fn processor_architecture<'a>() -> Option<&'a str> {
  use windows_sys::Win32::System::SystemInformation::{
    GetNativeSystemInfo, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM,
    PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
  };

  let mut system_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
  unsafe { GetNativeSystemInfo(&mut system_info) };
  match unsafe { system_info.Anonymous.Anonymous.wProcessorArchitecture } {
    PROCESSOR_ARCHITECTURE_INTEL => Some("x86"),
    PROCESSOR_ARCHITECTURE_AMD64 => Some("x64"),
    PROCESSOR_ARCHITECTURE_ARM => Some("arm"),
    PROCESSOR_ARCHITECTURE_ARM64 => Some("arm64"),
    _ => None,
  }
}
