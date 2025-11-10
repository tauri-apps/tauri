#![cfg(target_os = "macos")]

use crate::interface::{
  rust::{DevChild, RustupTarget},
  AppSettings, ExitReason, Options,
};
use crate::{error::ErrorExt, CommandExt, helpers::fs::copy_dir_all};

use serde::Serialize;
use shared_child::SharedChild;
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct InfoPlist {
  #[serde(rename = "CFBundleDevelopmentRegion")]
  cf_bundle_development_region: String,
  #[serde(rename = "CFBundleDisplayName")]
  cf_bundle_display_name: String,
  #[serde(rename = "CFBundleExecutable")]
  cf_bundle_executable: String,
  #[serde(rename = "CFBundleIdentifier")]
  cf_bundle_identifier: String,
  #[serde(rename = "CFBundleInfoDictionaryVersion")]
  cf_bundle_info_dictionary_version: String,
  #[serde(rename = "CFBundleName")]
  cf_bundle_name: String,
  #[serde(rename = "CFBundlePackageType")]
  cf_bundle_package_type: String,
  #[serde(rename = "CFBundleSignature")]
  cf_bundle_signature: String,
  #[serde(rename = "CFBundleVersion")]
  cf_bundle_version: String,
  #[serde(rename = "CFBundleShortVersionString")]
  cf_bundle_short_version_string: String,
  #[serde(rename = "LSEnvironment")]
  ls_environment: HashMap<String, String>,
  #[serde(rename = "LSFileQuarantineEnabled")]
  ls_file_quarantine_enabled: bool,
  #[serde(rename = "LSMinimumSystemVersion")]
  ls_minimum_system_version: String,
  #[serde(rename = "LSUIElement")]
  ls_ui_element: Option<String>,
  #[serde(rename = "NSSupportsAutomaticGraphicsSwitching")]
  ns_supports_automatic_graphics_switching: bool,
}

const EXEC_PATH: &str = "Contents/MacOS";
const FRAMEWORKS_PATH: &str = "Contents/Frameworks";
const FRAMEWORK: &str = "Chromium Embedded Framework.framework";

fn create_app_layout(app_path: &Path) -> crate::Result<()> {
  for p in [EXEC_PATH, "Contents/Resources", FRAMEWORKS_PATH] {
    std::fs::create_dir_all(app_path.join(p))
      .fs_context("failed to create directory", app_path.join(p).to_path_buf())?;
  }
  Ok(())
}

fn write_info_plist(
  contents_path: &Path,
  exec_name: &str,
  product_name: &str,
  version: &str,
  is_helper: bool,
) -> crate::Result<()> {
  let mut ls_env = HashMap::new();
  ls_env.insert("MallocNanoZone".into(), "0".into());
  let product_compact = product_name.replace(' ', "");
  let identifier = format!("dev.tauri.{product_compact}");
  let info_plist = InfoPlist {
    cf_bundle_development_region: "en".into(),
    cf_bundle_display_name: product_name.into(),
    cf_bundle_executable: exec_name.into(),
    cf_bundle_identifier: identifier,
    cf_bundle_info_dictionary_version: "6.0".into(),
    cf_bundle_name: product_name.into(),
    cf_bundle_package_type: "APPL".into(),
    cf_bundle_signature: "????".into(),
    cf_bundle_version: version.into(),
    cf_bundle_short_version_string: version.into(),
    ls_environment: ls_env,
    ls_file_quarantine_enabled: true,
    ls_minimum_system_version: "11.0".into(),
    ls_ui_element: if is_helper { Some("1".into()) } else { None },
    ns_supports_automatic_graphics_switching: true,
  };
  plist::to_file_xml(contents_path.join("Info.plist"), &info_plist)
    .map_err(|e| crate::Error::GenericError(e.to_string()))
}

