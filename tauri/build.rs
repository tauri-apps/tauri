use cfg_aliases::cfg_aliases;

fn main() {
  setup_env_aliases()
}

fn setup_env_aliases() {
  cfg_aliases! {
    embedded_server: { feature = "embedded-server" },
    no_server: { feature = "no-server" },
    assets: { any(feature = "embedded-server", feature = "no-server") },
    dev: { not(any(feature = "embedded-server", feature = "no-server")) },

    all_api: { feature = "all-api" },

    // fs
    read_text_file: { any(all_api, feature = "read-text-file") },
    read_binary_file: { any(all_api, feature = "read-binary-file") },
    write_file: { any(all_api, feature = "write-file") },
    write_binary_file: { any(all_api, feature = "write-binary-file") },
    read_dir: { any(all_api, feature = "read-dir") },
    copy_file: { any(all_api, feature = "copy-file") },
    create_dir: { any(all_api, feature = "create_dir") },
    remove_dir: { any(all_api, feature = "remove-dir") },
    remove_file: { any(all_api, feature = "remove-file") },
    rename_file: { any(all_api, feature = "rename-file") },

    // window
    set_title: { any(all_api, feature = "set-title") },
    open: { any(all_api, feature = "open") },

    // process
    execute: { any(all_api, feature = "execute") },

    // event
    event: { any(all_api, feature = "event") },

    // dialog
    open_dialog: { any(all_api, feature = "open-dialog") },
    save_dialog: { any(all_api, feature = "save-dialog") },

    // http
    http_request: { any(all_api, feature = "http-request") },

    // cli
    cli: { feature = "cli" },

    // notification
    notification: { any(all_api, feature = "notification") },
  }
}
