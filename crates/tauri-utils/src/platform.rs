// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Platform helper functions.

use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Env, PackageInfo};

mod starting_binary;

/// URI prefix of a Tauri asset.
///
/// This is referenced in the Tauri Android library,
/// which resolves these assets to a file descriptor.
#[cfg(target_os = "android")]
pub const ANDROID_ASSET_PROTOCOL_URI_PREFIX: &str = "asset://localhost/";

/// Platform target.
#[derive(PartialEq, Eq, Copy, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Target {
  /// MacOS.
  #[serde(rename = "macOS")]
  MacOS,
  /// Windows.
  Windows,
  /// Linux.
  Linux,
  /// Android.
  Android,
  /// iOS.
  #[serde(rename = "iOS")]
  Ios,
}

impl Display for Target {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::MacOS => "macOS",
        Self::Windows => "windows",
        Self::Linux => "linux",
        Self::Android => "android",
        Self::Ios => "iOS",
      }
    )
  }
}

impl Target {
  /// Parses the target from the given target triple.
  pub fn from_triple(target: &str) -> Self {
    if target.contains("darwin") {
      Self::MacOS
    } else if target.contains("windows") {
      Self::Windows
    } else if target.contains("android") {
      Self::Android
    } else if target.contains("ios") {
      Self::Ios
    } else {
      Self::Linux
    }
  }

  /// Gets the current build target.
  pub fn current() -> Self {
    if cfg!(target_os = "macos") {
      Self::MacOS
    } else if cfg!(target_os = "windows") {
      Self::Windows
    } else if cfg!(target_os = "ios") {
      Self::Ios
    } else if cfg!(target_os = "android") {
      Self::Android
    } else {
      Self::Linux
    }
  }

  /// Whether the target is mobile or not.
  pub fn is_mobile(&self) -> bool {
    matches!(self, Target::Android | Target::Ios)
  }

  /// Whether the target is desktop or not.
  pub fn is_desktop(&self) -> bool {
    !self.is_mobile()
  }
}

/// Retrieves the currently running binary's path, taking into account security considerations.
///
/// The path is cached as soon as possible (before even `main` runs) and that value is returned
/// repeatedly instead of fetching the path every time. It is possible for the path to not be found,
/// or explicitly disabled (see following macOS specific behavior).
///
/// # Platform-specific behavior
///
/// On `macOS`, this function will return an error if the original path contained any symlinks
/// due to less protection on macOS regarding symlinks. This behavior can be disabled by setting the
/// `process-relaunch-dangerous-allow-symlink-macos` feature, although it is *highly discouraged*.
///
/// # Security
///
/// If the above platform-specific behavior does **not** take place, this function uses the
/// following resolution.
///
/// We canonicalize the path we received from [`std::env::current_exe`] to resolve any soft links.
/// This avoids the usual issue of needing the file to exist at the passed path because a valid
/// current executable result for our purpose should always exist. Notably,
/// [`std::env::current_exe`] also has a security section that goes over a theoretical attack using
/// hard links. Let's cover some specific topics that relate to different ways an attacker might
/// try to trick this function into returning the wrong binary path.
///
/// ## Symlinks ("Soft Links")
///
/// [`std::path::Path::canonicalize`] is used to resolve symbolic links to the original path,
/// including nested symbolic links (`link2 -> link1 -> bin`). On macOS, any results that include
/// a symlink are rejected by default due to lesser symlink protections. This can be disabled,
/// **although discouraged**, with the `process-relaunch-dangerous-allow-symlink-macos` feature.
///
/// ## Hard Links
///
/// A [Hard Link] is a named entry that points to a file in the file system.
/// On most systems, this is what you would think of as a "file". The term is
/// used on filesystems that allow multiple entries to point to the same file.
/// The linked [Hard Link] Wikipedia page provides a decent overview.
///
/// In short, unless the attacker was able to create the link with elevated
/// permissions, it should generally not be possible for them to hard link
/// to a file they do not have permissions to - with exception to possible
/// operating system exploits.
///
/// There are also some platform-specific information about this below.
///
/// ### Windows
///
/// Windows requires a permission to be set for the user to create a symlink
/// or a hard link, regardless of ownership status of the target. Elevated
/// permissions users have the ability to create them.
///
/// ### macOS
///
/// macOS allows for the creation of symlinks and hard links to any file.
/// Accessing through those links will fail if the user who owns the links
/// does not have the proper permissions on the original file.
///
/// ### Linux
///
/// Linux allows for the creation of symlinks to any file. Accessing the
/// symlink will fail if the user who owns the symlink does not have the
/// proper permissions on the original file.
///
/// Linux additionally provides a kernel hardening feature since version
/// 3.6 (30 September 2012). Most distributions since then have enabled
/// the protection (setting `fs.protected_hardlinks = 1`) by default, which
/// means that a vast majority of desktop Linux users should have it enabled.
/// **The feature prevents the creation of hardlinks that the user does not own
/// or have read/write access to.** [See the patch that enabled this].
///
/// [Hard Link]: https://en.wikipedia.org/wiki/Hard_link
/// [See the patch that enabled this]: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=800179c9b8a1e796e441674776d11cd4c05d61d7
pub fn current_exe() -> std::io::Result<PathBuf> {
  self::starting_binary::STARTING_BINARY.cloned()
}

