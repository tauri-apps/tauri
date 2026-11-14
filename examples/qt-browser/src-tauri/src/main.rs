// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Qt Browser: a Chromium-style browser built with Tauri on the Qt runtime, and a showcase of
//! Tauri's features and official plugins.
//!
//! - Windows and webviews: frameless windows with a custom title bar, one child webview per tab
//!   (`browser.rs`), new windows, popups, fullscreen, always on top, theme, taskbar badge,
//!   progress bar and attention requests, zoom, print, devtools, cookies, file drops.
//! - Menus (`menus.rs`): the app menu as a menu bar, native popup menus (the "⋮" menu, the
//!   page's right-click menu, the site information menu) and the tray icon with its menu.
//! - IPC: commands, events, a channel (`downloads.rs`), capabilities that give web pages a few
//!   commands only (capabilities/web-pages.json).
//! - Protocols: the browser's own pages on a custom `qtbrowser://` scheme (`pages.rs`), and the
//!   asset protocol for download thumbnails.
//! - Plugins: single-instance, store (settings), window-state, notification, dialog,
//!   clipboard-manager, opener, global-shortcut, os, process, autostart and log.
//!
//!   cargo run -p qt-browser [url]

mod browser;
mod commands;
mod downloads;
mod menus;
mod pages;
mod settings;

use tauri::{DragDropEvent, Manager, RunEvent, Url, WebviewEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use browser::{Browser, browser};

fn main() {
  let start_url = std::env::args().nth(1);
  let app = tauri::Builder::default()
    .runtime(tauri_runtime_qt::Qt::default())
    // Launching the browser again opens the URL in the running one (must come first).
    .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
      let settings = browser(app).settings.clone();
      match (args.get(1), browser::current_window(app)) {
        (Some(input), Some(window)) => {
          let url = browser::resolve_input(input, &settings);
          let _ = browser::open_tab(app, &window, url, true, None);
          menus::show_browser(app);
        }
        (Some(input), None) => {
          let _ = browser::open_window(app, Some(browser::resolve_input(input, &settings)));
        }
        (None, _) => menus::show_browser(app),
      }
    }))
    .plugin(
      tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .build(),
    )
    .plugin(tauri_plugin_store::Builder::new().build())
    // Remembers each browser window's size, position and maximized state.
    .plugin(
      tauri_plugin_window_state::Builder::new()
        .with_filter(|label| label.starts_with("browser-"))
        .build(),
    )
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    // The browser opens its own links: no opener script sending target=_blank links to the
    // system browser.
    .plugin(
      tauri_plugin_opener::Builder::new()
        .open_js_links_on_click(false)
        .build(),
    )
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_autostart::Builder::new().build())
    .manage(Browser::default())
    .menu(menus::app_menu)
    .on_menu_event(menus::on_menu_event)
    .register_asynchronous_uri_scheme_protocol(browser::INTERNAL, pages::protocol)
    .invoke_handler(tauri::generate_handler![
      commands::browser_state,
      commands::new_tab,
      commands::close_tab,
      commands::activate_tab,
      commands::move_tab,
      commands::navigate,
      commands::go_back,
      commands::go_forward,
      commands::reload,
      commands::stop,
      commands::zoom,
      commands::new_window,
      commands::reopen_closed_tab,
      commands::focus_content,
      commands::shortcut,
      menus::open_menu,
      menus::page_context_menu,
      menus::site_info_menu,
      menus::page_html,
      pages::get_settings,
      pages::save_settings,
      pages::clear_browsing_data,
      pages::pick_download_dir,
      pages::set_autostart,
      pages::about_info,
      downloads::watch_downloads,
      downloads::download_action,
    ])
    .setup(move |app| {
      let handle = app.handle().clone();
      let settings = settings::load(&handle);
      browser(&handle).settings = settings.clone();
      pages::apply(&handle, None, &settings);
      menus::create_tray(&handle)?;
      let start = start_url.map(|u| browser::resolve_input(&u, &settings));
      browser::open_window(&handle, start)?;
      Ok(())
    })
    .on_window_event(|window, event| {
      let app = window.app_handle();
      match event {
        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
          browser::layout(app, window);
        }
        WindowEvent::Focused(true) if window.label().starts_with("browser-") => {
          browser(app).current_window = Some(window.label().to_string());
          menus::sync_checks(app);
        }
        // Closing a window with several tabs asks first (dialog plugin).
        WindowEvent::CloseRequested { api, .. } => {
          let tabs = browser(app)
            .windows
            .get(window.label())
            .map_or(0, |w| w.tabs.len());
          if tabs > 1 && browser(app).settings.confirm_close {
            api.prevent_close();
            let window = window.clone();
            app
              .dialog()
              .message(format!("This window has {tabs} tabs open."))
              .title("Close all tabs?")
              .buttons(MessageDialogButtons::OkCancelCustom(
                "Close tabs".into(),
                "Cancel".into(),
              ))
              .show(move |close| {
                if close {
                  let _ = window.destroy();
                }
              });
          }
        }
        WindowEvent::Destroyed => {
          let mut inner = browser(app);
          if let Some(w) = inner.windows.remove(window.label()) {
            for id in w.tabs {
              inner.tabs.remove(&id);
            }
          }
        }
        _ => {}
      }
    })
    // Files dropped on the title bar open in new tabs (tabs leave drops to their pages).
    .on_webview_event(|webview, event| {
      if let WebviewEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event
        && webview.label().starts_with("chrome-")
      {
        let app = webview.app_handle();
        let window = webview.window().label().to_string();
        for path in paths {
          if let Ok(url) = Url::from_file_path(path) {
            let _ = browser::open_tab(app, &window, url, true, None);
          }
        }
      }
    })
    .build(tauri::generate_context!())
    .expect("error while building Qt Browser");

  app.run(|app, event| {
    // Closing the last window quits, unless the browser keeps running in the tray.
    if let RunEvent::ExitRequested {
      code: None, api, ..
    } = event
      && browser(app).settings.keep_in_tray
    {
      api.prevent_exit();
    }
  });
}
