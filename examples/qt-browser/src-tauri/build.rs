// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  // An app ACL manifest, so web pages (remote origins) can be granted a few commands
  // (capabilities/web-pages.json) and the browser's own pages all of them
  // (capabilities/browser-ui.json).
  tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
    tauri_build::AppManifest::new().commands(&[
      // tabs and windows (src/browser.rs)
      "browser_state",
      "new_tab",
      "close_tab",
      "activate_tab",
      "move_tab",
      "navigate",
      "go_back",
      "go_forward",
      "reload",
      "stop",
      "zoom",
      "new_window",
      "reopen_closed_tab",
      "focus_content",
      "shortcut",
      // menus (src/menus.rs)
      "open_menu",
      "page_context_menu",
      "site_info_menu",
      "page_html",
      // settings, internal pages (src/pages.rs)
      "get_settings",
      "save_settings",
      "clear_browsing_data",
      "pick_download_dir",
      "set_autostart",
      "about_info",
      // downloads (src/downloads.rs)
      "watch_downloads",
      "download_action",
    ]),
  ))
  .expect("tauri-build failed");
}
