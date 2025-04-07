// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::AsShoutySnakeCase;
use tauri_utils::write_if_changed;

use std::{
  collections::BTreeMap,
  env, fs,
  path::{Path, PathBuf},
  sync::{Mutex, OnceLock},
};

static CHECKED_FEATURES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
const PLUGINS: &[(&str, &[(&str, bool)])] = &[
  // (plugin_name, &[(command, enabled-by_default)])
  // TODO: Enable this in v3
  // ("core:channel", &[("fetch", true)]),
  (
    "core:path",
    &[
      ("resolve_directory", true),
      ("resolve", true),
      ("normalize", true),
      ("join", true),
      ("dirname", true),
      ("extname", true),
      ("basename", true),
      ("is_absolute", true),
    ],
  ),
  (
    "core:event",
    &[
      ("listen", true),
      ("unlisten", true),
      ("emit", true),
      ("emit_to", true),
    ],
  ),
  (
    "core:window",
    &[
      ("create", false),
      // getters
      ("get_all_windows", true),
      ("scale_factor", true),
      ("inner_position", true),
      ("outer_position", true),
      ("inner_size", true),
      ("outer_size", true),
      ("is_fullscreen", true),
      ("is_minimized", true),
      ("is_maximized", true),
      ("is_focused", true),
      ("is_decorated", true),
      ("is_resizable", true),
      ("is_maximizable", true),
      ("is_minimizable", true),
      ("is_closable", true),
      ("is_visible", true),
      ("is_enabled", true),
      ("title", true),
      ("current_monitor", true),
      ("primary_monitor", true),
      ("monitor_from_point", true),
      ("available_monitors", true),
      ("cursor_position", true),
      ("theme", true),
      ("is_always_on_top", true),
      // setters
      ("center", false),
      ("request_user_attention", false),
      ("set_enabled", false),
      ("set_resizable", false),
      ("set_maximizable", false),
      ("set_minimizable", false),
      ("set_closable", false),
      ("set_title", false),
      ("maximize", false),
      ("unmaximize", false),
      ("minimize", false),
      ("unminimize", false),
      ("show", false),
      ("hide", false),
      ("close", false),
      ("destroy", false),
      ("set_decorations", false),
      ("set_shadow", false),
      ("set_effects", false),
      ("set_always_on_top", false),
      ("set_always_on_bottom", false),
      ("set_visible_on_all_workspaces", false),
      ("set_content_protected", false),
      ("set_size", false),
      ("set_min_size", false),
      ("set_size_constraints", false),
      ("set_max_size", false),
      ("set_position", false),
      ("set_fullscreen", false),
      ("set_focus", false),
      ("set_skip_taskbar", false),
      ("set_cursor_grab", false),
      ("set_cursor_visible", false),
      ("set_cursor_icon", false),
      ("set_cursor_position", false),
      ("set_ignore_cursor_events", false),
      ("start_dragging", false),
      ("start_resize_dragging", false),
      ("set_progress_bar", false),
      ("set_badge_count", false),
      ("set_overlay_icon", false),
      ("set_badge_label", false),
      ("set_icon", false),
      ("set_title_bar_style", false),
      ("set_theme", false),
      ("toggle_maximize", false),
      ("set_background_color", false),
      // internal
      ("internal_toggle_maximize", true),
    ],
  ),
  (
    "core:webview",
    &[
      ("create_webview", false),
      ("create_webview_window", false),
      // getters
      ("get_all_webviews", true),
      ("webview_position", true),
      ("webview_size", true),
      // setters
      ("webview_close", false),
      ("set_webview_size", false),
      ("set_webview_position", false),
      ("set_webview_focus", false),
      ("set_webview_zoom", false),
      ("webview_hide", false),
      ("webview_show", false),
      ("print", false),
      ("reparent", false),
      ("clear_all_browsing_data", false),
      ("set_webview_background_color", false),
      // internal
      ("internal_toggle_devtools", true),
    ],
  ),
  (
    "core:app",
    &[
      ("version", true),
      ("name", true),
      ("tauri_version", true),
      ("identifier", true),
      ("app_show", false),
      ("app_hide", false),
      ("fetch_data_store_identifiers", false),
      ("remove_data_store", false),
      ("default_window_icon", false),
      ("set_app_theme", false),
    ],
  ),
  (
    "core:image",
    &[
      ("new", true),
      ("from_bytes", true),
      ("from_path", true),
      ("rgba", true),
      ("size", true),
    ],
  ),
  ("core:resources", &[("close", true)]),
  (
    "core:menu",
    &[
      ("new", true),
      ("append", true),
      ("prepend", true),
      ("insert", true),
      ("remove", true),
      ("remove_at", true),
      ("items", true),
      ("get", true),
      ("popup", true),
      ("create_default", true),
      ("set_as_app_menu", true),
      ("set_as_window_menu", true),
      ("text", true),
      ("set_text", true),
      ("is_enabled", true),
      ("set_enabled", true),
      ("set_accelerator", true),
      ("set_as_windows_menu_for_nsapp", true),
      ("set_as_help_menu_for_nsapp", true),
      ("is_checked", true),
      ("set_checked", true),
      ("set_icon", true),
    ],
  ),
  (
    "core:tray",
    &[
      ("new", true),
      ("get_by_id", true),
      ("remove_by_id", true),
      ("set_icon", true),
      ("set_menu", true),
      ("set_tooltip", true),
      ("set_title", true),
      ("set_visible", true),
      ("set_temp_dir_path", true),
      ("set_icon_as_template", true),
      ("set_show_menu_on_left_click", true),
    ],
  ),
];

