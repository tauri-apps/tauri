// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Context;

use crate::{
  bundle::{linux::freedesktop, settings::Arch},
  utils::{
    fs_utils,
    http_utils::{download_and_verify, verify_file_hash, HashAlgorithm},
    CommandExt,
  },
  Settings,
};

use super::write_and_make_executable;

/// Pinned revision of the Anylinux-AppImages tooling.
///
/// `quick-sharun.sh` drives the entire deployment and upstream moves fast, so we
/// pin and checksum it instead of tracking a branch: a release build must not
/// change behaviour because of a push nobody reviewed. Bump both constants
/// together after testing the new revision.
///
/// Note that the script still downloads a few tools of its own (sharun,
/// appimagetool, uruntime, onelf, cross-libc-dlopen). The ones it pins itself
/// are pinned; the rest track their latest release.
const QUICK_SHARUN_REV: &str = "facb95e825cb082f634d48385f86b05a5c5cab66";
const QUICK_SHARUN_SHA256: &str =
  "af0851857ea505a6f600dfc47dcd421f5e719502f880d68cc3741bc23a7700a4";

/// Env var pointing at a local `quick-sharun.sh`, for testing changes to the
/// tooling without rebuilding the CLI. Skips the checksum, so it is unsupported
/// for release builds.
const QUICK_SHARUN_SCRIPT_ENV: &str = "TAURI_BUNDLER_QUICK_SHARUN_SCRIPT";

fn anylinux_raw_url(file: &str) -> String {
  // Note: `TAURI_BUNDLER_TOOLS_GITHUB_MIRROR` only rewrites github.com release
  // URLs, so it does not apply to these raw.githubusercontent.com paths.
  format!(
    "https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/{QUICK_SHARUN_REV}/useful-tools/{file}"
  )
}

/// Resolves the pinned `quick-sharun.sh`, downloading it only when the cached
/// copy is missing or does not match the pinned checksum. Keeping the revision
/// in the file name means a build is reproducible and, after the first run,
/// works offline.
fn quick_sharun_script(tools_path: &Path) -> crate::Result<PathBuf> {
  if let Some(path) = std::env::var_os(QUICK_SHARUN_SCRIPT_ENV) {
    let path = PathBuf::from(path);
    log::warn!(
      "Using the quick-sharun script at {} because {QUICK_SHARUN_SCRIPT_ENV} is set. The pinned revision and its checksum are ignored.",
      path.display()
    );
    return Ok(path);
  }

  let script = tools_path.join(format!("quick-sharun-{}.sh", &QUICK_SHARUN_REV[..12]));

  if verify_file_hash(&script, QUICK_SHARUN_SHA256, HashAlgorithm::Sha256).is_ok() {
    return Ok(script);
  }

  let data = download_and_verify(
    &anylinux_raw_url("quick-sharun.sh"),
    QUICK_SHARUN_SHA256,
    HashAlgorithm::Sha256,
  )?;
  write_and_make_executable(&script, data)?;

  Ok(script)
}

