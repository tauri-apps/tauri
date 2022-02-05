// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// A macOS application bundle package is laid out like:
//
// foobar.app    # Actually a directory
//     Contents      # A further subdirectory
//         Info.plist     # An xml file containing the app's metadata
//         MacOS          # A directory to hold executable binary files
//             foobar          # The main binary executable of the app
//             foobar_helper   # A helper application, possibly provitidng a CLI
//         Resources      # Data files such as images, sounds, translations and nib files
//             en.lproj        # Folder containing english translation strings/data
//         Frameworks     # A directory containing private frameworks (shared libraries)
//         ...            # Any other optional files the developer wants to place here
//
// See https://developer.apple.com/go/?id=bundle-structure for a full
// explanation.
//
// Currently, cargo-bundle does not support Frameworks, nor does it support placing arbitrary
// files into the `Contents` directory of the bundle.

use super::{
  super::common,
  icon::create_icns_file,
  sign::{notarize, notarize_auth_args, setup_keychain_if_needed, sign},
};
use crate::Settings;

use anyhow::Context;

use std::{
  fs,
  io::prelude::*,
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the .app was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // we should use the bundle name (App name) as a MacOS standard.
  // version or platform shouldn't be included in the App name.
  let app_product_name = format!("{}.app", settings.product_name());
  common::print_bundling(&app_product_name)?;
  let app_bundle_path = settings
    .project_out_directory()
    .join("bundle/macos")
    .join(&app_product_name);
  if app_bundle_path.exists() {
    fs::remove_dir_all(&app_bundle_path)
      .with_context(|| format!("Failed to remove old {}", app_product_name))?;
  }
  let bundle_directory = app_bundle_path.join("Contents");
  fs::create_dir_all(&bundle_directory).with_context(|| {
    format!(
      "Failed to create bundle directory at {:?}",
      bundle_directory
    )
  })?;

  let resources_dir = bundle_directory.join("Resources");
  let bin_dir = bundle_directory.join("MacOS");

  let bundle_icon_file: Option<PathBuf> =
    { create_icns_file(&resources_dir, settings).with_context(|| "Failed to create app icon")? };

  create_info_plist(&bundle_directory, bundle_icon_file, settings)
    .with_context(|| "Failed to create Info.plist")?;

  copy_frameworks_to_bundle(&bundle_directory, settings)
    .with_context(|| "Failed to bundle frameworks")?;

  settings.copy_resources(&resources_dir)?;

  settings
    .copy_binaries(&bin_dir)
    .with_context(|| "Failed to copy external binaries")?;

  copy_binaries_to_bundle(&bundle_directory, settings)?;

  let use_bootstrapper = settings.macos().use_bootstrapper.unwrap_or_default();
  if use_bootstrapper {
    create_bootstrapper(&bundle_directory, settings)
      .with_context(|| "Failed to create macOS bootstrapper")?;
  }

  if let Some(identity) = &settings.macos().signing_identity {
    // setup keychain allow you to import your certificate
    // for CI build
    setup_keychain_if_needed()?;
    // sign application
    sign(app_bundle_path.clone(), identity, &settings, true)?;
    // notarization is required for distribution
    match notarize_auth_args() {
      Ok(args) => {
        notarize(app_bundle_path.clone(), args, settings)?;
      }
      Err(e) => {
        common::print_info(format!("skipping app notarization, {}", e.to_string()).as_str())?;
      }
    }
  }

  Ok(vec![app_bundle_path])
}

// Copies the app's binaries to the bundle.
fn copy_binaries_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  let dest_dir = bundle_directory.join("MacOS");
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    common::copy_file(&bin_path, &dest_dir.join(bin.name()))
      .with_context(|| format!("Failed to copy binary from {:?}", bin_path))?;
  }
  Ok(())
}

