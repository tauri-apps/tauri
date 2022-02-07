// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
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
    fs_read_file: { any(fs_all, feature = "fs-read-file") },
    fs_write_file: { any(fs_all, feature = "fs-write-file") },
    fs_write_binary_file: { any(fs_all, feature = "fs-write-binary-file") },
    fs_read_dir: { any(fs_all, feature = "fs-read-dir") },
    fs_copy_file: { any(fs_all, feature = "fs-copy-file") },
    fs_create_dir: { any(fs_all, feature = "fs-create_dir") },
    fs_remove_dir: { any(fs_all, feature = "fs-remove-dir") },
    fs_remove_file: { any(fs_all, feature = "fs-remove-file") },
    fs_rename_file: { any(fs_all, feature = "fs-rename-file") },

    // window
    window_all: { any(api_all, feature = "window-all") },
    window_create: { any(window_all, feature = "window-create") },
    window_center: { any(window_all, feature = "window-center") },
    window_request_user_attention: { any(window_all, feature = "window-request-user-attention") },
    window_set_resizable: { any(window_all, feature = "window-set-resizable") },
    window_set_title: { any(window_all, feature = "window-set-title") },
    window_maximize: { any(window_all, feature = "window-maximize") },
    window_unmaximize: { any(window_all, feature = "window-unmaximize") },
    window_minimize: { any(window_all, feature = "window-minimize") },
    window_unminimize: { any(window_all, feature = "window-unminimize") },
    window_show: { any(window_all, feature = "window-show") },
    window_hide: { any(window_all, feature = "window-hide") },
    window_close: { any(window_all, feature = "window-close") },
    window_set_decorations: { any(window_all, feature = "window-set-decorations") },
    window_set_always_on_top: { any(window_all, feature = "window-set-always-on-top") },
    window_set_size: { any(window_all, feature = "window-set-size") },
    window_set_min_size: { any(window_all, feature = "window-set-min-size") },
    window_set_max_size: { any(window_all, feature = "window-set-max-size") },
    window_set_position: { any(window_all, feature = "window-set-position") },
    window_set_fullscreen: { any(window_all, feature = "window-set-fullscreen") },
    window_set_focus: { any(window_all, feature = "window-set-focus") },
    window_set_icon: { any(window_all, feature = "window-set-icon") },
    window_set_skip_taskbar: { any(window_all, feature = "window-set-skip-taskbar") },
    window_start_dragging: { any(window_all, feature = "window-start-dragging") },
    window_print: { any(window_all, feature = "window-print") },

    // shell
    shell_all: { any(api_all, feature = "shell-all") },
    shell_execute: { any(shell_all, feature = "shell-execute") },
    shell_sidecar: { any(shell_all, feature = "shell-sidecar") },
    shell_open: { any(shell_all, feature = "shell-open") },
    // helper for the shell scope functionality
    shell_scope: { any(shell_execute, shell_sidecar, feature = "shell-open-api") },

    // dialog
    dialog_all: { any(api_all, feature = "dialog-all") },
    dialog_open: { any(dialog_all, feature = "dialog-open") },
    dialog_save: { any(dialog_all, feature = "dialog-save") },
    dialog_message: { any(dialog_all, feature = "dialog-message") },
    dialog_ask: { any(dialog_all, feature = "dialog-ask") },
    dialog_confirm: { any(dialog_all, feature = "dialog-confirm") },

    // http
    http_all: { any(api_all, feature = "http-all") },
    http_request: { any(http_all, feature = "http-request") },

    // cli
    cli: { feature = "cli" },

    // notification
    notification_all: { any(api_all, feature = "notification-all") },

    // global shortcut
    global_shortcut_all: { any(api_all, feature = "global_shortcut-all") },

    // os
    os_all: { any(api_all, feature = "os-all") },

    // path
    path_all: { any(api_all, feature = "path-all") },

    // protocol
    protocol_all: { any(api_all, feature = "protocol-all") },
    protocol_asset: { any(protocol_all, feature = "protocol-asset") },

    // process
    process_all: { any(api_all, feature = "process-all") },
    process_relaunch: { any(protocol_all, feature = "process-relaunch") },
    process_relaunch_dangerous_allow_symlink_macos: { feature = "process-relaunch-dangerous-allow-symlink-macos" },
    process_exit: { any(protocol_all, feature = "process-exit") },

    // clipboard
    clipboard_all: { any(api_all, feature = "clipboard-all") },
    clipboard_write_text: { any(protocol_all, feature = "clipboard-write-text") },
    clipboard_read_text: { any(protocol_all, feature = "clipboard-read-text") },
  }
}
