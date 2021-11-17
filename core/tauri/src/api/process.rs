// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to child processes management.

use crate::Env;

use std::path::PathBuf;

#[cfg(feature = "command")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "command")))]
mod command;
#[cfg(feature = "command")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "command")))]
pub use command::*;

/// Finds the current running binary's path.
///
/// # Platform-specific behavior
///
/// On the `Linux` platform, this function will also **attempt** to detect if
/// it's currently running from a valid [AppImage] and use that path instead.
///
/// # Security
///
/// If the above Platform-specific behavior does not take place, this function
/// uses [`std::env::current_exe`]. Notably, it also has a security section
/// that goes over a theoretical attack using hard links. Let's cover some
/// specific topics that relate to different ways an attacker might try to
/// trick this function into returning the wrong binary path.
///
/// ## Symlinks ("Soft Links")
///
/// [`std::path::Path::canonicalize`] is used to resolve symbolic links to the
/// original path, including nested symbolic links (`link2 -> link1 -> bin`).
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
/// or have read/write access to.** [See the patch that enabled this.](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=800179c9b8a1e796e441674776d11cd4c05d61d7)
///
/// [AppImage]: https://appimage.org/
/// [Hard Link]: https://en.wikipedia.org/wiki/Hard_link
#[allow(unused_variables)]
pub fn current_binary(env: &Env) -> Option<PathBuf> {
  // if we are running from an AppImage, we ONLY want the set AppImage path
  #[cfg(target_os = "linux")]
  if let Some(app_image_path) = &env.appimage {
    return Some(PathBuf::from(app_image_path));
  }

  tauri_utils::platform::current_exe().ok()
}

/// Restarts the process.
///
/// See [`current_binary`] for the possible security implications.
pub fn restart(env: &Env) {
  use std::process::{exit, Command};

  if let Some(path) = current_binary(env) {
    Command::new(path)
      .spawn()
      .expect("application failed to start");
  }

  exit(0);
}