/// Try to determine the current target triple.
///
/// Returns a target triple (e.g. `x86_64-unknown-linux-gnu` or `i686-pc-windows-msvc`) or an
/// `Error::Config` if the current config cannot be determined or is not some combination of the
/// following values:
/// `linux, mac, windows` -- `i686, x86, armv7` -- `gnu, musl, msvc`
///
/// * Errors:
///     * Unexpected system config
pub fn target_triple() -> crate::Result<String> {
  let arch = if cfg!(target_arch = "x86") {
    "i686"
  } else if cfg!(target_arch = "x86_64") {
    "x86_64"
  } else if cfg!(target_arch = "arm") {
    "armv7"
  } else if cfg!(target_arch = "aarch64") {
    "aarch64"
  } else {
    return Err(crate::Error::Architecture);
  };

  let os = if cfg!(target_os = "linux") {
    "unknown-linux"
  } else if cfg!(target_os = "macos") {
    "apple-darwin"
  } else if cfg!(target_os = "windows") {
    "pc-windows"
  } else if cfg!(target_os = "freebsd") {
    "unknown-freebsd"
  } else {
    return Err(crate::Error::Os);
  };

  let os = if cfg!(target_os = "macos") || cfg!(target_os = "freebsd") {
    String::from(os)
  } else {
    let env = if cfg!(target_env = "gnu") {
      "gnu"
    } else if cfg!(target_env = "musl") {
      "musl"
    } else if cfg!(target_env = "msvc") {
      "msvc"
    } else {
      return Err(crate::Error::Environment);
    };

    format!("{os}-{env}")
  };

  Ok(format!("{arch}-{os}"))
}

#[cfg(all(not(test), not(target_os = "android")))]
fn is_cargo_output_directory(path: &std::path::Path) -> bool {
  path.join(".cargo-lock").exists()
}

#[cfg(test)]
const CARGO_OUTPUT_DIRECTORIES: &[&str] = &["debug", "release", "custom-profile"];

#[cfg(test)]
fn is_cargo_output_directory(path: &std::path::Path) -> bool {
  let last_component = path
    .components()
    .last()
    .unwrap()
    .as_os_str()
    .to_str()
    .unwrap();
  CARGO_OUTPUT_DIRECTORIES
    .iter()
    .any(|dirname| &last_component == dirname)
}

/// Computes the resource directory of the current environment.
///
/// On Windows, it's the path to the executable.
///
/// On Linux, when running in an AppImage the `APPDIR` variable will be set to
/// the mounted location of the app, and the resource dir will be
/// `${APPDIR}/usr/lib/${exe_name}`. If not running in an AppImage, the path is
/// `/usr/lib/${exe_name}`.  When running the app from
/// `src-tauri/target/(debug|release)/`, the path is
/// `${exe_dir}/../lib/${exe_name}`.
///
/// On MacOS, it's `${exe_dir}../Resources` (inside .app).
///
/// On iOS, it's `${exe_dir}/assets`.
///
/// Android uses a special URI prefix that is resolved by the Tauri file system plugin `asset://localhost/`
pub fn resource_dir(package_info: &PackageInfo, env: &Env) -> crate::Result<PathBuf> {
  #[cfg(target_os = "android")]
  return resource_dir_android(package_info, env);
  #[cfg(not(target_os = "android"))]
  {
    let exe = current_exe()?;
    resource_dir_from(exe, package_info, env)
  }
}

