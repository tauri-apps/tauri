// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  MobileTarget, OptionsHandle, detect_target_ok, ensure_init, env, get_app, get_config,
  inject_resources, load_pbxproj, log_finished, open_and_wait, project_config,
  synchronize_project_config,
};
use crate::{
  ConfigValue, Error, Result,
  build::Options as BuildOptions,
  error::{Context, ErrorExt},
  helpers::{
    app_paths::Dirs,
    config::{ConfigMetadata, get_config as get_tauri_config},
    flock,
    plist::merge_plist,
  },
  interface::{AppInterface, Options as InterfaceOptions},
  mobile::{CliOptions, TargetDevice, ios::ensure_ios_runtime_installed, write_options},
};
use clap::{ArgAction, Parser, ValueEnum};

use cargo_mobile2::{
  apple::{
    config::Config as AppleConfig,
    target::{ArchiveConfig, BuildConfig, ExportConfig, Target},
  },
  env::Env,
  opts::{NoiseLevel, Profile},
  target::{TargetInvalid, TargetTrait, call_for_targets_with_fallback},
};
use rand::distr::{Alphanumeric, SampleString};

use std::{
  collections::HashMap,
  env::{set_current_dir, var, var_os},
  fs,
  path::{Path, PathBuf},
};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode for iOS and generate IPAs",
  long_about = "Build your app in release mode for iOS and generate IPAs. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`."
)]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build.
  #[clap(
    short,
    long = "target",
    action = ArgAction::Append,
    num_args(0..),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Option<Vec<String>>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub features: Vec<String>,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json and tauri.ios.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Build number to append to the app version.
  #[clap(long)]
  pub build_number: Option<u32>,
  /// Open Xcode
  #[clap(short, long)]
  pub open: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
  /// Describes how Xcode should export the archive.
  ///
  /// Use this to create a package ready for the App Store (app-store-connect option) or TestFlight (release-testing option).
  #[clap(long, value_enum)]
  pub export_method: Option<ExportMethod>,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments.
  /// e.g. `tauri ios build -- [runnerArgs]`.
  #[clap(last(true))]
  pub args: Vec<String>,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  ///
  /// Only use this when you are sure the mismatch is incorrectly detected as version mismatched Tauri packages can lead to unknown behavior.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
  /// Skip code signing when bundling the app
  #[clap(long)]
  pub no_sign: bool,
  /// Only archive the app, skip generating the IPA.
  #[clap(long)]
  pub archive_only: bool,
  /// Target device of this build
  #[clap(skip)]
  pub target_device: Option<TargetDevice>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportMethod {
  AppStoreConnect,
  ReleaseTesting,
  Debugging,
}

impl ExportMethod {
  /// Xcode 15.4 deprecated these names (in this case we should use the Display impl).
  pub fn pre_xcode_15_4_name(&self) -> String {
    match self {
      Self::AppStoreConnect => "app-store".to_string(),
      Self::ReleaseTesting => "ad-hoc".to_string(),
      Self::Debugging => "development".to_string(),
    }
  }
}

impl std::fmt::Display for ExportMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::AppStoreConnect => write!(f, "app-store-connect"),
      Self::ReleaseTesting => write!(f, "release-testing"),
      Self::Debugging => write!(f, "debugging"),
    }
  }
}

impl std::str::FromStr for ExportMethod {
  type Err = &'static str;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "app-store-connect" => Ok(Self::AppStoreConnect),
      "release-testing" => Ok(Self::ReleaseTesting),
      "debugging" => Ok(Self::Debugging),
      _ => Err("unknown ios target"),
    }
  }
}

impl From<Options> for BuildOptions {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features,
      bundles: None,
      no_bundle: false,
      config: options.config,
      args: options.args,
      ci: options.ci,
      skip_stapling: false,
      ignore_version_mismatches: options.ignore_version_mismatches,
      no_sign: options.no_sign,
      no_binary_patching: false,
    }
  }
}

