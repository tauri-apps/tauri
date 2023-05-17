// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::SectionItem;
use super::Status;
use colored::Colorize;
#[cfg(windows)]
use serde::Deserialize;
use std::process::Command;

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VsInstanceInfo {
  display_name: String,
}

#[cfg(windows)]
const VSWHERE: &[u8] = include_bytes!("../../scripts/vswhere.exe");

#[cfg(windows)]
fn has_windows_sdk_libs() -> bool {
  #[cfg(target_arch = "x86")]
  let target = "i686";
  #[cfg(not(target_arch = "x86"))]
  let arch = std::env::consts::ARCH;

  let tool = cc::windows_registry::find_tool(&format!("{}-pc-windows-msvc", arch), "cl.exe");
  // Check if there's any Visual Studio installation present
  if tool.is_none() || cc::windows_registry::find_vs_version().is_err() {
    return false;
  }

  // We don't know the exact name of the Windows SDK Visual Studio Component so we search for files the SDK includes
  for envs in tool.unwrap().env() {
    if envs.0.to_ascii_lowercase() == "lib" {
      for mut path in std::env::split_paths(&envs.1) {
        path.push("kernel32.lib");
        if path.exists() {
          return true;
        }
      }
    }
  }

  false
}

#[cfg(windows)]
fn build_tools_version() -> crate::Result<Option<Vec<String>>> {
  let mut vswhere = std::env::temp_dir();
  vswhere.push("vswhere.exe");

  if !vswhere.exists() {
    if let Ok(mut file) = std::fs::File::create(&vswhere) {
      use std::io::Write;
      let _ = file.write_all(VSWHERE);
    }
  }

  // Check if there are Visual Studio installations that have the "MSVC - C++ Buildtools" component
  let output = Command::new(vswhere)
    .args([
      "-prerelease",
      "-products",
      "*",
      "-requires",
      "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
      "-format",
      "json",
    ])
    .output()?;

  Ok(if output.status.success() && has_windows_sdk_libs() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let instances: Vec<VsInstanceInfo> = serde_json::from_str(&stdout)?;

    Some(
      instances
        .iter()
        .map(|i| i.display_name.clone())
        .collect::<Vec<String>>(),
    )
  } else {
    None
  })
}

#[cfg(windows)]
fn webview2_version() -> crate::Result<Option<String>> {
  let powershell_path = std::env::var("SYSTEMROOT").map_or_else(
    |_| "powershell.exe".to_string(),
    |p| format!("{p}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
  );
  // check 64bit machine-wide installation
  let output = Command::new(&powershell_path)
      .args(["-NoProfile", "-Command"])
      .arg("Get-ItemProperty -Path 'HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' | ForEach-Object {$_.pv}")
      .output()?;
  if output.status.success() {
    return Ok(Some(
      String::from_utf8_lossy(&output.stdout).replace('\n', ""),
    ));
  }
  // check 32bit machine-wide installation
  let output = Command::new(&powershell_path)
        .args(["-NoProfile", "-Command"])
        .arg("Get-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' | ForEach-Object {$_.pv}")
        .output()?;
  if output.status.success() {
    return Ok(Some(
      String::from_utf8_lossy(&output.stdout).replace('\n', ""),
    ));
  }
  // check user-wide installation
  let output = Command::new(&powershell_path)
      .args(["-NoProfile", "-Command"])
      .arg("Get-ItemProperty -Path 'HKCU:\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' | ForEach-Object {$_.pv}")
      .output()?;
  if output.status.success() {
    return Ok(Some(
      String::from_utf8_lossy(&output.stdout).replace('\n', ""),
    ));
  }

  Ok(None)
}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd"
))]
fn pkg_conf_version(package: &str) -> Option<String> {
  Command::new("pkg-config")
    .args([package, "--print-provides"])
    .output()
    .map(|o| {
      String::from_utf8_lossy(&o.stdout)
        .split('=')
        .nth(1)
        .map(|s| s.trim().to_string())
    })
    .unwrap_or(None)
}
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd"
))]
fn webkit2gtk_ver() -> Option<String> {
  pkg_conf_version("webkit2gtk-4.0")
}
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd"
))]
fn rsvg2_ver() -> Option<String> {
  pkg_conf_version("librsvg-2.0")
}

#[cfg(target_os = "macos")]
fn is_xcode_command_line_tools_installed() -> bool {
  Command::new("xcode-select")
    .arg("-p")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

pub fn items() -> Vec<SectionItem> {
  vec![
    SectionItem::new(
      || {
        let os_info = os_info::get();
        Some((
          format!(
            "OS: {} {} {:?}",
            os_info.os_type(),
            os_info.version(),
            os_info.bitness()
          ),
          Status::Neutral,
        ))
      },
      || None,
      false,
    ),
    #[cfg(windows)]
    SectionItem::new(
      || {
        let error = || {
          format!(
            "Webview2: {}\nVisit {}",
            "not installed!".red(),
            "https://developer.microsoft.com/en-us/microsoft-edge/webview2/".cyan()
          )
        };
        Some(
          webview2_version()
            .map(|v| {
              v.map(|v| (format!("WebView2: {}", v), Status::Success))
                .unwrap_or_else(|| (error(), Status::Error))
            })
            .unwrap_or_else(|_| (error(), Status::Error)),
        )
      },
      || None,
      false,
    ),
    #[cfg(windows)]
    SectionItem::new(
      || {
        let build_tools = build_tools_version()
          .unwrap_or_default()
          .unwrap_or_default();
        if build_tools.is_empty() {
          Some((
            format!(
              "Couldn't detect Visual Studio or Visual Studio Build Tools. Download from {}",
              "https://aka.ms/vs/17/release/vs_BuildTools.exe".cyan()
            ),
            Status::Error,
          ))
        } else {
          Some((
            format!(
              "MSVC: {}{}",
              if build_tools.len() > 1 {
                format!("\n  {} ", "-".cyan())
              } else {
                "".into()
              },
              build_tools.join(format!("\n  {} ", "-".cyan()).as_str()),
            ),
            Status::Success,
          ))
        }
      },
      || None,
      false,
    ),
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd"
    ))]
    SectionItem::new(
      || {
        Some(
          webkit2gtk_ver()
            .map(|v| (format!("webkit2gtk-4.0: {v}"), Status::Success))
            .unwrap_or_else(|| {
              (
                format!(
                  "webkit2gtk-4.0: {}\nVisit {} to learn more about tauri prerequisites",
                  "not installed".red(),
                  "https://tauri.app/v1/guides/getting-started/prerequisites".cyan()
                ),
                Status::Error,
              )
            }),
        )
      },
      || None,
      false,
    ),
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd"
    ))]
    SectionItem::new(
      || {
        Some(
          rsvg2_ver()
            .map(|v| (format!("rsvg2: {v}"), Status::Success))
            .unwrap_or_else(|| {
              (
                format!(
                  "rsvg2: {}\nVisit {} to learn more about tauri prerequisites",
                  "not installed".red(),
                  "https://tauri.app/v1/guides/getting-started/prerequisites".cyan()
                ),
                Status::Error,
              )
            }),
        )
      },
      || None,
      false,
    ),
    #[cfg(target_os = "macos")]
    SectionItem::new(
      || {
        Some(if is_xcode_command_line_tools_installed() {
          (
            "Xcode Command Line Tools: installed".into(),
            Status::Success,
          )
        } else {
          (
            format!(
              "Xcode Command Line Tools: {}\n Run `{}`",
              "not installed!".red(),
              "xcode-select --install".cyan()
            ),
            Status::Error,
          )
        })
      },
      || None,
      false,
    ),
  ]
}
