// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cfg_aliases::cfg_aliases;

fn main() {
  cfg_aliases! {
    custom_protocol: { feature = "custom-protocol" },
    dev: { not(feature = "custom-protocol") },

    api_all: { feature = "api-all" },

    // fs
    fs_all: { any(api_all, feature = "fs-all") },
    fs_read_text_file: { any(fs_all, feature = "fs-read-text-file") },
    fs_read_binary_file: { any(fs_all, feature = "fs-read-binary-file") },
    fs_write_file: { any(fs_all, feature = "fs-write-file") },
    fs_write_binary_file: { any(fs_all, feature = "fs-write-binary-file") },
    fs_read_dir: { any(fs_all, feature = "fs-read-dir") },
    fs_copy_file: { any(fs_all, feature = "fs-copy-file") },
    fs_create_dir: { any(fs_all, feature = "fs-create_dir") },
    fs_remove_dir: { any(fs_all, feature = "fs-remove-dir") },
    fs_remove_file: { any(fs_all, feature = "fs-remove-file") },
    fs_rename_file: { any(fs_all, feature = "fs-rename-file") },
    fs_path: { any(fs_all, feature = "fs-path") },

    // window
    window_all: { any(api_all, feature = "window-all") },
    window_create: { any(window_all, feature = "window-create") },

    // shell
    shell_all: { any(api_all, feature = "shell-all") },
    shell_open: { any(shell_all, feature = "shell-open") },
    shell_execute: { any(shell_all, feature = "shell-execute") },

    // dialog
    dialog_all: { any(api_all, feature = "dialog-all") },
    dialog_open: { any(dialog_all, feature = "dialog-open") },
    dialog_save: { any(dialog_all, feature = "dialog-save") },

    // http
    http_all: { any(api_all, feature = "http-all") },
    http_request: { any(http_all, feature = "http-request") },

    // cli
    cli: { feature = "cli" },

    // notification
    notification_all: { any(api_all, feature = "notification-all") },

    // global shortcut
    global_shortcut_all: { any(api_all, feature = "global_shortcut-all") },
  }
}