pub struct BuiltApplication {
  pub config: AppleConfig,
  pub interface: AppInterface,
  // prevent drop
  #[allow(dead_code)]
  options_handle: OptionsHandle,
  #[allow(dead_code)]
  export_options_tmp: Option<tempfile::NamedTempFile>,
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<BuiltApplication> {
  let dirs = crate::helpers::app_paths::resolve_dirs();
  run(options, noise_level, &dirs)
}

pub fn run(options: Options, noise_level: NoiseLevel, dirs: &Dirs) -> Result<BuiltApplication> {
  let mut build_options: BuildOptions = options.clone().into();
  build_options.target = Some(
    Target::all()
      .get(
        options
          .targets
          .as_ref()
          .and_then(|t| t.first())
          .map(|t| t.as_str())
          .unwrap_or(Target::DEFAULT_KEY),
      )
      .unwrap()
      .triple
      .into(),
  );

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Ios,
    &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
    dirs.tauri,
  )?;
  let interface = AppInterface::new(&tauri_config, build_options.target.clone(), dirs.tauri)?;
  interface.build_options(&mut build_options.args, &mut build_options.features, true);

  let app = get_app(MobileTarget::Ios, &tauri_config, &interface, dirs.tauri);
  let (mut config, _) = get_config(
    &app,
    &tauri_config,
    &build_options.features,
    &CliOptions {
      dev: false,
      features: build_options.features.clone(),
      args: build_options.args.clone(),
      noise_level,
      vars: Default::default(),
      config: build_options.config.clone(),
      target_device: None,
    },
    dirs.tauri,
  )?;

  set_current_dir(dirs.tauri).context("failed to set current directory")?;

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::Ios,
    options.ci,
  )?;
  inject_resources(&config, &tauri_config)?;

  let mut plist = plist::Dictionary::new();
  plist.insert(
    "CFBundleShortVersionString".into(),
    config.bundle_version_short().into(),
  );

  let info_plist_path = config
    .project_dir()
    .join(config.scheme())
    .join("Info.plist");
  let mut src_plists = vec![info_plist_path.clone().into()];
  src_plists.push(plist::Value::Dictionary(plist).into());
  if dirs.tauri.join("Info.plist").exists() {
    src_plists.push(dirs.tauri.join("Info.plist").into());
  }
  if dirs.tauri.join("Info.ios.plist").exists() {
    src_plists.push(dirs.tauri.join("Info.ios.plist").into());
  }
  {
    if let Some(info_plist) = &tauri_config.bundle.ios.info_plist {
      src_plists.push(info_plist.clone().into());
    }
    if let Some(associations) = tauri_config.bundle.file_associations.as_ref() {
      if let Some(file_associations) = tauri_utils::config::file_associations_plist(associations) {
        src_plists.push(file_associations.into());
      }
    }
  }
  let merged_info_plist = merge_plist(src_plists)?;
  merged_info_plist
    .to_file_xml(&info_plist_path)
    .map_err(std::io::Error::other)
    .fs_context("failed to save merged Info.plist file", info_plist_path)?;

  let mut env = env().context("failed to load iOS environment")?;

  if !options.open {
    ensure_ios_runtime_installed()?;
  }

  // start from the user's export options so values only inferred from the Xcode project
  // (like the signing certificate) do not override what they explicitly configured
  let export_options_plist_path = config.project_dir().join("ExportOptions.plist");
  let mut export_options_plist = if export_options_plist_path.exists() {
    plist::Value::from_file(&export_options_plist_path)
      .with_context(|| {
        format!(
          "failed to parse plist from {}",
          export_options_plist_path.display()
        )
      })?
      .into_dictionary()
      .unwrap_or_default()
  } else {
    plist::Dictionary::new()
  };
  if let Some(method) = options.export_method {
    let xcode_version =
      crate::info::env_system::xcode_version().context("failed to determine Xcode version")?;
    let mut iter = xcode_version.split('.');
    let major = iter.next().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver"
    ))?;
    let minor = iter.next().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver"
    ))?;
    let major = major.parse::<u64>().ok().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver: major is not a number"
    ))?;
    let minor = minor.parse::<u64>().ok().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver: minor is not a number"
    ))?;

    if major < 15 || (major == 15 && minor < 4) {
      export_options_plist.insert("method".to_string(), method.pre_xcode_15_4_name().into());
    } else {
      export_options_plist.insert("method".to_string(), method.to_string().into());
    }
  }

  let (keychain, provisioning_profile) = super::signing_from_env()?;
  let project_config = project_config(keychain.as_ref(), provisioning_profile.as_ref())?;
  let mut pbxproj = load_pbxproj(&config)?;

  // synchronize pbxproj and exportoptions
  synchronize_project_config(
    &config,
    &tauri_config,
    &mut pbxproj,
    &mut export_options_plist,
    &project_config,
    options.debug,
  )?;
  if pbxproj.has_changes() {
    pbxproj
      .save()
      .fs_context("failed to save pbxproj file", pbxproj.path)?;
  }

  // write the merged export options to a temp file
  let export_options_tmp = if !export_options_plist.is_empty() {
    let export_options =
      tempfile::NamedTempFile::new().context("failed to create temporary file")?;

    plist::Value::from(export_options_plist)
      .to_file_xml(export_options.path())
      .map_err(std::io::Error::other)
      .fs_context(
        "failed to save export options plist file",
        export_options.path().to_path_buf(),
      )?;

    config.set_export_options_plist_path(export_options.path());

    Some(export_options)
  } else {
    None
  };

  let open = options.open;
  let options_handle = run_build(
    &interface,
    options,
    build_options,
    tauri_config,
    &mut config,
    &mut env,
    noise_level,
    dirs,
  )?;

  if open {
    open_and_wait(&config, &env);
  }

  Ok(BuiltApplication {
    config,
    interface,
    options_handle,
    export_options_tmp,
  })
}

