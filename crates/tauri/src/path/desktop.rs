// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Error, Result};
use crate::{AppHandle, Manager, Runtime};
use std::path::{Path, PathBuf};

/// The path resolver is a helper class for general and application-specific path APIs.
pub struct PathResolver<R: Runtime>(pub(crate) AppHandle<R>);

impl<R: Runtime> Clone for PathResolver<R> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<R: Runtime> PathResolver<R> {
  /// Returns the final component of the `Path`, if there is one.
  ///
  /// If the path is a normal file, this is the file name. If it's the path of a directory, this
  /// is the directory name.
  ///
  /// Returns [`None`] if the path terminates in `..`.
  ///
  /// On Android this also supports checking the file name of content URIs, such as the values returned by the dialog plugin.
  ///
  /// If you are dealing with plain file system paths or not worried about Android content URIs, prefer [`Path::file_name`].
  pub fn file_name(&self, path: &str) -> Option<String> {
    Path::new(path)
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
  }

  /// Returns the path to the user's audio directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_MUSIC_DIR`.
  /// - **macOS:** Resolves to `$HOME/Music`.
  /// - **Windows:** Resolves to `{FOLDERID_Music}`.
  pub fn audio_dir(&self) -> Result<PathBuf> {
    dirs::audio_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's cache directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_CACHE_HOME` or `$HOME/.cache`.
  /// - **macOS:** Resolves to `$HOME/Library/Caches`.
  /// - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
  pub fn cache_dir(&self) -> Result<PathBuf> {
    dirs::cache_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's config directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`.
  /// - **macOS:** Resolves to `$HOME/Library/Application Support`.
  /// - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
  pub fn config_dir(&self) -> Result<PathBuf> {
    dirs::config_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's data directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
  /// - **macOS:** Resolves to `$HOME/Library/Application Support`.
  /// - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
  pub fn data_dir(&self) -> Result<PathBuf> {
    dirs::data_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's local data directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
  /// - **macOS:** Resolves to `$HOME/Library/Application Support`.
  /// - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
  pub fn local_data_dir(&self) -> Result<PathBuf> {
    dirs::data_local_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's desktop directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DESKTOP_DIR`.
  /// - **macOS:** Resolves to `$HOME/Desktop`.
  /// - **Windows:** Resolves to `{FOLDERID_Desktop}`.
  pub fn desktop_dir(&self) -> Result<PathBuf> {
    dirs::desktop_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's document directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOCUMENTS_DIR`.
  /// - **macOS:** Resolves to `$HOME/Documents`.
  /// - **Windows:** Resolves to `{FOLDERID_Documents}`.
  pub fn document_dir(&self) -> Result<PathBuf> {
    dirs::document_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's download directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOWNLOAD_DIR`.
  /// - **macOS:** Resolves to `$HOME/Downloads`.
  /// - **Windows:** Resolves to `{FOLDERID_Downloads}`.
  pub fn download_dir(&self) -> Result<PathBuf> {
    dirs::download_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's executable directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_BIN_HOME/../bin` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`.
  /// - **macOS:** Not supported.
  /// - **Windows:** Not supported.
  pub fn executable_dir(&self) -> Result<PathBuf> {
    dirs::executable_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's font directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_DATA_HOME/fonts` or `$HOME/.local/share/fonts`.
  /// - **macOS:** Resolves to `$HOME/Library/Fonts`.
  /// - **Windows:** Not supported.
  pub fn font_dir(&self) -> Result<PathBuf> {
    dirs::font_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's home directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$HOME`.
  /// - **macOS:** Resolves to `$HOME`.
  /// - **Windows:** Resolves to `{FOLDERID_Profile}`.
  /// - **iOS**: Cannot be written to directly, use one of the app paths instead.
  pub fn home_dir(&self) -> Result<PathBuf> {
    dirs::home_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's picture directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PICTURES_DIR`.
  /// - **macOS:** Resolves to `$HOME/Pictures`.
  /// - **Windows:** Resolves to `{FOLDERID_Pictures}`.
  pub fn picture_dir(&self) -> Result<PathBuf> {
    dirs::picture_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's public directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PUBLICSHARE_DIR`.
  /// - **macOS:** Resolves to `$HOME/Public`.
  /// - **Windows:** Resolves to `{FOLDERID_Public}`.
  pub fn public_dir(&self) -> Result<PathBuf> {
    dirs::public_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's runtime directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_RUNTIME_DIR`.
  /// - **macOS:** Not supported.
  /// - **Windows:** Not supported.
  pub fn runtime_dir(&self) -> Result<PathBuf> {
    dirs::runtime_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's template directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_TEMPLATES_DIR`.
  /// - **macOS:** Not supported.
  /// - **Windows:** Resolves to `{FOLDERID_Templates}`.
  pub fn template_dir(&self) -> Result<PathBuf> {
    dirs::template_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's video dir
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_VIDEOS_DIR`.
  /// - **macOS:** Resolves to `$HOME/Movies`.
  /// - **Windows:** Resolves to `{FOLDERID_Videos}`.
  pub fn video_dir(&self) -> Result<PathBuf> {
    dirs::video_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the resource directory of this app.
  ///
  /// ## Platform-specific
  ///
  /// Although we provide the exact path where this function resolves to,
  /// this is not a contract and things might change in the future
  ///
  /// - **Windows:** Resolves to the directory that contains the main executable.
  /// - **Linux:** When running in an AppImage, the `APPDIR` variable will be set to
  ///   the mounted location of the app, and the resource dir will be `${APPDIR}/usr/lib/${exe_name}`.
  ///   If not running in an AppImage, the path is `/usr/lib/${exe_name}`.
  ///   When running the app from `src-tauri/target/(debug|release)/`, the path is `${exe_dir}/../lib/${exe_name}`.
  /// - **macOS:** Resolves to `${exe_dir}/../Resources` (inside .app).
  /// - **iOS:** Resolves to `${exe_dir}/assets`.
  /// - **Android:** Currently the resources are stored in the APK as assets so it's not a normal file system path,
  ///   we return a special URI prefix `asset://localhost/` here that can be used with the [file system plugin](https://tauri.app/plugin/file-system/),
  ///   with that, you can read the files through [`FsExt::fs`](https://docs.rs/tauri-plugin-fs/latest/tauri_plugin_fs/trait.FsExt.html#tymethod.fs)
  ///   like this: `app.fs().read_to_string(app.path().resource_dir().unwrap().join("resource"));`
  ///
  /// ## Development
  ///
  /// On desktop, when running with `tauri dev` or `cargo run`, resources are read from their
  /// source location instead of being copied next to the executable by the build script:
  ///
  /// - If all configured resources are plain relative paths, their bundle layout mirrors the
  ///   source layout, so this resolves to the directory containing `tauri.conf.json` and
  ///   resource files are read directly from the sources.
  /// - If resources are remapped (map notation, or paths outside the app directory), they are
  ///   mirrored to the directory containing the executable on the first call, so joined paths
  ///   keep working, and refreshed on each run.
  pub fn resource_dir(&self) -> Result<PathBuf> {
    #[cfg(all(dev, desktop))]
    if let Some(dir) = self.dev_resource_dir()? {
      return Ok(dir);
    }

    crate::utils::platform::resource_dir(self.0.package_info(), &self.0.env())
      .map_err(|_| Error::UnknownPath)
  }

  #[cfg(all(dev, desktop))]
  fn dev_resource_dir(&self) -> Result<Option<PathBuf>> {
    use tauri_utils::config::BundleResources;

    let manager = &self.0.manager;
    let Some(app_dir) = manager.config_parent() else {
      return Ok(None);
    };
    let Some(resources) = manager.config().bundle.resources.as_ref() else {
      return Ok(None);
    };

    match resources {
      BundleResources::List(patterns) if patterns.is_empty() => Ok(None),
      // the bundle layout mirrors the source layout, so the app directory
      // acts as the resource directory and files are read directly from the sources
      resources if resources_mirror_source_layout(resources) => Ok(Some(app_dir.clone())),
      // remapped resources: mirror the bundle layout next to the executable
      resources => {
        let mut dev_resources_dir = manager.dev_resources_dir.lock().unwrap();
        if let Some(dir) = dev_resources_dir.as_ref() {
          return Ok(Some(dir.clone()));
        }

        let exe_dir = crate::utils::platform::current_exe()?
          .parent()
          .ok_or(Error::UnknownPath)?
          .to_path_buf();
        mirror_resources(resources, app_dir, &exe_dir)?;
        dev_resources_dir.replace(exe_dir.clone());
        Ok(Some(exe_dir))
      }
    }
  }

  /// Returns the path to the suggested directory for your app's config files.
  ///
  /// Resolves to [`config_dir`](Self::config_dir)`/${bundle_identifier}`.
  pub fn app_config_dir(&self) -> Result<PathBuf> {
    dirs::config_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's data files.
  ///
  /// Resolves to [`data_dir`](Self::data_dir)`/${bundle_identifier}`.
  pub fn app_data_dir(&self) -> Result<PathBuf> {
    dirs::data_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's local data files.
  ///
  /// Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}`.
  pub fn app_local_data_dir(&self) -> Result<PathBuf> {
    dirs::data_local_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's cache files.
  ///
  /// Resolves to [`cache_dir`](Self::cache_dir)`/${bundle_identifier}`.
  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    dirs::cache_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's log files.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}/logs`.
  /// - **macOS:** Resolves to [`home_dir`](Self::home_dir)`/Library/Logs/${bundle_identifier}`
  /// - **Windows:** Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}/logs`.
  pub fn app_log_dir(&self) -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let path = dirs::home_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join("Library/Logs").join(&self.0.config().identifier));

    #[cfg(not(target_os = "macos"))]
    let path = dirs::data_local_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier).join("logs"));

    path
  }

  /// A temporary directory. Resolves to [`std::env::temp_dir`].
  pub fn temp_dir(&self) -> Result<PathBuf> {
    Ok(std::env::temp_dir())
  }
}

/// Whether the bundle target path of every configured resource matches its source path,
/// meaning the directory the resource patterns are relative to can be used
/// as the resource directory directly.
///
/// This is only the case for list notation with plain relative patterns:
/// map notation remaps targets, and patterns reaching outside the base directory
/// get their target rewritten (`..` -> `_up_`).
#[cfg(all(dev, desktop))]
fn resources_mirror_source_layout(resources: &tauri_utils::config::BundleResources) -> bool {
  match resources {
    tauri_utils::config::BundleResources::List(patterns) => patterns.iter().all(|pattern| {
      let path = Path::new(pattern);
      path.is_relative()
        && !path
          .components()
          .any(|c| matches!(c, std::path::Component::ParentDir))
    }),
    tauri_utils::config::BundleResources::Map(_) => false,
  }
}

/// Copies the configured resources, laid out as in the bundle, into `out_dir`.
///
/// Copies are skipped when the destination is already up to date, so this stays
/// cheap when called once per run.
#[cfg(all(dev, desktop))]
fn mirror_resources(
  resources: &tauri_utils::config::BundleResources,
  base_dir: &Path,
  out_dir: &Path,
) -> Result<()> {
  use tauri_utils::{config::BundleResources, resources::ResourcePaths};

  let resources = match resources {
    BundleResources::List(patterns) => ResourcePaths::new(patterns.as_slice(), true),
    BundleResources::Map(map) => ResourcePaths::from_map(map, true),
  };

  for resource in resources.with_base_dir(base_dir).iter() {
    let resource = resource.map_err(std::io::Error::other)?;
    let dest = out_dir.join(resource.target());
    if !up_to_date(resource.path(), &dest) {
      if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
      }
      std::fs::copy(resource.path(), &dest)?;
    }
  }

  Ok(())
}

#[cfg(all(dev, desktop))]
fn up_to_date(src: &Path, dest: &Path) -> bool {
  let (Ok(src), Ok(dest)) = (src.metadata(), dest.metadata()) else {
    return false;
  };
  src.len() == dest.len()
    && matches!(
      (src.modified(), dest.modified()),
      (Ok(src), Ok(dest)) if dest >= src
    )
}

#[cfg(all(test, dev, desktop))]
mod tests {
  use std::{collections::HashMap, fs};

  use tauri_utils::config::BundleResources;

  use super::{mirror_resources, resources_mirror_source_layout};

  fn list(patterns: &[&str]) -> BundleResources {
    BundleResources::List(patterns.iter().map(ToString::to_string).collect())
  }

  #[test]
  fn source_layout_mirror_detection() {
    assert!(resources_mirror_source_layout(&list(&[
      "assets/*",
      "lang/en.json",
      "./data",
      "files/**/*"
    ])));

    assert!(!resources_mirror_source_layout(&list(&["../assets/*"])));
    assert!(!resources_mirror_source_layout(&list(&[
      "assets/*",
      "dir/../../escape.txt"
    ])));
    #[cfg(windows)]
    assert!(!resources_mirror_source_layout(&list(&["C:\\abs\\file"])));
    #[cfg(not(windows))]
    assert!(!resources_mirror_source_layout(&list(&["/abs/file"])));

    assert!(!resources_mirror_source_layout(&BundleResources::Map(
      HashMap::from([("assets".into(), "assets".into())])
    )));
  }

  #[test]
  fn resource_dir_resolves_to_sources_in_dev() {
    use crate::Manager;

    let mut context = crate::test::mock_context(crate::test::noop_assets());
    context.config.bundle.resources = Some(list(&["assets/*", "lang/en.json"]));
    context.with_config_parent("/app/src-tauri");

    let app = crate::test::mock_builder().build(context).unwrap();
    assert_eq!(
      app.path().resource_dir().unwrap(),
      std::path::PathBuf::from("/app/src-tauri")
    );
    assert_eq!(
      app
        .path()
        .resolve("assets/logo.png", crate::path::BaseDirectory::Resource)
        .unwrap(),
      std::path::PathBuf::from("/app/src-tauri/assets/logo.png")
    );

    // without configured resources, the default resolution is used
    let context = crate::test::mock_context(crate::test::noop_assets());
    let app = crate::test::mock_builder().build(context).unwrap();
    assert_ne!(
      app.path().resource_dir().unwrap(),
      std::path::PathBuf::from("/app/src-tauri")
    );
  }

  #[test]
  fn mirrors_remapped_resources() {
    let mut random = [0; 8];
    getrandom::fill(&mut random).unwrap();
    let root = std::env::temp_dir().join(format!(
      "tauri_dev_resources_test_{}",
      random.map(|b| b.to_string()).join("")
    ));
    let _ = fs::remove_dir_all(&root);

    let app_dir = root.join("app/src-tauri");
    for (path, content) in [
      ("app/assets/logo.png", "logo"),
      ("app/assets/nested/deep.txt", "deep"),
      ("app/src-tauri/lang/en.json", "en"),
      ("app/src-tauri/data.bin", "data"),
    ] {
      let path = root.join(path);
      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, content).unwrap();
    }

    let resources = BundleResources::Map(HashMap::from([
      ("../assets".to_string(), "assets".to_string()),
      ("lang/en.json".to_string(), "locales/en.json".to_string()),
      ("data.bin".to_string(), String::new()),
    ]));

    let out_dir = root.join("out");
    fs::create_dir_all(&out_dir).unwrap();
    mirror_resources(&resources, &app_dir, &out_dir).unwrap();

    for (path, content) in [
      ("assets/logo.png", "logo"),
      ("assets/nested/deep.txt", "deep"),
      ("locales/en.json", "en"),
      ("data.bin", "data"),
    ] {
      assert_eq!(
        fs::read_to_string(out_dir.join(path)).unwrap(),
        content,
        "unexpected content for {path}"
      );
    }

    // a second run refreshes modified sources and keeps up-to-date copies
    let dest_modified = |p: &str| out_dir.join(p).metadata().unwrap().modified().unwrap();
    let untouched_before = dest_modified("data.bin");
    fs::write(app_dir.join("lang/en.json"), "en-updated").unwrap();
    mirror_resources(&resources, &app_dir, &out_dir).unwrap();
    assert_eq!(
      fs::read_to_string(out_dir.join("locales/en.json")).unwrap(),
      "en-updated"
    );
    assert_eq!(dest_modified("data.bin"), untouched_before);

    // resources reaching outside the app dir via list notation get `_up_` targets
    let out_dir = root.join("out-list");
    fs::create_dir_all(&out_dir).unwrap();
    mirror_resources(&list(&["../assets/*.png", "data.bin"]), &app_dir, &out_dir).unwrap();
    assert_eq!(
      fs::read_to_string(out_dir.join("_up_/assets/logo.png")).unwrap(),
      "logo"
    );
    assert_eq!(
      fs::read_to_string(out_dir.join("data.bin")).unwrap(),
      "data"
    );

    let _ = fs::remove_dir_all(&root);
  }
}