// TODO: Monitor TLS support / certificates - seems to be working in initial tests
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // for backwards compat we keep the amd64 and i386 rewrites in the filename
  let (appimage_arch, target_arch) = match settings.binary_arch() {
    Arch::X86_64 => ("amd64", "x86_64"),
    Arch::AArch64 => ("aarch64", "aarch64"),
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };

  // The deployment collects libraries from the build system itself, so the
  // bundle can only ever target the architecture we are running on. Without
  // this check a cross-compiled build silently produces an AppImage that is
  // named for the target but filled with the host's libraries.
  if target_arch != std::env::consts::ARCH {
    return Err(crate::Error::ArchError(format!(
      "cannot bundle a {target_arch} AppImage on a {host} host. The new AppImage format deploys the build system's own libraries, so it cannot cross-compile. Build on a {target_arch} machine or container instead.",
      host = std::env::consts::ARCH
    )));
  }

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }

  let product_name = settings.product_name();

  let appimage_filename = format!(
    "{}_{}_{appimage_arch}.AppImage",
    product_name,
    settings.version_string()
  );
  let appimage_path = output_path.join(&appimage_filename);

  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  let tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| {
      dirs::cache_dir().map_or_else(|| output_path.to_path_buf(), |p| p.join("tauri"))
    });

  fs::create_dir_all(&tools_path)?;

  let quick_sharun = quick_sharun_script(&tools_path)?;

  let main_binary = settings.main_binary()?;

  let mut settings = settings.clone();
  if main_binary.name().contains(' ') {
    let main_binary_path = settings.binary_path(main_binary);
    let project_out_dir = settings.project_out_directory();

    let main_binary_name_kebab = heck::AsKebabCase(main_binary.name()).to_string();
    let new_path = project_out_dir.join(&main_binary_name_kebab);
    fs::copy(main_binary_path, new_path)?;

    let main_binary = settings.main_binary_mut()?;
    main_binary.set_name(main_binary_name_kebab);
  }
  let settings = settings;

  fs::create_dir_all(&output_path)?;
  let app_dir = output_path.join(format!("{product_name}.AppDir"));
  let app_dir_bin = app_dir.join("bin/");
  let app_dir_lib = app_dir.join("lib/");

  let desktop_file = freedesktop::generate_desktop_file(&settings, &None, &app_dir)
    .with_context(|| "Failed to create desktop file")?
    .0;
  fs::rename(
    desktop_file,
    app_dir.join(format!("{product_name}.desktop")),
  )
  .with_context(|| "Failed to move desktop file")?;
  let _ = fs_utils::remove_dir_all(&app_dir.join("usr/"));

  // Copy external binaries (externalBin)
  let mut bins = settings
    .copy_binaries(&app_dir_bin)
    .with_context(|| "Failed to copy external binaries")?;

  // Copy Cargo project binaries
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    let trgt = app_dir_bin.join(bin.name());
    fs_utils::copy_file(&bin_path, &trgt)
      .with_context(|| format!("Failed to copy binary from {bin_path:?} to {trgt:?}"))?;
    bins.push(trgt);
  }

  settings
    .copy_resources(&app_dir_lib.join(product_name))
    .with_context(|| "Failed to copy resource files")?;

  fs_utils::copy_custom_files(&settings.appimage().files, &app_dir)
    .with_context(|| "Failed to copy custom files")?;

  let icons = freedesktop::list_icon_files(&settings, Path::new(""))
    .with_context(|| "Failed to create icon files")?;

  let largest_icon = icons
    .into_iter()
    .filter(|(i, _)| i.width == i.height)
    .max_by_key(|(i, _)| i.width)
    .ok_or_else(|| {
      crate::Error::GenericError(
        "couldn't find a square icon to use as AppImage icon. Add a square PNG icon to `bundle > icon` in your Tauri configuration.".into(),
      )
    })?;

  fs::copy(largest_icon.1, app_dir.join(format!("{product_name}.png")))
    .with_context(|| "Failed to copy icon file")?;

  // quick-sharun takes the binaries and libraries to deploy as positional
  // arguments.
  let mut deploy_args = bins;

  // Libraries the CLI injects through `appimage.files`, most notably
  // libappindicator for the tray icon. `copy_custom_files` puts them in the
  // AppDir under `usr/lib`, which sharun never looks at, so the host copy has
  // to be handed over explicitly to be deployed like any other dependency.
  let mut extra_libs = settings
    .appimage()
    .files
    .iter()
    .filter(|(target, _)| target.starts_with("/usr/lib"))
    .map(|(_, source)| source.clone())
    .collect::<Vec<_>>();
  // `files` is a HashMap, so sort to keep the command reproducible.
  extra_libs.sort();
  deploy_args.extend(extra_libs);

  // Deploys anything shipped as a resource that turns out to be a library or a
  // binary. The directory itself always exists by the time the script reads it.
  deploy_args.push(app_dir_lib);

  // quick-sharun runs each binary it deploys for a few seconds to see which
  // libraries get dlopened, then kills it with a process-group signal. That
  // only reaches the process if the shell put it in its own group, which is
  // what `set -m` is for. dash does not create the group when there is no
  // controlling terminal, so on Debian and Ubuntu, where /bin/sh is dash,
  // every terminal-less build - which is every CI run - hangs forever on the
  // first traced process that does not exit by itself. bash creates the group
  // either way, so prefer it and fall back to sh where it is missing.
  let shell = which::which("bash")
    .map(|p| p.to_string_lossy().into_owned())
    .unwrap_or_else(|_| "/bin/sh".into());

  // Passing the script to the shell as an argument rather than building a
  // `-c` string keeps paths containing spaces intact.
  let mut cmd = Command::new(shell);
  cmd
    .arg(&quick_sharun)
    .args(&deploy_args)
    .current_dir(&output_path)
    .env("APPDIR", &app_dir)
    .env("OUTNAME", &appimage_filename)
    .env("OUTPUT_APPIMAGE", "1")
    // Pin the helper library the script compiles into the bundle to the same
    // revision as the script itself.
    .env("ANYLINUX_LIB_SOURCE", anylinux_raw_url("lib/anylinux.c"));

  if settings.appimage().bundle_media_framework {
    cmd.env("DEPLOY_GSTREAMER", "1");
  }

  if let Some(upinfo) = std::env::var("UPINFO")
    .ok()
    .or_else(|| settings.appimage().update_information.clone())
  {
    cmd.env("UPINFO", upinfo);
  }

  // Streams the tooling's output instead of capturing it: this runs for
  // minutes, downloads tools and launches the app, and its own error messages
  // are the only useful diagnostics when something is missing on the system.
  let status = cmd.piped().context("Failed to run quick-sharun")?;
  if !status.success() {
    return Err(crate::Error::GenericError(
      "quick-sharun failed to build the AppImage, see the output above for details".into(),
    ));
  }

  if !appimage_path.exists() {
    return Err(crate::Error::GenericError(format!(
      "quick-sharun did not produce {}",
      appimage_path.display()
    )));
  }

  let mut bundles = vec![appimage_path];

  // appimagetool writes a zsync file next to the AppImage whenever update
  // information is embedded, which it also guesses from `GITHUB_REPOSITORY`
  // when nothing is configured. It is part of the release, so return it too.
  let zsync_path = output_path.join(format!("{appimage_filename}.zsync"));
  if zsync_path.exists() {
    bundles.push(zsync_path);
  }

  Ok(bundles)
}
