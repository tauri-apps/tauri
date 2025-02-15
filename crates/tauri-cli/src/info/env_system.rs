// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{SectionItem, Status};
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
fn build_tools_version() -> crate::Result<Vec<String>> {
  let mut vswhere = std::env::temp_dir();
  vswhere.push("vswhere.exe");

  if !vswhere.exists() {
    if let Ok(mut file) = std::fs::File::create(&vswhere) {
      use std::io::Write;
      let _ = file.write_all(VSWHERE);
    }
  }

  // Check if there are Visual Studio installations that have the "MSVC - C++ Buildtools" and "Windows SDK" components.
  // Both the Windows 10 and Windows 11 SDKs work so we need to query it twice.
  let output_sdk10 = Command::new(&vswhere)
    .args([
      "-prerelease",
      "-products",
      "*",
      "-requires",
      "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
      "-requires",
      "Microsoft.VisualStudio.Component.Windows10SDK.*",
      "-format",
      "json",
      "-utf8",
    ])
    .output()?;

  let output_sdk11 = Command::new(vswhere)
    .args([
      "-prerelease",
      "-products",
      "*",
      "-requires",
      "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
      "-requires",
      "Microsoft.VisualStudio.Component.Windows11SDK.*",
      "-format",
      "json",
      "-utf8",
    ])
    .output()?;

  let mut instances: Vec<VsInstanceInfo> = Vec::new();

  if output_sdk10.status.success() {
    let stdout = String::from_utf8_lossy(&output_sdk10.stdout);
    let found: Vec<VsInstanceInfo> = serde_json::from_str(&stdout)?;
    instances.extend(found);
  }

  if output_sdk11.status.success() {
    let stdout = String::from_utf8_lossy(&output_sdk11.stdout);
    let found: Vec<VsInstanceInfo> = serde_json::from_str(&stdout)?;
    instances.extend(found);
  }

  let mut instances: Vec<String> = instances
    .iter()
    .map(|i| i.display_name.clone())
    .collect::<Vec<String>>();

  instances.sort_unstable();
  instances.dedup();

  Ok(instances)
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
  pkg_conf_version("webkit2gtk-4.1")
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
fn de_and_session() -> String {
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
  ))]
  return {
    let de = std::env::var("XDG_SESSION_DESKTOP");
    let session = std::env::var("XDG_SESSION_TYPE");
    format!(
      " ({} on {})",
      de.as_deref().unwrap_or("Unknown DE"),
      session.as_deref().unwrap_or("Unknown Session")
    )
  };

  #[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
  )))]
  String::new()
}

pub fn items() -> Vec<SectionItem> {
  vec![
    SectionItem::new().action(|| {
      let os_info = os_info::get();
      format!(
        "OS: {} {} {} ({:?}){}",
        os_info.os_type(),
        os_info.version(),
        os_info.architecture().unwrap_or("Unknown Architecture"),
        os_info.bitness(),
        de_and_session(),
      ).into()
    }),
    #[cfg(windows)]
    SectionItem::new().action(|| {
      let error = format!(
          "Webview2: {}\nVisit {}",
          "not installed!".red(),
          "https://developer.microsoft.com/en-us/microsoft-edge/webview2/".cyan()
        );
      webview2_version()
        .map(|v| {
          v.map(|v| (format!("WebView2: {}", v), Status::Success))
            .unwrap_or_else(|| (error.clone(), Status::Error))
        })
        .unwrap_or_else(|_| (error, Status::Error)).into()
    }),
    #[cfg(windows)]
    SectionItem::new().action(|| {
      let build_tools = build_tools_version().unwrap_or_default();
      if build_tools.is_empty() {
        (
            format!(
              "Couldn't detect any Visual Studio or VS Build Tools instance with MSVC and SDK components. Download from {}",
              "https://aka.ms/vs/17/release/vs_BuildTools.exe".cyan()
            ),
            Status::Error,
          ).into()
      } else {
        (
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
        ).into()
      }
    }),
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd"
    ))]
    SectionItem::new().action(|| {
          webkit2gtk_ver()
            .map(|v| (format!("webkit2gtk-4.1: {v}"), Status::Success))
            .unwrap_or_else(|| {
              (
                format!(
                  "webkit2gtk-4.1: {}\nVisit {} to learn more about tauri prerequisites",
                  "not installed".red(),
                  "https://v2.tauri.app/start/prerequisites/".cyan()
                ),
                Status::Error,
              )
            }).into()
      },
    ),
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd"
    ))]
    SectionItem::new().action(|| {
          rsvg2_ver()
            .map(|v| (format!("rsvg2: {v}"), Status::Success))
            .unwrap_or_else(|| {
              (
                format!(
                  "rsvg2: {}\nVisit {} to learn more about tauri prerequisites",
                  "not installed".red(),
                  "https://v2.tauri.app/start/prerequisites/".cyan()
                ),
                Status::Error,
              )
            }).into()
      },
    ),
    #[cfg(target_os = "macos")]
    SectionItem::new().action(|| {
        if is_xcode_command_line_tools_installed() {
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
        }.into()
      },
    ),
  ]
}