#[cfg(target_os = "android")]
fn resource_dir_android(_package_info: &PackageInfo, _env: &Env) -> crate::Result<PathBuf> {
  Ok(PathBuf::from(ANDROID_ASSET_PROTOCOL_URI_PREFIX))
}

#[cfg(not(target_os = "android"))]
#[allow(unused_variables)]
fn resource_dir_from<P: AsRef<std::path::Path>>(
  exe: P,
  package_info: &PackageInfo,
  env: &Env,
) -> crate::Result<PathBuf> {
  let exe_dir = exe.as_ref().parent().expect("failed to get exe directory");
  let curr_dir = exe_dir.display().to_string();

  let parts: Vec<&str> = curr_dir.split(std::path::MAIN_SEPARATOR).collect();
  let len = parts.len();

  // Check if running from the Cargo output directory, which means it's an executable in a development machine
  // We check if the binary is inside a `target` folder which can be either `target/$profile` or `target/$triple/$profile`
  // and see if there's a .cargo-lock file along the executable
  // This ensures the check is safer so it doesn't affect apps in production
  // Windows also includes the resources in the executable folder so we check that too
  if cfg!(target_os = "windows")
    || ((len >= 2 && parts[len - 2] == "target") || (len >= 3 && parts[len - 3] == "target"))
      && is_cargo_output_directory(exe_dir)
  {
    return Ok(exe_dir.to_path_buf());
  }

  #[allow(unused_mut, unused_assignments)]
  let mut res = Err(crate::Error::UnsupportedPlatform);

  #[cfg(target_os = "linux")]
  {
    // (canonicalize checks for existence, so there's no need for an extra check)
    res = if let Ok(bundle_dir) = exe_dir
      .join(format!("../lib/{}", package_info.name))
      .canonicalize()
    {
      Ok(bundle_dir)
    } else if let Some(appdir) = &env.appdir {
      let appdir: &std::path::Path = appdir.as_ref();
      Ok(PathBuf::from(format!(
        "{}/usr/lib/{}",
        appdir.display(),
        package_info.name
      )))
    } else {
      // running bundle
      Ok(PathBuf::from(format!("/usr/lib/{}", package_info.name)))
    };
  }

  #[cfg(target_os = "macos")]
  {
    res = exe_dir
      .join("../Resources")
      .canonicalize()
      .map_err(Into::into);
  }

  #[cfg(target_os = "ios")]
  {
    res = exe_dir.join("assets").canonicalize().map_err(Into::into);
  }

  res
}

#[cfg(feature = "build")]
mod build {
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  use super::*;

  impl ToTokens for Target {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::platform::Target };

      tokens.append_all(match self {
        Self::MacOS => quote! { #prefix::MacOS },
        Self::Linux => quote! { #prefix::Linux },
        Self::Windows => quote! { #prefix::Windows },
        Self::Android => quote! { #prefix::Android },
        Self::Ios => quote! { #prefix::Ios },
      });
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::{Env, PackageInfo};

  #[test]
  fn resolve_resource_dir() {
    let package_info = PackageInfo {
      name: "MyApp".into(),
      version: "1.0.0".parse().unwrap(),
      authors: "",
      description: "",
      crate_name: "my-app",
    };
    let env = Env::default();

    let path = PathBuf::from("/path/to/target/aarch64-apple-darwin/debug/app");
    let resource_dir = super::resource_dir_from(&path, &package_info, &env).unwrap();
    assert_eq!(resource_dir, path.parent().unwrap());

    let path = PathBuf::from("/path/to/target/custom-profile/app");
    let resource_dir = super::resource_dir_from(&path, &package_info, &env).unwrap();
    assert_eq!(resource_dir, path.parent().unwrap());

    let path = PathBuf::from("/path/to/target/release/app");
    let resource_dir = super::resource_dir_from(&path, &package_info, &env).unwrap();
    assert_eq!(resource_dir, path.parent().unwrap());

    let path = PathBuf::from("/path/to/target/unknown-profile/app");
    let resource_dir = super::resource_dir_from(&path, &package_info, &env);
    #[cfg(target_os = "macos")]
    assert!(resource_dir.is_err());
    #[cfg(target_os = "linux")]
    assert_eq!(resource_dir.unwrap(), PathBuf::from("/usr/lib/my-app"));
    #[cfg(windows)]
    assert_eq!(resource_dir.unwrap(), path.parent().unwrap());
  }
}