#[allow(clippy::too_many_arguments)]
fn run_build(
  interface: &AppInterface,
  options: Options,
  mut build_options: BuildOptions,
  tauri_config: ConfigMetadata,
  config: &mut AppleConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
  dirs: &Dirs,
) -> Result<OptionsHandle> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  crate::build::setup(interface, &mut build_options, &tauri_config, dirs, true)?;

  let app_settings = interface.app_settings();
  let out_dir = app_settings.out_dir(
    &InterfaceOptions {
      debug: build_options.debug,
      target: build_options.target.clone(),
      args: build_options.args.clone(),
      ..Default::default()
    },
    dirs.tauri,
  )?;
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ios"), "iOS")?;

  let cli_options = CliOptions {
    dev: false,
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
    config: build_options.config.clone(),
    target_device: options.target_device.clone(),
  };
  let handle = write_options(MobileTarget::Ios, dirs.tauri, cli_options)?;

  if options.open {
    return Ok(handle);
  }

  let mut out_files = Vec::new();

  let force_skip_target_fallback = options.targets.as_ref().is_some_and(|t| t.is_empty());

  call_for_targets_with_fallback(
    options.targets.unwrap_or_default().iter(),
    if force_skip_target_fallback {
      &|_| None
    } else {
      &detect_target_ok
    },
    env,
    |target: &Target| -> Result<()> {
      let mut app_version = config.bundle_version().to_string();
      if let Some(build_number) = options.build_number {
        app_version.push('.');
        app_version.push_str(&build_number.to_string());
      }

      let credentials = auth_credentials_from_env()?;
      let skip_signing = options.no_sign || credentials.is_some();

      if !(options.archive_only || options.no_sign) {
        let mut build_config = BuildConfig::new().allow_provisioning_updates();
        if let Some(credentials) = &credentials {
          build_config = build_config.authentication_credentials(credentials.clone());
        }
        if skip_signing {
          build_config = build_config.skip_codesign();
        }

        target
          .build(None, config, env, noise_level, profile, build_config)
          .context("failed to build iOS app")?;
      }

      let mut archive_config = ArchiveConfig::new().allow_provisioning_updates();
      if let Some(credentials) = &credentials {
        archive_config = archive_config.authentication_credentials(credentials.clone());
      }
      if skip_signing {
        archive_config = archive_config.skip_codesign();
      }

      target
        .archive(
          config,
          env,
          noise_level,
          profile,
          Some(app_version),
          archive_config,
        )
        .context("failed to archive iOS app")?;

      if options.archive_only {
        out_files.push(
          config
            .archive_dir()
            .join(format!("{}.xcarchive", config.scheme())),
        );
        return Ok(());
      }

      let out_dir = config.export_dir().join(target.arch);

      if target.sdk == "iphonesimulator" {
        fs::create_dir_all(&out_dir)
          .fs_context("failed to create Xcode output directory", out_dir.clone())?;

        let app_path = config
          .archive_dir()
          .join(format!("{}.xcarchive", config.scheme()))
          .join("Products")
          .join("Applications")
          .join(config.app().stylized_name())
          .with_extension("app");

        let path = out_dir.join(app_path.file_name().unwrap());
        // `rename` does not replace a non-empty directory, such as the previous build
        if path.exists() {
          fs::remove_dir_all(&path).fs_context("failed to remove previous app", path.clone())?;
        }
        fs::rename(&app_path, &path).fs_context("failed to rename app", app_path)?;
        out_files.push(path);
      } else if options.no_sign {
        fs::create_dir_all(&out_dir)
          .fs_context("failed to create Xcode output directory", out_dir.clone())?;

        let app_path = config
          .archive_dir()
          .join(format!("{}.xcarchive", config.scheme()))
          .join("Products")
          .join("Applications")
          .join(config.app().stylized_name())
          .with_extension("app");

        let ipa_path = out_dir
          .join(config.app().stylized_name())
          .with_extension("ipa");
        create_ipa(&app_path, &ipa_path)?;
        out_files.push(ipa_path);
      } else {
        // if we skipped code signing, we do not have the entitlements applied to our exported IPA
        // we must force sign the app binary with a dummy certificate just to preserve the entitlements
        // target.export() will sign it with an actual certificate for us
        if skip_signing {
          let password = Alphanumeric.sample_string(&mut rand::rng(), 16);
          let certificate = tauri_macos_sign::certificate::generate_self_signed(
            tauri_macos_sign::certificate::SelfSignedCertificateRequest {
              algorithm: "rsa".to_string(),
              profile: tauri_macos_sign::certificate::CertificateProfile::AppleDistribution,
              team_id: "unset".to_string(),
              person_name: "Tauri".to_string(),
              country_name: "NL".to_string(),
              validity_days: 365,
              password: password.clone(),
            },
          )
          .map_err(Box::new)?;
          let tmp_dir = tempfile::tempdir().context("failed to create temporary directory")?;
          let cert_path = tmp_dir.path().join("cert.p12");
          std::fs::write(&cert_path, certificate)
            .fs_context("failed to write certificate", cert_path.clone())?;
          let self_signed_cert_keychain =
            tauri_macos_sign::Keychain::with_certificate_file(&cert_path, &password.into())
              .map_err(Box::new)?;

          let app_dir = config
            .export_dir()
            .join(format!("{}.xcarchive", config.scheme()))
            .join("Products/Applications")
            .join(format!("{}.app", config.app().stylized_name()));

          // nested code (app extensions, watch apps, app clips) is also left unsigned in the archive
          // so it must be signed with its own entitlements too, before the app that contains it
          sign_embedded_bundles(config, profile, &app_dir, &self_signed_cert_keychain)?;

          self_signed_cert_keychain
            .sign(
              &app_dir.join(config.app().stylized_name()),
              Some(
                &config
                  .project_dir()
                  .join(config.scheme())
                  .join(format!("{}.entitlements", config.scheme())),
              ),
              false,
            )
            .map_err(Box::new)?;
        }

        let mut export_config = ExportConfig::new().allow_provisioning_updates();
        if let Some(credentials) = &credentials {
          export_config = export_config.authentication_credentials(credentials.clone());
        }

        target
          .export(config, env, noise_level, export_config)
          .context("failed to export iOS app")?;

        if let Ok(ipa_path) = config.ipa_path() {
          fs::create_dir_all(&out_dir)
            .fs_context("failed to create Xcode output directory", out_dir.clone())?;
          let path = out_dir.join(ipa_path.file_name().unwrap());
          fs::rename(&ipa_path, &path).fs_context("failed to rename IPA", ipa_path)?;
          out_files.push(path);
        }
      }

      Ok(())
    },
  )
  .map_err(|e: TargetInvalid| Error::GenericError(e.to_string()))??;

  if !out_files.is_empty() {
    log_finished(out_files, "iOS Bundle");
  }

  Ok(handle)
}