// Creates the bootstrap script file.
fn create_bootstrapper(bundle_dir: &Path, settings: &Settings) -> crate::Result<()> {
  let file = &mut common::create_file(&bundle_dir.join("MacOS/__bootstrapper"))?;
  // Create a shell script to bootstrap the  $PATH for Tauri, so environments like node are available.
  write!(
    file,
    "#!/usr/bin/env sh
# This bootstraps the environment for Tauri, so environments are available.

if [ -e ~/.bash_profile ]
then 
  . ~/.bash_profile
fi
if [ -e ~/.zprofile ]
then 
  . ~/.zprofile
fi
if [ -e ~/.profile ]
then 
  . ~/.profile
fi
if [ -e ~/.bashrc ]
then 
  . ~/.bashrc
fi

if [ -e ~/.zshrc ]
then 
  . ~/.zshrc
fi

if pidof \"__bootstrapper\" >/dev/null; then
    exit 0
else
    exec \"`dirname \\\"$0\\\"`/{}\" $@ & disown
fi
exit 0",
    settings.product_name()
  )?;
  file.flush()?;

  // We have to make the __bootstrapper executable, or the bundle will not work
  let status = Command::new("chmod")
    .arg("+x")
    .arg("__bootstrapper")
    .current_dir(&bundle_dir.join("MacOS/"))
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .status()?;

  if !status.success() {
    return Err(anyhow::anyhow!("failed to make the bootstrapper an executable",).into());
  }

  Ok(())
}

// Creates the Info.plist file.
fn create_info_plist(
  bundle_dir: &Path,
  bundle_icon_file: Option<PathBuf>,
  settings: &Settings,
) -> crate::Result<()> {
  let format = time::format_description::parse("[year][month][day].[hour][minute][second]")
    .map_err(|e| time::error::Error::from(e))?;
  let build_number = time::OffsetDateTime::now_utc()
    .format(&format)
    .map_err(|e| time::error::Error::from(e))?;

  let bundle_plist_path = bundle_dir.join("Info.plist");
  let file = &mut common::create_file(&bundle_plist_path)?;
  let use_bootstrapper = settings.macos().use_bootstrapper.unwrap_or_default();
  write!(
    file,
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \
     \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
     <plist version=\"1.0\">\n\
     <dict>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleDevelopmentRegion</key>\n  \
     <string>English</string>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleDisplayName</key>\n  <string>{}</string>\n",
    settings.product_name()
  )?;
  write!(
    file,
    "  <key>CFBundleExecutable</key>\n  <string>{}</string>\n",
    if use_bootstrapper {
      "__bootstrapper"
    } else {
      settings.main_binary_name()
    }
  )?;
  if let Some(path) = bundle_icon_file {
    write!(
      file,
      "  <key>CFBundleIconFile</key>\n  <string>{}</string>\n",
      path.file_name().expect("No file name").to_string_lossy()
    )?;
  }
  write!(
    file,
    "  <key>CFBundleIdentifier</key>\n  <string>{}</string>\n",
    settings.bundle_identifier()
  )?;
  write!(
    file,
    "  <key>CFBundleInfoDictionaryVersion</key>\n  \
     <string>6.0</string>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleName</key>\n  <string>{}</string>\n",
    settings.product_name()
  )?;
  write!(
    file,
    "  <key>CFBundlePackageType</key>\n  <string>APPL</string>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleShortVersionString</key>\n  <string>{}</string>\n",
    settings.version_string()
  )?;
  write!(
    file,
    "  <key>CFBundleVersion</key>\n  <string>{}</string>\n",
    build_number
  )?;
  write!(file, "  <key>CSResourcesFileMapped</key>\n  <true/>\n")?;
  if let Some(category) = settings.app_category() {
    write!(
      file,
      "  <key>LSApplicationCategoryType</key>\n  \
       <string>{}</string>\n",
      category.macos_application_category_type()
    )?;
  }
  if let Some(version) = &settings.macos().minimum_system_version {
    write!(
      file,
      "  <key>LSMinimumSystemVersion</key>\n  \
       <string>{}</string>\n",
      version
    )?;
  }
  write!(file, "  <key>LSRequiresCarbon</key>\n  <true/>\n")?;
  write!(file, "  <key>NSHighResolutionCapable</key>\n  <true/>\n")?;
  if let Some(copyright) = settings.copyright_string() {
    write!(
      file,
      "  <key>NSHumanReadableCopyright</key>\n  \
       <string>{}</string>\n",
      copyright
    )?;
  }

  if let Some(exception_domain) = &settings.macos().exception_domain {
    write!(
      file,
      "  <key>NSAppTransportSecurity</key>\n  \
      <dict>\n  \
          <key>NSExceptionDomains</key>\n  \
          <dict>\n  \
              <key>{}</key>\n  \
              <dict>\n  \
                  <key>NSExceptionAllowsInsecureHTTPLoads</key>\n  \
                  <true/>\n  \
                  <key>NSIncludesSubdomains</key>\n  \
                  <true/>\n  \
              </dict>\n  \
          </dict>\n  \
      </dict>",
      exception_domain
    )?;
  }

  write!(file, "</dict>\n</plist>\n")?;
  file.flush()?;

  if let Some(user_plist_path) = &settings.macos().info_plist_path {
    let mut cmd = Command::new("/usr/libexec/PlistBuddy");
    cmd.args(&[
      "-c".into(),
      format!("Merge {}", user_plist_path.display()),
      bundle_plist_path.display().to_string(),
    ]);

    common::execute_with_verbosity(&mut cmd, settings).map_err(|_| {
      crate::Error::ShellScriptError(format!(
        "error running /usr/libexec/PlistBuddy{}",
        if settings.is_verbose() {
          ""
        } else {
          ", try running with --verbose to see command output"
        }
      ))
    })?;
  }

  Ok(())
}