pub fn run_dev_cef_macos<A: AppSettings, F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
  app_settings: &A,
  options: Options,
  run_args: Vec<String>,
  available_targets: &mut Option<Vec<RustupTarget>>,
  config_features: Vec<String>,
  on_exit: F,
) -> crate::Result<DevChild> {
  // Build the app
  let mut build_cmd = crate::interface::rust::cargo_command(
    false,
    options.clone(),
    available_targets,
    config_features,
  )?;
  build_cmd.env("CARGO_TERM_PROGRESS_WIDTH", "80");
  build_cmd.env("CARGO_TERM_PROGRESS_WHEN", "always");
  match build_cmd.piped() {
    Ok(status) if status.success() => {}
    Ok(status) => {
      return Err(crate::Error::CommandFailed {
        command: build_cmd.get_program().to_string_lossy().into_owned(),
        error: std::io::Error::other(format!("exit with status {status}")),
      })
    }
    Err(e) => {
      return Err(crate::Error::CommandFailed {
        command: build_cmd.get_program().to_string_lossy().into_owned(),
        error: e,
      })
    }
  }

  // Bundle the .app next to the built binary
  let out_dir = app_settings.out_dir(&options)?;
  let bin_path = app_settings.app_binary_path(&options)?;
  let exec_name = bin_path
    .file_name()
    .and_then(|s| s.to_str())
    .ok_or_else(|| crate::Error::GenericError("failed to determine executable name".into()))?;

  let product_name = app_settings.get_package_settings().product_name.clone();
  let version = app_settings.get_package_settings().version.clone();

  let app_bundle_path = out_dir.join(format!("{product_name}.app"));
  let _ = std::fs::remove_dir_all(&app_bundle_path);
  create_app_layout(&app_bundle_path)?;
  write_info_plist(
    &app_bundle_path.join("Contents"),
    exec_name,
    &product_name,
    &version,
    false,
  )?;
  std::fs::copy(&bin_path, app_bundle_path.join(EXEC_PATH).join(exec_name))
    .fs_context("failed to copy executable into bundle", bin_path.clone())?;

  // Create helper .app bundles inside Frameworks
  let helpers = vec![
    format!("{} Helper (GPU)", exec_name),
    format!("{} Helper (Renderer)", exec_name),
    format!("{} Helper (Plugin)", exec_name),
    format!("{} Helper (Alerts)", exec_name),
    format!("{} Helper", exec_name),
  ];
  for helper_name in helpers {
    let helper_app = app_bundle_path
      .join(FRAMEWORKS_PATH)
      .join(format!("{helper_name}.app"));
    create_app_layout(&helper_app)?;
    write_info_plist(
      &helper_app.join("Contents"),
      &helper_name,
      &product_name,
      &version,
      true,
    )?;
    let helper_exec = helper_app.join(EXEC_PATH).join(&helper_name);
    std::fs::copy(&bin_path, &helper_exec).fs_context(
      "failed to copy main executable as CEF helper",
      helper_exec.clone(),
    )?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = std::fs::metadata(&helper_exec).unwrap().permissions();
      perms.set_mode(0o755);
      std::fs::set_permissions(&helper_exec, perms).unwrap();
    }
  }

  // Copy CEF framework from CEF_PATH
  let cef_base = std::env::var("CEF_PATH").map_err(|_| {
    crate::Error::GenericError(
      "CEF_PATH environment variable is not set; set it to the unpacked CEF root".into(),
    )
  })?;
  let framework_src = PathBuf::from(cef_base).join(FRAMEWORK);
  if !framework_src.exists() {
    return Err(crate::Error::GenericError(format!(
      "CEF framework not found at {}",
      framework_src.display()
    )));
  }
  let framework_dst = app_bundle_path.join(FRAMEWORKS_PATH).join(FRAMEWORK);
  if framework_dst.exists() {
    let _ = std::fs::remove_dir_all(&framework_dst);
  }
  copy_dir_all(&framework_src, &framework_dst)?;

  // Launch the app executable from inside the .app
  let mut exec_cmd = Command::new(app_bundle_path.join(EXEC_PATH).join(exec_name));
  exec_cmd.stdout(os_pipe::dup_stdout().unwrap());
  exec_cmd.stderr(Stdio::piped());
  exec_cmd.args(run_args);

  let child = SharedChild::spawn(&mut exec_cmd).map_err(|e| crate::Error::CommandFailed {
    command: exec_cmd.get_program().to_string_lossy().into_owned(),
    error: e,
  })?;

  let dev_child = Arc::new(child);
  let dev_child_stderr = dev_child.take_stderr().unwrap();
  let mut stderr = BufReader::new(dev_child_stderr);
  let stderr_lines = Arc::new(Mutex::new(Vec::new()));
  let stderr_lines_ = stderr_lines.clone();
  std::thread::spawn(move || {
    let mut buf = Vec::new();
    let mut lines = stderr_lines_.lock().unwrap();
    let mut io_stderr = std::io::stderr();
    loop {
      buf.clear();
      if let Ok(0) = tauri_utils::io::read_line(&mut stderr, &mut buf) {
        break;
      }
      let _ = io_stderr.write_all(&buf);
      lines.push(String::from_utf8_lossy(&buf).into_owned());
    }
  });

  let manually_killed_app = Arc::new(AtomicBool::default());
  let manually_killed_app_ = manually_killed_app.clone();
  let dev_child_ = dev_child.clone();
  std::thread::spawn(move || {
    let status = dev_child_.wait().expect("failed to run app");
    if status.success() {
      on_exit(status.code(), ExitReason::NormalExit);
    } else {
      stderr_lines.lock().unwrap().clear();
      on_exit(
        status.code(),
        if manually_killed_app_.load(Ordering::Relaxed) {
          ExitReason::TriggeredKill
        } else {
          ExitReason::NormalExit
        },
      );
    }
  });

  Ok(DevChild {
    manually_killed_app,
    dev_child,
  })
}
