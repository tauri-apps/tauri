// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fs, path::PathBuf, process::Command};

use anyhow::Context;

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

  // generate deb_folder structure
  let (data_dir, icons) = debian::generate_data(&settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;
  fs_utils::copy_custom_files(&settings.appimage().files, &data_dir)
    .with_context(|| "Failed to copy custom files")?;

  fs::create_dir_all(data_dir.join("usr/bin/"))?;
  fs::create_dir_all(data_dir.join("usr/lib/"))?;
  fs::create_dir_all(data_dir.join("usr/lib/locales"))?;

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
    let dest = if f == "chrome-sandbox" {
      data_dir.join("usr/bin/").join(f)
    } else {
      data_dir.join("usr/lib/").join(f)
    };
    fs::copy(cef_path.join(f), &dest)?;
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
      data_dir.join("usr/lib/locales").join(f),
    )?;
  }

  fs::create_dir_all(&output_path)?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.product_name()));
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

  let bins = settings.copy_binaries(&app_dir_path.join("usr/bin/"))?;
  let bins = bins
    .iter()
    .map(|b| format!(" \"{}\"", b.to_string_lossy()))
    .collect::<String>();

  // quick-sharun checks the main binary with ldd so even though we manually add the cef files,
  // we'll add them to LD_LIBRARY_PATH to pass the pre-bundle checks
  let mut ld_lib_path = data_dir.join("usr/lib/").to_string_lossy().to_string();
  if let Ok(ld_env) = std::env::var("LD_LIBRARY_PATH") {
    ld_lib_path = format!("{}:{}", ld_lib_path, ld_env);
  }

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
    .env("LD_LIBRARY_PATH", ld_lib_path)
    .args([
      "-c",
      &format!(
        r#""{}" "{}" {bins} "{}" "{}""#,
        quick_sharun.to_string_lossy(),
        data_dir
          .join(format!("usr/bin/{}", main_binary.name()))
          .to_string_lossy(),
        // TODO: This may have to be in lib instead
        data_dir.join("usr/bin/chrome-sandbox").to_string_lossy(),
        data_dir.join("usr/lib/").to_string_lossy()
      ),
    ])
    .output_ok()
    .context("quick-sharun command failed to run.")?;

  {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&appimage_path, fs::Permissions::from_mode(0o770)).expect("perms");
  }

  fs::remove_dir_all(package_dir).expect("rmdir");
  Ok(vec![appimage_path])
}
