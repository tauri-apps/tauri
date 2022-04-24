// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use heck::ToSnakeCase;

fn has_feature(feature: &str) -> bool {
  std::env::var(format!(
    "CARGO_FEATURE_{}",
    feature.to_uppercase().replace('-', "_").replace('"', "")
  ))
  .map(|x| x == "1")
  .unwrap_or(false)
}

fn alias(alias: &str, has_feature: bool) {
  if has_feature {
    println!("cargo:rustc-cfg={}", alias);
  }
}

fn main() {
  alias("custom_protocol", has_feature("custom-protocol"));
  alias("dev", !has_feature("custom-protocol"));
  alias(
    "updater",
    has_feature("updater") || has_feature("__updater-docs"),
  );

  let api_all = has_feature("api-all");
  alias("api_all", api_all);

  let fs_any = alias_module(
    "fs",
    &[
      "read-file",
      "write-file",
      "write-binary-file",
      "read-dir",
      "copy-file",
      "create-dir",
      "remove-dir",
      "remove-file",
      "rename-file",
    ],
    api_all,
  );

  let window_any = alias_module(
    "window",
    &[
      "create",
      "center",
      "request-user-attention",
      "set-resizable",
      "set-title",
      "maximize",
      "unmaximize",
      "minimize",
      "unminimize",
      "show",
      "hide",
      "close",
      "set-decorations",
      "set-always-on-top",
      "set-size",
      "set-min-size",
      "set-max-size",
      "set-position",
      "set-fullscreen",
      "set-focus",
      "set-icon",
      "set-skip-taskbar",
      "set-cursor-grab",
      "set-cursor-visible",
      "set-cursor-icon",
      "set-cursor-position",
      "start-dragging",
      "print",
    ],
    api_all,
  );

  let shell_any = alias_module("shell", &["execute", "sidecar", "open"], api_all);
  // helper for the command module macro
  let shell_script = has_feature("shell-execute") || has_feature("shell-sidecar");
  alias("shell_script", shell_script);
  alias("shell_scope", shell_script || has_feature("shell-open-api"));

  let dialog_any = alias_module(
    "dialog",
    &["open", "save", "message", "ask", "confirm"],
    api_all,
  );

  let http_any = alias_module("http", &["request"], api_all);

  let cli = has_feature("cli");
  alias("cli", cli);

  let notification_any = alias_module("notification", &[], api_all);
  let global_shortcut_any = alias_module("global-shortcut", &[], api_all);
  let os_any = alias_module("os", &[], api_all);
  let path_any = alias_module("path", &[], api_all);

  let protocol_any = alias_module("protocol", &["asset"], api_all);

  let process_any = alias_module("process", &["relaunch", "exit"], api_all);

  let clipboard_any = alias_module("clipboard", &["write-text", "read-text"], api_all);

  let api_any = fs_any
    || window_any
    || shell_any
    || dialog_any
    || http_any
    || cli
    || notification_any
    || global_shortcut_any
    || os_any
    || path_any
    || protocol_any
    || process_any
    || clipboard_any;

  alias("api_any", api_any);
}

fn alias_module(module: &str, apis: &[&str], api_all: bool) -> bool {
  let all_feature_name = format!("{}-all", module);
  let all = api_all || has_feature(&all_feature_name);
  alias(&all_feature_name.to_snake_case(), all);

  let mut any = all;

  for api in apis {
    let has = all || has_feature(&format!("{}-{}", module, api));
    alias(
      &format!("{}_{}", module.to_snake_case(), api.to_snake_case()),
      has,
    );
    any = any || has;
  }

  alias(&format!("{}_any", module.to_snake_case()), any);

  any
}
