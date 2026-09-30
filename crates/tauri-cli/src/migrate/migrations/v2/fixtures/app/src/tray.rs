use tauri::{
  menu::{Menu, MenuItem},
  tray::{TrayIcon, TrayIconBuilder},
  AppHandle,
};

pub fn create(app: &AppHandle<tauri::Wry>) -> tauri::Result<TrayIcon<tauri::Wry>> {
  let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
  let menu = Menu::with_items(app, &[&quit])?;
  let tray = TrayIconBuilder::new().menu(&menu).build(app)?;
  tray.with_inner_tray_icon(|tray| {
    let _ = tray.id();
  })?;
  let _size = tauri::tao::dpi::PhysicalSize::new(32, 32);
  Ok(tray)
}