// checks if the given Cargo feature is enabled.
fn has_feature(feature: &str) -> bool {
  CHECKED_FEATURES
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .push(feature.to_string());

  // when a feature is enabled, Cargo sets the `CARGO_FEATURE_<name>` env var to 1
  // <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts>
  std::env::var(format!("CARGO_FEATURE_{}", AsShoutySnakeCase(feature)))
    .map(|x| x == "1")
    .unwrap_or(false)
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn alias(alias: &str, has_feature: bool) {
  println!("cargo:rustc-check-cfg=cfg({alias})");
  if has_feature {
    println!("cargo:rustc-cfg={alias}");
  }
}

fn main() {
  let custom_protocol = has_feature("custom-protocol");
  let dev = !custom_protocol;
  alias("custom_protocol", custom_protocol);
  alias("dev", dev);

  println!("cargo:dev={dev}");

  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
  let mobile = target_os == "ios" || target_os == "android";
  alias("desktop", !mobile);
  alias("mobile", mobile);

  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let checked_features_out_path = out_dir.join("checked_features");
  std::fs::write(
    checked_features_out_path,
    CHECKED_FEATURES.get().unwrap().lock().unwrap().join(","),
  )
  .expect("failed to write checked_features file");

  // workaround needed to prevent `STATUS_ENTRYPOINT_NOT_FOUND` error in tests
  // see https://github.com/tauri-apps/tauri/pull/4383#issuecomment-1212221864
  let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
  let is_tauri_workspace = std::env::var("__TAURI_WORKSPACE__").is_ok_and(|v| v == "true");
  if is_tauri_workspace && target_os == "windows" && Ok("msvc") == target_env.as_deref() {
    embed_manifest_for_tests();
  }

  if target_os == "android" {
    fn env_var(var: &str) -> String {
      std::env::var(var).unwrap_or_else(|_| {
        panic!("`{var}` is not set, which is needed to generate the kotlin files for android.")
      })
    }

    if let Ok(kotlin_out_dir) = std::env::var("WRY_ANDROID_KOTLIN_FILES_OUT_DIR") {
      let package = env_var("WRY_ANDROID_PACKAGE");
      let library = env_var("WRY_ANDROID_LIBRARY");

      let kotlin_out_dir = PathBuf::from(&kotlin_out_dir)
        .canonicalize()
        .unwrap_or_else(move |_| {
          panic!("Failed to canonicalize `WRY_ANDROID_KOTLIN_FILES_OUT_DIR` path {kotlin_out_dir}")
        });

      let kotlin_files_path =
        PathBuf::from(env_var("CARGO_MANIFEST_DIR")).join("mobile/android-codegen");
      println!("cargo:rerun-if-changed={}", kotlin_files_path.display());
      let kotlin_files =
        fs::read_dir(kotlin_files_path).expect("failed to read Android codegen directory");

      for file in kotlin_files {
        let file = file.unwrap();

        let content = fs::read_to_string(file.path())
          .expect("failed to read kotlin file as string")
          .replace("{{package}}", &package)
          .replace("{{library}}", &library);

        let out_path = kotlin_out_dir.join(file.file_name());
        // Overwrite only if changed to not trigger rebuilds
        write_if_changed(&out_path, &content).expect("Failed to write kotlin file");

        println!("cargo:rerun-if-changed={}", out_path.display());
      }
    }

    if let Some(project_dir) = env::var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
      let package_unescaped = env::var("TAURI_ANDROID_PACKAGE_UNESCAPED")
        .unwrap_or_else(|_| env_var("WRY_ANDROID_PACKAGE").replace('`', ""));
      let tauri_proguard =
        include_str!("./mobile/proguard-tauri.pro").replace("$PACKAGE", &package_unescaped);
      std::fs::write(
        project_dir.join("app").join("proguard-tauri.pro"),
        tauri_proguard,
      )
      .expect("failed to write proguard-tauri.pro");
    }

    let lib_path =
      PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("mobile/android");
    println!("cargo:android_library_path={}", lib_path.display());
  }

  #[cfg(target_os = "macos")]
  {
    if target_os == "ios" {
      let lib_path =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("mobile/ios-api");
      tauri_utils::build::link_apple_library("Tauri", &lib_path);
      println!("cargo:ios_library_path={}", lib_path.display());
    }
  }

  let tauri_global_scripts = PathBuf::from("./scripts/bundle.global.js")
    .canonicalize()
    .expect("failed to canonicalize tauri global API script path");
  tauri_utils::plugin::define_global_api_script_path(&tauri_global_scripts);
  // This should usually be done in `tauri-build`,
  // but we need to do this here for the examples in this workspace to work as they don't have build scripts
  if is_tauri_workspace {
    tauri_utils::plugin::save_global_api_scripts_paths(&out_dir, Some(tauri_global_scripts));
  }

  let permissions = define_permissions(&out_dir);
  tauri_utils::acl::build::generate_allowed_commands(&out_dir, permissions).unwrap();
}

