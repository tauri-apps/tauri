// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  io::Read,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::Context;
use walkdir::WalkDir;

use crate::{
  Settings,
  bundle::{linux::debian, settings::Arch},
  utils::{CommandExt, fs_utils, http_utils::download},
};

use super::write_and_make_executable;

// TODO: Test if bundling xdg-mime makes sense (eg does it even work if it's not on the host system?)
// TODO: Monitor TLS support / certificates - seems to be working in initial tests
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // for backwards compat we keep the amd64 and i386 rewrites in the filename
  let appimage_arch = match settings.binary_arch() {
    Arch::X86_64 => "amd64",
    //Arch::X86 => "i386",
    Arch::AArch64 => "aarch64",
    //Arch::Armhf => "armhf",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };
  //let tools_arch = settings.target().split('-').next().unwrap();

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    fs::remove_dir_all(&output_path)?;
  }

  let tools_path = settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| {
      dirs::cache_dir().map_or_else(|| output_path.to_path_buf(), |p| p.join("tauri"))
    });

  fs::create_dir_all(&tools_path)?;

  let quick_sharun = tools_path.join("quick-sharun.sh");
  // TODO: offline build support
  // github doesn't send a Last-Modified header
  // if !quick_sharun.exists() {}
  let data = download(
    "https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/quick-sharun.sh",
  )?;
  write_and_make_executable(&quick_sharun, data)?;

  let package_dir = settings
    .project_out_directory()
    .join("bundle/appimage_deb/");

  let main_binary = settings.main_binary()?;
  let product_name = settings.product_name();

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

  fs::create_dir_all(&output_path)?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.product_name()));

  // generate deb_folder structure
  let (data_dir, icons) = debian::generate_data(&settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;

  fs_utils::copy_dir(&data_dir.join("usr/bin/"), &app_dir_path.join("bin/"))
    .with_context(|| "Failed to copy bin files")?;
  // Only exists when resources feature is used
  if data_dir.join("usr/lib/").exists() {
    fs_utils::copy_dir(&data_dir.join("usr/lib/"), &app_dir_path.join("lib/"))
      .with_context(|| "Failed to copy lib files")?;
  }

  fs_utils::copy_custom_files(&settings.appimage().files, &app_dir_path)
    .with_context(|| "Failed to copy custom files")?;

  fs::create_dir_all(app_dir_path.join("bin/locales/"))?;

  let cef_path = settings
    .bundle_settings()
    .cef_path
    .clone()
    .expect("this module is only called when cef_path is set");

  let cef_files = [
    // required
    "libcef.so",
    "icudtl.dat",
    "v8_context_snapshot.bin",
    // required end
    // "optional" - but not really since we want support for all of this
    "chrome_100_percent.pak",
    "chrome_200_percent.pak",
    "resources.pak",
    // ANGEL support
    "libEGL.so",
    "libGLESv2.so",
    // SwANGLE support
    "libvk_swiftshader.so",
    "vk_swiftshader_icd.json",
    "libvulkan.so.1",
    // sandbox - may need to be behind a setting?
    "chrome-sandbox",
    // TODO: seccomp
  ];

  for f in cef_files {
    let dest = app_dir_path.join("bin/").join(f);
    fs::copy(cef_path.join(f), &dest)
      .with_context(|| format!("Failed to copy cef file {f} to {}", dest.display()))?;
    // quick-sharun checks for the NO_STRIP env but libcef.so is 1.5GB so we make sure it's stripped anyway.
    let _ = Command::new("strip").arg(&dest).output_ok();
  }
  let locales = [
    "en-US.pak",
    "en-US_FEMININE.pak",
    "en-US_MASCULINE.pak",
    "en-US_NEUTER.pak",
  ];

  for f in locales {
    fs::copy(
      cef_path.join("locales").join(f),
      app_dir_path.join("bin/locales").join(f),
    )
    .with_context(|| format!("Failed to copy cef locales file {f}"))?;
  }

  let appimage_filename = format!(
    "{}_{}_{appimage_arch}.AppImage",
    settings.product_name(),
    settings.version_string()
  );
  let appimage_path = output_path.join(&appimage_filename);

  fs::create_dir_all(&tools_path)?;
  let larger_icon = icons
    .iter()
    .filter(|i| i.width == i.height)
    .max_by_key(|i| i.width)
    .expect("couldn't find a square icon to use as AppImage icon");

  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  // TODO:
  let _verbosity = match settings.log_level() {
    log::Level::Error => "-q", // errors only
    log::Level::Info => "",    // errors + "normal logs" (mostly rpath)
    log::Level::Trace => "-v", // You can expect way over 1k lines from just lib4bin on this level
    _ => "",
  };

  // We need to give quick-sharun the list of binaries AND libraries to include.
  // To support weird `appimage.files` settings we just walk through the whole AppDir we set up.
  // TODO: In some cases we may have to give quick-sharun the path to some directories as well.
  let mut elfs = Vec::new();
  for entry in WalkDir::new(&app_dir_path) {
    if let Ok(entry) = entry {
      if entry.file_type().is_file() && is_elf(entry.path()) {
        elfs.push(format!(" \"{}\"", entry.path().to_string_lossy()));
      }
    }
  }
  let elfs = elfs
    .into_iter()
    .collect::<String>();

  // TODO: Consider to not rely on quick-sharun when we have more time
  Command::new("/bin/sh")
    .current_dir(&output_path)
    .env("APPDIR", &app_dir_path)
    .env("OUTNAME", &appimage_filename)
    .env(
      "DESKTOP",
      data_dir.join(format!("usr/share/applications/{product_name}.desktop")),
    )
    .env("ICON", &larger_icon.path)
    .env("OUTPUT_APPIMAGE", "1")
    .env("HOOKSRC", "https://raw.githubusercontent.com/FabianLars/Anylinux-AppImages/refs/heads/main/useful-tools/hooks")
    .env("DEPLOY_CHROMIUM", "1")
    .env("ADD_HOOKS", "fix-namespaces.hook")
    .args([
      "-c",
      &format!(
        r#""{}" {elfs}"#,
        quick_sharun.to_string_lossy()
      ),
    ])
    .output_ok()
    .context("quick-sharun command failed to run.")?;

  fs::remove_dir_all(package_dir).expect("rmdir");
  Ok(vec![appimage_path])
}

fn is_elf(path: &Path) -> bool {
  let mut buf = [0; 4];
  if let Ok(mut file) = fs::File::open(path) {
    if let Ok(_) = file.read_exact(&mut buf) {
      return buf == [0x7f, b'E', b'L', b'F'];
    }
  }
  false
}
