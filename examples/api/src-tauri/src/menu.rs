use tauri::{CustomMenuItem, Menu, MenuItem};

pub fn get_menu() -> Vec<Menu> {
  // `Primary` is a platform-agnostic accelerator modifier.
  // On Windows and Linux, `Primary` maps to the `Ctrl` key,
  // and on macOS it maps to the `command` key.
  let custom_print_menu =
    MenuItem::Custom(CustomMenuItem::new("Print").with_accelerators("<Primary>p"));
  let other_test_menu =
    MenuItem::Custom(CustomMenuItem::new("Custom").with_accelerators("<Primary>M"));
  let quit_menu = MenuItem::Custom(CustomMenuItem::new("Quit").with_accelerators("<Primary>q"));

  // macOS require to have at least Copy, Paste, Select all etc..
  // to works fine. You should always add them.
  #[cfg(any(target_os = "linux", target_os = "macos"))]
  let menu = vec![
    Menu::new(
      // on macOS first menu is always app name
      "Tauri API",
      vec![
        // All's non-custom menu, do NOT return event's
        // they are handled by the system automatically
        MenuItem::About("Todos"),
        MenuItem::Services,
        MenuItem::Separator,
        MenuItem::Hide,
        MenuItem::HideOthers,
        MenuItem::ShowAll,
        MenuItem::Separator,
        quit_menu,
      ],
    ),
    Menu::new(
      "File",
      vec![
        custom_print_menu,
        MenuItem::Separator,
        other_test_menu,
        MenuItem::CloseWindow,
      ],
    ),
    Menu::new(
      "Edit",
      vec![
        MenuItem::Undo,
        MenuItem::Redo,
        MenuItem::Separator,
        MenuItem::Cut,
        MenuItem::Copy,
        MenuItem::Paste,
        MenuItem::Separator,
        MenuItem::SelectAll,
      ],
    ),
    Menu::new("View", vec![MenuItem::EnterFullScreen]),
    Menu::new("Window", vec![MenuItem::Minimize, MenuItem::Zoom]),
    Menu::new(
      "Help",
      vec![MenuItem::Custom(
        CustomMenuItem::new("Custom help").with_accelerators("<Primary><Shift>h"),
      )],
    ),
  ];

  // Attention, Windows only support custom menu for now.
  // If we add any `MenuItem::*` they'll not render
  // We need to use custom menu with `Menu::new()` and catch
  // the events in the EventLoop.
  #[cfg(target_os = "windows")]
  let menu = vec![
    Menu::new("File", vec![other_test_menu]),
    Menu::new("Other menu", vec![quit_menu]),
  ];
  menu
}