const LICENSE_HEADER: &str = r"# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT
";

fn define_permissions(
  out_dir: &Path,
) -> BTreeMap<String, Vec<tauri_utils::acl::manifest::PermissionFile>> {
  let mut all_permissions = BTreeMap::new();
  for (plugin, commands) in PLUGINS {
    let plugin_directory_name = plugin.strip_prefix("core:").unwrap_or(plugin);
    let permissions_out_dir = out_dir.join("permissions").join(plugin_directory_name);
    let autogenerated =
      permissions_out_dir.join(tauri_utils::acl::build::AUTOGENERATED_FOLDER_NAME);
    let commands_dir = autogenerated.join("commands");

    tauri_utils::acl::build::autogenerate_command_permissions(
      &commands_dir,
      &commands.iter().map(|(cmd, _)| *cmd).collect::<Vec<_>>(),
      LICENSE_HEADER,
      false,
    );
    let default_permissions: Vec<_> = commands.iter().filter(|(_cmd, default)| *default).collect();
    let all_commands_enabled_by_default = commands.len() == default_permissions.len();
    let default_permissions = default_permissions
      .into_iter()
      .map(|(cmd, _)| {
        let slugified_command = cmd.replace('_', "-");
        format!("\"allow-{slugified_command}\"")
      })
      .collect::<Vec<_>>()
      .join(", ");

    let all_enable_by_default = if all_commands_enabled_by_default {
      ", which enables all commands"
    } else {
      ""
    };

    let default_toml = format!(
      r###"{LICENSE_HEADER}# Automatically generated - DO NOT EDIT!

[default]
description = "Default permissions for the plugin{all_enable_by_default}."
permissions = [{default_permissions}]
"###,
    );

    let out_path = autogenerated.join("default.toml");
    write_if_changed(out_path, default_toml)
      .unwrap_or_else(|_| panic!("unable to autogenerate default permissions"));

    let permissions = tauri_utils::acl::build::define_permissions(
      &PathBuf::from(glob::Pattern::escape(
        &permissions_out_dir.to_string_lossy(),
      ))
      .join("**")
      .join("*.toml")
      .to_string_lossy(),
      &format!("tauri:{plugin}"),
      out_dir,
      |_| true,
    )
    .unwrap_or_else(|e| panic!("failed to define permissions for {plugin}: {e}"));

    let docs_out_dir = Path::new("permissions")
      .join(plugin_directory_name)
      .join("autogenerated");
    fs::create_dir_all(&docs_out_dir).expect("failed to create plugin documentation directory");
    tauri_utils::acl::build::generate_docs(
      &permissions,
      &docs_out_dir,
      plugin.strip_prefix("tauri-plugin-").unwrap_or(plugin),
    )
    .expect("failed to generate plugin documentation page");
    all_permissions.insert(plugin.to_string(), permissions);
  }

  let default_permissions = define_default_permission_set(out_dir);
  all_permissions.insert("core".to_string(), default_permissions);

  all_permissions
}

fn define_default_permission_set(
  out_dir: &Path,
) -> Vec<tauri_utils::acl::manifest::PermissionFile> {
  let permissions_out_dir = out_dir.join("permissions");
  fs::create_dir_all(&permissions_out_dir)
    .expect("failed to create core:default permissions directory");

  let default_toml = permissions_out_dir.join("default.toml");
  let toml_content = format!(
    r#"{LICENSE_HEADER}

[default]
description = "Default core plugins set."
permissions = [{}]
"#,
    PLUGINS
      .iter()
      .map(|(k, _)| format!("\"{k}:default\""))
      .collect::<Vec<_>>()
      .join(",")
  );

  write_if_changed(default_toml, toml_content)
    .unwrap_or_else(|_| panic!("unable to autogenerate core:default set"));

  tauri_utils::acl::build::define_permissions(
    &PathBuf::from(glob::Pattern::escape(
      &permissions_out_dir.to_string_lossy(),
    ))
    .join("*.toml")
    .to_string_lossy(),
    "tauri:core",
    out_dir,
    |_| true,
  )
  .unwrap_or_else(|e| panic!("failed to define permissions for `core:default` : {e}"))
}

fn embed_manifest_for_tests() {
  static WINDOWS_MANIFEST_FILE: &str = "windows-app-manifest.xml";

  let manifest = std::env::current_dir()
    .unwrap()
    .join("../tauri-build/src")
    .join(WINDOWS_MANIFEST_FILE);

  println!("cargo:rerun-if-changed={}", manifest.display());
  // Embed the Windows application manifest file.
  println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
  println!(
    "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
    manifest.to_str().unwrap()
  );
  // Turn linker warnings into errors.
  println!("cargo:rustc-link-arg=/WX");
}