// Copies the framework under `{src_dir}/{framework}.framework` to `{dest_dir}/{framework}.framework`.
fn copy_framework_from(dest_dir: &Path, framework: &str, src_dir: &Path) -> crate::Result<bool> {
  let src_name = format!("{}.framework", framework);
  let src_path = src_dir.join(&src_name);
  if src_path.exists() {
    common::copy_dir(&src_path, &dest_dir.join(&src_name))?;
    Ok(true)
  } else {
    Ok(false)
  }
}

// Copies the macOS application bundle frameworks to the .app
fn copy_frameworks_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  let frameworks = settings
    .macos()
    .frameworks
    .as_ref()
    .cloned()
    .unwrap_or_default();
  if frameworks.is_empty() {
    return Ok(());
  }
  let dest_dir = bundle_directory.join("Frameworks");
  fs::create_dir_all(&bundle_directory)
    .with_context(|| format!("Failed to create Frameworks directory at {:?}", dest_dir))?;
  for framework in frameworks.iter() {
    if framework.ends_with(".framework") {
      let src_path = PathBuf::from(framework);
      let src_name = src_path
        .file_name()
        .expect("Couldn't get framework filename");
      common::copy_dir(&src_path, &dest_dir.join(&src_name))?;
      continue;
    } else if framework.contains('/') {
      return Err(crate::Error::GenericError(format!(
        "Framework path should have .framework extension: {}",
        framework
      )));
    }
    if let Some(home_dir) = dirs_next::home_dir() {
      if copy_framework_from(&dest_dir, framework, &home_dir.join("Library/Frameworks/"))? {
        continue;
      }
    }
    if copy_framework_from(&dest_dir, framework, &PathBuf::from("/Library/Frameworks/"))?
      || copy_framework_from(
        &dest_dir,
        framework,
        &PathBuf::from("/Network/Library/Frameworks/"),
      )?
    {
      continue;
    }
    return Err(crate::Error::GenericError(format!(
      "Could not locate framework: {}",
      framework
    )));
  }
  Ok(())
}