/// Signs the bundles embedded in `app_dir` (app extensions, watch apps, app clips) with the
/// entitlements declared by their own Xcode target.
///
/// When code signing is skipped while archiving, these bundles are left unsigned and
/// `xcodebuild -exportArchive` only preserves entitlements that are already present in a signature,
/// so they would otherwise be exported with the provisioning profile defaults only,
/// silently dropping e.g. their App Groups.
fn sign_embedded_bundles(
  config: &AppleConfig,
  profile: Profile,
  app_dir: &Path,
  keychain: &tauri_macos_sign::Keychain,
) -> Result<()> {
  let entitlements = match embedded_bundle_entitlements(config, profile) {
    Ok(entitlements) => entitlements,
    Err(error) => {
      log::warn!(
        "Failed to read the Xcode project build settings, embedded app extensions might lose their entitlements: {error}"
      );
      return Ok(());
    }
  };
  if entitlements.is_empty() {
    return Ok(());
  }

  let mut bundles = Vec::new();
  let mut stack = vec![app_dir.to_path_buf()];
  while let Some(dir) = stack.pop() {
    for entry in fs::read_dir(&dir).fs_context("failed to read directory", dir.clone())? {
      let path = entry
        .fs_context("failed to read directory entry", dir.clone())?
        .path();
      // do not follow symlinks, bundles are never symlinked inside the app
      if !path.is_dir() || path.is_symlink() {
        continue;
      }
      if let Some(entitlements_path) = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| entitlements.get(name))
      {
        bundles.push((path.clone(), entitlements_path.clone()));
      }
      stack.push(path);
    }
  }

  // sign the innermost bundles first, signing a bundle seals the code nested in it
  bundles.sort_by_key(|(path, _)| std::cmp::Reverse(path.components().count()));

  for (bundle, entitlements_path) in bundles {
    keychain
      .sign(&bundle, Some(&entitlements_path), false)
      .map_err(Box::new)?;
  }

  Ok(())
}

