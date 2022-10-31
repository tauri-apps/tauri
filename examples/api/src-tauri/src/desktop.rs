use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
  api::dialog::ask, CustomMenuItem, GlobalShortcutManager, Manager, RunEvent, SystemTray,
  SystemTrayEvent, SystemTrayMenu, WindowBuilder, WindowEvent, WindowUrl,
};

pub fn main() {
  api::AppBuilder::new()
    .setup(|app| {
      create_tray(app)?;
      Ok(())
    })
    .on_event(|app_handle, e| match e {
      // Application is ready (triggered only once)
      RunEvent::Ready => {
        let app_handle = app_handle.clone();
        app_handle
          .global_shortcut_manager()
          .register("CmdOrCtrl+1", move || {
            let app_handle = app_handle.clone();
            let window = app_handle.get_window("main").unwrap();
            window.set_title("New title!").unwrap();
          })
          .unwrap();
      }

      // Triggered when a window is trying to close
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::CloseRequested { api, .. },
        ..
      } => {
        // for other windows, we handle it in JS
        if label == "main" {
          let app_handle = app_handle.clone();
          let window = app_handle.get_window(&label).unwrap();
          // use the exposed close api, and prevent the event loop to close
          api.prevent_close();
          // ask the user if he wants to quit
          ask(
            Some(&window),
            "Tauri API",
            "Are you sure that you want to close this window?",
            move |answer| {
              if answer {
                // .close() cannot be called on the main thread
                std::thread::spawn(move || {
                  app_handle.get_window(&label).unwrap().close().unwrap();
                });
              }
            },
          );
        }
      }
      _ => (),
    })
    .run()
}

fn create_tray(app: &tauri::App) -> tauri::Result<()> {
  let mut tray_menu1 = SystemTrayMenu::new()
    .add_item(CustomMenuItem::new("toggle", "Toggle"))
    .add_item(CustomMenuItem::new("new", "New window"))
    .add_item(CustomMenuItem::new("icon_1", "Tray Icon 1"))
    .add_item(CustomMenuItem::new("icon_2", "Tray Icon 2"));

  #[cfg(target_os = "macos")]
  {
    tray_menu1 = tray_menu1.add_item(CustomMenuItem::new("set_title", "Set Title"));
  }

  tray_menu1 = tray_menu1
    .add_item(CustomMenuItem::new("switch_menu", "Switch Menu"))
    .add_item(CustomMenuItem::new("exit_app", "Quit"))
    .add_item(CustomMenuItem::new("destroy", "Destroy"));

  let tray_menu2 = SystemTrayMenu::new()
    .add_item(CustomMenuItem::new("toggle", "Toggle"))
    .add_item(CustomMenuItem::new("new", "New window"))
    .add_item(CustomMenuItem::new("switch_menu", "Switch Menu"))
    .add_item(CustomMenuItem::new("exit_app", "Quit"))
    .add_item(CustomMenuItem::new("destroy", "Destroy"));
  let is_menu1 = AtomicBool::new(true);

  let handle = app.handle();
  let tray_id = "my-tray".to_string();
  SystemTray::new()
    .with_id(&tray_id)
    .with_menu(tray_menu1.clone())
    .on_event(move |event| {
      let tray_handle = handle.tray_handle_by_id(&tray_id).unwrap();
      match event {
        SystemTrayEvent::LeftClick {
          position: _,
          size: _,
          ..
        } => {
          let window = handle.get_window("main").unwrap();
          window.show().unwrap();
          window.set_focus().unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
          let item_handle = tray_handle.get_item(&id);
          match id.as_str() {
            "exit_app" => {
              // exit the app
              handle.exit(0);
            }
            "destroy" => {
              tray_handle.destroy().unwrap();
            }
            "toggle" => {
              let window = handle.get_window("main").unwrap();
              let new_title = if window.is_visible().unwrap() {
                window.hide().unwrap();
                "Show"
              } else {
                window.show().unwrap();
                "Hide"
              };
              item_handle.set_title(new_title).unwrap();
            }
            "new" => {
              WindowBuilder::new(&handle, "new", WindowUrl::App("index.html".into()))
                .title("Tauri")
                .build()
                .unwrap();
            }
            "set_title" => {
              #[cfg(target_os = "macos")]
              tray_handle.set_title("Tauri").unwrap();
            }
            "icon_1" => {
              #[cfg(target_os = "macos")]
              tray_handle.set_icon_as_template(true).unwrap();

              tray_handle
                .set_icon(tauri::Icon::Raw(
                  include_bytes!("../../../.icons/tray_icon_with_transparency.png").to_vec(),
                ))
                .unwrap();
            }
            "icon_2" => {
              #[cfg(target_os = "macos")]
              tray_handle.set_icon_as_template(true).unwrap();

              tray_handle
                .set_icon(tauri::Icon::Raw(
                  include_bytes!("../../../.icons/icon.ico").to_vec(),
                ))
                .unwrap();
            }
            "switch_menu" => {
              let flag = is_menu1.load(Ordering::Relaxed);
              tray_handle
                .set_menu(if flag {
                  tray_menu2.clone()
                } else {
                  tray_menu1.clone()
                })
                .unwrap();
              is_menu1.store(!flag, Ordering::Relaxed);
            }
            _ => {}
          }
        }
        _ => {}
      }
    })
    .build(app)
    .map(|_| ())
}