/// Maps the product file name (e.g. `ShareExtension.appex`) of every Xcode target
/// that declares an entitlements file to the absolute path of that file.
fn embedded_bundle_entitlements(
  config: &AppleConfig,
  profile: Profile,
) -> Result<HashMap<String, PathBuf>> {
  let project_path = config
    .project_dir()
    .join(format!("{}.xcodeproj", config.app().name()));
  let output = std::process::Command::new("xcodebuild")
    .arg("-project")
    .arg(&project_path)
    .arg("-alltargets")
    .args(["-configuration", profile.as_str()])
    .args(["-sdk", "iphoneos"])
    .arg("-showBuildSettings")
    .arg("-json")
    .output()
    .context("failed to run xcodebuild")?;
  if !output.status.success() {
    return Err(Error::GenericError(format!(
      "xcodebuild -showBuildSettings failed: {}",
      String::from_utf8_lossy(&output.stderr)
    )));
  }
  let main_app_product_name = format!("{}.app", config.app().stylized_name());
  parse_bundle_entitlements(&output.stdout, &main_app_product_name)
}

fn parse_bundle_entitlements(
  build_settings_json: &[u8],
  main_app_product_name: &str,
) -> Result<HashMap<String, PathBuf>> {
  #[derive(serde::Deserialize)]
  struct TargetBuildSettings {
    #[serde(rename = "buildSettings")]
    build_settings: HashMap<String, serde_json::Value>,
  }

  let targets: Vec<TargetBuildSettings> = serde_json::from_slice(build_settings_json)
    .context("failed to parse the xcodebuild build settings")?;

  let mut entitlements = HashMap::new();
  for target in targets {
    let settings = target.build_settings;
    let setting = |key: &str| settings.get(key).and_then(|value| value.as_str());
    let (Some(product_name), Some(entitlements_path)) = (
      setting("FULL_PRODUCT_NAME"),
      setting("CODE_SIGN_ENTITLEMENTS"),
    ) else {
      continue;
    };
    // the main app is signed with the entitlements file managed by the Tauri CLI
    if product_name == main_app_product_name || entitlements_path.is_empty() {
      continue;
    }
    // CODE_SIGN_ENTITLEMENTS is relative to SRCROOT unless it is already absolute
    let entitlements_path = match setting("SRCROOT") {
      Some(src_root) => Path::new(src_root).join(entitlements_path),
      None => PathBuf::from(entitlements_path),
    };
    entitlements.insert(product_name.to_string(), entitlements_path);
  }

  Ok(entitlements)
}

fn create_ipa(app_path: &Path, ipa_path: &Path) -> Result<()> {
  let ipa_file =
    fs::File::create(ipa_path).fs_context("failed to create IPA file", ipa_path.to_path_buf())?;
  let mut zip = zip::ZipWriter::new(ipa_file);
  let options = zip::write::SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated)
    .unix_permissions(0o755);

  zip
    .add_directory("Payload/", options)
    .context("failed to add Payload directory to zip")?;

  let mut app_files = Vec::new();
  let mut stack = vec![app_path.to_path_buf()];
  while let Some(path) = stack.pop() {
    if path.is_dir() {
      app_files.push(path.clone());
      for entry in fs::read_dir(&path).fs_context("failed to read directory", path.clone())? {
        stack.push(
          entry
            .fs_context("failed to read directory entry", path.clone())?
            .path(),
        );
      }
    } else {
      app_files.push(path);
    }
  }

  for file_path in app_files {
    let name = file_path.strip_prefix(app_path.parent().unwrap()).unwrap();
    let mut name_str = name.to_string_lossy().to_string();
    // zip expects forward slashes
    if std::path::MAIN_SEPARATOR == '\\' {
      name_str = name_str.replace('\\', "/");
    }
    let mut name_in_zip = format!("Payload/{name_str}");

    if file_path.is_dir() {
      name_in_zip.push('/');
      zip
        .add_directory(name_in_zip, options)
        .context("failed to add directory to zip")?;
    } else {
      zip
        .start_file(name_in_zip, options)
        .context("failed to start file in zip")?;
      let mut f = fs::File::open(&file_path).fs_context("failed to open file", file_path)?;
      std::io::copy(&mut f, &mut zip).context("failed to copy file to zip")?;
    }
  }

  zip.finish().context("failed to finish zip")?;
  Ok(())
}

fn auth_credentials_from_env() -> Result<Option<cargo_mobile2::apple::AuthCredentials>> {
  match (
    var("APPLE_API_KEY"),
    var("APPLE_API_ISSUER"),
    var_os("APPLE_API_KEY_PATH").map(PathBuf::from),
  ) {
    (Ok(key_id), Ok(key_issuer_id), Some(key_path)) => {
      Ok(Some(cargo_mobile2::apple::AuthCredentials {
        key_path,
        key_id,
        key_issuer_id,
      }))
    }
    (Err(_), Err(_), None) => Ok(None),
    _ => crate::error::bail!(
      "APPLE_API_KEY, APPLE_API_ISSUER and APPLE_API_KEY_PATH must be provided for code signing"
    ),
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  #[test]
  fn parse_bundle_entitlements() {
    let json = br#"[
      {
        "action": "build",
        "target": "app_iOS",
        "buildSettings": {
          "FULL_PRODUCT_NAME": "App.app",
          "CODE_SIGN_ENTITLEMENTS": "app_iOS/app_iOS.entitlements",
          "SRCROOT": "/project/gen/apple"
        }
      },
      {
        "action": "build",
        "target": "ShareExtension",
        "buildSettings": {
          "FULL_PRODUCT_NAME": "ShareExtension.appex",
          "CODE_SIGN_ENTITLEMENTS": "ShareExtension/ShareExtension.entitlements",
          "SRCROOT": "/project/gen/apple"
        }
      },
      {
        "action": "build",
        "target": "Widget",
        "buildSettings": {
          "FULL_PRODUCT_NAME": "Widget.appex",
          "CODE_SIGN_ENTITLEMENTS": "/absolute/Widget.entitlements",
          "SRCROOT": "/project/gen/apple"
        }
      },
      {
        "action": "build",
        "target": "Framework",
        "buildSettings": {
          "FULL_PRODUCT_NAME": "Framework.framework",
          "SRCROOT": "/project/gen/apple"
        }
      }
    ]"#;

    let entitlements = super::parse_bundle_entitlements(json, "App.app").unwrap();

    assert_eq!(entitlements.len(), 2);
    assert_eq!(
      entitlements.get("ShareExtension.appex"),
      Some(&PathBuf::from(
        "/project/gen/apple/ShareExtension/ShareExtension.entitlements"
      ))
    );
    assert_eq!(
      entitlements.get("Widget.appex"),
      Some(&PathBuf::from("/absolute/Widget.entitlements"))
    );
  }
}
